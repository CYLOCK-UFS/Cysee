FROM rust:1.82-slim-bookworm AS builder

WORKDIR /usr/src/app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    iputils-ping \
    iproute2 \
    netcat-openbsd \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /etc/cysee /var/lib/cysee

COPY --from=builder /usr/src/app/target/release/cysee-central /usr/local/bin/cysee-central

EXPOSE 50051 8080
CMD ["cysee-central"]