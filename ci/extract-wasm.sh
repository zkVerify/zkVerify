#!/bin/bash
# --image argument and its value are required

set -eEuo pipefail

workdir="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )"
common_file_location="${COMMON_FILE_LOCATION:-ci/common.sh}"
image=""
image_artifact=""

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
    --image)
      if [ -z "${2:-}" ]; then  # Check if the value for --image is empty
        fn_die "ERROR: --image argument's value CAN NOT be empty !!!"
      fi
      echo "Option --image argument has value: ${2}"
      image="${2}"
      shift 2 ;;  # Skip both the option and its value
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

# Check if just one of image and image_artifact is provided and not empty
if (( (!!${#image}) == (!!${#image_artifact}) )); then
  fn_die "ERROR: You should provide at least and just one of --image or --image-artifact"
fi


####
# Main
####
cd "${workdir}"

if [ -n "${image_artifact}" ]; then
  if [ "${DRY_RUN}" != "true" ]; then
    image="$(docker load -i "${GITHUB_WORKSPACE}/${image_artifact}.tar" | awk '/Loaded image:/ { print $3 }')"
  else
    log_info "GITHUB_WORKSPACE=${GITHUB_WORKSPACE} image_artifact=${image_artifact}"
    image="__from-artifact--dry-run__"
  fi
fi

log_info "=== Extract runtime artifacts ==="
if [ "${DRY_RUN}" != "true" ]; then
  # It is not possible to get files from container using wildcards. We should run the container to get the list
  # and then copy them.
  container_id="$(docker create "${image}")"
  for rt in $(docker run --rm --entrypoint "ls" "${image}" /app | grep -E ".compact.compressed.wasm$");
  do
    docker cp "${container_id}:/app/${rt}" .
  done
else
  log_warn "WARNING: 'DRY_RUN' variable is set to 'true'. CREATE FAKE WASMS"
  echo "${image} -> zkv_runtime WASM" > ./zkv_runtime.compact.compressed.wasm
  echo "${image} -> volta_runtime WASM" > ./volta_runtime.compact.compressed.wasm
fi

exit 0
