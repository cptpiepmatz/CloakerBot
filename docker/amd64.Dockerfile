FROM rust:latest AS build
WORKDIR /cloaker-bot

# update container
RUN apt update -y && apt upgrade -y
RUN apt install -y cmake build-essential
RUN rustup up

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

COPY --from=build /cloaker-bot/target/debug/CloakerBot .
ENTRYPOINT [ "./CloakerBot" ]
