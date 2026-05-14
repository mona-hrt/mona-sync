# Build stage
FROM rust:1.81-slim-bullseye as builder

WORKDIR /usr/src/mona-sync
COPY . .

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/mona-sync/target/release/mona_backend .
COPY --from=builder /usr/src/mona-sync/index.html .
COPY --from=builder /usr/src/mona-sync/migrations ./migrations

# Default environment variables
ENV DATABASE_URL=sqlite://data/database.db
ENV SERVER_IP=0.0.0.0
ENV SERVER_PORT=3000
ENV RUST_LOG=mona_backend=info

RUN mkdir data

EXPOSE 3000

CMD ["./mona_backend"]
