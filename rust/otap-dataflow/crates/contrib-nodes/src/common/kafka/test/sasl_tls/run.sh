#!/usr/bin/env bash
set -euo pipefail

fixture_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
compose=(docker compose --project-directory "$fixture_dir" -f "$fixture_dir/docker-compose.yaml")
: "${KAFKA_SASL_TLS_PORT:=39093}"
KAFKA_SASL_TLS_HOST=host.docker.internal
OTEL_ARROW_REPO_ROOT="$(git -C "$fixture_dir" rev-parse --show-toplevel)"
export KAFKA_SASL_TLS_PORT
export KAFKA_SASL_TLS_HOST
export OTEL_ARROW_REPO_ROOT

cleanup() {
  "${compose[@]}" down --volumes --remove-orphans
}

failure_logs() {
  "${compose[@]}" logs broker
}

trap cleanup EXIT
trap failure_logs ERR

"${compose[@]}" down --volumes --remove-orphans
"${compose[@]}" up --detach --wait --wait-timeout 180 broker
"${compose[@]}" run --build --rm --no-deps receiver-test
