#!/bin/bash
set -eEuo pipefail

IS_A_RELEASE="false"
PROD_RELEASE="false"
RC_RELEASE="false"
TEST_RELEASE="false"
workdir="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." &> /dev/null && pwd )"
github_tag="${GITHUB_REF_NAME:-}"
runtime="${RUNTIME:-zkverify}"
release_branch="${RELEASE_BRANCH:-release}"
maintainers_keys="${MAINTAINERS_KEYS:-}"
version_regex='[0-9]+\.[0-9]+\.[0-9]+'
rc_regex='rc[0-9]+'
test_regex='[a-zA-Z][a-zA-Z0-9_]*'

# rt-<runtime>-<version>
tag_runtime="$(echo ${github_tag} | cut -d '-' -f 2 )"
version="$(echo ${github_tag} | cut -d '-' -f 3 )"
version_str="$(echo ${github_tag} | cut -d '-' -f 3- )"
version_ext="$(echo ${github_tag} | cut -d '-' -f 4- )"

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
log_info "Runtime: ${runtime}"
log_info "Tag Runtime: ${tag_runtime}"
log_info "Main Version: ${version}"
log_info "Version Extension: ${version_ext}"
log_info "Version Str: ${version_str}"

if [ "${tag_runtime}" != "${runtime}" ]; then
  fn_die "ERROR: Runtime in the tag (${tag_runtime}) does not match the expected runtime (${runtime}). Exiting ..."
fi

if [[ "${runtime}" = "zkverify" && -z "${version_ext}" ]]; then
  log_info "INFO: checking that zkverify and volta production release tags point to the same commit..."
  volta_tag="rt-volta-${version}"
  if [ "$(git_tag_ref "${github_tag}")" != "$(git_tag_ref "${volta_tag}")" ]; then
    fn_die "ERROR: commit pointed to by zkverify tag (${github_tag}) does not match commit of volta tag (${volta_tag}). Exiting..."
  fi
fi

# Checking if it is a release build
if git branch -r --contains "${github_tag}" | grep -xqE ". origin\/${release_branch}\/rt-${runtime}\/${version_str}$"; then
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
    if [[ "${version}" =~ ${version_regex} ]]; then
      if [[ "${version}" = "${version_str}" ]]; then
        log_info "Prod Release: GitHub tag: ${github_tag} is a production release [${version_str}]"
        PROD_RELEASE="true"
      elif [[ "${version_ext}" =~ ${rc_regex} ]]; then
        log_info "RC Release: GitHub tag: ${github_tag} is a release candidate [${version_str}]"
        RC_RELEASE="true"
      elif [[ "${version_str}" =~ ${test_regex} ]]; then
        log_info "Test Release: GitHub tag: ${github_tag} is a test release [${version_str}]"
        TEST_RELEASE="true"
      else
        IS_A_RELEASE="false"
      fi
    else
      IS_A_RELEASE="false"
    fi
    if [ "${IS_A_RELEASE}" = "false" ]; then
      log_warn "WARNING: GitHub tag: ${github_tag} is in the wrong format for PRODUCTION, RC or TEST release. Expecting the following format for the release: PRODUCTION = 'rt-<runtime>-d.d.d' | DEVELOPMENT = 'rt-<runtime>-d.d.d-rc[0-9]+' | TEST = 'rt-<runtime>-d.d.d-[a-zA-Z][a-zA-Z0-9_]*'. The build is not going to be released ..."
    fi
  fi
else
  log_warn "WARNING: GitHub tag = ${github_tag} does NOT derive from any '${release_branch}/rt-${runtime}/*' branches. The build is not going to be released ..."
fi