#!/bin/bash
set -eEuo pipefail

# Set output file locations
unit_test_file="./test-output/unit_tests_output.txt"
integration_test_file="./test-output/integration_tests_output.txt"
coverage_report_file="./coverage-output/coverage_report.json"
zombienet_test_file="./zombienet-test-output/zombienet_test_output.txt"

# Initialize counters for unit tests
unit_total_passed=0
unit_total_failed=0
unit_total_ignored=0
unit_total_runtime=0.0

# Initialize counters for integration tests
integration_total_passed=0
integration_total_failed=0
integration_total_ignored=0
integration_total_runtime=0.0

# Initialize counters for zombienet tests
zombienet_total_passed=0
zombienet_total_failed=0
zombienet_total_runtime=0.0

process_test_file() {
  local file_type="$1" # "unit" or "integration"
  local file_path="$2"
  local output_not_found="${file_type} test output not found."
  
  if [ -f "${file_path}" ]; then
    while IFS= read -r line; do
      if [[ "${line}" == *"test result:"* ]]; then

        passed=$(echo "${line}" | awk '{for (i=1; i<=NF; i++) if ($i=="passed;") print $(i-1)}')
        failed=$(echo "${line}" | awk '{for (i=1; i<=NF; i++) if ($i=="failed;") print $(i-1)}')
        ignored=$(echo "${line}" | awk '{for (i=1; i<=NF; i++) if ($i=="ignored;") print $(i-1)}')
        runtime=$(echo "${line}" | awk -F'finished in ' '{n=split($2,a," "); print (n>0)?a[1]:0}')

        # Update counters based on file type
        if [[ "${file_type}" == "unit" ]]; then
          unit_total_passed=$((unit_total_passed + passed))
          unit_total_failed=$((unit_total_failed + failed))
          unit_total_ignored=$((unit_total_ignored + ignored))
          unit_total_runtime=$(awk -v tr="${unit_total_runtime}" -v rt="${runtime}" 'BEGIN {printf "%.2f", tr+rt}')
        elif [[ "${file_type}" == "integration" ]]; then
          integration_total_passed=$((integration_total_passed + passed))
          integration_total_failed=$((integration_total_failed + failed))
          integration_total_ignored=$((integration_total_ignored + ignored))
          integration_total_runtime=$(awk -v tr="${integration_total_runtime}" -v rt="${runtime}" 'BEGIN {printf "%.2f", tr+rt}')
        fi
      fi
    done < "${file_path}"
  else
    echo "$(echo ${file_type} | awk '{print toupper($0)}')_TEST_SUMMARY=${output_not_found}" >> $GITHUB_ENV
  fi
}

# Process unit and integration test files
process_test_file "unit" "${unit_test_file}"
process_test_file "integration" "${integration_test_file}"

# Format the run times to 2 decimal places
unit_total_runtime=$(printf "%.2f" ${unit_total_runtime})
integration_total_runtime=$(printf "%.2f" ${integration_total_runtime})

# Format and add summaries to $GITHUB_ENV
passed_formatted="*Passed*: ${unit_total_passed}"
failed_formatted="*Failed*: ${unit_total_failed}"
ignored_formatted="*Ignored*: ${unit_total_ignored}"
runtime_formatted="*Runtime*: ${unit_total_runtime}s"
echo "UNIT_TEST_SUMMARY=*Unit Tests |* ${passed_formatted}, ${failed_formatted}, ${ignored_formatted}, ${runtime_formatted}" >> $GITHUB_ENV

passed_formatted="*Passed*: ${integration_total_passed}"
failed_formatted="*Failed*: ${integration_total_failed}"
ignored_formatted="*Ignored*: ${integration_total_ignored}"
runtime_formatted="*Runtime*: ${integration_total_runtime}s"
echo "INTEGRATION_TEST_SUMMARY=*Integration Tests |* ${passed_formatted}, ${failed_formatted}, ${ignored_formatted}, ${runtime_formatted}" >> $GITHUB_ENV

# Extract and summarize overall test coverage data to $GITHUB_ENV
if [ -f "${coverage_report_file}" ]; then
    coverage_totals=$(cat "${coverage_report_file}" | jq '.data[0].totals')
    
    functions_count=$(echo "${coverage_totals}" | jq '.functions.count')
    functions_percent=$(echo "${coverage_totals}" | jq -r '.functions.percent' | awk '{printf "%.2f", $0}')
    lines_count=$(echo "${coverage_totals}" | jq '.lines.count')
    lines_percent=$(echo "${coverage_totals}" | jq -r '.lines.percent' | awk '{printf "%.2f", $0}')
    regions_count=$(echo "${coverage_totals}" | jq '.regions.count')
    regions_percent=$(echo "${coverage_totals}" | jq -r '.regions.percent' | awk '{printf "%.2f", $0}')
    instantiations_count=$(echo "${coverage_totals}" | jq '.instantiations.count')
    instantiations_percent=$(echo "${coverage_totals}" | jq -r '.instantiations.percent' | awk '{printf "%.2f", $0}')

    coverage_summary="*Test Coverage Summary (${lines_percent}%)*\n*Functions:* ${functions_count} (${functions_percent}%), *Lines:* ${lines_count} (${lines_percent}%), *Regions:* ${regions_count} (${regions_percent}%), *Instantiations:* ${instantiations_count} (${instantiations_percent}%)"
    echo "COVERAGE_SUMMARY=${coverage_summary}" >> $GITHUB_ENV
    echo "LINE_COVERAGE_PERCENT=${lines_percent}" >> $GITHUB_ENV
else
    echo "COVERAGE_SUMMARY=Coverage data not found." >> $GITHUB_ENV
fi

# Process zombienet test file
if [ -f "${zombienet_test_file}" ]; then
  while IFS= read -r line; do
    if [[ "{$line}" == *"Passed tests:"* ]]; then
      zombienet_total_passed=$(echo "${line}" | grep -o -E '[[:space:]]+[0-9]+[[:space:]]+' | grep -o -E '[0-9]+') # colors handling
    elif [[ "{$line}" == *"Failed tests:"* ]]; then
      zombienet_total_failed=$(echo "${line}" | grep -o -E '[[:space:]]+[0-9]+[[:space:]]+' | grep -o -E '[0-9]+') # colors handling
    elif [[ "{$line}" == *"Done in"* ]]; then
      zombienet_total_runtime=$(echo "${line}" | sed 's/..$//' | grep -o -E '[0-9.]+')
    fi
  done < "$zombienet_test_file"
else
  echo "ZOMBIENET_TEST_SUMMARY=zombienet test output not found." >> $GITHUB_ENV
fi
passed_formatted="*Passed*: ${zombienet_total_passed}"
failed_formatted="*Failed*: ${zombienet_total_failed}"
runtime_formatted="*Runtime*: ${zombienet_total_runtime}s"
echo "ZOMBIENET_TEST_SUMMARY=*ZOMBIENET Tests |* ${passed_formatted}, ${failed_formatted}, ${runtime_formatted}" >> $GITHUB_ENV