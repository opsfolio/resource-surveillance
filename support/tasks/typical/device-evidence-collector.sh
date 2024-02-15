#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# Set environment variable
export SURVEILR_STATEDB_FS_PATH="/tmp/resource-surveillance-$(hostname).sqlite.db"

# Remove the current file if it exists
if [ -e "$SURVEILR_STATEDB_FS_PATH" ]; then
  rm "$SURVEILR_STATEDB_FS_PATH" || true  # Ignore errors during removal
fi

# Continue script execution even if the file removal fails

# Define the GitHub repository and directory URL
GITHUB_REPO_URL="https://api.github.com/repos/opsfolio/resource-surveillance/contents/support/tasks/typical"

# Fetch all file URLs using GitHub API and filter based on criteria
DEVICE_JSONL=$(curl -s "$GITHUB_REPO_URL" | grep -o 'https://raw.githubusercontent.com[^"]*' | grep 'device-.*\.jsonl$')

# Loop through the filtered URLs and execute surveilr ingest tasks
for url in $DEVICE_JSONL; do
    filename=$(basename "$url")
    echo "Processing file: $filename"
    curl -sL "$url" | surveilr ingest tasks
done

# Copy the created file to AWS using rclone
rclone -vv copy "${SURVEILR_STATEDB_FS_PATH}" sftp:/home/resource-surveillance/RSSD
