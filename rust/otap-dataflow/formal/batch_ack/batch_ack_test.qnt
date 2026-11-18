// -*- mode: Bluespec; -*-
// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Short deterministic scenarios, suitable for CI: quint test batch_ack_test.qnt

/// Scenarios with the F1/F2 fix on.
module batch_ack_test {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = false,
    MUTATION = "none", COVERAGE = true,
  ).* from "batch_ack"

  pure def flush_with(tok: int -> str, retain: bool): Choice =
    { batchError: false, tok: tok, retainTail: retain, finishFail: -1 }

  def answer_of(i: int): List[str] =
    if (st.answers.keys().contains(i)) st.answers.get(i) else []

  /// Every range list the abstraction generates passes the validator rules,
  /// and the rules reject the malformed examples from the Rust validator test.
  run derivedRangesPassValidatorTest =
    init.expect(and {
      1.to(4).forall(n => 0.to(n - 1).setOfMaps(TOKENS).forall(t =>
        contract_ok(derive_ranges(n, t), completed_of(n, t), n))),
      contract_ok([(0, 0), (0, 1), (1, 2)], Set(), 3),
      contract_ok([(1, 2), (2, 2)], Set(0, 3), 4),
      not(contract_ok([(0, 1), (0, 1)], Set(), 2)),
      not(contract_ok([(1, 1)], Set(), 2)),
      not(contract_ok([(0, 0)], Set(0), 1)),
      not(contract_ok([(0, 2)], Set(), 2)),
    })

  /// Mirrors test_otap_residual_holds_input_until_flushed: A is buffered,
  /// B triggers a size flush with outputs [A..B] and [B], and [B] is
  /// retained. B is answered only after its tail is flushed and Acked.
  run residualHoldsInputTest =
    init
      .then(accept_buffer(true))
      .then(accept_flush(true, flush_with(Map(0 -> "K", 1 -> "S"), true)))
      .expect(and {
        st.pending == [{ input: 1, tracked: true }],
        st.outbound == Map(0 -> [0, 1]),
        st.liveWakeup, all_invariants,
      })
      .then(downstream(0, true))
      .expect(answer_of(0) == ["ack"] and answer_of(1) == [] and all_invariants)
      .then(wakeup(flush_with(Map(0 -> "K"), false)))
      .then(downstream(1, true))
      .expect(and {
        answer_of(1) == ["ack"], st.inbound == Map(), st.pending == [],
        all_invariants,
      })

  /// One input split into two outputs: the first Nack answers at once and
  /// the later Ack of the sibling output is ignored.
  run nackWinsTest =
    init
      .then(accept_flush(true, flush_with(Map(0 -> "SB"), false)))
      .expect(st.outbound.keys() == Set(0, 1) and st.inbound == Map(0 -> 2))
      .then(downstream(0, false))
      .expect(answer_of(0) == ["nack"] and st.events.contains("nack_sibling_inflight"))
      .then(downstream(1, true))
      .expect(answer_of(0) == ["nack"] and st.inbound == Map() and all_invariants)

  /// An input the batcher completes without output is Acked by the flush.
  run completedInputAckedAtFlushTest =
    init
      .then(accept_buffer(true))
      .then(accept_flush(true, flush_with(Map(0 -> "C", 1 -> "K"), false)))
      .expect(and {
        answer_of(0) == ["ack"], answer_of(1) == [],
        st.outbound == Map(0 -> [1]), all_invariants,
      })

  /// D4: outbound exhaustion Nacks the owner but the output is still sent.
  run outboundExhaustedNacksButSendsTest =
    init
      .then(accept_buffer(true))
      .then(accept_flush(true, flush_with(Map(0 -> "SB", 1 -> "B"), false)))
      .expect(and {
        st.outbound.keys() == Set(0, 1),
        answer_of(1) == ["nack"],
        st.outResult.get(2) == "untracked",
        st.events.contains("outbound_exhausted"),
        all_invariants,
      })

  /// F1 with the fix: input during a drain stays buffered and is flushed by
  /// Shutdown. Its answer is left to the engine shutdown protocol (D3).
  run drainInputBufferedTest =
    init
      .then(begin_drain)
      .then(accept_buffer(true))
      .expect(not(st.crashed) and st.pending.length() == 1 and not(st.liveWakeup))
      .then(shutdown(flush_with(Map(0 -> "K"), false)))
      .expect(and {
        st.phase == "stopped", st.pending == [], st.outbound == Map(0 -> [0]),
        safety,
      })

  /// F2 with the fix: a size flush that retains a tail during a drain sends
  /// its outputs, keeps the tail, and Shutdown flushes the tail.
  run drainTailFlushTest =
    init
      .then(accept_buffer(true))
      .then(begin_drain)
      .then(accept_flush(true, flush_with(Map(0 -> "K", 1 -> "S"), true)))
      .expect(and {
        not(st.crashed), st.pending == [{ input: 1, tracked: true }],
        st.outbound == Map(0 -> [0, 1]), all_invariants,
      })
      .then(downstream(0, true))
      .then(shutdown(flush_with(Map(0 -> "K"), false)))
      .expect(and {
        answer_of(0) == ["ack"], st.outbound == Map(1 -> [1]), safety,
      })
}

/// The same scenarios on main (fix off) reproduce F1 and F2.
module batch_ack_main_test {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = false, EARLY_WAKEUPS = false,
    MUTATION = "none", COVERAGE = true,
  ).* from "batch_ack"

  pure def flush_with(tok: int -> str, retain: bool): Choice =
    { batchError: false, tok: tok, retainTail: retain, finishFail: -1 }

  /// F1: set_arrival fails during a drain; the buffered input is stranded.
  run f1CrashOnMainTest =
    init
      .then(begin_drain)
      .then(accept_buffer(true))
      .expect(and {
        st.crashed, st.events.contains("crash_accept"),
        st.inbound == Map(0 -> 1), st.answers == Map(),
      })

  /// F2: the flush stops after taking references; nothing is sent and
  /// neither input is answered.
  run f2CrashOnMainTest =
    init
      .then(accept_buffer(true))
      .then(begin_drain)
      .then(accept_flush(true, flush_with(Map(0 -> "K", 1 -> "S"), true)))
      .expect(and {
        st.crashed, st.events.contains("crash_flush"),
        st.outbound == Map(), st.answers == Map(), st.nextOutput == 0,
        st.inbound == Map(0 -> 2, 1 -> 3),
      })
}
