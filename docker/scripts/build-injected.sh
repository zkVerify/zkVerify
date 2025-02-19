#!/usr/bin/env bash
set -e

# This script allows building a Container Image starting from a
# base image (define in DOCKERFILE env variable) that inject 
# a list of Linux binaries where the first will be the entrypoint.

ENGINE=${ENGINE:-docker}

if [ "$ENGINE" == "podman" ]; then
  ENGINE_FLAGS="--format docker"
else
  ENGINE_FLAGS=""
fi


CONTEXT=$(mktemp -d)
REGISTRY=${REGISTRY:-docker.io}

# The following line ensure we know the project root
PROJECT_ROOT=${PROJECT_ROOT:-$(git rev-parse --show-toplevel)}
DOCKERFILE=${DOCKERFILE:-docker/dockerfiles/binary_injected.Dockerfile}
VERSION_TOML=$(grep "^version " "${PROJECT_ROOT}/node/Cargo.toml" | grep -oE "([0-9\.]+-?[0-9]+)")

#n The following VAR have default that can be overridden
DOCKER_OWNER=${DOCKER_OWNER:-horizenlabs}

# We may get 1..n binaries, comma separated
BINARY=${BINARY:-zkv-node}
IFS=',' read -r -a BINARIES <<< "$BINARY"

VERSION=${VERSION:-$VERSION_TOML}
ARTIFACTS_FOLDER=${ARTIFACTS_FOLDER:-.}
SPEC_FOLDER=${SPEC_FOLDER:-${ARTIFACTS_FOLDER}/specs}

IMAGE=${IMAGE:-${REGISTRY}/${DOCKER_OWNER}/zkverify}
DESCRIPTION_DEFAULT="Injected Container image built for ${BINARY}"
DESCRIPTION=${DESCRIPTION:-$DESCRIPTION_DEFAULT}

# Build the image
echo "Using engine: $ENGINE"
echo "Using Dockerfile: $DOCKERFILE"
echo "Using context: $CONTEXT"
echo "Building ${IMAGE}:latest container image for ${BINARY} v${VERSION} from ${ARTIFACTS_FOLDER} hang on!"

# We need all binaries and resources available in the Container build "CONTEXT"
mkdir -p "${CONTEXT}/bin"
for bin in "${BINARIES[@]}"
do
  echo "Copying $ARTIFACTS_FOLDER/$bin to context: $CONTEXT/bin"
  ls -al "$ARTIFACTS_FOLDER/$bin"
  cp -r "$ARTIFACTS_FOLDER/$bin" "${CONTEXT}/bin"
done

cp "$PROJECT_ROOT/docker/scripts/entrypoint.sh" "${CONTEXT}"

echo "Building image: ${IMAGE}"

#shellcheck disable=SC2124
TAGS=${TAGS[@]:-latest}
IFS=',' read -r -a TAG_ARRAY <<< "$TAGS"
TAG_ARGS=" "

echo "The image ${IMAGE} will be tagged with ${TAG_ARRAY[*]}"
for tag in "${TAG_ARRAY[@]}"; do
  TAG_ARGS+="--tag ${IMAGE}:${tag} "
done

echo "TAG_ARGS: $TAG_ARGS"

IMAGES_PRE=$(docker image ls -q -f 'dangling=true')

# time \
# shellcheck disable=SC2086
$ENGINE build \
    ${ENGINE_FLAGS} \
    --build-arg BINARY="${BINARY}" \
    --build-arg DESCRIPTION="${DESCRIPTION}" \
    ${TAG_ARGS} \
    -f "${PROJECT_ROOT}/${DOCKERFILE}" \
    "${CONTEXT}"

# cleaning images made dangling due to build
IMAGES_POST=$(docker image ls -q -f 'dangling=true')
for IMAGE_POST in ${IMAGES_POST}; do
  if ! echo "${IMAGES_PRE[*]}" | grep -qw $IMAGE_POST; then
    docker image rm "${IMAGE_POST}"
  fi
done

echo "Your Container image for ${IMAGE} is ready"
$ENGINE images

if [[ -z "${SKIP_IMAGE_VALIDATION}" ]]; then
  echo "Check the image ${IMAGE}:${TAG_ARRAY[0]}"
  $ENGINE run --rm --entrypoint "${BINARIES[0]}" -i "${IMAGE}:${TAG_ARRAY[0]}" --version

  echo "Query binaries"
  $ENGINE run --rm -i --entrypoint /bin/bash "${IMAGE}:${TAG_ARRAY[0]}" -c "echo BINARY: ${BINARY}"
fi
