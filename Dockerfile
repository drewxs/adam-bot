FROM rust:1.75 as builder

RUN apt-get update && apt-get install -y cmake

WORKDIR /bot
COPY . .
RUN cargo build --release


FROM python:3.12-slim

RUN apt-get update && \
  apt-get install -y curl ca-certificates ffmpeg libopus-dev
RUN pip3 install yt-dlp

WORKDIR /bot
COPY --from=builder /bot/target/release/adam .

CMD ["./adam"]
