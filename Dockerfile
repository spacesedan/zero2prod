FROM rust:1.66.1

WORKDIR /app

COPY . .

RUN cargo build --release

ENTRYPOINT [ "./target/release/zero2prod" ]
