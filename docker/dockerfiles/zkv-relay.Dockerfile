FROM rust:1-bookworm AS builder

RUN apt-get update -qq \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      clang \
      lld \
      protobuf-compiler \
    && apt-get -y clean \
    && apt-get -y autoclean \
    && apt-get -y autoremove \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*.deb

# Add Rust targets and components
RUN rustup target add wasm32-unknown-unknown && \
    rustup component add rust-src

ARG PROFILE="release"
ARG FEATURES=""

WORKDIR /usr/src/node
COPY . .

RUN cargo build -p zkv-relay --profile "${PROFILE}" --features "${FEATURES}"

FROM ubuntu:24.04 AS node

SHELL ["/bin/bash", "-c"]

ARG BINARY="zkv-relay"
ARG DESCRIPTION="zkVerify Relay"
ARG AUTHORS="infrastructure@zkverify.io"
ARG VENDOR="zkVerify"
ARG PROFILE="release"
ARG FEATURES=""

ENV BINARY="${BINARY}" \
    RUN_USER="user"

LABEL io.image.authors="${AUTHORS}" \
      io.image.vendor="${VENDOR}" \
      io.image.description="${DESCRIPTION}" \
      io.image.profile="${PROFILE}" \
      io.image.features="${FEATURES}"

USER root
WORKDIR /app

RUN apt-get update -qq \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      aria2 \
      ca-certificates \
      curl \
      jq \
      gosu \
    && useradd -m -U -s /bin/bash -d "/${RUN_USER}" "${RUN_USER}" \
    && mkdir -p /data "/${RUN_USER}/.local/share" \
    && chown -R "${RUN_USER}:${RUN_USER}" /data "/${RUN_USER}" \
    && ln -s /data "/${RUN_USER}/.local/share" \
    && apt-get -y clean \
    && apt-get -y autoclean \
    && apt-get -y autoremove \
    && rm -rf /var/{lib/apt/lists/*,cache/apt/archives/*.deb} /tmp/*

COPY --from=builder "/usr/src/node/target/${PROFILE}/${BINARY}" "/usr/local/bin/"
COPY --from=builder "/usr/src/node/target/${PROFILE}/${BINARY}-execute-worker" "/usr/local/bin/"
COPY --from=builder "/usr/src/node/target/${PROFILE}/${BINARY}-prepare-worker" "/usr/local/bin/"
COPY --from=builder "/usr/src/node/target/${PROFILE}/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm" "./zkv_runtime.compact.compressed.wasm"
COPY --from=builder "/usr/src/node/target/${PROFILE}/wbuild/volta-runtime/volta_runtime.compact.compressed.wasm" "./volta_runtime.compact.compressed.wasm"
RUN chmod -R a+rx "/usr/local/bin"

COPY docker/scripts/entrypoint.sh .
RUN chmod +x entrypoint.sh

# ENTRYPOINT
ENTRYPOINT ["/app/entrypoint.sh"]
