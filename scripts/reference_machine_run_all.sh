#!/usr/bin/env bash

set -eEuo pipefail

ECHO_CMD="${ECHO_CMD:-false}"
[ "${ECHO_CMD}" = "true" ] && set -x

# functions

check_root () {
  local extra_msg="$1"
  if [ "$(whoami)" != 'root' ]; then
    echo "$(date --utc +%FT%T.%3NZ) Error: Run this script with 'sudo $0'.${extra_msg}"
    return 1
  fi
}

prereqs () {
  # make sure we have all dependencies
  local INSTALL=""
  ! command -v cpupower &> /dev/null && INSTALL+="linux-tools-common "
  ! command -v jq &> /dev/null && INSTALL+="jq "
  ! command -v lscpu &> /dev/null && INSTALL+="util-linux "
  ! command -v docker &> /dev/null && INSTALL+="docker-buildx-plugin docker-ce docker-ce-cli docker-ce-rootless-extras docker-compose-plugin"
  ! command -v sudo &> /dev/null && INSTALL+="sudo "
  ! command -v mkfs.ext4 &> /dev/null && INSTALL+="e2fsprogs "
  if [ -n "$INSTALL" ]; then
    check_root " We need to install dependencies: ${INSTALL}"
    apt-get update
    # shellcheck disable=SC2086
    DEBIAN_FRONTEND=noniteractive apt-get -y --no-install-recommends install $INSTALL
  fi
  cpuinfo="$(lscpu)"
  if ! [[ "$(hostname)" =~ ^bench.* ]] ||
  [ "$(grep '^CPU family' <<< "${cpuinfo}" | awk '{print $3}')" -ne 25 ] ||
  ! grep -q "Ryzen\|EPYC" <<< "${cpuinfo}" ||
  grep -q hypervisor <<< "${cpuinfo}"; then
    echo "$(date --utc +%FT%T.%3NZ) Error: This script can only be run on the benchmark machine with AMD ZEN4 CPU."
    exit 1
  fi
  IS_BENCHMACHINE="true"
}

set_cpu () {
  # save values currently used
  FREQUENCY="$1" # kHz
  check_root " Elevated permissions required to set fixed CPU clock for reproducible benchmark results."
  FIRSTCPU="$(cut -f1 -d- /sys/devices/system/cpu/online)"
  ORIG_GOVERNOR="$(cat "/sys/devices/system/cpu/cpu$FIRSTCPU/cpufreq/scaling_governor")"
  ORIG_AMD_PSTATE="$(cat /sys/devices/system/cpu/amd_pstate/status)"
  MIN_FREQ="$(cpupower frequency-info -l | tail -n 1 | cut -d " " -f1)"
  MAX_FREQ="$(cpupower frequency-info -l | tail -n 1 | cut -d " " -f2)"
  if [ "${FREQUENCY}" -lt "${MIN_FREQ}" ] || [ "${FREQUENCY}" -gt "${MAX_FREQ}" ]; then
    echo "$(date --utc +%FT%T.%3NZ) Error: Requested frequency $FREQUENCY has to be > $MIN_FREQ and < $MAX_FREQ."
    exit 1
  fi
  echo "$(date --utc +%FT%T.%3NZ) Info:  Setting fixed CPU frequency of ${FREQUENCY}kHz, performance governor and disabling turbo boost."
  echo "passive" > /sys/devices/system/cpu/amd_pstate/status
  cpupower frequency-set -g performance > /dev/null 2>&1
  cpupower frequency-set -d "$FREQUENCY" > /dev/null 2>&1
  cpupower frequency-set -u "$FREQUENCY" > /dev/null 2>&1
  HAVE_SET_CPU="true"
}

restore_cpu () {
  # restore CPU frequency settings
  [ "$HAVE_SET_CPU" = "false" ] && return 0
  check_root " Elevated permissons required to reset CPU clock to default settings."
  echo "$(date --utc +%FT%T.%3NZ) Info:  Restoring CPU frequency settings of MIN_FREQ: ${MIN_FREQ}kHz, MAX_FREQ: ${MAX_FREQ}kHz, governor: $ORIG_GOVERNOR and enabling turbo boost."
  echo "$ORIG_AMD_PSTATE" > /sys/devices/system/cpu/amd_pstate/status
  cpupower frequency-set -g "$ORIG_GOVERNOR" > /dev/null 2>&1
  cpupower frequency-set -d "$MIN_FREQ" > /dev/null 2>&1
  cpupower frequency-set -u "$MAX_FREQ" > /dev/null 2>&1
}

