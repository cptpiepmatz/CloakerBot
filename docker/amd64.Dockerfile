FROM rust:latest AS build
WORKDIR /cloaker-bot

# update container
RUN apt update -y && apt upgrade -y
RUN apt install -y cmake build-essential
RUN rustup up

# build cloaker bot
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY .cargo ./.cargo
COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY common ./common
COPY macros ./macros
COPY ./assets ./assets
COPY ./src ./src
RUN cargo build --locked


FROM debian:bookworm-slim AS run
WORKDIR /cloaker-bot

# update container
RUN apt update -y && apt upgrade -y
RUN apt install -y ca-certificates

COPY --from=build /cloaker-bot/target/debug/CloakerBot .
ENTRYPOINT [ "./CloakerBot" ]
