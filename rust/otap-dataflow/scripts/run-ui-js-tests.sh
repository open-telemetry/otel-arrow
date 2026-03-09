#!/bin/bash
# Run admin UI JavaScript module tests.
#
# Usage:
#   ./scripts/run-ui-js-tests.sh
#   ./scripts/run-ui-js-tests.sh <test-file-or-glob>
#
# Examples:
#   ./scripts/run-ui-js-tests.sh
#   ./scripts/run-ui-js-tests.sh crates/admin/ui/js/tests/graph-renderer.test.mjs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

NODE_BIN="${NODE_BIN:-node}"
DEFAULT_PATTERN="crates/admin/ui/js/tests/*.test.mjs"
TARGET_PATTERN="${1:-$DEFAULT_PATTERN}"

if ! command -v "$NODE_BIN" > /dev/null 2>&1; then
  echo "ERROR: Node.js executable not found: $NODE_BIN"
  echo "Set NODE_BIN=/path/to/node if needed."
  exit 1
fi

cd "$PROJECT_DIR"

shopt -s nullglob
matches=( $TARGET_PATTERN )
shopt -u nullglob

if [ "${#matches[@]}" -eq 0 ]; then
  echo "ERROR: No JS tests matched pattern: $TARGET_PATTERN"
  exit 1
fi

echo "Running UI JS module tests (${#matches[@]} files)..."
"$NODE_BIN" --test --experimental-default-type=module "${matches[@]}"
