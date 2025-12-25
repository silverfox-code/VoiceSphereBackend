FROM rust:1.83-slim-bookworm

WORKDIR /usr/src/app

# Install system dependencies for ScyllaDB driver (if needed) and build tools
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests first for caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build
RUN rm -rf src

# Copy source code
COPY . .

EXPOSE 8080

# Use cargo run for development (recompiles on restart)
CMD ["cargo", "run"]
