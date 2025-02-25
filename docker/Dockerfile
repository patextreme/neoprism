FROM node:20 AS assets-builder
COPY . /workspace
WORKDIR /workspace
RUN npm install
RUN npm install -D tailwindcss@3.4.17
WORKDIR /workspace/prism-node
RUN npx tailwindcss -i tailwind.css -o ./assets/tailwind.css

FROM rust:1.84 AS builder
RUN rustup default nightly-2025-02-24
RUN apt update -y && apt install -y protobuf-compiler
WORKDIR /workspace
# dummy lib with real dependencies for docker caching
COPY Cargo.toml /workspace/Cargo.toml
COPY prism-core/Cargo.toml /workspace/prism-core/Cargo.toml
COPY prism-node/Cargo.toml /workspace/prism-node/Cargo.toml
COPY prism-storage/Cargo.toml /workspace/prism-storage/Cargo.toml
COPY prism-migration/Cargo.toml /workspace/prism-migration/Cargo.toml
RUN mkdir -p prism-core/src && touch prism-core/src/lib.rs && \
  mkdir -p prism-node/src && touch prism-node/src/lib.rs && \
  mkdir -p prism-storage/src && touch prism-storage/src/lib.rs && \
  mkdir -p prism-migration/src && touch prism-migration/src/lib.rs
RUN cargo b
# build the actual thing
COPY . /workspace
RUN cargo clean
RUN cargo b -r

FROM debian:bookworm
COPY --from=builder /workspace/target/release/prism-node /usr/local/bin/prism-node
COPY --from=assets-builder /workspace/prism-node/assets /workspace/assets
ENTRYPOINT prism-node

