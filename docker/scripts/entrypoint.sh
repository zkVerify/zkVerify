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
# 2. if underscores (`_`) are present, they are replaced with dashes (`-`)
# 3. letters are replaced with lower case
# 4. prefix `--` is added
# 
# Examples:
# 
# - `ZKV_CONF_BASE_PATH` -> `--base-path`
# - `ZKV_CONF_BOOTNODES` -> `--bootnodes`
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

  # Extract the base name by removing the prefix
  local base_name="${env_name:${#prefix}}"

  # Replace underscores with hyphens and convert to lowercase
  arg_name="${base_name//_/-}"
  arg_name="${arg_name,,}"

  # Prefix the argument name with --
  echo "--${arg_name}"
}

get_arg_value_from_env_value() {
  local env_value="$1"
  local arg_value="${env_value}"

  # Check if the value is exactly "true".
  if [ "${arg_value}" == "true" ]; then
    # If it is "true", set arg_value to an empty string (""), indicating no flag should be set.
    arg_value=""
  fi

  # Output the processed arg_value
  echo "${arg_value}"
}

# Function to validate chain specification and download if necessary
validate_and_download() {
  local CHAIN_VAR_NAME="$1"
  local URL_VAR_NAME="$2"

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
  fn_die "ERROR: Required environment variable 'BINARY' is not defined. This should never happen. Aborting ..."
else
  # If the user built the image with multiple binaries,
  # we consider the first one to be the canonical one
  # To start with another binary, the user can either:
  #  - use the --entrypoint option
  #  - pass the ENV BINARY with a single binary
  IFS=',' read -r -a BINARIES <<< "${BINARY}"
  ZKV_NODE="${BINARIES[0]}"
  command -v "${ZKV_NODE}" &>/dev/null || fn_die "ERROR: '${ZKV_NODE}' binary can not be found on the run user's 'PATH'=${PATH}"
  log_bold_green "🔧 zkVerify node binary: ${ZKV_NODE}"
fi

####
# zkVerify node's configuration
####
log_bold_green "=== zkVerify node's configuration:"
ZKV_CONF_BASE_PATH=${ZKV_CONF_BASE_PATH:-}
ZKV_CONF_CHAIN=${ZKV_CONF_CHAIN:-}
ZKV_SPEC_FILE_URL="${ZKV_SPEC_FILE_URL:-}"
### TO BE IMPLEMENTED
#ZKV_SECRET_PASSWORD="${ZKV_SECRET_PASSWORD:-}"
#ZKV_SECRET_PASSWORD_FILE_PATH=""

# Loop through all arguments to check for --dev
for arg in "$@"; do
  if [ "${arg}" == "--dev" ]; then
    log_green "INFO: '--dev' flag detected! Running in development mode."
    # You can place any logic for development mode here
    DEV_MODE="true"
    break
  fi
done

if [ "${DEV_MODE:-false}" != "true" ]; then
  # Call the function for ZKV_CONF_CHAIN
  validate_and_download "ZKV_CONF_CHAIN" "ZKV_SPEC_FILE_URL"
fi

ZKV_SECRET_PHRASE_PATH=${ZKV_SECRET_PHRASE_PATH:-"/data/config/secret_phrase.dat"}
ZKV_NODE_KEY_FILE=${ZKV_NODE_KEY_FILE:-"/data/config/node_key.dat"}
echo "  ZKV_SECRET_PHRASE_PATH=${ZKV_SECRET_PHRASE_PATH}"
echo -e "  ZKV_NODE_KEY_FILE=${ZKV_NODE_KEY_FILE}\n"

prefix="ZKV_CONF_"
conf_args=()
while IFS='=' read -r -d '' var_name var_value; do
  if [[ "${var_name}" == "${prefix}"* ]]; then
    # Get argument name from the environment variable name
    arg_name="$(get_arg_name_from_env_name "${var_name}" "${prefix}")"

    # If the value contains commas, handle it by splitting the values
    if [[ "${var_value}" == *","* ]]; then
      IFS=',' read -ra values <<< "${var_value}"
      for value in "${values[@]}"; do
        # Add the argument name and each value
        conf_args+=("${arg_name}")
        conf_args+=("${value}")
      done
    else
      # If there is no comma, just add the argument with the value
      if [ "${var_value}" != "true" ]; then
        conf_args+=("${arg_name}")
        conf_args+=("${var_value}")
      else
        conf_args+=("${arg_name}")
      fi
    fi

    # Debug output
    echo "  ${var_name}=${var_value} -> ${arg_name} ${var_value}"
  fi
done < <(env -0)

### TO BE IMPLEMENTED
## Password file mgmt if provided. Saving under on tmpfs device
#if [ -n "${ZKV_SECRET_PASSWORD}" ]; then
#  ZKV_SECRET_PASSWORD_FILE_PATH="/tmp/password.txt"
#  printf "%s" "${ZKV_SECRET_PASSWORD}" > "${ZKV_SECRET_PASSWORD_FILE_PATH}"
#
#  if [ -f "${ZKV_SECRET_PASSWORD_FILE_PATH}" ] && [ -s "${ZKV_SECRET_PASSWORD_FILE_PATH}" ]; then
#    sed -i -z 's/\n$//' "${ZKV_SECRET_PASSWORD_FILE_PATH}" # make sure there is no ending newline
#    conf_args+=("--password-filename" "$(get_arg_value_from_env_value "${ZKV_SECRET_PASSWORD_FILE_PATH}")")
#  else
#    fn_die "ERROR: The 'ZKV_SECRET_PASSWORD' environment variable is set, but the ${ZKV_SECRET_PASSWORD_FILE} file could not be created or populated. Exiting..."
#  fi
#fi

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
  ### TO BE IMPLEMENTED
#  if [ -f "${ZKV_SECRET_PASSWORD_FILE_PATH}" ] && [ -s "${ZKV_SECRET_PASSWORD_FILE_PATH}" ]; then
#    injection_args+=("--password-filename" "$(get_arg_value_from_env_value "${ZKV_SECRET_PASSWORD_FILE_PATH}")")
#  fi
  log_green "INFO: injecting keys with ${injection_args[*]} ..."

  log_green "INFO: injecting key (Babe) ..."
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type babe

  log_green "INFO: injecting key (Grandpa) ..."
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Ed25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type gran

  log_green "INFO: injecting key (Imonline) ..."
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type imon

  log_green "INFO: injecting key (Parachain) ..."
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type para

  log_green "INFO: injecting key (Authorities Discovery) ..."
  ${ZKV_NODE} key insert "${injection_args[@]}" \
    --scheme Sr25519 \
    --suri "${ZKV_SECRET_PHRASE_PATH}" \
    --key-type audi
fi

# Node-key (used for p2p) handling
if [[ (-n "${ZKV_CONF_BASE_PATH}") && (-n "${ZKV_CONF_CHAIN}") && (-f "${ZKV_NODE_KEY_FILE}") ]]; then
  base_path="$(get_arg_value_from_env_value "${ZKV_CONF_BASE_PATH}")"
  chain="$(get_arg_value_from_env_value "${ZKV_CONF_CHAIN}")"
  chain_id="$("${ZKV_NODE}" build-spec --chain "${chain}" 2>/dev/null | jq -r '.id')" || chain_id='null'
  if [ -z "${chain_id}" ] || [ "${chain_id}" == 'null' ]; then
    fn_die "ERROR: could not find 'id' under node's spec file. Aborting ..."
  fi
  destination="${base_path}/chains/${chain_id}/network"

  mkdir -p "${destination}"
  log_green "INFO: copying node's key file to ${destination} location ..."
  cp "${ZKV_NODE_KEY_FILE}" "${destination}/secret_ed25519"
fi

### TO BE IMPLEMENTED
#if [ -f "${ZKV_SECRET_PASSWORD_FILE_PATH}" ] && [ -s "${ZKV_SECRET_PASSWORD_FILE_PATH}" ]; then
#  log_green "INFO: launching password file remover ..."
#  ./remove_password_file.sh "${ZKV_NODE}" "${ZKV_SECRET_PASSWORD_FILE_PATH}" >> "/${RUN_USER}/remove_password_file_$(date +%Y-%m-%d_%H-%M-%S).log" 2>&1 &
#fi

####
# Relaychain collator's configuration (env->arg)
####
prefix="RC_CONF_"
if env | grep -q "^${prefix}"; then
  log_bold_green "=== Relaychain collator's configuration:"
  relaychain_appended_any=""
  while IFS='=' read -r -d '' var_name var_value; do
    if [[ "${var_name}" == "${prefix}"* ]]; then
      # Append separator only once
      if [[ -z "${relaychain_appended_any}" ]]; then
        relaychain_appended_any="true"
        conf_args+=("--")
      fi

      # Get argument name from the environment variable name
      arg_name="$(get_arg_name_from_env_name "${var_name}" "${prefix}")"

      # If the value contains commas, handle it by splitting the values
      if [[ "${var_value}" == *","* ]]; then
        IFS=',' read -ra values <<< "${var_value}"
        for value in "${values[@]}"; do
          # Add the argument name and each value
          conf_args+=("${arg_name}")
          conf_args+=("${value}")
        done
      else
        # If there is no comma, just add the argument with or without the value depending on the condition
        if [ "${var_value}" != "true" ]; then
          conf_args+=("${arg_name}")
          conf_args+=("${var_value}")
        else
          conf_args+=("${arg_name}")
        fi
      fi

      # Debug output
      echo "  ${var_name}=${var_value} -> ${arg_name} ${var_value}"
    fi
  done < <(env -0)
fi

log_green "INFO: launching ${ZKV_NODE} with the following args:"
echo "  ${conf_args[*]}" "$@"

exec "${ZKV_NODE}" "${conf_args[@]}" "$@"
