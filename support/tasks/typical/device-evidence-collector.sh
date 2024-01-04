#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# Set environment variable
export SURVEILR_STATEDB_FS_PATH="/tmp/resource-surveillance-$(hostname).sqlite.db"

# Remove the current file if it exists
if [ -e "$SURVEILR_STATEDB_FS_PATH" ]; then
  rm "$SURVEILR_STATEDB_FS_PATH"
fi


# Define the GitHub repository and API URL
GITHUB_REPO_URL="https://api.github.com/repos/opsfolio/resource-surveillance/contents/support/tasks/typical"

# Fetch the JSONL file URLs using GitHub API
JSONL_URLS=($(curl -s "$GITHUB_REPO_URL" | jq -r '.[].download_url'))

# Loop through the URLs and execute the curl command
for JSONL_URL in "${JSONL_URLS[@]}"; do
  curl -sL "$JSONL_URL" | surveilr ingest tasks
done

# Copy the created file to AWS using rclone
rclone -vv copy "${SURVEILR_STATEDB_FS_PATH}" sftp:/home/resource-surveillance/RSSD
