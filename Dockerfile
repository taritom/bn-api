FROM rust:1.28

WORKDIR /usr/src/bn-api
ADD api ./api/
ADD db ./db/
ADD Cargo.lock Cargo.toml ./

RUN cargo build --release
WORKDIR /usr/src/bn-api/db
RUN cargo install
WORKDIR /usr/src/bn-api/api
RUN cargo install

CMD ["server"]