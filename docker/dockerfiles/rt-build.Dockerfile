ARG PROFILE="release"
ARG FEATURES=""
ARG RUNTIME_NAME=""

FROM rust:1-bookworm AS builder

# Add Rust targets and components
RUN rustup target add wasm32-unknown-unknown && \
    rustup component add rust-src

WORKDIR /usr/src/node
COPY . .

ARG PROFILE
ARG FEATURES
ARG RUNTIME_NAME

RUN cargo build -p zkv-runtime --profile "${PROFILE}" --features "${FEATURES}"

FROM alpine:latest AS runtime

ARG PROFILE
ARG FEATURES
ARG RUNTIME_NAME

ARG DESCRIPTION="zkVerify Runtime"
ARG AUTHORS="infrastructure@zkverify.io"
ARG VENDOR="zkVerify"

LABEL io.image.authors="${AUTHORS}" \
      io.image.vendor="${VENDOR}" \
      io.image.description="${DESCRIPTION} - ${RUNTIME_NAME}" \
      io.image.profile="${PROFILE}" \
      io.image.features="${FEATURES}"

WORKDIR /app

COPY --from=builder "/usr/src/node/target/${PROFILE}/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm" "."
