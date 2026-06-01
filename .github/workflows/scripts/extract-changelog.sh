#!/bin/bash
# extract-changelog.sh - Extract a release section from a CHANGELOG.md file.
#
# This script is used by the release workflows to pull out a specific
# version's notes from a chloggen-managed CHANGELOG (go/CHANGELOG.md or
# rust/otap-dataflow/CHANGELOG.md). It matches the chloggen heading
# format `## vX.Y.Z`.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 --version <ver> [--file <path>] [--allow-missing] [output_file]"
    echo ""
    echo "Extract a release section from a chloggen-managed CHANGELOG.md."
    echo "Output preserves the section as rendered by chloggen and can be"
    echo "embedded directly in a PR / release body."
    echo ""
    echo "Required:"
    echo "  --version <ver>     Version to extract (e.g., 0.40.0). Matches '## v<ver>'."
    echo ""
    echo "Options:"
    echo "  --file <path>       CHANGELOG file to read (default: CHANGELOG.md)."
    echo "  --allow-missing     Exit 0 with empty output if the version section is absent."
    echo "  output_file         Output file (default: /tmp/release_content.txt)."
    echo ""
    echo "Examples:"
    echo "  $0 --version 0.48.0 --file go/CHANGELOG.md /tmp/go_notes.txt"
    echo "  $0 --version 0.48.0 --file rust/otap-dataflow/CHANGELOG.md --allow-missing"
    exit 1
}

VERSION=""
FILE="CHANGELOG.md"
OUTPUT_FILE=""
ALLOW_MISSING=false

while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="${2:-}"
            shift 2
            ;;
        --file)
            FILE="${2:-}"
            shift 2
            ;;
        --allow-missing)
            ALLOW_MISSING=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        --*)
            echo -e "${RED}Error: unknown option: $1${NC}"
            usage
            ;;
        *)
            if [ -n "$OUTPUT_FILE" ]; then
                echo -e "${RED}Error: too many positional arguments${NC}"
                usage
            fi
            OUTPUT_FILE="$1"
            shift
            ;;
    esac
done

if [ -z "$VERSION" ]; then
    echo -e "${RED}Error: --version is required${NC}"
    usage
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Version must be in format X.Y.Z (e.g., 0.40.0)${NC}"
    exit 1
fi

if [ -z "$OUTPUT_FILE" ]; then
    OUTPUT_FILE="/tmp/release_content.txt"
fi

if [ ! -f "$FILE" ]; then
    echo -e "${RED}Error: file not found: $FILE${NC}"
    exit 1
fi

echo -e "${YELLOW}Extracting changelog content for version ${BLUE}${VERSION}${NC} from ${BLUE}${FILE}${NC}..."

# Pull the section between '## v<VERSION>' and the next '## ' heading.
CONTENT=$(awk -v ver="v${VERSION}" '
    BEGIN { found = 0 }
    {
        if (!found) {
            if ($0 == "## " ver) { found = 1; next }
            next
        }
        if ($0 ~ /^## /) { exit }
        if ($0 ~ /^<!-- previous-version -->/) { next }
        print
    }
' "$FILE")

# Strip leading/trailing blank lines but preserve internal blank lines so
# chloggen section headers (### ...) keep their spacing.
CONTENT=$(echo "$CONTENT" | awk '
    { lines[NR] = $0 }
    END {
        first = 1
        for (i = 1; i <= NR; i++) if (lines[i] != "") { first = i; break }
        last = NR
        for (i = NR; i >= 1; i--) if (lines[i] != "") { last = i; break }
        for (i = first; i <= last; i++) print lines[i]
    }
')

if [ -z "$CONTENT" ]; then
    if [ "$ALLOW_MISSING" = true ]; then
        echo -e "${YELLOW}No section found for v${VERSION} in ${FILE}; writing empty output (allow-missing).${NC}"
        : > "$OUTPUT_FILE"
        exit 0
    fi
    echo -e "${RED}Error: No release content found for version ${VERSION} in ${FILE}${NC}"
    echo -e "${YELLOW}Available versions in ${FILE}:${NC}"
    grep -E "^## v[0-9]" "$FILE" || echo "No versioned entries found"
    exit 1
fi

echo -e "${GREEN}✓ Found release content for version ${VERSION}${NC}"

echo "$CONTENT" > "$OUTPUT_FILE"

echo -e "${GREEN}✓ Content saved to: ${OUTPUT_FILE}${NC}"
echo ""
echo -e "${YELLOW}Changelog content:${NC}"
echo "----------------------------------------"
cat "$OUTPUT_FILE"
echo "----------------------------------------"
echo ""
echo -e "${GREEN}✓ Successfully extracted changelog content for version ${VERSION}${NC}"
