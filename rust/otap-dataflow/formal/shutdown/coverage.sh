#!/usr/bin/env bash
# Vacuity check for Shutdown: checks that every path named in Coverage.tla
# that applies to a config is reachable with that config's constants. One TLC
# run per path; each run is expected to find a violation of TargetUnreached.
#
# Usage: ./coverage.sh /path/to/tla2tools.jar [/path/to/tlc-state-dir]
# TLC state goes to the second argument (default: a new temporary directory)
# and is left in place.
set -euo pipefail

jar="${1:?usage: $0 /path/to/tla2tools.jar [/path/to/tlc-state-dir]}"
here="$(cd "$(dirname "$0")" && pwd)"
work="${2:-$(mktemp -d)}"
mkdir -p "$work"

labels_for() {
    case "$1" in
        T1) echo "real_ack real_nack forced_nack shut_nack deadline_fired
                  all_stopped mgr_pending_sends disp_pending_sends
                  receiver_forced_stop" ;;
        T2) echo "real_ack wakeup_after_latch holding_input_after_latch
                  deadline_fired all_stopped" ;;
        # Retry's shutdown Nack (rejected requeue) cannot reach a waiting
        # receiver: its scheduler latches only after the receiver drained.
        T3) echo "real_ack real_nack retained_converted
                  closed_before_shutdown burst_path all_stopped" ;;
        T4) echo "real_ack shut_nack fanout_inflight_at_shutdown all_stopped" ;;
        T4_timeout) echo "timeout_nack tick_during_drain" ;;
        T5) echo "real_ack shut_nack router_parked_at_latch
                  wakeup_after_latch all_stopped" ;;
    esac
}

status=0
for topo in T1 T2 T3 T4 T4_timeout T5; do
    for label in $(labels_for "$topo"); do
        # Same constants as the topology's config, with the coverage spec,
        # the target, and only the TargetUnreached invariant.
        cfg="$work/$topo-$label.cfg"
        {
            echo "SPECIFICATION CovSpec"
            sed -n '/^CONSTANTS/,/^$/p' "$here/$topo.cfg"
            echo "    CovTarget = \"$label\""
            echo "INVARIANT TargetUnreached"
            echo "CHECK_DEADLOCK FALSE"
        } > "$cfg"
        out="$(cd "$here" && java -XX:+UseParallelGC -cp "$jar" tlc2.TLC \
            -workers auto -noGenerateSpecTE -metadir "$work/$topo-$label" \
            -config "$cfg" Coverage.tla 2>&1 || true)"
        if grep -q "Invariant TargetUnreached is violated" <<<"$out"; then
            echo "reached      $topo $label"
        else
            echo "NOT REACHED  $topo $label"
            status=1
        fi
    done
done

exit "$status"
