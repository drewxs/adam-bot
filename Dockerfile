FROM rust:1.74 as builder

WORKDIR /usr/src
RUN apt-get update && apt-get upgrade -y \
  && apt-get install -y cmake ca-certificates libssl-dev

WORKDIR /usr/src/bot
COPY . .
RUN cargo build --release

FROM debian:sid-slim

WORKDIR /usr/src/bot
COPY --from=builder /usr/src/bot/target/release/adam .

CMD ["./adam"]
