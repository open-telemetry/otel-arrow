#!/bin/bash
# extract-changelog.sh - Extract unreleased content from CHANGELOG.md

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

OUTPUT_FILE="${1:-/tmp/changelog_content.txt}"

echo -e "${YELLOW}Extracting changelog content...${NC}"

# Check if CHANGELOG.md exists
if [ ! -f "CHANGELOG.md" ]; then
    echo -e "${RED}Error: CHANGELOG.md not found${NC}"
    exit 1
fi

# Get unreleased content
UNRELEASED_CONTENT=$(awk '/^## Unreleased/,/^## \[/ {
    if (/^## Unreleased/) next
    if (/^## \[/) exit
    print
}' CHANGELOG.md | sed '/^$/d')

if [ -z "$UNRELEASED_CONTENT" ]; then
    echo -e "${RED}Error: No unreleased content found in CHANGELOG.md${NC}"
    exit 1
fi

# Save to file
echo "$UNRELEASED_CONTENT" > "$OUTPUT_FILE"

echo -e "${GREEN}✓ Changelog content extracted to: $OUTPUT_FILE${NC}"
echo "Content:"
echo "----------------------------------------"
cat "$OUTPUT_FILE"
echo "----------------------------------------"
