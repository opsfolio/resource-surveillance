#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# Set environment variable
export SURVEILR_STATEDB_FS_PATH="/tmp/resource-surveillance-$(hostname).sqlite.db"

# Set the directory for cnquery-packs
CNQUERY_PACKS_DIR="$HOME/cnquery-packs"

# Check if cnquery-packs directory already exists
if [ ! -d "$CNQUERY_PACKS_DIR" ]; then
  # Clone Cnquery-packs to the user home folder
  cd "$HOME"
  git clone https://github.com/mondoohq/cnquery-packs
else
  echo "cnquery-packs repository already exists. Skipping cloning."
fi

# Remove the current file if it exists
if [ -e "$SURVEILR_STATEDB_FS_PATH" ]; then
  rm "$SURVEILR_STATEDB_FS_PATH" || true  # Ignore errors during removal
fi

# Continue script execution even if the file removal fails

# Define the GitHub repository and directory URL
GITHUB_REPO_URL="https://api.github.com/repos/opsfolio/resource-surveillance/contents/support/tasks/typical"

# Fetch all file URLs using GitHub API
ALL_URLS=($(curl -s "$GITHUB_REPO_URL" | grep -o 'https://raw.githubusercontent.com[^"]*'))

# Loop through the URLs and execute the curl command for those ending with ".jsonl"
for URL in "${ALL_URLS[@]}"; do
  if [[ "$URL" == *".jsonl" ]]; then
    curl -sL "$URL" | surveilr ingest tasks
  fi
done

# Copy the created file to AWS using rclone
rclone -vv copy "${SURVEILR_STATEDB_FS_PATH}" sftp:/home/resource-surveillance/RSSD
