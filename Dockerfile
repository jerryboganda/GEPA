# Multi-stage production build for GEPA v2 (Cloud Run Container Deployment)
# Reference: docs/02_ARCHITECTURE.md §1 & §8

# Stage 1: Build Astro Client
FROM node:20-alpine AS client-builder
WORKDIR /app/client
COPY client/package*.json ./
RUN npm ci
COPY client/ ./
RUN npm run build

# Stage 2: Build Rust Backend Server
FROM rust:1.80-bullseye AS server-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY tools ./tools
COPY server ./server
RUN cargo build --release --bin server

# Stage 3: Minimal Runtime Image
FROM debian:bullseye-slim AS runner
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries and assets
COPY --from=server-builder /app/target/release/server /app/server
COPY --from=client-builder /app/client/dist /app/client/dist
COPY seed /app/seed
COPY assets /app/assets
# Migrations (server/migrations/001_init.sql) are embedded into the binary at
# compile time via `include_str!` — nothing to copy for them at runtime.

# Configure runtime environment
ENV PORT=8080 \
    CLIENT_DIST=/app/client/dist \
    SEED_DIR=/app/seed \
    RUST_LOG=server=info,tower_http=info

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1

ENTRYPOINT ["/app/server"]
