FROM rust:1.43-slim

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --all-features

EXPOSE 3003

CMD [ "./target/release/mobile-api" ]
