#!/usr/bin/env bash

set -euo pipefail

artifact_dir="${OTEL_ARROW_INTEGRATION_ARTIFACT_DIR:?artifact directory is required}"
mkdir -p "$artifact_dir"
output="$artifact_dir/cargo-test.log"

cd rust/otap-dataflow
OTAP_DF_RUN_USEREVENTS_E2E=1 cargo test --locked \
  -p otel-arrow-dfe-contrib-nodes \
  user_events_linux_e2e_smoke_when_available \
  --features user-events,otel-arrow-dfe-otap/crypto-ring \
  -- --nocapture 2>&1 | tee "$output"

if ! grep -Eq 'test result: ok\. 1 passed; 0 failed; 0 ignored;' "$output"; then
  echo "Expected exactly one User Events smoke test to execute." >&2
  exit 1
fi

if ! grep -Fq 'OTEL_ARROW_INTEGRATION_TEST_COMPLETED=user-events-linux' "$output"; then
  echo "User Events smoke test exited before exercising the integration path." >&2
  exit 1
fi
