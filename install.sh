#!/bin/sh

# Set the SURVEILR_HOME environment variable to the current directory if not already set
: ${SURVEILR_HOME:=$(pwd)}

# Set the repository owner and repository name
REPO_OWNER="opsfolio"
REPO_NAME="resource-surveillance"

# GitHub API URL for the latest release
API_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"

# Fetch the latest release tag from GitHub API and construct the download URL
DOWNLOAD_URL=$(curl -s $API_URL | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | xargs -I {} echo "https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/{}/resource-surveillance_{}_x86_64-unknown-linux-musl.tar.gz")

# Download and extract the binary to the SURVEILR_HOME directory
curl -sL $DOWNLOAD_URL | tar -xz -C $SURVEILR_HOME

echo "Download and extraction complete. Binary is in $SURVEILR_HOME"
