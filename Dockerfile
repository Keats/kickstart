# Build stage
FROM rust:1.87 AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY rustfmt.toml ./

# Copy source code
COPY src ./src
COPY examples ./examples

# Build the application with CLI features
RUN cargo build --release --features cli

# Runtime stage - Debian 12 slim
FROM debian:12-slim

# Install ca-certificates
RUN apt-get update && \
    apt-get install -y ca-certificates git && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/kickstart /usr/local/bin/kickstart

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/kickstart"]
