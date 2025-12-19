#!/bin/bash
set -euo pipefail

# Script to consolidate benchmark JSON files and trim commit messages
# Usage: consolidate-benchmarks.sh <input_dir> <output_file>

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <input_dir> <output_file>"
    exit 1
fi

INPUT_DIR="$1"
OUTPUT_FILE="$2"

echo "Consolidating benchmark JSON files from ${INPUT_DIR}..."
find "${INPUT_DIR}" -name "*.json" -type f | while read file; do
  echo "Processing: $file"
  cat "$file"
  echo
done

# Combine all benchmark JSON files into a single output (find them recursively)
find "${INPUT_DIR}" -name "*.json" -type f -exec cat {} \; | jq -s 'map(.[])' > "${OUTPUT_FILE}"

echo "Consolidated benchmark data:"
cat "${OUTPUT_FILE}"
