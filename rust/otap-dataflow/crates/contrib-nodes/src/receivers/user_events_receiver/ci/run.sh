#!/usr/bin/env bash

set -euo pipefail

cd rust/otap-dataflow
OTAP_DF_RUN_USEREVENTS_E2E=1 cargo test --locked \
  -p otel-arrow-dfe-contrib-nodes \
  user_events_linux_e2e_smoke_when_available \
  --features user-events,otel-arrow-dfe-otap/crypto-ring
