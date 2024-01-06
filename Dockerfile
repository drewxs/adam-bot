FROM rust:1.74 as builder

RUN apt-get update && apt-get install -y cmake 

WORKDIR /bot
COPY . .
RUN cargo build --release


FROM debian:sid-slim

RUN apt-get update && apt-get install -y curl ca-certificates ffmpeg libopus-dev

WORKDIR /bot
COPY --from=builder /bot/target/release/adam .

CMD ["./adam"]
