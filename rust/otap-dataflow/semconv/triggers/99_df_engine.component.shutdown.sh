#!/usr/bin/env bash
# Trigger for `df_engine.component.shutdown`.
#
# Prefixed with `99_` so it sorts LAST in the glob — this trigger must
# run after all other triggers because it terminates the engine.
#
# Initiates a graceful shutdown via the admin API so pipeline nodes
# emit their shutdown events before the process is hard-killed by CI.
# The `wait=true` parameter blocks until shutdown completes or the
# timeout expires, ensuring the events have time to flow through the
# internal_telemetry pipeline to weaver.
set -euo pipefail

curl -X POST --max-time 15 -fsS \
  "http://127.0.0.1:14319/api/v1/groups/shutdown?wait=true&timeout_secs=10" || true

# Brief pause for the internal telemetry exporter to flush the
# shutdown events to weaver before the hard-kill step runs.
sleep 2
