#!/bin/bash
# --image-artifact argument and its value are required

set -eEuo pipefail

workdir="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )"
docker_image_build_name="${DOCKER_IMAGE_BUILD_NAME:-zkverify}"
docker_hub_org="${DOCKER_HUB_ORG:-horizenlabs}"
docker_hub_username="${DOCKER_HUB_USERNAME:-}"
docker_hub_token="${DOCKER_HUB_TOKEN:-}"
is_a_release="${IS_A_RELEASE:-false}"
prod_release="${PROD_RELEASE:-false}"
dev_release="${DEV_RELEASE:-false}"
test_release="${TEST_RELEASE:-false}"
fastruntime_release="${FAST_RUNTIME_RELEASE:-false}"
github_ref_name="${GITHUB_REF_NAME:-}"
common_file_location="${COMMON_FILE_LOCATION:-not-set}"
image_artifact=""
docker_tag_full=""
extract_runtime="${EXTRACT_RUNTIME:-true}"

# Requirement
if ! [ -f "${common_file_location}" ]; then
  echo -e "\n\033[1;31mERROR: ${common_file_location} file is missing !!!  Exiting ...\033[0m\n"
  exit 1
else
  # shellcheck disable=SC1090
  source "${common_file_location}"
fi

log_info "DRY_RUN: '${DRY_RUN}'"

# Check for command-line options
while [ $# -gt 0 ]; do
  case "${1:-}" in
    --image-artifact)
      if [ -z "${2:-}" ]; then  # Check if the value for --image-artifact is empty
        fn_die "ERROR: --image-artifact argument's value CAN NOT be empty !!!"
      fi
      echo "Option --image-artifact argument has value: ${2}"
      image_artifact="${2}"
      shift 2 ;;  # Skip both the option and its value
    *) shift ;;  # Shift for any other options
  esac
done

# Check if image_artifact is still empty (i.e., the option was not provided or had no value)
if [ -z "${image_artifact}" ]; then
  fn_die "ERROR: --image-artifact is a required argument"
fi

####
# Main
####
cd "${workdir}"

if [ -z "${docker_hub_token:-}" ]; then
  fn_die "ERROR: DOCKER_HUB_TOKEN variable is not set. Exiting ..."
fi

if [ -z "${docker_hub_username:-}" ]; then
  fn_die "ERROR: DOCKER_HUB_USERNAME variable is not set. Exiting ..."
fi

# Load docker image
if [ "${is_a_release}" = "true" ]; then
  docker_tag_full="${github_ref_name}"

  log_info "=== Using Docker image artifact from upstream ==="
  if [ "${DRY_RUN}" != "true" ]; then
    image_name="$(docker load -i "${GITHUB_WORKSPACE}/${image_artifact}.tar" | awk '/Loaded image:/ { print $3 }')"
  else
    log_info "GITHUB_WORKSPACE=${GITHUB_WORKSPACE} image_artifact=${image_artifact}"
    image_name="__dry_run__"
  fi
  log_info "=== Loaded already built image '${image_name}' ==="

  # Publishing to DockerHub
  log_info "=== Publishing Docker image(s) on Docker Hub ==="
  echo "${docker_hub_token}" | docker login -u "${docker_hub_username}" --password-stdin

  # Docker image(s) tags for PROD vs DEV release
  if [ "${prod_release}" = "true" ]; then
    docker_tag_node="$(cut -d '-' -f1 <<< "${docker_tag_full}")"
    publish_tags=("${docker_tag_full}" "${docker_tag_node}" "latest")
  elif [ "${dev_release}" = "true" ]; then
    docker_tag_node="$(cut -d '-' -f1 <<< "${docker_tag_full}")-$(cut -d '-' -f3- <<< "${docker_tag_full}")"
    publish_tags=("${docker_tag_full}" "${docker_tag_node}")
  elif [ "${test_release}" = "true" ]; then
    publish_tags=("${docker_tag_full}")
  elif [ "${fastruntime_release}" = "true" ]; then
    publish_tags=("fast-runtime")
    extract_runtime="false"
  fi

  # Append -relay to tag names for relay chain images
  if [[ "${image_artifact}" == "${docker_image_build_name}-relay" ]]; then
    docker_tag_full="${docker_tag_full}-relay"
    for publish_tag in "${!publish_tags[@]}"; do
      publish_tags["${publish_tag}"]="${publish_tags["${publish_tag}"]}-relay"
    done
  fi

  for publish_tag in "${publish_tags[@]}"; do
    log_info "Publishing docker image: ${docker_image_build_name}:${publish_tag}"
    if [ "${DRY_RUN}" != "true" ]; then
      docker tag "${image_name}" "index.docker.io/${docker_hub_org}/${docker_image_build_name}:${publish_tag}"
      docker push "index.docker.io/${docker_hub_org}/${docker_image_build_name}:${publish_tag}"
    else
      log_warn "WARNING: 'DRY_RUN' variable is set to 'true'. Don't PUBLISH"
      log_info "image_name='${image_name}' docker_hub_org='${docker_hub_org}' docker_image_build_name='${docker_image_build_name}' publish_tag='${publish_tag}'"
    fi
  done

  # Extract runtime artifact
  if [ "${extract_runtime}" == "true" ]; then
    log_info "=== Extract runtime artifact ==="
    if [ "${DRY_RUN}" != "true" ]; then
      container_id="$(docker create "index.docker.io/${docker_hub_org}/${docker_image_build_name}:${docker_tag_full}")"
      docker cp "${container_id}":/app/zkv_runtime.compact.compressed.wasm ./zkv_runtime.compact.compressed.wasm
      docker rm "${container_id}"  # Clean up the container
    else
      log_warn "WARNING: 'DRY_RUN' variable is set to 'true'. CREATE FAKE WASM"
      echo "index.docker.io/${docker_hub_org}/${docker_image_build_name}:${docker_tag_full} -> WASM" > ./zkv_runtime.compact.compressed.wasm
    fi
  else
    log_info "=== Skipping runtime artifact extraction ==="
  fi
else
  fn_die "ERROR: the build did NOT satisfy RELEASE build requirements(IS_A_RELEASE variable=${is_a_release}). Docker image(s) was(were) NOT published."
fi

exit 0
