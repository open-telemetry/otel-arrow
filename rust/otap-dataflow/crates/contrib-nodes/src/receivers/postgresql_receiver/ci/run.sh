#!/usr/bin/env bash
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail
umask 077

artifact_dir="${OTEL_ARROW_INTEGRATION_ARTIFACT_DIR:?artifact directory is required}"
component_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 "$component_dir/ci/test_harness.py"
mkdir -p "$artifact_dir"
artifact_dir="$(cd "$artifact_dir" && pwd)"
export OTEL_ARROW_INTEGRATION_ARTIFACT_DIR="$artifact_dir"

if [[ -e "$artifact_dir/fixture-directory" || -e "$artifact_dir/compose-project" ]]; then
  echo "Previous PostgreSQL fixture state exists; run ci/cleanup.sh first." >&2
  exit 1
fi
finish() {
  status=$?
  trap - EXIT
  if ! bash "$component_dir/ci/cleanup.sh"; then
    echo "PostgreSQL fixture cleanup failed." >&2
    if [[ "$status" == 0 ]]; then
      status=1
    fi
  fi
  exit "$status"
}
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

fixture_directory="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/otel-pg-ci.XXXXXXXXXX")"
printf '%s\n' "$fixture_directory" > "$artifact_dir/fixture-directory"
project="otel-pg-ci-$(openssl rand -hex 8)"
printf '%s\n' "$project" > "$artifact_dir/compose-project"
export POSTGRESQL_FIXTURE_EXPORT_DIR="$fixture_directory"
export POSTGRESQL_FIXTURE_PORT=0

# These are disposable fixture credentials, never production credentials.
echo "::add-mask::postgres-admin"
echo "::add-mask::otel-reader-password"
compose=(docker compose --project-name "$project" --file "$component_dir/fixtures/tls/compose.yaml")
timeout --signal=TERM --kill-after=15s 5m \
  "${compose[@]}" up --build --detach --wait --wait-timeout 120

binding="$("${compose[@]}" port postgres 5432)"
if [[ ! "$binding" =~ ^127\.0\.0\.1:([0-9]+)$ ]]; then
  echo "PostgreSQL fixture did not publish exactly one loopback port." >&2
  exit 1
fi
export OTEL_ARROW_POSTGRESQL_TEST_PORT="${BASH_REMATCH[1]}"
export OTEL_ARROW_POSTGRESQL_TEST_CA_FILE="$fixture_directory/ca.pem"
test -s "$OTEL_ARROW_POSTGRESQL_TEST_CA_FILE"

cd "$component_dir/../../../../.."
test_name="receivers::postgresql_receiver::worker::tls_tests::live_scalar_and_snapshot_native_execution"
test_command=(
  cargo test --locked -p otel-arrow-dfe-contrib-nodes
  --no-default-features --features postgresql --lib "$test_name"
)
timeout --signal=TERM --kill-after=15s 30m \
  "${test_command[@]}" -- --ignored --exact --list \
  2>&1 \
  | tee "$artifact_dir/postgresql-test-list.log"
if ! grep -Fx "$test_name: test" "$artifact_dir/postgresql-test-list.log"; then
  echo "The native PostgreSQL live test was not selected." >&2
  exit 1
fi
timeout --signal=TERM --kill-after=15s 2m \
  "${test_command[@]}" -- --ignored --exact --nocapture \
  2>&1 \
  | tee "$artifact_dir/postgresql-receiver-test.log"
if ! grep -F "test result: ok. 1 passed; 0 failed; 0 ignored;" \
  "$artifact_dir/postgresql-receiver-test.log"; then
  echo "Expected exactly one executed, passing PostgreSQL live test." >&2
  exit 1
fi
