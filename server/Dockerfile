FROM rust:bullseye AS builder
WORKDIR /usr/leaderboard

COPY . /usr/leaderboard/

RUN cd /usr/leaderboard/src

RUN cargo build --release --features=testing

FROM debian:bullseye-slim AS runtime

WORKDIR /usr/leaderboard/prod

RUN apt-get update && apt-get install openssl ca-certificates -y && \ 
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/leaderboard/target/release/piss-fan /usr/local/bin/piss-fan

COPY --from=builder /usr/leaderboard/entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 3000/tcp

