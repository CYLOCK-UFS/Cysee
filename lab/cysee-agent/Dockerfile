# =============================================================================
# ESTÁGIO 1: Builder (Ambiente de compilação oficial do Rust)
# =============================================================================
FROM rust:1.82-slim-bookworm AS builder

WORKDIR /usr/src/app
COPY Cargo.toml ./
COPY src ./src

# Compila o binário otimizado de produção
RUN cargo build --release

# =============================================================================
# ESTÁGIO 2: Runtime (Imagem enxuta apenas com o binário e ferramentas de rede)
# =============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    iputils-ping \
    iproute2 \
    iptables \
    netcat-openbsd \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cria diretórios padronizados pela Arquitetura v1.3 (/etc/cysee e /var/lib/cysee)
RUN mkdir -p /etc/cysee /var/lib/cysee

# Copia APENAS o binário compilado no Estágio 1
COPY --from=builder /usr/src/app/target/release/cysee-agent /usr/local/bin/cysee-agent

CMD ["cysee-agent"]