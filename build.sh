#docker pull messense/rust-musl-cross:arm-musleabihf
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:arm-musleabihf'
rust-musl-builder cargo build --release
