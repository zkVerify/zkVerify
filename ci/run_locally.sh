#!/bin/bash
set -eEuo pipefail

root_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." &>/dev/null && pwd)"
common_file_location="${root_dir}/ci/common.sh"
conf_file_location="${root_dir}/ci/run_locally"
workflows_dir="${root_dir}/.github/workflows"
PRE_PUSH_HOOK="${PRE_PUSH_HOOK:-false}"

show_help() {
  echo "Usage: ${0} [-i | --help]"
  echo
  echo "Options:"
  echo "  -i        Run the script in interactive mode, allowing you to select workflows."
  echo "  --help    Show this help message and exit."
}

interactive_mode=false
# Parse command-line arguments
if [[ "${1:-}" == "-i" ]]; then
  interactive_mode=true
elif [[ "${1:-}" == "--help" ]]; then
  show_help
  exit 0
elif [[ "${1:-}" != "" ]]; then
  echo "Invalid option: ${1}"
  show_help
  exit 1
fi


####
# Checking all the requirement(s)
####
if ! [ -f "${common_file_location}" ]; then
  echo -e "\n\033[1;31mERROR: ${common_file_location} file is missing !!!  Exiting ...\033[0m\n"
  exit 1
else
  # shellcheck disable=SC1090
  source "${common_file_location}"
fi
check_requirements

# Starting from fresh act environment
act_dangling_containers="$(docker ps -a -q -f "name=act")" || fn_die "ERROR: Failed to execute the 'docker ps' command. Please ensure the 'docker' service is running. Exiting ..."
if [ -n "${act_dangling_containers}" ]; then
  log_info "\n=== Cleaning up dangling 'act' containers ==="
  docker rm -f "${act_dangling_containers}" || fn_die "ERROR: Failed to execute the 'docker rm -f' command. Please ensure the 'docker' service is running. Exiting ..."
fi


####
# Running workflow(s)
####
workflows_orchestrator="CI-build CI-test CI-coverage CI-lint-format CI-zombienet-test"
workflows_extra="CI-rustdoc CI-build-docker-image"
if [ "${interactive_mode}" == 'true' ];then
  workflows="${workflows_orchestrator} ${workflows_extra}"
  while true; do
    # choose one of the available workflows
    log_warn "\nPlease select a workflow to run:"
    workflow="$(selection "${workflows}")" || fn_die "ERROR: Failed to execute 'selection' function for listing workflows to run. Exiting ..."
    if [ "${workflow}" == 'QUIT' ]; then
      break
    else
      declare -a opt_args
      env_file="${conf_file_location}/${workflow}.vars"
      if [ -f "${env_file}" ]; then
        log_debug "\n=== Using envs from ${env_file} ==="
        opt_args+=(--var-file "${env_file}")
      fi
      inputs_file="${conf_file_location}/${workflow}.inputs"
      if [ -f "${inputs_file}" ]; then
        log_debug "\n=== Using inputs from ${env_file} ==="
        opt_args+=(--input-file "${inputs_file}")
      fi
      log_debug "\n=== Running ${workflows_dir}/${workflow}.yml workflow ==="
      act "${opt_args[@]}" --detect-event --rm -W "${workflows_dir}/${workflow}.yml" || fn_die "ERROR: attempt to run ${workflows_dir}/${workflow}.yml workflow locally has failed. Exiting ..."
    fi
  done
elif [ "${interactive_mode}" == 'false' ]; then
  log_debug "\nRunning all the following workflows sequentially: ${workflows_orchestrator}"
  for workflow in ${workflows_orchestrator}; do
    log_debug "\n=== Running ${workflows_dir}/${workflow}.yml workflow ==="
    act --detect-event --rm -W "${workflows_dir}/${workflow}.yml" || fn_die "ERROR: attempt to run ${workflows_dir}/${workflow}.yml workflow locally has failed. Exiting ..."
  done
fi


####
# End
####
log_info "\n=== Done ==="
exit 0