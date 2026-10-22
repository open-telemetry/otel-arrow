#!/usr/bin/env bash
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

artifact_dir="${OTEL_ARROW_INTEGRATION_ARTIFACT_DIR:?artifact directory is required}"
component_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cleanup_status=0

if [[ -f "$artifact_dir/compose-project" ]]; then
  project="$(cat "$artifact_dir/compose-project")"
  if [[ ! "$project" =~ ^otel-pg-ci-[0-9a-f]{16}$ ]]; then
    echo "Invalid PostgreSQL CI Compose project; refusing cleanup." >&2
    exit 1
  fi
  export POSTGRESQL_FIXTURE_PORT=0
  export POSTGRESQL_FIXTURE_EXPORT_DIR="$(cat "$artifact_dir/fixture-directory")"
  compose=(docker compose --project-name "$project" --file "$component_dir/fixtures/tls/compose.yaml")
  # Do not upload container logs or Compose config: they may contain credentials.
  if ! timeout --signal=TERM --kill-after=5s 30s \
    "${compose[@]}" ps --all --format json > "$artifact_dir/postgresql-container-state.json"; then
    echo "Warning: failed to capture PostgreSQL container state." >&2
  fi
  if timeout --signal=TERM --kill-after=15s 60s "${compose[@]}" down --volumes; then
    rm -f "$artifact_dir/compose-project"
  else
    echo "Failed to remove the PostgreSQL CI Compose project." >&2
    cleanup_status=1
  fi
fi

if [[ -f "$artifact_dir/fixture-directory" && "$cleanup_status" == 0 ]]; then
  fixture_directory="$(cat "$artifact_dir/fixture-directory")"
  if [[ ! "$(basename "$fixture_directory")" =~ ^otel-pg-ci\.[a-zA-Z0-9]{10}$ ]]; then
    echo "Invalid PostgreSQL CI fixture directory; refusing cleanup." >&2
    exit 1
  fi
  rm -f "$fixture_directory/ca.pem" "$fixture_directory/username" "$fixture_directory/password"
  rmdir "$fixture_directory"
  rm -f "$artifact_dir/fixture-directory"
fi
exit "$cleanup_status"
