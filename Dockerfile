FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml ./
COPY crates ./crates

RUN cargo build --release --package llm-gateway-cli --bin llm-gateway

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install --yes --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/llm-gateway /usr/local/bin/llm-gateway

EXPOSE 4141

ENTRYPOINT ["/usr/local/bin/llm-gateway"]
CMD ["serve", "--host", "0.0.0.0", "--allow-remote"]
