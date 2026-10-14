#!/usr/bin/env bash
# Vacuity check for Completion: checks that every path named in Coverage.tla
# that applies to a topology is reachable with that topology's config. One TLC
# run per path; each run is expected to find a violation of TargetUnreached.
#
# Usage: ./coverage.sh /path/to/tla2tools.jar
set -euo pipefail

jar="${1:?usage: $0 /path/to/tla2tools.jar}"
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

labels_for() {
    case "$1" in
        T1) echo "ack_answered nack_answered send_blocked pending_used
                  expired_delivery abandoned error_answer split_partial" ;;
        T2) echo "ack_answered nack_answered send_blocked
                  detached_copy_answered fanout_all_partial
                  secondary_ignored" ;;
        T3) echo "ack_answered nack_answered send_blocked retry_parked
                  retried_then_acked" ;;
        # Fanout `none` only answers Ack, so retry never sees a Nack in T4.
        T4) echo "ack_answered detached_copy_answered" ;;
        T5) echo "ack_answered nack_answered detached_copy_answered
                  sink_ack" ;;
    esac
}

status=0
for topo in T1 T2 T3 T4 T5; do
    for label in $(labels_for "$topo"); do
        # Same constants as the topology's config, with the coverage spec,
        # the target, and only the TargetUnreached invariant.
        {
            echo "SPECIFICATION CovSpec"
            sed -n '/^CONSTANTS/,/^$/p' "$here/$topo.cfg"
            echo "    Target = \"$label\""
            echo "INVARIANT TargetUnreached"
            echo "CHECK_DEADLOCK FALSE"
        } > "$work/c.cfg"
        out="$(cd "$here" && java -XX:+UseParallelGC -cp "$jar" tlc2.TLC \
            -workers auto -noGenerateSpecTE -metadir "$work/states" \
            -config "$work/c.cfg" Coverage.tla 2>&1 || true)"
        rm -rf "$work/states"
        if grep -q "Invariant TargetUnreached is violated" <<<"$out"; then
            echo "reached      $topo $label"
        else
            echo "NOT REACHED  $topo $label"
            status=1
        fi
    done
done

exit "$status"
