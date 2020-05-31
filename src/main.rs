use clap::{App, Arg};
use libavif_sys::{
    avifDecoderCreate, avifDecoderRead, avifImage, avifImageCreateEmpty, avifImageYUVToRGB,
    avifROData, AVIF_RESULT_OK,
};
use std::{
    convert::TryFrom,
    fs::File,
    io::{self, BufWriter, Read},
    path::Path,
    ptr,
};

fn main() {
    let matches = App::new("davif")
        .version("1.0")
        .author("Kan-Ru Chen <kanru@kanru.info>")
        .about("decompress an AVIF file to an image file")
        .arg(
            Arg::with_name("output_file")
                .short("o")
                .value_name("string")
                .help("Specify the name of the output file (as PNG format by default).")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("resize")
                .short("r")
                .long("resize")
                .visible_alias("scale")
                .help("Rescale the decoded picture to dimension width x height.")
                .long_help(
                    "Rescale the decoded picture to dimension width x height.
If either (but not both) of the width or height parameters is 0,
the value will be calculated preserving the aspect-ratio.",
                )
                .value_names(&["width", "height"])
                .takes_value(true)
                .number_of_values(2)
                .validator(|v| match v.parse::<usize>() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }),
        )
        .arg(
            Arg::with_name("input_file")
                .takes_value(true)
                .value_name("input_file.avif")
                .required(true),
        )
        .get_matches();

    let input_file = matches.value_of("input_file").expect("parse input_file");
    let output_file = matches.value_of("output_file").expect("parse output_file");
    let (width, height) = match matches.values_of("resize") {
        Some(mut dimension) => {
            let width: usize = dimension.next().unwrap().parse().unwrap();
            let height: usize = dimension.next().unwrap().parse().unwrap();
            (width, height)
        }
        None => (0, 0),
    };

    convert_avif_to_png(input_file, output_file, width, height).expect("convert avif to png");
}

struct RgbImage {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl RgbImage {
    fn resize(&mut self, mut width: usize, mut height: usize) {
        if width != 0 || height != 0 {
            if width == 0 {
                width = ((self.width as f32 / self.height as f32) * height as f32) as usize;
            } else if height == 0 {
                height = ((self.height as f32 / self.width as f32) * width as f32) as usize;
            }
        } else {
            width = self.width;
            height = self.height;
        }
        let mut resizer = resize::new(
            self.width,
            self.height,
            width,
            height,
            resize::Pixel::RGB24,
            resize::Type::Lanczos3,
        );
        let mut resized = vec![0; width * height * 3];
        resizer.resize(&self.data, &mut resized);
        self.data = resized;
        self.width = width;
        self.height = height;
    }

    fn save_as_png<T: AsRef<Path>>(&self, path: T) -> io::Result<()> {
        let file = File::create(path)?;
        let w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.data)?;
        Ok(())
    }
}

impl TryFrom<avifROData> for RgbImage {
    type Error = io::Error;
    fn try_from(mut avif_data: avifROData) -> Result<Self, Self::Error> {
        unsafe {
            let decoder = avifDecoderCreate();
            let image = avifImageCreateEmpty();
            match avifDecoderRead(decoder, image, &mut avif_data) {
                AVIF_RESULT_OK => Ok(image.into()),
                error_code => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to decode AVIF image ({})", error_code),
                )),
            }
        }
    }
}

impl From<*mut avifImage> for RgbImage {
    fn from(image: *mut avifImage) -> Self {
        unsafe {
            avifImageYUVToRGB(image);
        }
        let (width, height) = unsafe { ((*image).width, (*image).height) };
        let capacity = (width * height) as usize;
        let mut data = Vec::with_capacity(capacity);
        let rgb_planes = unsafe { (*image).rgbPlanes };
        for v in 0..height {
            for u in 0..width {
                for p in 0..3 {
                    data.push(
                        unsafe { &*ptr::slice_from_raw_parts(rgb_planes[p], capacity) }
                            [(u + v * unsafe { (*image).rgbRowBytes[p] }) as usize],
                    );
                }
            }
        }
        RgbImage {
            data,
            width: width as usize,
            height: height as usize,
        }
    }
}

fn convert_avif_to_png<T: AsRef<Path>>(
    input_file: T,
    output_file: T,
    width: usize,
    height: usize,
) -> io::Result<()> {
    let avif_data = open_avif(input_file)?;
    let avif_image = as_avif(&avif_data);
    let mut rgb_image = RgbImage::try_from(avif_image)?;
    rgb_image.resize(width, height);
    rgb_image.save_as_png(output_file)?;
    Ok(())
}

fn open_avif<T: AsRef<Path>>(path: T) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn as_avif(buf: &[u8]) -> avifROData {
    avifROData {
        data: buf.as_ptr(),
        size: buf.len(),
    }
}
