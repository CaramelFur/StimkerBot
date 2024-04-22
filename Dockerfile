FROM rust:1.77.2-slim-bookworm as builder

RUN apt update && apt install -y pkg-config libssl-dev

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt update && apt install -y libssl3 ca-certificates && update-ca-certificates

VOLUME [ "/data" ]
ENV DATABASE_LOCATION=/data/sticker_bot.db
ENV TELOXIDE_TOKEN=
ENV RUST_LOG=info

COPY --from=builder /usr/src/app/target/release/sticker_bot /usr/local/bin/sticker_bot

CMD ["sticker_bot"]

