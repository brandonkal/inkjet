# Copyright 2020 Brandon Kalinowski (brandonkal)
# SPDX-License-Identifier: MIT

VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.1 AS rust

FROM rust:slim-bookworm
RUN apt-get update && apt-get install -y binutils pkg-config openssl libssl-dev sudo php python3 ruby curl lcov && apt-get clean
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && apt-get install -y nodejs
RUN curl -fsSL https://deno.land/install.sh | sh
ENV YAEGI_VERSION v0.16.1
RUN curl -LO https://github.com/traefik/yaegi/releases/download/${YAEGI_VERSION}/yaegi_${YAEGI_VERSION}_linux_amd64.tar.gz \
    && tar -xzf yaegi_${YAEGI_VERSION}_linux_amd64.tar.gz && sudo mv yaegi /bin/ && rm yaegi_${YAEGI_VERSION}_linux_amd64.tar.gz
ENV DENO_INSTALL="/root/.deno"
ENV PATH="$DENO_INSTALL/bin:$PATH"
RUN rustup toolchain install nightly && rustup component add llvm-tools-preview # for coverage
RUN cargo install grcov rust-covfix
WORKDIR /build

source:
    # CARGO function adds caching to cargo runs.
    # See https://github.com/earthly/lib/tree/main/rust
    DO rust+INIT --keep_fingerprints=true
    COPY --keep-ts --dir src Cargo.lock Cargo.toml .
    COPY inkjet-icon.ico .
    COPY build.rs .
# build creates the binary target/release/example-rust
build:
    FROM +source
    DO rust+CARGO --args="build --release --bin inkjet" --output="release/[^/\.]+"
    RUN strip target/release/inkjet
    SAVE ARTIFACT target/release/inkjet AS LOCAL inkjet
# test executes all unit and integration tests via Cargo
test:
    FROM +source
    ENV PATH="/build/target/debug:$PATH"
    COPY inkjet.md .
    COPY tests tests
    DO rust+CARGO --args="test" --output="debug/inkjet"
coverage:
    FROM +test
    ARG EARTHLY_GIT_SHORT_HASH
    RUN inkjet cov
    RUN mkdir -p /output && zip -9 /output/coverage-inkjet-$EARTHLY_GIT_SHORT_HASH.zip /build/target/cov/* && \
        mv /build/target/lcov.info /output/coverage-inkjet-$EARTHLY_GIT_SHORT_HASH.lcov.info
    SAVE ARTIFACT /output
# all runs all targets in parallel
all:
    BUILD +test
    BUILD +coverage
    BUILD +build
