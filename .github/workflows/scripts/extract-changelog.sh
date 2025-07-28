#!/bin/bash
# extract-changelog.sh - Extract changelog content from CHANGELOG.md
# Can extract either unreleased content or content for a specific version

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to show usage
usage() {
    echo "Usage: $0 [version] [output_file]"
    echo ""
    echo "Extract changelog content from CHANGELOG.md"
    echo ""
    echo "Arguments:"
    echo "  version       Version to extract (e.g., 0.40.0). If not provided, extracts 'Unreleased' content"
    echo "  output_file   Output file (default: /tmp/changelog_content.txt or /tmp/release_content.txt)"
    echo ""
    echo "Examples:"
    echo "  $0                           # Extract unreleased content"
    echo "  $0 /tmp/my_file.txt          # Extract unreleased content to custom file"
    echo "  $0 0.40.0                    # Extract content for version 0.40.0"
    echo "  $0 0.40.0 /tmp/my_file.txt   # Extract version content to custom file"
    exit 1
}

# Parse arguments - handle different calling patterns
VERSION=""
OUTPUT_FILE=""

if [ $# -eq 0 ]; then
    # No arguments - extract unreleased content
    OUTPUT_FILE="/tmp/changelog_content.txt"
elif [ $# -eq 1 ]; then
    # One argument - could be version or output file
    if [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        # It's a version
        VERSION="$1"
        OUTPUT_FILE="/tmp/release_content.txt"
    else
        # It's an output file for unreleased content
        OUTPUT_FILE="$1"
    fi
elif [ $# -eq 2 ]; then
    # Two arguments - version and output file
    VERSION="$1"
    OUTPUT_FILE="$2"
else
    echo -e "${RED}Error: Too many arguments${NC}"
    usage
fi

# Check if CHANGELOG.md exists
if [ ! -f "CHANGELOG.md" ]; then
    echo -e "${RED}Error: CHANGELOG.md not found${NC}"
    exit 1
fi

if [ -z "$VERSION" ]; then
    # Extract unreleased content
    echo -e "${YELLOW}Extracting unreleased changelog content...${NC}"
    
    CONTENT=$(awk '/^## Unreleased/,/^## \[/ {
        if (/^## Unreleased/) next
        if (/^## \[/) exit
        print
    }' CHANGELOG.md | sed '/^$/d')

    if [ -z "$CONTENT" ]; then
        echo -e "${RED}Error: No unreleased content found in CHANGELOG.md${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Found unreleased content${NC}"
else
    # Extract content for specific version
    echo -e "${YELLOW}Extracting changelog content for version ${BLUE}${VERSION}${NC}..."
    
    # Validate version format
    if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo -e "${RED}Error: Version must be in format X.Y.Z (e.g., 0.40.0)${NC}"
        exit 1
    fi

    CONTENT=$(awk "BEGIN{found=0} /^## \[$VERSION\]/ {found=1; next} found && /^## \[/ {exit} found {print}" CHANGELOG.md | sed '/^$/d')

    if [ -z "$CONTENT" ]; then
        echo -e "${RED}Error: No release content found for version $VERSION in CHANGELOG.md${NC}"
        echo -e "${YELLOW}Available versions in CHANGELOG.md:${NC}"
        grep "^## \[" CHANGELOG.md || echo "No versioned entries found"
        exit 1
    fi

    echo -e "${GREEN}✓ Found release content for version ${VERSION}${NC}"
fi

# Save to file
echo "$CONTENT" > "$OUTPUT_FILE"

echo -e "${GREEN}✓ Content saved to: ${OUTPUT_FILE}${NC}"
echo ""
echo -e "${YELLOW}Changelog content:${NC}"
echo "----------------------------------------"
cat "$OUTPUT_FILE"
echo "----------------------------------------"

echo ""
if [ -z "$VERSION" ]; then
    echo -e "${GREEN}✓ Successfully extracted unreleased changelog content${NC}"
else
    echo -e "${GREEN}✓ Successfully extracted changelog content for version ${VERSION}${NC}"
fi
