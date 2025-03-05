FROM ubuntu:24.04

SHELL ["/bin/bash", "-c"]

# That can be a single one or a comma separated list
ARG BINARY="zkv-node"
ARG DESCRIPTION="zkVerify Development"
ARG AUTHORS="infrastructure@zkverify.io"
ARG VENDOR="zkVerify"

ENV BINARY="${BINARY}" \
    RUN_USER="user"

LABEL io.image.authors="${AUTHORS}" \
      io.image.vendor="${VENDOR}" \
      io.image.description="${DESCRIPTION}"

USER root
WORKDIR /app

COPY "bin/*" "/usr/local/bin/"
RUN chmod -R a+rx "/usr/local/bin"

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

COPY entrypoint.sh .
RUN chmod +x entrypoint.sh

# ENTRYPOINT
ENTRYPOINT ["/app/entrypoint.sh"]
