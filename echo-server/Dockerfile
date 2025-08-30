# ==========================
# --- Builder Stage ---
# ==========================
FROM rust:1.89.0-bookworm AS builder

# Set working directory
WORKDIR /app

# Install build-time dependencies
RUN apt-get update && \
    apt-get install -y \
        libpq-dev pkg-config \
        libavutil-dev libavcodec-dev libavformat-dev \
        libswscale-dev libavfilter-dev libavdevice-dev \
        clang llvm make cmake build-essential && \
    rm -rf /var/lib/apt/lists/*

# --- Cache dependencies ---
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

# --- Copy full source and build release binary ---
COPY . .
RUN cargo build --release

# ==========================
# --- Production Stage ---
# ==========================
FROM debian:bookworm-slim AS prod

# Set working directory
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libpq5 \
        libmariadb3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /app/target/release/echo /app/echo

# Copy migrations, info, and env file
COPY migrations /app/migrations
COPY info /app/info
COPY .env /app/.env

# Create a non-root user for security
RUN useradd -m appuser
USER appuser

# Expose port and run
EXPOSE 8080
CMD ["/app/echo"]

# ==========================
# --- Development Stage ---
# ==========================
FROM rust:1.89.0 AS dev

WORKDIR /app

# Install dev dependencies
RUN apt-get update && \
    apt-get install -y \
       libpq-dev pkg-config \
       libavutil-dev libavcodec-dev libavformat-dev \
       libswscale-dev libavfilter-dev libavdevice-dev \
       clang llvm make cmake build-essential && \
    rm -rf /var/lib/apt/lists/*

# Copy full source code
COPY . .

# Install cargo-watch for hot reloading
RUN cargo install cargo-watch

# Copy env
COPY .env /app/.env

# Expose port
EXPOSE 8080

# Start dev mode with live reload
CMD ["cargo", "watch", "-x", "run"]
