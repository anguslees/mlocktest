ARG CROSSVER=0.2.1

FROM --platform=$BUILDPLATFORM rustembedded/cross:aarch64-unknown-linux-musl-${CROSSVER} AS build-linux-arm64
ENV RUST_TARGET=aarch64-unknown-linux-musl

FROM --platform=$BUILDPLATFORM rustembedded/cross:x86_64-unknown-linux-musl-${CROSSVER} AS build-linux-amd64
ENV RUST_TARGET=x86_64-unknown-linux-musl

FROM --platform=$BUILDPLATFORM rustembedded/cross:x86_64-pc-windows-gnu-${CROSSVER} AS build-windows-amd64
ENV RUST_TARGET=x86_64-pc-windows-gnu

FROM --platform=$BUILDPLATFORM rustembedded/cross:arm-unknown-linux-musleabi-${CROSSVER} AS build-linux-armv6
ENV RUST_TARGET=arm-unknown-linux-musleabi

FROM --platform=$BUILDPLATFORM rustembedded/cross:armv7-unknown-linux-musleabihf-${CROSSVER} AS build-linux-armv7
ENV RUST_TARGET=armv7-unknown-linux-musleabihf

FROM --platform=$BUILDPLATFORM build-${TARGETOS}-${TARGETARCH}${TARGETVARIANT} AS crossbuild

ENV \
        CARGO_HOME=/usr/local/cargo \
        PATH=/usr/local/cargo/bin:$PATH

ARG RUSTVER=1.48.0

# Ideally, rustembedded images would build on a shared base that
# had these pre-installed...
# https://github.com/rust-embedded/cross/issues/468
RUN curl https://sh.rustup.rs -o rustup-init.sh
RUN sh ./rustup-init.sh -y --profile=minimal --default-toolchain ${RUSTVER} --target ${RUST_TARGET}

FROM --platform=$BUILDPLATFORM crossbuild AS build

WORKDIR /usr/src/mlocktest
COPY . .
RUN cargo install --target=${RUST_TARGET} --path .

FROM --platform=$TARGETPLATFORM alpine:3.15@sha256:4edbd2beb5f78b1014028f4fbb99f3237d9561100b6881aabbf5acce2c4f9454

COPY --from=build /usr/local/cargo/bin/ /usr/local/bin/

CMD ["mlocktest"]
