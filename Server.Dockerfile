FROM lukemathwalker/cargo-chef:latest-rust-1.69.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim AS runtime
WORKDIR /app
# OpenSSL is dynamically linked by some of our dependencies
# ca-certificates is needed to verify TLS certificates when establishing HTTPS connections
RUN apt-get update -y && apt-get install -y --no-install-recommends openssl ca-certificates
# Clean up
RUN apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY settings.* .
ENV RUN_MODE production
ENTRYPOINT ["./zero2prod"]
