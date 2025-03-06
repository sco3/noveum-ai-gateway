# Build stage
FROM --platform=linux/amd64 rust:1.82-slim-bookworm as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy only necessary files first
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the real source code
COPY src ./src

# Build the application (removed static linking flag)
RUN cargo build --release && \
    strip target/release/noveum-ai-gateway

# Runtime stage
FROM --platform=linux/amd64 debian:bookworm-slim

# Add LABEL to identify the image
LABEL org.opencontainers.image.source="https://github.com/noveum-ai/ai-gateway"
LABEL org.opencontainers.image.description="Noveum AI Gateway"
LABEL org.opencontainers.image.version="latest"

# Install runtime dependencies including ca-certificates for HTTPS connections
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/noveum-ai-gateway /usr/local/bin/

# Set the startup command
CMD ["noveum-ai-gateway"] 