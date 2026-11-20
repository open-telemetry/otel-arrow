#!/usr/bin/env bash
# Copyright The OpenTelemetry Authors
# SPDX-License-Identifier: Apache-2.0
#
# Runs the batch_ack model checks. Requires quint (npm i -g
# @informalsystems/quint) and Java 17+ for the TLC backend.
#
# Usage: ./check.sh [--large]
#   --large  also run random simulation on the large and wide bounds
#            (about 15 minutes). Exhaustive TLC on the large bounds takes
#            hours; see README.md.

set -euo pipefail
cd "$(dirname "$0")"

failures=0

run() {
  # run <label> <expect: ok|violation> <command...>
  local label=$1 expect=$2
  shift 2
  local start out status
  start=$(date +%s)
  out=$("$@" 2>&1) || true
  if grep -q '\[violation\]' <<<"$out"; then status=violation
  elif grep -q '\[ok\]' <<<"$out"; then status=ok
  else status=error
  fi
  printf '%-40s %-10s %4ss' "$label" "$status" "$(( $(date +%s) - start ))"
  if [[ $status == "$expect" ]]; then
    echo
  else
    echo "  (expected $expect)"
    echo "$out" | tail -20
    failures=$((failures + 1))
  fi
}

TLC=(quint verify mc.qnt --backend=tlc)
SIM=(quint run mc.qnt --max-samples=5000 --max-steps=25 --seed=1)
WITNESSES=(w_act_accept_buffer w_act_accept_flush w_act_wakeup
  w_act_begin_drain w_act_shutdown w_act_downstream_ack w_act_downstream_nack
  w_tail_retained w_tail_flushed_timer w_tail_flushed_size
  w_tail_flushed_shutdown w_completed_input w_split_boundary w_batch_error
  w_finish_failed w_inbound_exhausted w_outbound_exhausted
  w_nack_sibling_inflight w_data_after_nack w_ack_answer w_nack_answer
  w_timer_flush w_shutdown_flush w_accept_during_drain w_set_arrival_latched
  w_shutdown_after_drain w_shutdown_flushes_drain_input)

echo "== typecheck and scenarios"
quint typecheck batch_ack.qnt
quint typecheck mc.qnt
quint test batch_ack_test.qnt --main=batch_ack_test
quint test batch_ack_test.qnt --main=batch_ack_main_test

echo "== exhaustive checks (TLC)"
run "small: all invariants" ok "${TLC[@]}" --main=small --invariant=all_invariants
if [[ ${1:-} == "--large" ]]; then
  echo "== random simulation, 100k traces of 40 steps"
  for main in large wide; do
    run "$main: all invariants" ok quint run mc.qnt --main=$main \
      --invariant=all_invariants --max-samples=100000 --max-steps=40 --seed=1
  done
fi

echo "== fix toggle and mutations (TLC, counterexample expected)"
run "nofix: no_crash (F1)" violation "${TLC[@]}" --main=nofix --invariant=no_crash
run "nofix: no_crash_in_flush (F2)" violation "${TLC[@]}" --main=nofix --invariant=no_crash_in_flush
run "early: no_stuck_buffer (F4)" violation "${TLC[@]}" --main=early --invariant=no_stuck_buffer
run "mut_release: safety" violation "${TLC[@]}" --main=mut_release --invariant=safety
run "mut_tail: tail_attribution" violation "${TLC[@]}" --main=mut_tail --invariant=tail_attribution
run "mut_nack_delay: nack_answers_at_once" violation "${TLC[@]}" --main=mut_nack_delay --invariant=nack_answers_at_once

echo "== coverage (simulation, every witness must be reached)"
cov=$("${SIM[@]}" --main=cov --invariant=all_invariants --witnesses "${WITNESSES[@]}" 2>&1) || true
echo "$cov" | grep 'was witnessed'
if grep -q 'witnessed in 0 trace' <<<"$cov" || ! grep -q '\[ok\]' <<<"$cov"; then
  echo "coverage: FAILED"
  failures=$((failures + 1))
fi
crash=$("${SIM[@]}" --main=nofix --witnesses w_crash_accept w_crash_flush 2>&1) || true
echo "$crash" | grep 'was witnessed'
if grep -q 'witnessed in 0 trace' <<<"$crash"; then
  echo "crash coverage: FAILED"
  failures=$((failures + 1))
fi
early=$("${SIM[@]}" --main=early_cov --invariant=safety --witnesses w_early_wakeup 2>&1) || true
echo "$early" | grep 'was witnessed'
if grep -q 'witnessed in 0 trace' <<<"$early" || ! grep -q '\[ok\]' <<<"$early"; then
  echo "early wakeup coverage: FAILED"
  failures=$((failures + 1))
fi
# idle is enabled only once the node has stopped or crashed, so a trace
# shorter than --max-steps ended in a deadlock.
if ! grep -q 'min=26' <<<"$cov"; then
  echo "deadlock before stopped: FAILED"
  grep 'Trace length' <<<"$cov"
  failures=$((failures + 1))
fi

echo
if [[ $failures -eq 0 ]]; then echo "all checks passed"; else echo "$failures check(s) failed"; exit 1; fi
