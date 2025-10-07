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

# Runtime stage - Debian 12 distroless
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy the binary from builder
COPY --from=builder /app/target/release/kickstart /usr/local/bin/kickstart

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/kickstart"]
