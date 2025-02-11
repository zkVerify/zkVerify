#!/usr/bin/env bash

# This script performs the following tasks:
# 
# - translation of environment variables to command line arguments
# - preparation before the node start (example keys injection)
# - launch of the actual node
# 
# Environment variables should generally be in the form `ZKV_*`
# Environment variables in the form `ZKV_CONF_*` are translated to command line arguments based on these rules:
#
# 1. `ZKV_CONF_` prefix is removed
# 2. if both a trailing underscore (`_`) and number are present, they are removed
# 3. if underscores (`_`) are present, they are replaced with dashes (`-`)
# 4. letters are replaced with lower case
# 5. prefix `--` is added
# 
# Examples:
# 
# - `ZKV_CONF_BASE_PATH` -> `--base-path`
# - `ZKV_CONF_BOOTNODES` -> `--bootnodes`
# - `ZKV_CONF_BOOTNODES_2` -> `--bootnodes`
#
# Values of environment variables are used unmodified as values of command line arguments with the exception
# of `true` being dropped (as a flag, example `ZKV_CONF_VALIDATOR`/`--validator`)

set -eEuo pipefail

####
# Function(s)
####
fn_die() {
  echo -e "\n\033[1;31m${1}\033[0m\n" >&2
  exit "${2:-1}"
}

log_bold_green() {
  echo -e "\n\033[1;32m${1}\033[0m\n"
}

log_green() {
  echo -e "\n\033[0;32m${1}\033[0m\n"
}

log_yellow() {
  echo -e "\n\033[1;33m${1}\033[0m\n"
}

get_arg_name_from_env_name() {
    local env_name="$1"
    local prefix="$2"
    arg_name="${env_name:${#prefix}}"
    arg_name=$(echo "${arg_name}" | sed -r 's/^(.+)_[0-9]*$/\1/')
    arg_name="${arg_name//_/-}"
    arg_name="${arg_name,,}"
    arg_name=--"${arg_name}"
    arg_name="$(echo "$arg_name" | sed -r 's/^---+/--/')"
    echo "${arg_name}"
}

get_arg_value_from_env_value() {
    local env_value="$1"
    arg_value="${env_value}"
    if [ "$arg_value" == "true" ]; then
      arg_value=""
    fi
    echo "${arg_value}"
}

# Function to validate chain specification and download if necessary
validate_and_download() {
  local CHAIN_VAR_NAME="$1"
  local URL_VAR_NAME="$2"

  echo " --- validate and download chain-spec ---"

  # Dynamically retrieve the values of the variables using indirect expansion
  local CHAIN_VALUE="${!CHAIN_VAR_NAME}"
  local SPEC_FILE_URL="${!URL_VAR_NAME}"

  # Check if the chain variable is empty
  if [ -z "${CHAIN_VALUE}" ]; then
    fn_die "ERROR: '${CHAIN_VAR_NAME}' variable can not be empty or undefined. Aborting ..."
  fi

  # Echo the chain value
  echo "  ${CHAIN_VAR_NAME}=${CHAIN_VALUE}"

  # Check if CHAIN_VALUE points to an existing .json file and download it otherwise
  if [[ "${CHAIN_VALUE}" == *.json ]] && [ ! -f "${CHAIN_VALUE}" ] ; then
    # Attempt to download the file if it doesn't exist
    if [ -n "${SPEC_FILE_URL}" ]; then
      log_green "INFO: Spec file '${CHAIN_VALUE}' does not exist. Downloading it from '${SPEC_FILE_URL}' ..."
      mkdir -p "$(dirname "${CHAIN_VALUE}")" || fn_die "ERROR: could not create directory '$(dirname "${CHAIN_VALUE}")' for spec file. Aborting ..."
      cd "$(dirname "${CHAIN_VALUE}")"
      aria2c --file-allocation=none -s16 -x16 --max-tries=3 --continue=true "${SPEC_FILE_URL}" -o "$(basename "${CHAIN_VALUE}")" || fn_die "ERROR: Failed to download spec file from '${SPEC_FILE_URL}' url. Aborting ..."
    else
      fn_die "ERROR: The variable '${CHAIN_VAR_NAME}' (spec file) is set to '${CHAIN_VALUE}', which is a .json file that does not exist. The variable '${URL_VAR_NAME}' is empty, therefore the file can not be downloaded. Aborting ..."
    fi
  fi
}

####
# Main
####

# Sanity check
if [ -z "${BINARY:-}" ]; then
    echo "BINARY ENV not defined, this should never be the case. Aborting..."
    exit 1
fi

# If the user built the image with multiple binaries,
# we consider the first one to be the canonical one
# To start with another binary, the user can either:
#  - use the --entrypoint option
#  - pass the ENV BINARY with a single binary
IFS=',' read -r -a BINARIES <<< "$BINARY"
ZKV_NODE="${BINARIES[0]}"
echo "ZKV_NODE=${ZKV_NODE}"

ZKV_SECRET_PHRASE_PATH=${ZKV_SECRET_PHRASE_PATH:-"/data/config/secret_phrase.dat"}
echo "ZKV_SECRET_PHRASE_PATH=${ZKV_SECRET_PHRASE_PATH}"
ZKV_NODE_KEY_FILE=${ZKV_NODE_KEY_FILE:-"/data/config/node_key.dat"}
echo "ZKV_NODE_KEY_FILE=${ZKV_NODE_KEY_FILE}"

