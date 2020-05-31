## DAVIF

Decompress an [AVIF][] file to an image file.
As of 2020-05-31 only PNG format output is implemented.

I wrote this program to use it as a thumbnailer for Gnome desktop.
The commandline switch is compatiable with the `dwebp` utility for webp.

[AVIF]: https://aomediacodec.github.io/av1-avif/

## Install

First install the binary

```sh
cargo install davif
sudo install $HOME/.cargo/bin/davif /usr/bin
```

Install the mime type definition

```sh
mkdir -p $HOME/.local/share/mime/packages
cp avif.xml $HOME/.local/share/mime/packages
update-mime-database $HOME/.local/share/mime
```

Then install the thumbnailer config

```sh
install -m644 avif.thumbnailer $HOME/.local/share/thumbnailers/
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.