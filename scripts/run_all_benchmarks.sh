#!/usr/bin/env bash

# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# - Pallet benchmarking to update the pallet weights
# - Overhead benchmarking for the Extrinsic and Block weights
# - Machine benchmarking
#
# Should be run on a reference machine to gain accurate benchmarks

set -eEuo pipefail

ECHO_CMD="${ECHO_CMD:-false}"
[ "${ECHO_CMD}" = "true" ] && set -x

# The following line ensures we know the project root
SOURCE_ROOT="${SOURCE_ROOT:-$(git rev-parse --show-toplevel)}"

# shellcheck source=/dev/null
source "${SOURCE_ROOT}/scripts/bench_cfg.sh"

USE_DOCKER="${USE_DOCKER:-false}"
ENABLE_PALLETS="${ENABLE_PALLETS:-false}"
# Default all available pallets
PALLETS="${PALLETS:-}"
ENABLE_OVERHEAD="${ENABLE_OVERHEAD:-true}"
ENABLE_MACHINE="${ENABLE_MACHINE:-true}"
BENCH_BASE_PATH="${BENCH_BASE_PATH:-}"
BASE_PATH_ARG=""
[ -n "${BENCH_BASE_PATH:-}" ] && BASE_PATH_ARG="--base-path=${BENCH_BASE_PATH}"

BENCH_SH="${BENCH_SH:-${SOURCE_ROOT}/scripts/bench.sh}"
# Define the error file.
ERR_FILE="${ERR_FILE:-${SOURCE_ROOT}/benchmarking_errors.txt}"

RUNTIME_CRATE="zkv-runtime"


if [ "${USE_DOCKER}" = "false" ]; then
  echo "[+] Compiling zkv-relay benchmarks..."
  cargo build \
    --locked \
    --features=runtime-benchmarks \
    --profile=production \
    --bin zkv-relay

  # The executable to use.
  ZKV_NODE="${PROJECT_ROOT}/target/production/zkv-relay"
  ZKV_WASM="${PROJECT_ROOT}/target/production/wbuild/${RUNTIME_CRATE}/${RUNTIME_CRATE//-/_}.compact.compressed.wasm"
else
  IMAGE="zkverify"
  TAG="$(git rev-parse --short HEAD)"
  dirty="$(git status --porcelain --untracked-files=no | grep "Cargo\|docker/\|native/\|node/\|pallets/\|primitives/\|rpc/\|verifiers/\|runtime/" | grep -v "runtime/src/weights" || true)"
  build="false"
  [ -n "${dirty:-}" ] && build="true" && TAG="${TAG}-dirty"
  [ -z "$(docker image ls -q "${IMAGE}:${TAG}")" ] && build="true"
  compose_file="${SOURCE_ROOT}/scripts/docker-compose-local.yml"
  [ "${IS_BENCHMACHINE:-}" = "true" ] && compose_file="${SOURCE_ROOT}/scripts/docker-compose-bench.yml"
  USER_ID="$(id -u)"
  GROUP_ID="$(id -g)"
  export IMAGE TAG SOURCE_ROOT USER_ID GROUP_ID ECHO_CMD

  if [ "${build}" = "true" ]; then
    echo -e "[+] Building ${IMAGE}:${TAG} runtime-benchmarks docker image...\n\n"
    docker compose --progress=plain -f "${compose_file}" build
    docker image prune -f
  fi
  # The executable to use.
  ZKV_NODE="docker compose -f ${compose_file} run -T --rm --remove-orphans zkverify-bench /usr/local/bin/zkv-relay"
  ZKV_WASM="/app/${RUNTIME_CRATE//-/_}.compact.compressed.wasm"

  # Now PROJECT_ROOT become the docker folder
  PROJECT_ROOT="/data/benchmark"
fi

DEFAULT_DEPLOY_WEIGHT_TEMPLATE="${PROJECT_ROOT}/relay-node/benchmarks/zkv-deploy-weight-template.hbs"
DEFAULT_DEPLOY_WEIGHT_TEMPLATE_XCM="${PROJECT_ROOT}/relay-node/benchmarks/zkv-deploy-weight-template-xcm.hbs"

WEIGTH_TEMPLATE="${WEIGTH_TEMPLATE:-${DEFAULT_DEPLOY_WEIGHT_TEMPLATE}}"
WEIGTH_TEMPLATE_XCM="${WEIGTH_TEMPLATE_XCM:-${DEFAULT_DEPLOY_WEIGHT_TEMPLATE_XCM}}"

WEIGHTS_FOLDER="${WEIGHTS_FOLDER:-${PROJECT_ROOT}/runtime/zkverify/src/weights}"

CODE_HEADER="${PROJECT_ROOT}/HEADER-APACHE2"

