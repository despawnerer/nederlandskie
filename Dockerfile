FROM rust:1.72

RUN USER=root cargo new --bin nederlandskie
WORKDIR /nederlandskie

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/nederlandskie*
RUN cargo build --release

EXPOSE 3000
CMD ["./target/release/nederlandskie"]
