#!/bin/bash
# validate-version.sh - Validate version format and repository state

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <version>"
    echo "  version: Version in format X.Y.Z (e.g., 0.40.0)"
    exit 1
}

if [ $# -ne 1 ]; then
    usage
fi

VERSION="$1"

echo -e "${YELLOW}Validating version and repository state...${NC}"

# Validate version format
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Version must be in format X.Y.Z (e.g., 0.40.0)${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Version format is valid: $VERSION${NC}"

# Check if repository is clean
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: Repository has uncommitted changes${NC}"
    git status
    exit 1
fi

echo -e "${GREEN}✓ Repository is clean${NC}"

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

echo -e "${GREEN}✓ In a git repository${NC}"

echo -e "${GREEN}All validations passed!${NC}"
