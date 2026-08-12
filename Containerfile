# ── Stage 1: Build ────────────────────────────────────────────────────────────
# Uses the official Rust image with musl target for a fully static binary.
# No dynamic linking means the final image can be FROM scratch.
FROM rust:1-slim AS builder

RUN rustup target add x86_64-unknown-linux-musl \
    && apt-get update -qq \
    && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ── Cache dependencies ────────────────────────────────────────────────────────
# Copy only the manifests first so dependency compilation is cached separately
# from source changes.
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml crates/api/Cargo.toml
COPY crates/frontend/Cargo.toml crates/frontend/Cargo.toml

# Create dummy source files so cargo can resolve the dependency graph
RUN mkdir -p crates/api/src crates/frontend/src \
    && echo '' > crates/api/src/lib.rs \
    && echo 'fn main() {}' > crates/frontend/src/main.rs \
    && cargo build --release --target x86_64-unknown-linux-musl -p opr-frontend 2>/dev/null || true \
    && rm -rf crates/

# ── Build the real source ─────────────────────────────────────────────────────
COPY . .

# Touch source files to invalidate the dummy build cache
RUN touch crates/api/src/lib.rs crates/frontend/src/main.rs

RUN cargo build --release --target x86_64-unknown-linux-musl -p opr-frontend

# ── Stage 2: Scratch ─────────────────────────────────────────────────────────
# Only the static binary plus the static UI assets it serves. The API is
# fully in-process (no database or outbound network calls), so no CA
# certificates are required.
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/opr-frontend /opr-frontend
COPY --from=builder /app/crates/frontend/static /static

# The binary falls back to compile-time paths / loopback when these are
# unset, so they must be baked in for the container to serve correctly.
ENV BIND_ADDR=0.0.0.0:3000 \
    STATIC_DIR=/static

EXPOSE 3000

ENTRYPOINT ["/opr-frontend"]