ZKV_CONF_BASE_PATH=${ZKV_CONF_BASE_PATH:-}
ZKV_CONF_CHAIN=${ZKV_CONF_CHAIN:-}
ZKV_SPEC_FILE_URL="${ZKV_SPEC_FILE_URL:-}"
ZKV_SECRET_PASSWORD_PATH="${ZKV_SECRET_PASSWORD_PATH:-}"

# Node configurations (env->arg)
prefix="ZKV_CONF_"
conf_args=()
echo "Node configuration:"
while IFS='=' read -r -d '' var_name var_value; do
  if [[ "$var_name" == ${prefix}* ]]; then
    # rules above
    arg_name=$(get_arg_name_from_env_name "${var_name}" "${prefix}")
    conf_args+=("${arg_name}")
    # rules above
    arg_value=$(get_arg_value_from_env_value "${var_value}")
    if [ -n "${arg_value}" ]; then
      conf_args+=("${arg_value}")
    fi
    echo "  ${var_name}=${var_value} -> ${arg_name} ${arg_value}"
  fi
done < <(env -0)
if [ -n "${ZKV_SECRET_PASSWORD_PATH}" ]; then
  sed -i -z 's/\n$//' "${ZKV_SECRET_PASSWORD_PATH}" # make sure there is no ending newline
  conf_args+=("--password-filename" "$(get_arg_value_from_env_value "${ZKV_SECRET_PASSWORD_PATH}")")
fi

# Relaychain's collator configurations (env->arg)
prefix="RC_CONF_"
echo "Relaycain's collator configuration:"
relaychain_appended_any=""
while IFS='=' read -r -d '' var_name var_value; do
  if [[ "$var_name" == ${prefix}* ]]; then
    if [[ -z ${relaychain_appended_any} ]]; then
      relaychain_appended_any="true"
      # Add separator
      conf_args+=("--")  
    fi
    # rules above
    arg_name=$(get_arg_name_from_env_name "${var_name}" "${prefix}")
    conf_args+=("${arg_name}")
    # rules above
    arg_value=$(get_arg_value_from_env_value "${var_value}")
    if [ -n "${arg_value}" ]; then
      conf_args+=("${arg_value}")
    fi
    echo "  ${var_name}=${var_value} -> ${arg_name} ${arg_value}"
  fi
done < <(env -0)

if [ -n "${ZKV_CONF_BASE_PATH}" ]; then
  BASE_CHAINS="${ZKV_CONF_BASE_PATH}/chains"

  for chain in local testnet ; do
    source_chain_dir="${BASE_CHAINS}/nh_${chain}";
    dest_chain_dir="${BASE_CHAINS}/zkv_${chain}";
    [ -d "$source_chain_dir" ] && [ ! -e "$dest_chain_dir" ] && \
      echo "Move ${source_chain_dir} to ${dest_chain_dir}" && \
      mv "${source_chain_dir}" "${dest_chain_dir}"
  done
fi

# Call the function for ZKV_CONF_CHAIN
validate_and_download "ZKV_CONF_CHAIN" "ZKV_SPEC_FILE_URL"

# Keys handling
if [ -f "${ZKV_SECRET_PHRASE_PATH}" ]; then
  injection_args=()
  if [ -n "${ZKV_CONF_BASE_PATH}" ]; then
    injection_args+=("$(get_arg_name_from_env_name ZKV_CONF_BASE_PATH ${prefix})")
    injection_args+=("$(get_arg_value_from_env_value "${ZKV_CONF_BASE_PATH}")")
  fi
  if [ -n "${ZKV_CONF_CHAIN}" ]; then
    injection_args+=("$(get_arg_name_from_env_name ZKV_CONF_CHAIN ${prefix})")
    injection_args+=("$(get_arg_value_from_env_value "${ZKV_CONF_CHAIN}")")
  fi
  if [ -n "${ZKV_SECRET_PASSWORD_PATH}" ]; then
    injection_args+=("--password-filename" "$(get_arg_value_from_env_value "${ZKV_SECRET_PASSWORD_PATH}")")
  fi
  echo "Injecting keys with ${injection_args[*]}"
  echo "Injecting key (Babe)"
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type babe
  echo "Injecting key (Grandpa)"
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Ed25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type gran
  echo "Injecting key (Imonline)"
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type imon
  echo "Injecting key (Parachain)"
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type para
  echo "Injecting key (Authorities Discovery)"
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type audi
fi

# Node-key handling
if [[ (-n "${ZKV_CONF_BASE_PATH}") && (-n "${ZKV_CONF_CHAIN}") && (-f "${ZKV_NODE_KEY_FILE}") ]]; then
  base_path=("$(get_arg_value_from_env_value "${ZKV_CONF_BASE_PATH}")")
  chain=("$(get_arg_value_from_env_value "${ZKV_CONF_CHAIN}")")
  chain_id=$("${ZKV_NODE}" build-spec --chain "${chain}" 2> /dev/null | grep \"id\": | awk -F'"' '{print $4}')
  destination="${base_path}/chains/${chain_id}/network"
  mkdir -p "${destination}"
  echo "Copying node key file"
  cp "${ZKV_NODE_KEY_FILE}" "${destination}/secret_ed25519"
fi

if [ -n "${ZKV_SECRET_PASSWORD_PATH}" ]; then
    echo "Launching password file remover"
    ./remove_password_file.sh ${ZKV_NODE} ${ZKV_SECRET_PASSWORD_PATH} &
fi

echo "Launching ${ZKV_NODE} with args ${conf_args[*]}"
exec "${ZKV_NODE}" "${conf_args[@]}"
