#!/usr/bin/env bash

set -eEuo pipefail

ECHO_CMD="${ECHO_CMD:-false}"
[ "${ECHO_CMD}" = "true" ] && set -x

# functions

check_root() {
  local extra_msg="$1"
  if [ "$(whoami)" != 'root' ]; then
    echo "$(date --utc +%FT%T.%3NZ) Error: Run this script with 'sudo $0'.${extra_msg}"
    return 1
  fi
}

prereqs() {
  # make sure we have all dependencies
  local INSTALL=""
  ! command -v cpupower &> /dev/null && INSTALL+="linux-tools-common "
  ! command -v jq &> /dev/null && INSTALL+="jq "
  ! command -v lscpu &> /dev/null && INSTALL+="util-linux "
  ! command -v docker &> /dev/null && INSTALL+="docker-buildx-plugin docker-ce docker-ce-cli docker-ce-rootless-extras docker-compose-plugin "
  ! command -v sudo &> /dev/null && INSTALL+="sudo "
  ! command -v mkfs.vfat &> /dev/null && INSTALL+="dosfstools "
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

set_cpu() {
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

restore_cpu() {
  # restore CPU frequency settings
  [ "$HAVE_SET_CPU" = "false" ] && return 0
  check_root " Elevated permissons required to reset CPU clock to default settings."
  echo "$(date --utc +%FT%T.%3NZ) Info:  Restoring CPU frequency settings of MIN_FREQ: ${MIN_FREQ}kHz, MAX_FREQ: ${MAX_FREQ}kHz, governor: $ORIG_GOVERNOR and enabling turbo boost."
  echo "$ORIG_AMD_PSTATE" > /sys/devices/system/cpu/amd_pstate/status
  cpupower frequency-set -g "$ORIG_GOVERNOR" > /dev/null 2>&1
  cpupower frequency-set -d "$MIN_FREQ" > /dev/null 2>&1
  cpupower frequency-set -u "$MAX_FREQ" > /dev/null 2>&1
}

setup_disk() {
  # create a simulated vfat formatted brd block device in memory
  local mountpoint="$1"
  local USER_ID="$2"
  local GROUP_ID="$3"
  DEVICE="/dev/ram0"
  # rd_size in KiB, allocate 1GiB
  modprobe brd rd_nr=1 rd_size="$((1024**2*1))" max_part=1 && BRD_CREATED="true"
  # fill with 1GiB of random data
  dd if=/dev/urandom of="${DEVICE}" bs=1024 count="$((1024**2*1))"
  # use FAT32 because it supports -o sync, logical sector size == brd physical sector size == page size == 4096
  mkfs.vfat -F32 -S4096 "${DEVICE}"
  # mount with -o sync to force O_DIRECT disk access bypassing the page cache, which makes cgroup.blkio namesspace limits work in docker
  mount -odefaults,noatime,sync,umask=000 "${DEVICE}" "${mountpoint?err_unset}" && DEVICE_MOUNTED="true"
}

disable_swap() {
  echo 3 > /proc/sys/vm/drop_caches
  swapoff -a && SWAP_DISABLED="true"
}

enable_swap() {
  [ "${SWAP_DISABLED}" = "true" ] && swapon -a
}

cleanup_disk() {
  local mountpoint="$1"
  sync
  if [ "${DEVICE_MOUNTED}" = "true" ]; then
    while read -r pid; do
      wait "${pid}"
    done < <(lsof -F p "${mountpoint?err_unset}" | tr -d 'p')
    sync
    umount "${mountpoint?err_unset}"
  fi
  sync
  [ "${MOUNT_IS_TMP}" = "true" ] && rm -rf "${mountpoint?err_unset}"
  sync
  if [ "${BRD_CREATED}" = "true" ]; then
    i=0
    while ! modprobe -r brd 2>/dev/null && [ "${i}" -lt 60 ]; do
      sync
      sleep 0.1
      i="$((i+1))"
    done
  fi
}

kill_orphaned_containers() {
  if [ "${BENCHMARK_STARTED}" = "true" ]; then
    docker ps --format '{{.Names}}' | grep "scripts-zkverify-bench-run" | xargs -I{} docker kill {} 2>/dev/null
  fi
}

# locking
LOCKFILE="/var/lock/.$(basename "${BASH_SOURCE[0]}").lock"
LOCKFD=99
# shellcheck disable=SC2086
_lock()        { flock -$1 $LOCKFD; }
remove_lock()  { _lock u; _lock xn && rm -f "${LOCKFILE}"; }
prepare_lock() { eval "exec $LOCKFD>\"${LOCKFILE}\""; }
exlock_now()   { _lock xn; }  # obtain an exclusive lock immediately or fail

prepare_lock
check_root ""
prereqs

# performance profiles
declare -A cpu_profiles
declare -A io_profiles

# Default CPU profile for 'Relay' chain reference machine: aws.c7a.2xlarge
# Default CPU profile for 'Para' chain reference machine: linode.g6-standard-8
CPU_PROFILE="${CPU_PROFILE:-aws.c7a.2xlarge}"
IO_PROFILE="${IO_PROFILE:-aws.ebs.io2_8000}"
echo -e "\n==== INFO: Using CPU profile '${CPU_PROFILE}' and IO profile '${IO_PROFILE}' ====\n"

# frequency in kHz
cpu_profiles["aws.c7a.2xlarge"]="3500000"
cpu_profiles["linode.g6-standard-8"]="2800000"
cpu_profiles["unconfined"]="5389000"
io_profiles["aws.ebs.io2_8000"]='{"read_io":97000,"write_io":97000,"read_bps":575438848,"write_bps":575438848}'
io_profiles["unconfined"]='{"read_io":1000000000,"write_io":1000000000,"read_bps":53687091200,"write_bps":53687091200}'

# config
HAVE_SET_CPU="false"
DEVICE_MOUNTED="false"
MOUNT_IS_TMP="false"
SWAP_DISABLED="false"
BRD_CREATED="false"
BENCHMARK_STARTED="false"
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
USE_DOCKER="true"
ENABLE_PALLETS="${ENABLE_PALLETS:-true}"
ENABLE_OVERHEAD="${ENABLE_OVERHEAD:-true}"
ENABLE_MACHINE="${ENABLE_MACHINE:-true}"
# Network to benchmark, used in run_all_benchmarks.sh
NETWORK="${NETWORK:-zkverify}"
# The space separated pallet list to benchmark (empty means all). Use the inent
# version like `pallet_aggregate` and not the one with `-`
PALLETS="${PALLETS:-}"

export IS_BENCHMACHINE READ_IO WRITE_IO READ_BPS WRITE_BPS BENCH_BASE_PATH DEVICE USE_DOCKER ENABLE_PALLETS ECHO_CMD ROOT_DIR PALLETS

# add exit handler to restore machine to base settings on exit or error
exit_handler() {
  restore_cpu
  kill_orphaned_containers || true
  cleanup_disk "${BENCH_BASE_PATH}" || true
  enable_swap
  remove_lock
}

trap exit_handler EXIT

exlock_now || { echo -e "Error: this script cannot be run concurrently. Check no other users are running it and that the previous run completed without issues.\n\nWhen in doubt reboot the machine with 'sudo reboot'."; exit 1; }

# run run_all_benchmarks.sh once without any enabled benchmarks
# this will trigger a docker build if needed
# doing this before underclocking the CPU will be faster
IS_BENCHMACHINE="false" ENABLE_PALLETS="false" ENABLE_OVERHEAD="false" ENABLE_MACHINE="false" \
  sudo --preserve-env=IS_BENCHMACHINE,READ_IO,WRITE_IO,READ_BPS,WRITE_BPS,BENCH_BASE_PATH,DEVICE,USE_DOCKER,ENABLE_PALLETS,ENABLE_OVERHEAD,ENABLE_MACHINE,PALLETS,ECHO_CMD,ROOT_DIR,NETWORK \
  -u "${USER}" bash -c 'cd "${ROOT_DIR}"; "${ROOT_DIR}/scripts/run_all_benchmarks.sh"'

# run benchmark
disable_swap
setup_disk "${BENCH_BASE_PATH}" "${USER_ID}" "${GROUP_ID}"
set_cpu "${cpu_profiles[${CPU_PROFILE}]}"
BENCHMARK_STARTED="true"
sudo --preserve-env=IS_BENCHMACHINE,READ_IO,WRITE_IO,READ_BPS,WRITE_BPS,BENCH_BASE_PATH,DEVICE,USE_DOCKER,ENABLE_PALLETS,ENABLE_OVERHEAD,ENABLE_MACHINE,PALLETS,ECHO_CMD,ROOT_DIR,NETWORK \
  -u "${USER}" bash -c 'cd "${ROOT_DIR}"; "${ROOT_DIR}/scripts/run_all_benchmarks.sh"'
