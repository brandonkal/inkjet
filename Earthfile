# Copyright 2020 Brandon Kalinowski (brandonkal)
# SPDX-License-Identifier: MIT

VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.1 AS rust

FROM rust:slim-bookworm
RUN apt-get update && apt-get install -y binutils pkg-config openssl libssl-dev && apt-get clean
WORKDIR /build

source:
    # CARGO function adds caching to cargo runs.
    # See https://github.com/earthly/lib/tree/main/rust
    DO rust+INIT --keep_fingerprints=true
    COPY --keep-ts --dir src Cargo.lock Cargo.toml .
# build creates the binary target/release/example-rust
build:
    FROM +source
    DO rust+CARGO --args="build --release --bin inkjet" --output="release/[^/\.]+"
    RUN strip target/release/inkjet
    SAVE ARTIFACT target/release/inkjet AS LOCAL inkjet
# test executes all unit and integration tests via Cargo
test:
  FROM +source
  DO rust+CARGO --args="test"
