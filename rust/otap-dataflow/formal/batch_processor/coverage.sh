#!/usr/bin/env bash
# Vacuity check for BatchAggregation: checks that every risky path named in
# Coverage.tla is reachable in every sizing mode. One TLC run per path; each
# run is expected to find a violation of TargetUnreached.
#
# Usage: ./coverage.sh /path/to/tla2tools.jar
set -euo pipefail

jar="${1:?usage: $0 /path/to/tla2tools.jar}"
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

labels="residual_retained split_request split_request_acked
        residual_request_acked outbound_full_nack inbound_full_nack
        batch_error downstream_nack tracked_pass_through"
status=0

for mode in aligned rows bytes; do
    mode_labels="$labels"
    if [ "$mode" = bytes ]; then
        mode_labels="$labels fragment fragment_retained fragment_residual_acked"
    fi
    for label in $mode_labels; do
        cat > "$work/c.cfg" <<EOF
SPECIFICATION CovSpec
CONSTANTS
    Target = "$label"
    NumRequests = 2
    Mode = "$mode"
    MaxReqSize = 4
    MinSizes = {2, 3}
    MaxSizes = {0, 3, 4}
    InboundLimits = {1, 2}
    OutboundLimits = {1, 2}
    Overheads = {0, 1}
    AllowBatchError = TRUE
    AllowFallback = TRUE
    FreeSplit = TRUE
INVARIANT TargetUnreached
CHECK_DEADLOCK FALSE
EOF
        out="$(cd "$here" && java -XX:+UseParallelGC -cp "$jar" tlc2.TLC \
            -workers auto -noGenerateSpecTE -metadir "$work/states" \
            -config "$work/c.cfg" Coverage.tla 2>&1 || true)"
        rm -rf "$work/states"
        if grep -q "Invariant TargetUnreached is violated" <<<"$out"; then
            echo "reached      $mode $label"
        else
            echo "NOT REACHED  $mode $label"
            status=1
        fi
    done
done

exit "$status"
