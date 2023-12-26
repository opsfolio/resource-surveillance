#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# Set environment variable
export SURVEILR_STATEDB_FS_PATH="/tmp/resource-surveillance-$(hostname).sqlite.db"

# Remove old resource-surveillance DB file
rm -f ${SURVEILR_STATEDB_FS_PATH}

# Define an array of task URLs
tasks_urls=(
  "https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-security.jsonl"
  "https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-elaboration.jsonl"
  "https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-memory.jsonl"
  "https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-security.jsonl"
  "https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-storage.jsonl"
  "https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-containers.jsonl"
)

# Loop through the URLs and execute the curl command
for url in "${tasks_urls[@]}"; do
  curl -sL "$url" | surveilr ingest tasks
done