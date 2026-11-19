# syntax=docker/dockerfile:1

FROM rust:bookworm AS dependencies
WORKDIR /cloaker-bot

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates cmake build-essential git pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY infra/Cargo.toml ./infra/Cargo.toml
COPY macros/Cargo.toml ./macros/Cargo.toml
COPY macros/proc/Cargo.toml ./macros/proc/Cargo.toml
COPY .cargo ./.cargo

# compile external dependencies using placeholders, without copying application sources
RUN mkdir -p src infra/src macros/src macros/proc/src \
    && printf 'fn main() {}\n' > src/main.rs \
    && printf 'fn main() {}\n' > build.rs \
    && touch infra/src/lib.rs macros/src/lib.rs macros/proc/src/lib.rs \
    && cargo build --locked --release --workspace \
    && cargo clean --release --workspace

FROM dependencies AS macros
COPY macros ./macros
RUN cargo build --locked --release -p cloaker-bot-macros

FROM macros AS infra
COPY infra ./infra
RUN cargo build --locked --release -p cloaker-bot-infra

FROM infra AS build
COPY build.rs ./
COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY assets ./assets
COPY icon ./icon
COPY src ./src
COPY .git ./.git

# these arguments affect only the bot, leaving the preceding build layers cached
ARG PROD=0
ARG DISCORD_CLIENT_ID
ARG HTTP_PORT
ARG HTTP_HOST
ARG DISCORD_REDIRECT_URI
ARG BACKUP_INTERVAL_SECONDS
RUN set -eu; \
    set --; \
    if [ "$PROD" = "1" ]; then set -- --cfg prod; fi; \
    cargo rustc --locked --release -p cloaker-bot --bin CloakerBot -- "$@"

FROM debian:bookworm-slim AS run
WORKDIR /cloaker-bot

# assets and migrations are embedded, the bot creates data/ at runtime
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /cloaker-bot/target/release/CloakerBot ./CloakerBot
EXPOSE 25625
ENTRYPOINT ["./CloakerBot"]
