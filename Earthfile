VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.1 AS rust

FROM rust:slim-bookworm
RUN apt-get update && apt-get install -y binutils pkg-config openssl libssl-dev && apt-get clean
WORKDIR /rustexample

# build creates the binary target/release/example-rust
build:
    # CARGO function adds caching to cargo runs.
    # See https://github.com/earthly/lib/tree/main/rust
    DO rust+INIT --keep_fingerprints=true
    COPY --keep-ts --dir src Cargo.lock Cargo.toml .
    DO rust+CARGO --args="build --release --bin inkjet" --output="release/[^/\.]+"
    RUN strip releasae/inkjet
    SAVE ARTIFACT target/release/inkjet AS LOCAL inkjet