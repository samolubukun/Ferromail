# Builder stage
FROM rust:1.88-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY db ./db
COPY smtp ./smtp
COPY api ./api
COPY worker ./worker

RUN cargo build --release --workspace

# Final API stage
FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/api /app/api
EXPOSE 3000
CMD ["/app/api"]

# Final Worker stage
FROM debian:bookworm-slim AS worker
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/worker /app/worker
CMD ["/app/worker"]

# Final SMTP stage
FROM debian:bookworm-slim AS smtp
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/smtp /app/smtp
EXPOSE 465 587
CMD ["/app/smtp"]
