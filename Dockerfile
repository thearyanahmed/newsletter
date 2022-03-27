FROM rust:1.59.0

WORKDIR /app

RUN apt update && apt install lld clang -y

# should use volume
COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release

ENTRYPOINT ["./target/release/newsletter"]