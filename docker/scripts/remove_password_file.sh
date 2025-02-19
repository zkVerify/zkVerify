#!/usr/bin/env bash

# Check if both arguments are provided
if [ -z "${1}" ] || [ -z "${2}" ]; then
  echo "Usage: ${0} <node_pid_name> <password_file>"
  exit 1
fi

NODE_PID_NAME="${1}"
PASSWORD_FILE="${2}"

# Wait for the node process to start
echo -e "INFO: waiting for node to launch ...\n"

# Ensure that pidof returns a valid process ID, otherwise keep retrying
until pid="$(pidof "${NODE_PID_NAME}")"; do
  sleep 1
done

echo -e "INFO: node process started with PID: ${pid}\n"
echo -e "INFO: waiting for node to complete startup ...\n"

# Check if the node has finished syncing by reading stderr (fd 2)
if grep -qe "🏆 Imported" /proc/"${pid}"/fd/2; then
  echo -e "INFO: Node started and synced; wiping password ...\n"

  # Remove the password file if the node has completed startup
  if [ -f "${PASSWORD_FILE}" ]; then
    rm -f "${PASSWORD_FILE}"
    echo "INFO: password file removed."
  else
    echo "WARNING: password file not found: ${PASSWORD_FILE}"
  fi
else
  echo "ERROR: node has not completed startup yet. Exiting ..."
  exit 1
fi
