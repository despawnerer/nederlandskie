# FIXME: Use mariner once they support the latest Rust.
# FROM mcr.microsoft.com/cbl-mariner/base/rust:1 as builder
FROM rust:1-bullseye AS builder

COPY Cargo.lock /build/
COPY Cargo.toml /build/
COPY src /build/src

# Build the default page
WORKDIR /build

RUN cargo build --release
RUN mkdir -p /app && mv target/release/nederlandskie /app/

# FROM mcr.microsoft.com/cbl-mariner/distroless/base:2.0
FROM debian:bullseye-slim

COPY --from=builder /app /app

WORKDIR /app
EXPOSE 8000

ENTRYPOINT [ "/app/nederlandskie" ]
