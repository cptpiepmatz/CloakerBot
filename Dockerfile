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
^apt-get install -y --no-install-recommends ca-certificates cmake build-essential
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
COPY common ./common
COPY macros ./macros
COPY assets ./assets
COPY src ./src
ARG BOT_ENV=dev
RUN <<EOF
if $env.BOT_ENV not-in [prod dev] {
    error make { msg: "BOT_ENV must be prod or dev" }
}
let target = open --raw /rust-target | str trim
^cargo build --locked --release --no-default-features --features $env.BOT_ENV --target $target
mkdir /out
cp $"target/($target)/release/CloakerBot" /out/CloakerBot
EOF

FROM debian:bookworm-slim AS run
WORKDIR /cloaker-bot

# copy runtime files without emulation
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /out/CloakerBot ./CloakerBot
ENTRYPOINT ["./CloakerBot"]
