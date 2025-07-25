#!/bin/bash
# get-last-version.sh - Get the last released version from git tags

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we want verbose output (default is verbose unless --quiet is passed)
QUIET=false
if [ $# -eq 1 ] && [ "$1" = "--quiet" ]; then
    QUIET=true
fi

if [ "$QUIET" = false ]; then
    echo -e "${YELLOW}Getting last released version...${NC}" >&2
fi

# Get the latest tag
LAST_TAG=$(git tag --list 'v*' --sort=-version:refname | head -n1)

if [ -z "$LAST_TAG" ]; then
    if [ "$QUIET" = false ]; then
        echo "No previous tags found" >&2
    fi
    LAST_VERSION="0.0.0"
else
    LAST_VERSION=${LAST_TAG#v}
fi

if [ "$QUIET" = false ]; then
    echo -e "${GREEN}Last released version: $LAST_VERSION${NC}" >&2
fi
echo "$LAST_VERSION"
