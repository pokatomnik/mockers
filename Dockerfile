# syntax=docker/dockerfile:1

FROM rust:1.88-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY doc ./doc

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/mocks

COPY --from=builder /app/target/release/mockers /usr/local/bin/mockers

EXPOSE 8080
EXPOSE 8443

ENTRYPOINT ["/usr/local/bin/mockers"]
