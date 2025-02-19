#!/bin/sh

PROJECT_ROOT="${PROJECT_ROOT:-$(git rev-parse --show-toplevel)}"

IMAGE="horizenlabs/zkv-relay" \
BINARY="zkv-relay,zkv-relay-execute-worker,zkv-relay-prepare-worker" \
ARTIFACTS_FOLDER="target/release" \
"${PROJECT_ROOT}"/docker/scripts/build-injected.sh
