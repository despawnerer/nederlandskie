FROM rust:1-bookworm AS dev
RUN cargo install cargo-watch
WORKDIR /app

FROM rust:1-bookworm AS builder
WORKDIR /build
COPY Cargo.lock ./
COPY Cargo.toml ./
COPY core ./core
COPY processes ./processes
COPY tools ./tools
RUN cargo build --release
RUN mkdir -p /bin && mv target/release/nederlandskie-* /bin/

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates
COPY --from=builder /bin /bin
WORKDIR /bin
EXPOSE 8000
