#!/bin/bash
# validate-version-increment.sh - Validate that new version is greater than last version

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <last_version> <new_version>"
    echo "  last_version: Previous version (e.g., 0.39.0)"
    echo "  new_version: New version (e.g., 0.40.0)"
    exit 1
}

if [ $# -ne 2 ]; then
    usage
fi

LAST_VERSION="$1"
NEW_VERSION="$2"

echo -e "${YELLOW}Validating version increment...${NC}"
echo "Last version: $LAST_VERSION"
echo "New version: $NEW_VERSION"

# Simple version comparison (assumes semantic versioning)
if [ "$LAST_VERSION" != "0.0.0" ]; then
    if ! printf '%s\n%s\n' "$LAST_VERSION" "$NEW_VERSION" | sort -V -C; then
        echo -e "${RED}Error: New version $NEW_VERSION is not greater than last version $LAST_VERSION${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✓ Version increment is valid${NC}"