declare -a SELECTED_PALLETS=()
if [ "${ENABLE_PALLETS:-}" = "true" ]; then
  if [ -n "${PALLETS}" ] ; then 
    echo "PALLETS='${PALLETS}'"
    mapfile -t SELECTED_PALLETS < <(for p in ${PALLETS}; do echo "${p}"; done | sort | uniq)
  else
    # get all
    mapfile -t SELECTED_PALLETS < <(${ZKV_NODE} benchmark pallet \
      --list \
      --runtime "${ZKV_WASM}" \
      --genesis-builder=runtime | \
      cut -d',' -f1 | \
      sort | \
      uniq \
      )
    fi
fi

EXCLUDED_PALLETS=(
        "frame_benchmarking"
        # Pallets without automatic benchmarking
        "pallet_babe" "pallet_grandpa"
        "pallet_offences"

        # SLOW
        # "pallet_im_online" "frame_benchmarking" "frame_system" "pallet_balances" "pallet_staking"

        # Manually disabled to keep artificial weights
        "parachains::on_demand"
    )

echo "[+] Benchmarking ${#SELECTED_PALLETS[@]} zkVerify pallets. (IGNORE SET [${#EXCLUDED_PALLETS[@]}])"


is_pallet_excluded() {
  local pallet=$1;

  for exluded in "${EXCLUDED_PALLETS[@]}"; do
      if [ "${exluded}" = "${pallet}" ]; then
          return 0
      fi
  done

  return 1
}

# Delete the error file before each run.
rm -f "${ERR_FILE:?err_unset}"

# Benchmark each pallet.
for PALLET in "${SELECTED_PALLETS[@]}"; do
  if [ "${PALLET}" = "pallet" ]; then
    # Skip header
    continue
  fi

  if is_pallet_excluded "${PALLET}"; then
    echo "[+] Skipping - $PALLET"
    continue
  fi

  PALLET_NAME="$(tr '_' '-' <<< "${PALLET}")"
  MODULE_NAME=$(echo "${PALLET}.rs" | sed 's/ :: /\//g');
  WEIGHT_FILE="${WEIGHTS_FOLDER}/${MODULE_NAME}"
  echo "[+] Benchmarking $PALLET with weight file $WEIGHT_FILE"

  TEMPLATE="${WEIGTH_TEMPLATE}"
  # XCM pallet-xcm-benchmarks benchmarks need a different template
  if [[ "${PALLET}" == *pallet_xcm_benchmarks_* ]] ; then
    TEMPLATE="${WEIGTH_TEMPLATE_XCM}"
  fi

  OUTPUT="$( \
    SOURCE_ROOT="${SOURCE_ROOT}" \
    WEIGTH_OUT_PATH="${WEIGHT_FILE}" \
    SKIP_BUILD="true" \
    ZKV_NODE_EXE="${ZKV_NODE}" \
    ZKV_RUNTIME="${ZKV_WASM}" \
    WEIGTH_TEMPLATE="${TEMPLATE}" \
    CODE_HEADER="${CODE_HEADER}" \
    BM_STEPS="${BM_STEPS}" \
    BM_REPEAT="${BM_REPEAT}" \
    BM_HEAP_PAGES="${BM_HEAP_PAGES}" \
    BASE_PATH_ARG="${BASE_PATH_ARG}" \
    "${BENCH_SH}" "${PALLET_NAME}" 2>&1
  )" || {
    echo "$OUTPUT" >> "$ERR_FILE";
    echo "[-] Failed to benchmark $PALLET. Error written to $ERR_FILE; continuing...";
  }
done

if [ "${ENABLE_OVERHEAD:-}" = "true" ]; then
  # Update the block and extrinsic overhead weights.
  echo "[+] Benchmarking block and extrinsic overheads..."
  # shellcheck disable=SC2086
  OUTPUT="$(
    ${ZKV_NODE} benchmark overhead \
    --chain=dev \
    --weight-path="${WEIGHTS_FOLDER}" \
    --header="${CODE_HEADER}" \
    --warmup=10 \
    --repeat=100 \
    ${BASE_PATH_ARG} 2>&1
  )" || {
    echo "$OUTPUT" >> "$ERR_FILE";
    echo "[-] Failed to benchmark the block and extrinsic overheads. Error written to $ERR_FILE; continuing...";
  }
else
  echo "[+] Skipping - Benchmarking block and extrinsic overheads..."
fi

if [ "${ENABLE_MACHINE:-}" = "true" ]; then
  echo "[+] Benchmarking the machine..."
  # shellcheck disable=SC2086
  OUTPUT="$(
    ${ZKV_NODE} benchmark machine --chain=dev ${BASE_PATH_ARG} 2>&1
  )" || {
    # Do not write the error to the error file since it is not a benchmarking error.
    echo -e "[-] Failed the machine benchmark:\n";
  }
  echo "${OUTPUT}"
else
  echo "[+] Skipping - Benchmarking the machine..."
fi

# Check if the error file exists.
if [ -f "$ERR_FILE" ]; then
  echo "[-] Some benchmarks failed. See: $ERR_FILE"
  exit 1
else
  echo "[+] All benchmarks passed."
  exit 0
fi
