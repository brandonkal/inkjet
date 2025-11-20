# Copyright 2020 Brandon Kalinowski (brandonkal)
# SPDX-License-Identifier: MIT

VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.1 AS rust

source:
    FROM rust:1.91.1-slim-bullseye
    RUN apt-get update && apt-get install -y binutils pkg-config openssl libssl-dev \
        && apt-get install -y --no-install-recommends sudo php python3 ruby curl lcov unzip zip p7zip-full && apt-get clean
    # Install cross-compilation tools
    RUN apt-get update && apt-get install -y \
        gcc-aarch64-linux-gnu \
        libc6-dev-arm64-cross \
        && apt-get clean
    RUN curl -fsSL https://deb.nodesource.com/setup_24.x | bash - && apt-get install -y nodejs
    RUN curl -fsSL https://deno.land/install.sh | sh
    ENV YAEGI_VERSION v0.16.1
    # Detect architecture and download appropriate yaegi binary
    RUN ARCH=$(dpkg --print-architecture) && \
        if [ "$ARCH" = "amd64" ]; then YAEGI_ARCH="amd64"; \
        elif [ "$ARCH" = "arm64" ]; then YAEGI_ARCH="arm64"; \
        else echo "Unsupported architecture: $ARCH" && exit 1; fi && \
        curl -LO https://github.com/traefik/yaegi/releases/download/${YAEGI_VERSION}/yaegi_${YAEGI_VERSION}_linux_${YAEGI_ARCH}.tar.gz \
        && tar -xzf yaegi_${YAEGI_VERSION}_linux_${YAEGI_ARCH}.tar.gz && sudo mv yaegi /bin/ && rm yaegi_${YAEGI_VERSION}_linux_${YAEGI_ARCH}.tar.gz
    ENV DENO_INSTALL="/root/.deno"
    ENV PATH="$DENO_INSTALL/bin:$PATH"
    RUN rustup component add llvm-tools # for coverage
    RUN rustup target add x86_64-unknown-linux-musl
    RUN rustup target add aarch64-unknown-linux-gnu
    RUN rustup target add aarch64-unknown-linux-musl
    RUN cargo install grcov
    RUN rustup component add clippy && rustup component add rustfmt # for lint checks
    # Configure cross-compilation for both GNU and musl targets
    ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    ENV AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
    # For musl targets, use the ARM64 GCC as linker driver (it will link against musl)
    ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
    RUN mkdir -p /output
    WORKDIR /build
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
    COPY version.txt .
    DO rust+CARGO --args="build --release --locked --target x86_64-unknown-linux-gnu --bin inkjet" --output="x86_64-unknown-linux-gnu/release/[^/\.]+"
    ENV RUST_TARGET=
    ENV BINARY=/build/target/x86_64-unknown-linux-gnu/release/inkjet
    RUN strip $BINARY \
        && version=$(cat version.txt) \
        && name=inkjet-v${version}-x86_64-unknown-linux-gnu \
        && tar -czf /output/${name}.tar.gz  -C "$(dirname "$BINARY")" inkjet \
        && shasum -a 256 /output/* > /output/${name}.sha256
    SAVE ARTIFACT /output
build-musl:
    FROM +source
    COPY version.txt .
    DO rust+CARGO --args="build --release --locked --target x86_64-unknown-linux-musl --bin inkjet" --output="x86_64-unknown-linux-musl/release/[^/\.]+"
    ENV BINARY=/build/target/x86_64-unknown-linux-musl/release/inkjet
    RUN strip $BINARY \
        && version=$(cat version.txt) \
        && name=inkjet-v${version}-x86_64-unknown-linux-musl \
        && tar -czf /output/${name}.tar.gz -C "$(dirname "$BINARY")" inkjet \
        && shasum -a 256 /output/* > /output/${name}.sha256
    SAVE ARTIFACT /output
build-arm64:
    FROM +source
    COPY version.txt .
    DO rust+CARGO --args="build --release --locked --target aarch64-unknown-linux-gnu --bin inkjet" --output="aarch64-unknown-linux-gnu/release/[^/\.]+"
    ENV BINARY=/build/target/aarch64-unknown-linux-gnu/release/inkjet
    RUN aarch64-linux-gnu-strip $BINARY \
        && version=$(cat version.txt) \
        && name=inkjet-v${version}-aarch64-unknown-linux-gnu \
        && tar -czf /output/${name}.tar.gz -C "$(dirname "$BINARY")" inkjet \
        && shasum -a 256 /output/* > /output/${name}.sha256
    SAVE ARTIFACT /output
build-arm64-musl:
    FROM +source
    COPY version.txt .
    DO rust+CARGO --args="build --release --locked --target aarch64-unknown-linux-musl --bin inkjet" --output="aarch64-unknown-linux-musl/release/[^/\.]+"
    ENV BINARY=/build/target/aarch64-unknown-linux-musl/release/inkjet
    RUN aarch64-linux-gnu-strip $BINARY \
        && version=$(cat version.txt) \
        && name=inkjet-v${version}-aarch64-unknown-linux-musl \
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
    ENV RUSTFLAGS="-Cinstrument-coverage"
    RUN cargo build
    ENV LLVM_PROFILE_FILE="inkjet-%p-%m.profraw"
    RUN cargo test
    RUN grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
    RUN zip -9 /output/inkjet-coverage-$EARTHLY_GIT_SHORT_HASH.zip /build/target/debug/coverage/*
    SAVE ARTIFACT /output
# man builds the man page
man:
    FROM pandoc/core:3.7-alpine
    RUN apk add groff util-linux
    COPY README.md man-filter.lua .
    RUN pandoc README.md -s -t man --lua-filter=man-filter.lua -V adjusting=l > inkjet.1
    SAVE ARTIFACT inkjet.1 AS LOCAL ./generated/inkjet.1
debian-amd64:
    FROM ./packager+base
    COPY --dir .fpm completions .
    COPY +build-musl/output .
    COPY +man/inkjet.1 .
    COPY version.txt .
    RUN tar xf *.tar.gz \
        && version=$(cat version.txt) \
        && fpm -v ${version} --architecture amd64 \
        && mkdir /output && mv *.deb /output
    SAVE ARTIFACT /output
debian-arm64:
    FROM ./packager+base
    COPY --dir .fpm completions .
    COPY +build-arm64-musl/output .
    COPY +man/inkjet.1 .
    COPY version.txt .
    RUN tar xf *.tar.gz \
        && version=$(cat version.txt) \
        && fpm -v ${version} --architecture arm64 \
        && mkdir /output && mv *.deb /output
    SAVE ARTIFACT /output
gather-release:
    LOCALLY
    COPY +man/inkjet.1 ./output/
    RUN inkjet build mac
    COPY +man/inkjet.1 ./output/
    COPY +build/output ./output/linux-gnu
    COPY +build-musl/output ./output/linux-musl
    COPY +build-arm64/output ./output/linux-arm64-gnu
    COPY +build-arm64-musl/output ./output/linux-arm64-musl
    COPY +debian-amd64/output ./output/deb
    RUN mv ./output/deb/*.deb ./output/zips
    COPY +debian-arm64/output ./output/deb
    RUN mv ./output/deb/*.deb ./output/zips
    RUN cp ./output/*.7z ./output/zips/
    RUN cp ./output/linux*/*.tar.gz ./output/zips/ && rm ./output/zips/*.sha256.txt || true && cd ./output/zips && shasum -a 256 * > checksums.sha256.txt
# all runs all targets in parallel
all:
    BUILD +fmt
    BUILD +lint
    BUILD +test
    BUILD +coverage
    BUILD +build
    BUILD +build-musl
    BUILD +build-arm64
    BUILD +build-arm64-musl
    BUILD +debian-amd64
    BUILD +debian-arm64
    BUILD +man
