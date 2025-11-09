# Use official Rust image for building
FROM rust:1.91-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --color=always --profile release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates openssl curl

# Create non-root user
RUN addgroup -S app && adduser -S app -G app

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/epovistka /app/epovistka

# Change to non-root user
USER app

# Expose port (adjust if your app uses a different port)
EXPOSE 3000

# Health check - make sure this matches your actual health endpoint
HEALTHCHECK --interval=3s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

# Run the application
CMD ["/app/epovistka"]