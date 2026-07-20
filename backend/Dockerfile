FROM rust:1.88-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY backend/ ./backend/
RUN cargo build --release -p api_server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install --no-install-recommends -y libssl3 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api_server /usr/local/bin/api_server
EXPOSE 8080
CMD ["api_server"]
