FROM rust:1.83-slim AS builder

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
       pkg-config \
       libclang-dev clang \
       zlib1g-dev \
       coinor-cbc coinor-libcbc-dev

COPY rust-toolchain.toml ./

# For build cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() { println!(\"hello\"); }" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN cargo build --release

# runtime stage
FROM alpine:latest

COPY --from=builder /app/target/release/qmp_scheduler /usr/local/bin/qmp_scheduler

ENTRYPOINT ["/usr/local/bin/qmp_scheduler"]

