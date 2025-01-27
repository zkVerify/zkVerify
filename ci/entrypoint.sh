#!/bin/bash
set -euo pipefail

# Add local zenbuilder user, either use LOCAL_USER_ID:LOCAL_GRP_ID
# if set via environment or fallback to 9001:9001
USER_ID="${LOCAL_USER_ID:-9001}"
GRP_ID="${LOCAL_GRP_ID:-9001}"
DOCKER_BUILD_DIR="${DOCKER_BUILD_DIR:-/build}"
DOCKER_CARGO_HOME="${DOCKER_CARGO_HOME:-/tmp/.cargo}"
CARGO_BINARIES_INSTALL="${CARGO_BINARIES_INSTALL:-}"
NODEJS_VERSION_INSTALL="${NODEJS_VERSION_INSTALL:-}"
CMAKE_INSTALL="${CMAKE_INSTALL:-}"
LLD_INSTALL="${LLD_INSTALL:-}"
TARGET_DIR="${DOCKER_BUILD_DIR}/target"

fn_die() {
  echo -e "\n\033[1;31m${1}\033[0m\n" >&2
  exit "${2:-1}"
}

# Using generic cargo home dir location. 'CARGO_HOME' is a special env var that defines home dir for CARGO
if [ -n "${DOCKER_CARGO_HOME}" ]; then
  export CARGO_HOME="${DOCKER_CARGO_HOME}"
fi

# Run user mgmt
if [ "$USER_ID" != "0" ]; then
  export USERNAME=zenbuilder
  getent group "$GRP_ID" &> /dev/null || groupadd -g "$GRP_ID" "$USERNAME"
  id -u "$USERNAME" &> /dev/null || useradd --shell /bin/bash -u "$USER_ID" -g "$GRP_ID" -o -c "" -m "$USERNAME"
  CURRENT_UID="$(id -u "$USERNAME")"
  CURRENT_GID="$(id -g "$USERNAME")"
  export HOME=/home/"$USERNAME"
  if [ "$USER_ID" != "$CURRENT_UID" ] || [ "$GRP_ID" != "$CURRENT_GID" ]; then
    echo -e "\nWARNING: User with differing UID ${CURRENT_UID}/GID ${CURRENT_GID} already exists, most likely this container was started before with a different UID/GID. Re-create it to change UID/GID.\n"
  fi
else
  export USERNAME=root
  export HOME=/root
  CURRENT_UID="$USER_ID"
  CURRENT_GID="$GRP_ID"
  echo -e "\nWARNING: Starting container processes as root. This has some security implications and goes against docker best practice.\n"
fi

# Installing extra dependencies if required
if [ -n "${CARGO_BINARIES_INSTALL}" ]; then
  echo -e "\nInstalling extra cargo binaries: ${CARGO_BINARIES_INSTALL}\n"
  for binary in $(tr "," " " <<< "${CARGO_BINARIES_INSTALL}"); do
    cargo install --force "${binary}" || fn_die "ERROR: Failed to install cargo binary: ${binary}"
  done
  PATH=$PATH:"${CARGO_HOME}/bin"
fi

# Node.js install if required
if [ -n "${NODEJS_VERSION_INSTALL}" ]; then
  echo -e "\n=== Installing Node.js version: ${NODEJS_VERSION_INSTALL} ===\n"
  mkdir -p /etc/apt/keyrings
  curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
  echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODEJS_VERSION_INSTALL}.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
  export DEBIAN_FRONTEND=noninteractive
  apt update -qq
  apt --no-install-recommends install -y nodejs
  npm install -g yarn
  echo -e "Nodejs environment was successfully deployed. Node.js version: $(node -v) | npm version: $(npm -v) | yarn version: $(yarn -v)\n"
fi

# cmake install if required
if [ -n "${CMAKE_INSTALL}" ]; then
  echo -e "\n=== Installing cmake ===\n"
  export DEBIAN_FRONTEND=noninteractive
  apt update -qq
  apt --no-install-recommends install -y cmake
  echo -e "cmake was successfully installed. cmake version: $(cmake --version | grep -P -o -e '\d+\.\d+\.\d+')\n"
fi

# lld install if required
if [ -n "${LLD_INSTALL}" ]; then
  echo -e "\n=== Installing lld ===\n"
  export DEBIAN_FRONTEND=noninteractive
  apt update -qq
  apt --no-install-recommends install -y lld
  echo -e "lld was successfully installed.\n"
fi

# System info
rustup show
num_cpus=$(lscpu | grep '^CPU(s):' | awk '{print $2}')
total_ram=$(free -h | grep '^Mem:' | awk '{print $2}')
echo -e "\nCPU count: ${num_cpus}\nTotal RAM: ${total_ram}"
echo -e "\nUsername: $USERNAME, HOME: $HOME, UID: $CURRENT_UID, GID: $CURRENT_GID"
echo "CARGOARGS: ${CARGOARGS:-unset} | RUSTFLAGS: ${RUSTFLAGS:-unset} | RUST_CROSS_TARGETS: ${RUST_CROSS_TARGETS:-unset} | RUST_COMPONENTS: ${RUST_COMPONENTS:-unset} | RUSTUP_TOOLCHAIN: ${RUSTUP_TOOLCHAIN:-unset}"

# Adding safe.directory since 'act' runs as root user
git config --global --add safe.directory "${DOCKER_BUILD_DIR}"

# Taking care of cache directories ownership after mounting
if [ ! -d "${CARGO_HOME}" ]; then
  mkdir "${CARGO_HOME}"
fi
chown -fR "$CURRENT_UID":"$CURRENT_GID" "${CARGO_HOME}" || fn_die "ERROR: Failed to change ownership of ${CARGO_HOME} directory. Exiting ..."
chown -fR "$CURRENT_UID":"$CURRENT_GID" "${TARGET_DIR}" || fn_die "ERROR: Failed to  change ownership of ${DOCKER_BUILD_DIR}/target directory. Exiting ..."

export DONT_CACHE_NATIVE="true"
# On CI use CARGO_INCREMENTAL=1 is useless because the local source files are always newer
# then the compiled artifact (cargo use timestamp to identify what to recompile). So we can
# save 20/30% time on disable it without to lose any advantage.
export CARGO_INCREMENTAL=0

# Run
if [ "$USERNAME" = "root" ]; then
  exec "$@"
else
  exec gosu "$USERNAME" "$@"
fi
