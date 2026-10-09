#!/usr/bin/env bash

set -euo pipefail

artifact_dir="${OTEL_ARROW_INTEGRATION_ARTIFACT_DIR:?artifact directory is required}"
mkdir -p "$artifact_dir"

cd rust/otap-dataflow
OTAP_DF_RUN_USEREVENTS_E2E=1 cargo test --locked \
  -p otel-arrow-dfe-contrib-nodes \
  user_events_linux_e2e_smoke_when_available \
  --features user-events,otel-arrow-dfe-otap/crypto-ring
