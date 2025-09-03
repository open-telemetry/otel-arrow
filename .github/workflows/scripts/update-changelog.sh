#!/bin/bash
# update-changelog.sh - Update CHANGELOG.md with new release section

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <version> [changelog_content_file]"
    echo "  version: Release version (e.g., 0.40.0)"
    echo "  changelog_content_file: File containing unreleased content (default: /tmp/changelog_content.txt)"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

NEW_VERSION="$1"
CHANGELOG_CONTENT_FILE="${2:-/tmp/changelog_content.txt}"

echo -e "${YELLOW}Updating CHANGELOG.md...${NC}"

# Check if CHANGELOG.md exists
if [ ! -f "CHANGELOG.md" ]; then
    echo -e "${RED}Error: CHANGELOG.md not found${NC}"
    exit 1
fi

# Check if changelog content file exists
if [ ! -f "$CHANGELOG_CONTENT_FILE" ]; then
    echo -e "${RED}Error: Changelog content file not found: $CHANGELOG_CONTENT_FILE${NC}"
    exit 1
fi

# Create new changelog with release section
RELEASE_DATE=$(date -u +"%Y-%m-%d")

# Create temporary file with new content
{
    # Keep header content and unreleased section header
    awk '/^## Unreleased/ {print; exit} {print}' CHANGELOG.md
    echo ""

    # Add new release section
    echo "## [${NEW_VERSION}](https://github.com/open-telemetry/otel-arrow/releases/tag/v${NEW_VERSION}) - ${RELEASE_DATE}"
    echo ""

    # Add unreleased content
    cat "$CHANGELOG_CONTENT_FILE"
    echo ""

    # Add rest of changelog (starting from first existing release)
    awk '/^## \[.*\]/ {found=1} found {print}' CHANGELOG.md
} > CHANGELOG.md.tmp

mv CHANGELOG.md.tmp CHANGELOG.md
echo -e "${GREEN}✓ Updated CHANGELOG.md with release $NEW_VERSION${NC}"
