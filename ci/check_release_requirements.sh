#!/bin/bash
set -eEuo pipefail

IS_A_RELEASE="false"
PROD_RELEASE="false"
RC_RELEASE="false"
TEST_RELEASE="false"
FAST_RUNTIME_RELEASE="${FAST_RUNTIME_RELEASE:-false}"
workdir="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )"
github_tag="${GITHUB_REF_NAME:-}"
release_branch="${RELEASE_BRANCH:-release}"
maintainers_keys="${MAINTAINERS_KEYS:-}"
version_regex='[0-9]+\.[0-9]+\.[0-9]+'
date_regex='20[2-4][0-9]((0[1-9])|(1[0-2]))((0[1-9])|([1-2][0-9])|(3[0-1]))'
rc_regex='rc[0-9]+'
test_regex='[a-zA-Z][a-zA-Z0-9_]*'

prod_release_regex="^${version_regex}-${date_regex}?$"
rc_release_regex="^${version_regex}-${rc_regex}$"
test_release_regex="^${version_regex}-${test_regex}$"

version_str="${github_tag#v}"

export VERSION_STR="${version_str}"
export COMMON_FILE_LOCATION='ci/common.sh'
# Requirement
if ! [ -f "${workdir}/${COMMON_FILE_LOCATION}" ]; then
  echo -e "\n\033[1;31mERROR: ${COMMON_FILE_LOCATION} file is missing !!! \033[0m\n"
  return
else
  # shellcheck disable=SC1090
  source "${COMMON_FILE_LOCATION}"
fi

####
# Main
####
log_info "DRY_RUN: '${DRY_RUN}'"
log_info "Release branch is: ${release_branch}"
log_info "Github tag is: ${github_tag}"
log_info "Version string is: ${version_str}"

# Checking if it is a release build
if git branch -r --contains "${github_tag}" | grep -xqE ". origin\/${release_branch}\/${version_str}$"; then
  IS_A_RELEASE="true"

  if [ -z "${maintainers_keys:-}" ]; then
    log_warn "WARNING: 'MAINTAINERS_KEYS' variable is not set. The build is not going to be released ..."
  fi

  if [ "${DRY_RUN}" != "true" ]; then
    import_gpg_keys "${maintainers_keys}"
    check_signed_tag "${github_tag}"
  else
    log_warn "WARNING: 'DRY_RUN' variable is set to 'true'. Don't check GPG keys"
  fi

  # Release test
  if [ "${IS_A_RELEASE}" = "true" ]; then
    if [[ "${version_str}" =~ ${prod_release_regex} ]]; then
      log_info "Prod Release: GitHub tag: ${github_tag} is a production release [${version_str}]"
      PROD_RELEASE="true"
    elif [[ "${version_str}" =~ ${rc_release_regex} ]]; then
      log_info "RC Release: GitHub tag: ${github_tag} is a release candidate [${version_str}]"
      RC_RELEASE="true"
    elif [[ "${version_str}" =~ ${test_release_regex} ]]; then
      log_info "Test Release: GitHub tag: ${github_tag} is a test release [${version_str}]"
      TEST_RELEASE="true"
    else
      log_warn "WARNING: GitHub tag: ${github_tag} is in the wrong format for PRODUCTION, RC or TEST release. Expecting the following format for the release: PRODUCTION = 'd.d.d(-YYYYMMDD)?' | DEVELOPMENT = 'd.d.d-rc[0-9]+' | TEST = 'd.d.d-[a-zA-Z][a-zA-Z0-9_]*'. The build is not going to be released ..."
      IS_A_RELEASE="false"
    fi
  fi
else
  log_warn "WARNING: GitHub tag = ${github_tag} does NOT derive from any '${release_branch}/*' branches. The build is not going to be released ..."
fi