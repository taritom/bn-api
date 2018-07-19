FROM rust:1.26

WORKDIR /usr/src/bn-api
ADD Cargo.toml Cargo.lock ./
ADD tests tests/
ADD src src/

RUN cargo build --release
RUN cargo install

CMD ["server"]