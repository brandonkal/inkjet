# Copyright 2020 Brandon Kalinowski (brandonkal)
# SPDX-License-Identifier: MIT

VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.1 AS rust

FROM rust:slim-bookworm
RUN apt-get update && apt-get install -y binutils pkg-config openssl libssl-dev \
    && apt-get install -y --no-install-recommends sudo php python3 ruby curl lcov unzip zip p7zip-full && apt-get clean
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && apt-get install -y nodejs
RUN curl -fsSL https://deno.land/install.sh | sh
ENV YAEGI_VERSION v0.16.1
RUN curl -LO https://github.com/traefik/yaegi/releases/download/${YAEGI_VERSION}/yaegi_${YAEGI_VERSION}_linux_amd64.tar.gz \
    && tar -xzf yaegi_${YAEGI_VERSION}_linux_amd64.tar.gz && sudo mv yaegi /bin/ && rm yaegi_${YAEGI_VERSION}_linux_amd64.tar.gz
ENV DENO_INSTALL="/root/.deno"
ENV PATH="$DENO_INSTALL/bin:$PATH"
RUN rustup toolchain install nightly && rustup component add llvm-tools-preview # for coverage
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install grcov rust-covfix
RUN rustup component add clippy && rustup component add rustfmt # for lint checks
RUN mkdir -p /output
WORKDIR /build

source:
    # CARGO function adds caching to cargo runs.
    # See https://github.com/earthly/lib/tree/main/rust
    DO rust+INIT --keep_fingerprints=true
    COPY --keep-ts --dir src Cargo.lock Cargo.toml .
    COPY --keep-ts inkjet-icon.ico .
    COPY --keep-ts build.rs .
# build creates the binary target/release/inkjet and generates a tar.gz file in /output
build:
    FROM +source
    ARG RUST_TARGET=
    DO rust+CARGO --args="build --release --locked --target x86_64-unknown-linux-gnu --bin inkjet" --output="x86_64-unknown-linux-gnu/release/[^/\.]+"
    ENV RUST_TARGET=
    ENV BINARY=/build/target/x86_64-unknown-linux-gnu/release/inkjet
    RUN strip $BINARY \
        && version=$($BINARY --version | awk '{print $2}') \
        && name=inkjet-v${version}-x86_64-unknown-linux-gnu \
        && tar -czf /output/${name}.tar.gz  -C "$(dirname "$BINARY")" inkjet \
        && shasum -a 256 /output/* > /output/${name}.sha256
    SAVE ARTIFACT /output
build-musl:
    FROM +source
    DO rust+CARGO --args="build --release --locked --target x86_64-unknown-linux-musl --bin inkjet" --output="x86_64-unknown-linux-musl/release/[^/\.]+"
    ENV BINARY=/build/target/x86_64-unknown-linux-musl/release/inkjet
    RUN strip $BINARY \
        && version=$($BINARY --version | awk '{print $2}') \
        && name=inkjet-v${version}-x86_64-unknown-linux-musl \
        && tar -czf /output/${name}.tar.gz -C "$(dirname "$BINARY")" inkjet \
        && shasum -a 256 /output/* > /output/${name}.sha256
    SAVE ARTIFACT /output
# test executes all unit and integration tests via Cargo
test:
    FROM +source
    ENV PATH="/build/target/debug:$PATH"
    COPY --keep-ts inkjet.md .
    COPY --keep-ts tests tests
    DO rust+CARGO --args="test" --output="debug/inkjet"
# fmt checks whether Rust code is formatted according to style guidelines
fmt:
  FROM +source
  DO rust+CARGO --args="fmt --check"
# lint runs cargo clippy on the source code
lint:
    FROM +source
    DO rust+CARGO --args="clippy --all-features --all-targets"
coverage:
    FROM +test
    ARG EARTHLY_GIT_SHORT_HASH
    RUN inkjet cov
    RUN zip -9 /output/inkjet-coverage-$EARTHLY_GIT_SHORT_HASH.zip /build/target/cov/* \
        && mv /build/target/lcov.info /output/inkjet-coverage-$EARTHLY_GIT_SHORT_HASH.lcov.info
    SAVE ARTIFACT /output
# man builds the man page
man:
    FROM pandoc/core:3.2-alpine
    RUN apk add groff util-linux
    COPY README.md man-filter.lua .
    RUN pandoc README.md -s -t man --lua-filter=man-filter.lua -V adjusting=l > inkjet.1
    SAVE ARTIFACT inkjet.1 AS LOCAL ./output/inkjet.1
debian:
    FROM ./packager+base
    COPY --dir .fpm completions .
    COPY +build/output .
    COPY +man/inkjet.1 .
    RUN tar xf *.tar.gz
    RUN fpm -v 0.15.0
    SAVE ARTIFACT *.deb
gather-release:
    LOCALLY
    COPY +man/inkjet.1 ./output/
    RUN ./target/release/inkjet build mac
    COPY +man/inkjet.1 ./output/
    RUN cp README.md ./output/
    COPY +build/output ./output/linux-gnu
    COPY +build-musl/output ./output/linux-musl
    COPY +debian/inkjet_0.15.0_amd64.deb ./output/zips/
    RUN cp ./output/linux*/*.tar.gz ./output/ && rm ./output/zips/*.sha256 && cd ./output/zips && shasum -a 256 * > checksums.sha256
# all runs all targets in parallel
all:
    BUILD +fmt
    BUILD +lint
    BUILD +test
    BUILD +coverage
    BUILD +build
    BUILD +build-musl
    BUILD +man
