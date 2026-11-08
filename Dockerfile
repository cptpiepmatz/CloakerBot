# syntax=docker/dockerfile:1

# build shell
FROM --platform=$BUILDPLATFORM hustcer/nushell:0.116.1-alpine AS nushell

# native build environment
FROM --platform=$BUILDPLATFORM rust:bookworm AS build
COPY --from=nushell /usr/bin/nu /usr/local/bin/nu
SHELL ["/usr/local/bin/nu", "--no-config-file", "-c"]
ARG BUILDARCH
ARG TARGETARCH
WORKDIR /cloaker-bot

# install build dependencies
RUN <<EOF
let platform = match $env.TARGETARCH {
    amd64 => { target: x86_64-unknown-linux-gnu, compiler: x86-64-linux-gnu }
    arm64 => { target: aarch64-unknown-linux-gnu, compiler: aarch64-linux-gnu }
    _ => { error make { msg: $"Unsupported target architecture: ($env.TARGETARCH)" } }
}
^apt-get update
^apt-get install -y --no-install-recommends ca-certificates cmake build-essential git
if $env.BUILDARCH != $env.TARGETARCH {
    ^apt-get install -y --no-install-recommends $"gcc-($platform.compiler)" $"g++-($platform.compiler)"
}
rm -rf /var/lib/apt/lists/*
^rustup target add $platform.target
$platform.target | save /rust-target
EOF

# configure cross compilation
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
    CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++ \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

# build cloaker bot
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY .cargo ./.cargo
COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY macros ./macros
COPY assets ./assets
COPY icon ./icon
COPY src ./src
COPY .git ./.git
ARG PROD=0
ARG DISCORD_CLIENT_ID
ARG HTTP_PORT
ARG HTTP_HOST
ARG DISCORD_REDIRECT_URI
ARG BACKUP_INTERVAL_SECONDS
RUN <<EOF
let target = open --raw /rust-target | str trim
if $env.PROD == '1' {
    $env.RUSTFLAGS = $"($env.RUSTFLAGS? | default '') --cfg prod"
}
^cargo build --locked --release --target $target
mkdir /out
cp $"target/($target)/release/CloakerBot" /out/CloakerBot
EOF

FROM debian:bookworm-slim AS run
WORKDIR /cloaker-bot

# copy runtime files without emulation
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /out/CloakerBot ./CloakerBot
EXPOSE 25625
ENTRYPOINT ["./CloakerBot"]
