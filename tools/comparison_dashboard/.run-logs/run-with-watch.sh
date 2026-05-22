#!/bin/bash
# Run a single comparison-dashboard suite (or a --tests subset) under a stall
# watcher. Kills the orchestrator if its log file goes silent for STALL_LIMIT
# seconds, and enforces an absolute MAX_RUNTIME ceiling. Also tears down stray
# load-generator / backend-service / go-collector containers on kill.
#
# IMPORTANT: dashboard.py is launched with `python -u`. Without -u, Python's
# stdout buffering can hide progress and the watcher will false-positive kill
# a healthy run. Do not remove -u.
#
# Usage:
#   run-with-watch.sh <suite_path> [extra dashboard.py args...]
#
# Env vars:
#   STALL_LIMIT  seconds of log silence before kill (default 300)
#   MAX_RUNTIME  absolute wall-clock ceiling in seconds (default 2400)
#   PY           python interpreter (default: tools/comparison_dashboard venv)
#
# Exit codes:
#   0    suite ran to completion, dashboard.py exited 0
#   124  watcher killed it (stalled or hit MAX_RUNTIME)
#   *    dashboard.py's own non-zero exit
set -u
SUITE="${1:?suite_path required}"; shift || true
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# When dropped into the comparison_dashboard directory, override ROOT.
if [[ -f "$ROOT/dashboard.py" ]]; then :; else ROOT="$(pwd)"; fi
cd "$ROOT"
PY="${PY:-/home/jakedern/repos/otel-arrow/.venv/bin/python}"
LOG_DIR="${LOG_DIR:-$ROOT/.run-logs}"
mkdir -p "$LOG_DIR"
STALL_LIMIT="${STALL_LIMIT:-300}"
MAX_RUNTIME="${MAX_RUNTIME:-2400}"
base="$(basename "$SUITE" .yaml)"
log="$LOG_DIR/${base}.log"
: > "$log"

"$PY" -u dashboard.py run "$SUITE" "$@" >>"$log" 2>&1 &
pid=$!
start=$(date +%s)
killed_reason=""
while kill -0 "$pid" 2>/dev/null; do
    sleep 30
    now=$(date +%s)
    elapsed=$((now - start))
    if [[ $elapsed -ge $MAX_RUNTIME ]]; then
        killed_reason="MAX_RUNTIME (${elapsed}s) exceeded"
        break
    fi
    if [[ -f "$log" ]]; then
        mtime=$(stat -c '%Y' "$log")
        silent=$((now - mtime))
        if [[ $silent -ge $STALL_LIMIT ]]; then
            killed_reason="log silent for ${silent}s"
            break
        fi
    fi
done
if [[ -n "$killed_reason" ]]; then
    echo "WATCH-KILL: $killed_reason" >>"$log"
    kill -TERM "$pid" 2>/dev/null
    sleep 3
    kill -KILL "$pid" 2>/dev/null
    docker rm -f load-generator backend-service go-collector >/dev/null 2>&1
    wait "$pid" 2>/dev/null
    echo "STALLED $base: $killed_reason"
    exit 124
fi
wait "$pid"
rc=$?
echo "RC=$rc $base"
exit $rc
