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
# Floating "1" tag (not a pinned minor version) so this build always matches
# a current stable toolchain, the same way CI's own build does via
# dtolnay/rust-toolchain@stable — a hardcoded 1.80 here is what broke once a
# transitive dependency (hmac v0.13.0) started requiring Cargo's edition2024
# feature, unstable before Rust 1.85. bookworm (not bullseye) to match the
# runner stage below — bullseye is now old enough that its security mirror
# has started pruning individual package files out from under still-listed
# index entries, not just serving a stale-but-otherwise-fine Release file.
FROM rust:1-bookworm AS server-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY tools ./tools
COPY server ./server
RUN cargo build --release --bin server

# Stage 3: Minimal Runtime Image
FROM debian:bookworm-slim AS runner
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
