#!/bin/sh
set -e

PROJECT_ROOT=${PROJECT_ROOT:-$(git rev-parse --show-toplevel)}
SCRIPTS=${PROJECT_ROOT}/docker/scripts
DOCKERS=${PROJECT_ROOT}/docker/dockerfiles
CARGO=${SCRIPTS}/my_cargo
BUILD_PROFILE="${BUILD_PROFILE:---release}"

# Features
FAST_RUNTIME="${FAST_RUNTIME:-true}"                    # for dev, limit an epoch to 1min. Useful for testing with parachains

# Build rbuilder
echo "----------------------------------------------------------"
echo "Building rbuilder"
docker build -f ${DOCKERS}/zkv-builder.Dockerfile -t rbuilder ${PROJECT_ROOT}

RELAY_FEATURES=""

if [ "$FAST_RUNTIME" = "true" ]; then
  RELAY_FEATURES="fast-runtime"
fi

if [ "$RELAY_FEATURES" ]; then
  RELAY_FEATURES="--features ${RELAY_FEATURES}"
fi

echo "----------------------------------------------------------"
echo "Compile relay node"
${CARGO} build -p zkv-relay "${BUILD_PROFILE}" "${RELAY_FEATURES}"

echo "----------------------------------------------------------"
echo "Compile test parachain node"
${CARGO} build -p paratest-node "${BUILD_PROFILE}"

# Create docker images
echo "----------------------------------------------------------"
echo "Building relay node image"
"${SCRIPTS}/build-zkv-relay-image-injected.sh"

echo "----------------------------------------------------------"
echo "Building parachain node image"
"${SCRIPTS}/build-paratest-image-injected.sh"
