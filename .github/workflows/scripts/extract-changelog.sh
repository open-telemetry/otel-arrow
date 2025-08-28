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
    echo "Usage: $0 [version] [output_file] [--normalize]"
    echo ""
    echo "Extract changelog content from CHANGELOG.md"
    echo ""
    echo "Arguments:"
    echo "  version       Version to extract (e.g., 0.40.0). If not provided, extracts 'Unreleased' content"
    echo "  output_file   Output file (default: /tmp/changelog_content.txt or /tmp/release_content.txt)"
    echo "  --normalize   Normalize whitespace for PR body embedding (optional)"
    echo ""
    echo "Examples:"
    echo "  $0                           # Extract unreleased content"
    echo "  $0 /tmp/my_file.txt          # Extract unreleased content to custom file"
    echo "  $0 0.40.0                    # Extract content for version 0.40.0"
    echo "  $0 0.40.0 /tmp/my_file.txt   # Extract version content to custom file"
    echo "  $0 /tmp/my_file.txt --normalize  # Extract and normalize unreleased content"
    exit 1
}

VERSION=""
OUTPUT_FILE=""
NORMALIZE=false

for arg in "$@"; do
    case $arg in
        --normalize)
            NORMALIZE=true
            ;;
        *)
            if [[ "$arg" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                VERSION="$arg"
            else
                OUTPUT_FILE="$arg"
            fi
            ;;
    esac
done

# Set default output file if not specified
if [ -z "$OUTPUT_FILE" ]; then
    if [ -n "$VERSION" ]; then
        OUTPUT_FILE="/tmp/release_content.txt"
    else
        OUTPUT_FILE="/tmp/changelog_content.txt"
    fi
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

# Normalize whitespace for PR/Release body embedding if requested
if [ "$NORMALIZE" = true ]; then
    echo -e "${YELLOW}Normalizing whitespace for PR/Release body...${NC}"

    CONTENT=$(echo "$CONTENT" | awk '
    function clean_and_print_item() {
        if (current_item != "") {
            # Clean up extra spaces in content (but preserve sub-bullet indentation)
            if (current_item ~ /^  - /) {
                # For sub-bullets, only clean spaces after the "  - " prefix
                prefix = "  - "
                content = substr(current_item, 5)
                gsub(/  +/, " ", content)
                current_item = prefix content
            } else {
                # For main items, clean all extra spaces
                gsub(/  +/, " ", current_item)
            }
            print current_item
        }
    }
    
    BEGIN {
        current_item = ""
        in_list_item = 0
    }
    {
        # Main list item or sub-bullet (starts with "- " or "  - ")
        if ($0 ~ /^- / || $0 ~ /^  - /) {
            # Print previous item if we have one
            clean_and_print_item()
            current_item = $0
            in_list_item = 1
        }
        # Continuation line for main item (2 spaces, not sub-bullet)
        else if ($0 ~ /^  / && !($0 ~ /^  - /) && in_list_item) {
            # Remove leading spaces and append to current item
            continuation = $0
            sub(/^  /, "", continuation)
            current_item = current_item " " continuation
        }
        # Continuation line for sub-bullet (4+ spaces)
        else if ($0 ~ /^    / && in_list_item) {
            # Remove leading spaces and append to current item
            continuation = $0
            sub(/^    /, "", continuation)
            current_item = current_item " " continuation
        }
        # Non-list content or empty line
        else {
            # Print previous item if we have one
            clean_and_print_item()
            current_item = ""
            in_list_item = 0
            
            # Print non-empty, non-list lines
            if ($0 != "" && !($0 ~ /^- /)) {
                print $0
            }
        }
    }
    END {
        # Print final item if we have one
        clean_and_print_item()
    }')
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
    if [ "$NORMALIZE" = true ]; then
        echo -e "${GREEN}✓ Successfully extracted and normalized unreleased changelog content${NC}"
    else
        echo -e "${GREEN}✓ Successfully extracted unreleased changelog content${NC}"
    fi
else
    if [ "$NORMALIZE" = true ]; then
        echo -e "${GREEN}✓ Successfully extracted and normalized changelog content for version ${VERSION}${NC}"
    else
        echo -e "${GREEN}✓ Successfully extracted changelog content for version ${VERSION}${NC}"
    fi
fi
