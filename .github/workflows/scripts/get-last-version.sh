#!/bin/bash
# get-last-version.sh - Get the last released version from git tags

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Getting last released version...${NC}" >&2

# Get the latest tag
LAST_TAG=$(git tag --list 'v*' --sort=-version:refname | head -n1)

if [ -z "$LAST_TAG" ]; then
    echo "No previous tags found" >&2
    LAST_VERSION="0.0.0"
else
    LAST_VERSION=${LAST_TAG#v}
fi

echo -e "${GREEN}Last released version: $LAST_VERSION${NC}" >&2
echo "$LAST_VERSION"