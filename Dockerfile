FROM rust:1.83-slim-bullseye AS builder

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
       pkg-config \
       libclang-dev clang \
       zlib1g-dev \
       coinor-cbc coinor-libcbc-dev \
  && rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# runtime stage
FROM debian:bullseye-slim

WORKDIR /ftqmp

COPY --from=builder /app/target/release/qmp_scheduler /usr/local/bin/qmp_scheduler
COPY examples ./examples

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
       coinor-cbc coinor-libcbc-dev \
  && rm -rf /var/lib/apt/lists/*

ENTRYPOINT [ "/bin/bash" ]

