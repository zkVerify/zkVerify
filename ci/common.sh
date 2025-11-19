#!/bin/bash

IS_A_RELEASE=${IS_A_RELEASE:-"false"}

log() {
  # Usage: log style color "message"
  # style: bold, italic, normal, light
  # color: black, red, green
  # Example: log bold red "ERROR: Something went wrong"

  # styles
  # shellcheck disable=SC2034
  local normal=0
  local bold=1
  # shellcheck disable=SC2034
  local shadow=2
  # shellcheck disable=SC2034
  local italic=3
  # colors
  # shellcheck disable=SC2034
  local black=30
  local red=31
  # shellcheck disable=SC2034
  local green=32
  # shellcheck disable=SC2034
  local yellow=33

  local usage="Usage: ${FUNCNAME[0]} style color \"message\"\nStyles: bold, italic, normal, light\nColors: black, red, green, yellow\nExample: log bold red \"ERROR: Something went wrong\""
  [ "$#" -lt 3 ] && {
    echo -e "\033[${bold};${red}m${FUNCNAME[0]} error: function requires three arguments.\n${usage}\033[0m"
    exit 1
  }
  # vars
  local style="${1}"
  local color="${2}"
  local message="${3}"
  # validate style is in bold, italic, normal, shadow
  if [[ ! "${style}" =~ ^(bold|italic|normal|shadow)$ ]]; then
    message="ERROR: Invalid style. Must be one of normal, bold, italic, shadow."
    echo -e "\033[${bold};${red}m${message}\033[0m"
    exit 1
  fi
  # validate color is in black, red, green
  if [[ ! "${color}" =~ ^(black|red|green|yellow)$ ]]; then
    message="ERROR: Invalid color. Must be one of black, red, green or yellow."
    echo -e "\033[${bold};${red}m${message}\033[0m"
    exit 1
  fi
  echo -e "\033[${!style};${!color}m${message}\033[0m"
}

log_info() {
  local usage="Log a message in bold green - Usage: ${FUNCNAME[0]} {message}"
  [ "${1:-}" = "usage" ] && log_debug "${usage}" && return
  [ "$#" -ne 1 ] && fn_die "\n${FUNCNAME[0]} error: function requires exactly one argument.\n\n${usage}"
  log bold green "${1}" >&2
}

log_debug() {
  local usage="Log a message in normal green - Usage: ${FUNCNAME[0]} {message}"
  [ "${1:-}" = "usage" ] && log_debug "${usage}" && return
  [ "$#" -ne 1 ] && fn_die "\n${FUNCNAME[0]} error: function requires exactly one argument.\n\n${usage}"
  log italic green "${1}" >&2
}

log_warn() {
  local usage="Log a message in normal yellow - Usage: ${FUNCNAME[0]} {message}"
  [ "${1:-}" = "usage" ] && log_debug "${usage}" && return
  [ "$#" -ne 1 ] && fn_die "\n${FUNCNAME[0]} error: function requires exactly one argument.\n\n${usage}"
  log bold yellow "${1}" >&2
}

log_error() {
  local usage="Log a message in bold red - Usage: ${FUNCNAME[0]} {message}"
  [ "${1:-}" = "usage" ] && log_debug "${usage}" && return
  [ "$#" -ne 1 ] && fn_die "\n${FUNCNAME[0]} error: function requires exactly one argument.\n\n${usage}"
  log bold red "${1}" >&2
}

fn_die() {
  log_error "${1}" >&2
  exit "${2:-1}"
}

selection() {
  local usage="Use select method for multi choice interaction with user - usage: ${FUNCNAME[0]} {string_to_be_used}"
  [ "${1:-}" = "usage" ] && log_debug "${usage}" && return
  [ "$#" -ne 1 ] && fn_die "\n${FUNCNAME[0]} error: function requires exactly one argument.\n\n${usage}"

  local select_from_string="${1:-}"
  select item in ${select_from_string} "QUIT"; do
    case "${item}" in
    "")
      log_warn "\nInvalid selection. Please type the number of the option you want to choose."
      ;;
    *)
      log_info "\nYou have selected: ${item}"
      echo "${item}"
      break
      ;;
    esac
  done
}

verify_required_commands() {
  command -v act &>/dev/null || fn_die "${FUNCNAME[0]} ERROR: 'act' is required to run this script, see installation instructions at 'https://nektosact.com/installation/index.html'"
  command -v docker &>/dev/null || fn_die "${FUNCNAME[0]} ERROR: 'docker' is required to run this script, see installation instructions at 'https://docs.docker.com/engine/install/'"
  (docker compose version 2>&1 | grep -q "v2\|version 2") || fn_die "${FUNCNAME[0]} ERROR: 'docker compose' is required to run this script, see installation instructions at 'https://docs.docker.com/compose/install/'"
}

check_requirements() {
  log_info "\n=== Checking all the requirements ==="
  verify_required_commands
  docker info >/dev/null 2>&1 || fn_die "${FUNCNAME[0]} ERROR: 'docker daemon' is not running, start it before running this script."
}

# Functions
import_gpg_keys() {
  # shellcheck disable=SC2207
  declare -r keys=( $(echo "${@}" | tr " " "\n") )

  if [ "${#keys[@]}" -eq 0 ]; then
    log_warn "WARNING: there are ZERO gpg keys to import. Please check if 'MAINTAINERS_KEYS' variable is set correctly. The build is not going to be released ..."
    IS_A_RELEASE="false"
  else
    # shellcheck disable=SC2145
    printf "%s\n" "Tagged build, fetching keys:" "${@}" ""
    for key in "${keys[@]}"; do
      gpg -v --batch --keyserver hkps://keys.openpgp.org --recv-keys "${key}" ||
      gpg -v --batch --keyserver hkp://keyserver.ubuntu.com --recv-keys "${key}" ||
      gpg -v --batch --keyserver hkp://pgp.mit.edu:80 --recv-keys "${key}" ||
      gpg -v --batch --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys "${key}" ||
      { log_warn "WARNING: ${key} can not be found on GPG key servers. Please upload it to at least one of the following GPG key servers:\nhttps://keys.openpgp.org/\nhttps://keyserver.ubuntu.com/\nhttps://pgp.mit.edu/"
        IS_A_RELEASE="false"
      }
    done
  fi
}

check_signed_tag() {
  local tag="${1}"

  if git verify-tag -v "${tag}"; then
    log_info "INFO: ${tag} is a valid signed tag"
  else
    log_warn "WARNING: GIT's tag = ${tag} signature is NOT valid. The build is not going to be released ..."
    IS_A_RELEASE="false"
  fi
}

function git_tag_commit {
  local tag="$1"
  git rev-list -n 1 "${tag}"
}