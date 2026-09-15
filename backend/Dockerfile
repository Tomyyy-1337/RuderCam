# syntax=docker/dockerfile:1.7

ARG BUILD_PLATFORM=linux/amd64
FROM --platform=${BUILD_PLATFORM} rust:latest AS builder

RUN dpkg --add-architecture arm64 && \
    apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    pkg-config \
    libudev-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-gnu

ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
ENV AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_SYSROOT_DIR=/usr/aarch64-linux-gnu
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

# ENV RUSTFLAGS="-C target-cpu=cortex-a53"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    RUSTFLAGS="-D warnings" cargo build --release --target aarch64-unknown-linux-gnu && \
    cp /app/target/aarch64-unknown-linux-gnu/release/backend /tmp/backend

FROM alpine:latest
RUN apk add --no-cache bash
COPY --from=builder /tmp/backend /backend

# Use: docker build -t pi-backend . ; docker create --name temp pi-backend ; docker cp temp:/backend ./backend_bin ; docker rm temp