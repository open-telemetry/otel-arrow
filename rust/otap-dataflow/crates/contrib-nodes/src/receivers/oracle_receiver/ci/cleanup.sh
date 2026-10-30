#!/usr/bin/env bash

set -euo pipefail

artifact_dir="${OTEL_ARROW_INTEGRATION_ARTIFACT_DIR:?artifact directory is required}"
runner_temp="${RUNNER_TEMP:-$artifact_dir/tmp}"
mkdir -p "$artifact_dir"

cleanup_status=0
if docker inspect oracle-receiver-ci > /dev/null 2>&1; then
  if ! docker logs oracle-receiver-ci > "$artifact_dir/oracle-container.log" 2>&1; then
    echo "Warning: failed to capture Oracle container logs." >&2
  fi
  if ! docker rm --force oracle-receiver-ci > /dev/null 2>&1; then
    echo "Failed to remove Oracle container." >&2
    cleanup_status=1
  fi
fi
rm -f "$runner_temp/oracle-password"
exit "$cleanup_status"