setup_disk () {
  local file="$1"
  local mountpoint="$2"
  local USER_ID="$3"
  local GROUP_ID="$4"
  # create a simulated ext4 formatted block device in memory
  dd if=/dev/zero of="${file?err_unset}" bs=1024 count="$((1024**2*10))" && IMG_CREATED="true"
  LOOP_DEV="$(losetup -f)"
  losetup --sector-size=4096 --direct-io=on "${LOOP_DEV}" "${file}" && LOOP_CREATED="true"
  mkfs.ext4 -O ^has_journal -E nodiscard "${LOOP_DEV}"
  mount -odefaults,noatime "${LOOP_DEV}" "${mountpoint?err_unset}" && DEVICE_MOUNTED="true"
  chown -R "${USER_ID}:${GROUP_ID}" "${mountpoint}"
}

cleanup_disk () {
  local file="$1"
  local mountpoint="$2"
  [ "${DEVICE_MOUNTED}" = "true" ] && umount "${mountpoint?err_unset}"
  [ "${LOOP_CREATED}" = "true" ] && losetup -d "${LOOP_DEV}"
  [ "${IMG_CREATED}" = "true" ] && rm -f "${file?err_unset}"
  [ "${MOUNT_IS_TMP}" ] && rm -rf "${mountpoint?err_unset}"
}

check_root ""
prereqs

# performance profiles
declare -A cpu_profiles
declare -A io_profiles
CPU_PROFILE="${CPU_PROFILE:-aws.c7a.2xlarge}"
IO_PROFILE="${IO_PROFILE:-aws.ebs.io2_8000}"

# frequency in kHz
cpu_profiles["aws.c7a.2xlarge"]="3500000"
cpu_profiles["unconfined"]="5389000"
io_profiles["aws.ebs.io2_8000"]='{"read_io":450,"write_io":450,"read_bps":1457520640,"write_bps":1457520640}'
io_profiles["unconfined"]='{"read_io":1000000000,"write_io":1000000000,"read_bps":53687091200,"write_bps":53687091200}'

# config
HAVE_SET_CPU="false"
IMG_CREATED="false"
LOOP_CREATED="false"
DEVICE_MOUNTED="false"
MOUNT_IS_TMP="false"
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." &> /dev/null && pwd)"
USER="$(stat -c '%U' "${ROOT_DIR}")"
USER_ID="$(stat -c '%u' "${ROOT_DIR}")"
GROUP_ID="$(stat -c '%g' "${ROOT_DIR}")"
READ_IO="$(jq -rc '.read_io' <<< "${io_profiles[${IO_PROFILE}]}")"
WRITE_IO="$(jq -rc '.write_io' <<< "${io_profiles[${IO_PROFILE}]}")"
READ_BPS="$(jq -rc '.read_bps' <<< "${io_profiles[${IO_PROFILE}]}")"
WRITE_BPS="$(jq -rc '.write_bps' <<< "${io_profiles[${IO_PROFILE}]}")"
BENCH_BASE_PATH="$(mktemp -d)"
grep -q "^/tmp/tmp\..*$" <<< "${BENCH_BASE_PATH}" && MOUNT_IS_TMP="true"
EXT4_IMG="/dev/shm/ext4.img"
USE_DOCKER="true"
ENABLE_PALLETS="${ENABLE_PALLETS:-true}"
ENABLE_OVERHEAD="${ENABLE_OVERHEAD:-true}"
ENABLE_MACHINE="${ENABLE_MACHINE:-true}"
# The space separated pallet list to benchmark (empty means all). Use the inent
# version like `pallet_aggregate` and not the one with `-` 
PALLETS="${PALLETS:-}"

export IS_BENCHMACHINE READ_IO WRITE_IO READ_BPS WRITE_BPS BENCH_BASE_PATH LOOP_DEV USE_DOCKER ENABLE_PALLETS ECHO_CMD ROOT_DIR

# add exit handler to restore machine to base settings on exit or error
exit_handler () {
  restore_cpu
  cleanup_disk "${EXT4_IMG}" "${BENCH_BASE_PATH}"
}

trap exit_handler EXIT

# run benchmark
setup_disk "${EXT4_IMG}" "${BENCH_BASE_PATH}" "${USER_ID}" "${GROUP_ID}"
set_cpu "${cpu_profiles[${CPU_PROFILE}]}"
sudo --preserve-env=IS_BENCHMACHINE,READ_IO,WRITE_IO,READ_BPS,WRITE_BPS,BENCH_BASE_PATH,LOOP_DEV,USE_DOCKER,ENABLE_PALLETS,ENABLE_OVERHEAD,ENABLE_MACHINE,PALLETS,ECHO_CMD,ROOT_DIR \
  -u "${USER}" bash -c 'cd "${ROOT_DIR}"; "${ROOT_DIR}/scripts/run_all_benchmarks.sh"'
