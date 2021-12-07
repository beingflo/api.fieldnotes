# Builder stage
FROM rust:1.57.0 AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build

# Runtime stage
FROM rust:1.57.0-slim AS runtime
WORKDIR /app
# Copy the compiled binary from the builder environment # to our runtime environment
COPY --from=builder /app/target/debug/fieldnotes-api fieldnotes-api
COPY .env .env
ENTRYPOINT ["./fieldnotes-api"]