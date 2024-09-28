FROM --platform=$BUILDPLATFORM rust:latest AS build
WORKDIR /cloaker-bot

# update container
RUN apt update -y && apt upgrade -y
RUN apt install -y cmake build-essential gcc-aarch64-linux-gnu
RUN rustup up
RUN rustup target add aarch64-unknown-linux-gnu

# configure cargo
ENV CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# build dependencies
RUN cargo init --bin
COPY Cargo.toml Cargo.lock ./
RUN cargo build

# build cloaker bot
COPY dev.token .
COPY ./assets ./assets
COPY ./src ./src
RUN touch ./src/main.rs
RUN cargo build


FROM debian:bookworm-slim AS run
WORKDIR /cloaker-bot

# update container
RUN apt update -y && apt upgrade -y
RUN apt install -y ca-certificates

COPY --from=build /cloaker-bot/target/aarch64-unknown-linux-gnu/debug/CloakerBot .
ENTRYPOINT [ "./CloakerBot" ]
