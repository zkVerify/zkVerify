FROM rust:1-bookworm as builder

RUN apt-get update -qq \
  && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  protobuf-compiler \
  cmake \
  clang \
  lld \
  && rustup target add wasm32-unknown-unknown \
  && rustup component add rust-src \
  && apt-get -y clean \
  && apt-get -y autoclean \
  && apt-get -y autoremove \
  && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*.deb

ARG PROFILE="release"
ARG FEATURES=""

WORKDIR /usr/src/node
COPY . .

RUN echo "SUBSTRATE_CLI_GIT_COMMIT_HASH=`git rev-parse --short=11 HEAD`" >> .docker.env
RUN . ./.docker.env \
  && cargo build -p mainchain --profile ${PROFILE} --features "${FEATURES}"

FROM ubuntu:22.04 as node

SHELL ["/bin/bash", "-c"]

# That can be a single one or a comma separated list
ARG BINARY=zkv-node
ARG DESCRIPTION="zkVerify Core"
ARG AUTHORS="mainchain-team@horizenlabs.io"
ARG VENDOR="Horizen Labs"
ARG PROFILE="release"
ARG FEATURES=""

ENV BINARY="${BINARY}" \
  RUN_USER=hl

LABEL io.hl.image.authors="${AUTHORS}" \
  io.hl.image.vendor="${VENDOR}" \
  io.hl.image.description="${DESCRIPTION}" \
  io.hl.image.profile="${PROFILE}" \
  io.hl.image.features="${FEATURES}"

USER root
WORKDIR /app

COPY docker/scripts/entrypoint.sh .
COPY docker/scripts/remove_password_file.sh .
COPY --from=builder "/usr/src/node/target/${PROFILE}/zkv-node" "/usr/local/bin/"
COPY --from=builder "/usr/src/node/target/${PROFILE}/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm" "./zkv_runtime.compact.compressed.wasm"
RUN chmod -R a+rx "/usr/local/bin"

RUN apt-get update \
  && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  aria2 \
  ca-certificates \
  curl \
  jq \
  && useradd -m -U -s /bin/bash -d /${RUN_USER} ${RUN_USER} \
  && mkdir -p /data /${RUN_USER}/.local/share \
  && chown -R ${RUN_USER}:${RUN_USER} /data /${RUN_USER} \
  && ln -s /data /${RUN_USER}/.local/share \
  && apt-get -y clean \
  && apt-get -y autoclean \
  && apt-get -y autoremove \
  && rm -rf /var/{lib/apt/lists/*,cache/apt/archives/*.deb} /tmp/*

USER ${RUN_USER}

# ENTRYPOINT
ENTRYPOINT ["/app/entrypoint.sh"]
