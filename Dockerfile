# FIXME: Use mariner once they support the latest Rust.
# FROM mcr.microsoft.com/cbl-mariner/base/rust:1 as builder
FROM rust:1-bullseye AS builder

COPY Cargo.lock /build/
COPY Cargo.toml /build/
COPY core /build/core
COPY processes /build/processes
COPY tools /build/tools

# Build the default page
WORKDIR /build

RUN cargo build
RUN mkdir -p /bin && mv target/debug/nederlandskie-* /bin/

# FROM mcr.microsoft.com/cbl-mariner/distroless/base:2.0
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates
RUN update-ca-certificates

COPY --from=builder /bin /bin
COPY .env /bin

WORKDIR /bin
EXPOSE 8000
