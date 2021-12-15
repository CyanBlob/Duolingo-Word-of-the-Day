docker run --rm -v `pwd`:/project -w /project ekidd/rust-musl-builder:latest cargo build --release --target arm-unknown-linux-musleabihf
