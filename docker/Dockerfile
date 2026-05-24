FROM rust:latest AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release && rm -f target/release/deps/dataharvest*
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/dataharvest .
COPY --from=builder /app/migrations ./migrations
EXPOSE 4000
CMD ["./dataharvest"]
