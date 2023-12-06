FROM rust:latest

WORKDIR /usr/src/adam
COPY . .

RUN cargo install --path .

CMD ["adam"]
