// -*- mode: Bluespec; -*-
// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Ack/Nack aggregation of the batch processor
/// (crates/core-nodes/src/processors/batch_processor/mod.rs).
///
/// One SignalBuffer is modeled: buffers never share slots or wakeups. Payloads
/// are abstracted to input identities and the batcher returns any range list
/// that validate_input_ranges accepts (see spec.md, "Model abstraction").
/// Inputs get unique ids; this matches the generational slot keys, which make
/// a freed or reused slot unreachable from a stale response.
///
/// Action and helper names mirror the Rust functions. Every step is one
/// process() call, which the engine runs to completion before the next one.
module batch_ack {
  // ---------------------------------------------------------------------
  // Parameters
  // ---------------------------------------------------------------------

  /// Number of inputs the environment may deliver.
  const MAX_INPUTS: int
  /// inbound_request_limit.
  const INBOUND_CAP: int
  /// outbound_request_limit.
  const OUTBOUND_CAP: int
  /// F1/F2 fix: set_arrival treats WakeupError::ShuttingDown as success
  /// without an armed wakeup. false reproduces main.
  const FIX_SHUTDOWN_WAKEUP: bool
  /// D9/F4: allow the live wakeup to be delivered before its deadline.
  const EARLY_WAKEUPS: bool
  /// Vacuity check: "none", "release_before_refs", "tail_untracked" or
  /// "nack_delay".
  const MUTATION: str
  /// Record coverage events. Adds state, so it is off for the big runs.
  const COVERAGE: bool

  // ---------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------

  /// BatchPortion: the delivery owner of one buffered entry.
  type Portion = { input: int, tracked: bool }

  /// Inclusive range of indices into the drained pending list.
  type Range = (int, int)

  /// One batcher result. tok encodes the ranges (see derive_ranges);
  /// finishFail is the index of the output whose finish_batch fails, or -1.
  type Choice = {
    batchError: bool,
    tok: int -> str,
    retainTail: bool,
    finishFail: int,
  }

  type State = {
    // SignalBuffer and scheduler
    phase: str,                  // "running" | "draining" | "stopped"
    latched: bool,               // local scheduler shutdown latch
    crashed: bool,               // process() returned Err
    pending: List[Portion],      // Inputs.context (payloads abstracted)
    inbound: int -> int,         // live inbound slot -> BatchContext.outbound
    outbound: int -> List[int],  // live outbound slot -> owner inputs
    liveWakeup: bool,            // scheduler holds this buffer's wakeup
    wakeupArmed: bool,           // SignalBuffer.wakeup_armed
    nextInput: int,
    nextOutput: int,
    // Ghost state for the invariants
    trackedInputs: Set[int],
    answers: int -> List[str],   // upstream answers per input
    outputsOf: int -> Set[int],  // outputs that carried data of the input
    outResult: int -> str,       // "inflight" | "ack" | "nack" | "dropped"
                                 // | "untracked"
    events: Set[str],            // coverage
    tailInputs: Set[int],        // coverage: inputs with a retained tail
    drainInputs: Set[int],       // coverage: inputs accepted during a drain
    failedIn: Set[int],          // mutation nack_delay: Nacked, not answered
  }

  var st: State

  /// Tokens per input index, consumed by derive_ranges:
  /// C = completed without output, K = keep the output open,
  /// B = close the output after this input, S = split (close an output at
  /// this input and continue the next one from it), SB = split and close.
  pure val TOKENS = Set("C", "K", "B", "S", "SB")

  // ---------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------

  pure def remove(m: int -> a, k: int): int -> a =
    m.keys().exclude(Set(k)).mapBy(x => m.get(x))

  pure def has(l: List[int], i: int): bool =
    l.foldl(false, (acc, x) => acc or x == i)

  pure def count(l: List[int], i: int): int =
    l.foldl(0, (acc, x) => if (x == i) acc + 1 else acc)

  pure def owners_of(ps: List[Portion]): List[int] =
    ps.foldl([], (acc, p) => if (p.tracked) acc.append(p.input) else acc)

  def ev(s: State, e: str): State =
    if (COVERAGE) { ...s, events: s.events.union(Set(e)) } else s

  def ev_if(s: State, cond: bool, e: str): State =
    if (cond) ev(s, e) else s

  pure def answer(s: State, i: int, kind: str): State =
    val prev = if (s.answers.keys().contains(i)) s.answers.get(i) else []
    { ...s, answers: s.answers.put(i, prev.append(kind)) }

  // ---------------------------------------------------------------------
  // Batcher abstraction
  // ---------------------------------------------------------------------

  type RangeAcc = { outs: List[Range], open: bool, start: int }

  pure def tok_step(a: RangeAcc, j: int, t: str): RangeAcc =
    if (t == "C")
      { outs: if (a.open) a.outs.append((a.start, j - 1)) else a.outs,
        open: false, start: a.start }
    else
      tok_cover(a, j, t)

  pure def tok_cover(a: RangeAcc, j: int, t: str): RangeAcc =
    val s0 = if (a.open) a.start else j
    val split = t == "S" or t == "SB"
    val outs1 = if (split) a.outs.append((s0, j)) else a.outs
    val s1 = if (split) j else s0
    if (t == "B" or t == "SB")
      { outs: outs1.append((s1, j)), open: false, start: s1 }
    else
      { outs: outs1, open: true, start: s1 }

  /// Output ranges for n drained entries. By construction the result passes
  /// validate_input_ranges together with completed_of (checked in tests).
  pure def derive_ranges(n: int, tok: int -> str): List[Range] =
    val acc = range(0, n).foldl({ outs: [], open: false, start: 0 },
                                (a, j) => tok_step(a, j, tok.get(j)))
    if (acc.open) acc.outs.append((acc.start, n - 1)) else acc.outs

  pure def completed_of(n: int, tok: int -> str): Set[int] =
    0.to(n - 1).filter(j => tok.get(j) == "C")

  /// The structural part of validate_input_ranges: ordered non-empty
  /// in-bounds ranges, overlap only at a shared boundary, and every index
  /// either covered by an output or completed, never both.
  pure def contract_ok(outs: List[Range], completed: Set[int], n: int): bool =
    and {
      outs.indices().forall(k => and {
        0 <= outs[k]._1, outs[k]._1 <= outs[k]._2, outs[k]._2 < n,
        k == 0 or outs[k]._1 >= outs[k - 1]._2,
        k == 0 or outs[k]._1 <= outs[k - 1]._2 + 1
          or 0.to(n - 1).filter(j => outs[k - 1]._2 < j and j < outs[k]._1)
               .subseteq(completed),
      }),
      0.to(n - 1).forall(j =>
        outs.indices().exists(k => outs[k]._1 <= j and j <= outs[k]._2)
          != completed.contains(j)),
    }

  /// All batcher results for n drained entries.
  pure def choices(n: int): Set[Choice] =
    val ok = tuples(0.to(n - 1).setOfMaps(TOKENS), Set(true, false),
                    (-1).to(1))
      .map(t => { batchError: false, tok: t._1, retainTail: t._2,
                  finishFail: t._3 })
    val err = Set({ batchError: true, tok: 0.to(n - 1).mapBy(j => "K"),
                    retainTail: false, finishFail: -1 })
    ok.union(err)

  // ---------------------------------------------------------------------
  // SignalBuffer::handle_partial_responses
  // ---------------------------------------------------------------------

  def partial_one(s: State, ok: bool, i: int): State =
    if (not(s.inbound.keys().contains(i)))
      s
    else if (MUTATION == "nack_delay" and not(ok) and s.inbound.get(i) > 1)
      // A Nack only lowers the count; the input is answered at its last
      // reference.
      { ...s, inbound: s.inbound.set(i, s.inbound.get(i) - 1), failedIn: s.failedIn.union(Set(i)) }
    else if (not(ok))
      answer({ ...s, inbound: remove(s.inbound, i) }, i, "nack")
        .ev("nack_answer")
    else if (s.inbound.get(i) > 1)
      { ...s, inbound: s.inbound.set(i, s.inbound.get(i) - 1) }
    else
      answer({ ...s, inbound: remove(s.inbound, i) }, i, if (s.failedIn.contains(i)) "nack" else "ack")
        .ev("ack_answer")

  def handle_partial_responses(s: State, ok: bool, owners: List[int]): State =
    owners.foldl(s, (a, i) => partial_one(a, ok, i))

  // ---------------------------------------------------------------------
  // SignalBuffer::set_arrival
  // ---------------------------------------------------------------------

  def set_arrival(s: State, site: str): State =
    if (not(s.latched))
      { ...s, liveWakeup: true, wakeupArmed: true }
    else if (FIX_SHUTDOWN_WAKEUP)
      ev({ ...s, wakeupArmed: false }, "set_arrival_latched")
    else
      ev({ ...s, crashed: true }, if (site == "accept") "crash_accept" else "crash_flush")

  // ---------------------------------------------------------------------
  // BatchProcessorSignal::flush_signal_impl
  // ---------------------------------------------------------------------

  /// One reference per (output, tracked live input in its range).
  def add_refs(s: State, drained: List[Portion], outs: List[Range]): State =
    outs.foldl(s, (a, r) => range(r._1, r._2 + 1).foldl(a, (b, j) =>
      val p = drained[j]
      if (p.tracked and b.inbound.keys().contains(p.input))
        { ...b, inbound: b.inbound.set(p.input, b.inbound.get(p.input) + 1) }
      else b))

  /// SignalBuffer::take_remaining followed by set_arrival.
  def take_remaining(s: State, part: Portion): State =
    val kept = if (MUTATION == "tail_untracked") { ...part, tracked: false }
               else part
    set_arrival(ev({ ...s, pending: s.pending.append(kept),
                     tailInputs: if (COVERAGE) s.tailInputs.union(Set(part.input))
                                 else s.tailInputs }, "tail_retained"),
                "flush")

  /// Releases the pending reference of every drained entry.
  def release_pending(s: State, drained: List[Portion]): State =
    handle_partial_responses(s, true, owners_of(drained))

  /// One output: output_context, finish_batch, outbound slot, send.
  def emit_one(a: State, drained: List[Portion], r: Range, fails: bool): State =
    val oid = a.nextOutput
    val idx = range(r._1, r._2 + 1)
    val carried = idx.foldl(Set(), (acc, j) => acc.union(Set(drained[j].input)))
    val owners = idx.foldl([], (l, j) =>
      if (drained[j].tracked and a.inbound.keys().contains(drained[j].input))
        l.append(drained[j].input)
      else l)
    // D4: this output is handed off (not dropped) and carries data of an
    // input that was already Nacked.
    val nackedData = idx.foldl(false, (acc, j) => acc or (drained[j].tracked
      and (if (a.answers.keys().contains(drained[j].input))
             a.answers.get(drained[j].input) == ["nack"]
           else false)))
    val a1 = { ...a,
      nextOutput: oid + 1,
      outputsOf: carried.fold(a.outputsOf, (m, i) =>
        m.put(i, (if (m.keys().contains(i)) m.get(i) else Set()).union(Set(oid)))),
    }.ev_if(nackedData and not(fails), "data_after_nack")
    if (fails)
      handle_partial_responses({ ...a1, outResult: a1.outResult.put(oid, "dropped") },
                               false, owners).ev("finish_failed")
    else if (owners.length() == 0)
      { ...a1, outResult: a1.outResult.put(oid, "untracked") }
    else if (a1.outbound.keys().size() >= OUTBOUND_CAP)
      handle_partial_responses({ ...a1, outResult: a1.outResult.put(oid, "untracked") },
                               false, owners).ev("outbound_exhausted")
    else
      { ...a1,
        outbound: a1.outbound.put(oid, owners),
        outResult: a1.outResult.put(oid, "inflight") }

  def emit_all(s: State, drained: List[Portion], outs: List[Range], ff: int): State =
    range(0, outs.length()).foldl(s, (a, k) => emit_one(a, drained, outs[k], k == ff))

  def flush_outputs(s: State, reason: str, c: Choice, drained: List[Portion]): State =
    val n = drained.length()
    val outs = derive_ranges(n, c.tok)
    val last = if (outs.length() > 0) outs[outs.length() - 1] else (0, -1)
    val retain = and {
      reason == "size", outs.length() > 1, c.retainTail,
      last._1 == last._2, last._2 == n - 1,
    }
    val sent = if (retain) outs.slice(0, outs.length() - 1) else outs
    val early = MUTATION == "release_before_refs"
    val s1 = if (early) add_refs(release_pending(s, drained), drained, outs)
             else add_refs(s, drained, outs)
    val s2 = if (retain) take_remaining(s1, drained[n - 1]) else s1
    if (s2.crashed)
      s2
    else
      val s3 = if (early) s2 else release_pending(s2, drained)
      val s4 = emit_all(s3, drained, sent, c.finishFail)
        .ev_if(completed_of(n, c.tok).size() > 0, "completed_input")
        .ev_if(outs.indices().exists(k => k > 0 and outs[k]._1 == outs[k - 1]._2),
               "split_boundary")
      if (retain) s4 else { ...s4, wakeupArmed: false }

  /// True if an output numbered `firstOut` or later carries data of an input in
  /// `ins` and was handed off downstream (not dropped by finish_batch).
  def handed_off(s: State, firstOut: int, ins: Set[int]): bool =
    ins.exists(i =>
      if (s.outputsOf.keys().contains(i))
        s.outputsOf.get(i).exists(o => o >= firstOut and s.outResult.get(o) != "dropped")
      else false)

  def flush_signal_impl(s: State, reason: str, c: Choice): State =
    val s0 = if (reason == "timer") { ...s, wakeupArmed: false } else s
    if (s0.pending.length() == 0)
      s0
    else
      // cancel_wakeup is ignored once the scheduler has latched shutdown,
      // but the latch has already cleared the live wakeup.
      val s1 = if (reason != "timer" and s0.wakeupArmed)
                 { ...s0, liveWakeup: false, wakeupArmed: false }
               else s0
      val drained = s1.pending
      val tails = drained.foldl(Set(), (a, p) =>
        if (s1.tailInputs.contains(p.input)) a.union(Set(p.input)) else a)
      val drainIns = drained.foldl(Set(), (a, p) =>
        if (s1.drainInputs.contains(p.input)) a.union(Set(p.input)) else a)
      val s2 = { ...s1, pending: [], tailInputs: Set() }
      if (c.batchError)
        handle_partial_responses(s2, false, owners_of(drained)).ev("batch_error")
      else
        val s3 = flush_outputs(s2, reason, c, drained)
        // Flush events count only data that was handed off downstream.
        val firstOut = s2.nextOutput
        s3.ev_if(handed_off(s3, firstOut, tails),
                 if (reason == "timer") "tail_flushed_timer"
                 else if (reason == "size") "tail_flushed_size"
                 else "tail_flushed_shutdown")
          .ev_if(reason == "shutdown" and handed_off(s3, firstOut, drainIns),
                 "shutdown_flushes_drain_input")
          .ev_if(firstOut.to(s3.nextOutput - 1).exists(o => s3.outResult.get(o) != "dropped"),
                 if (reason == "timer") "timer_flush"
                 else if (reason == "size") "size_flush"
                 else "shutdown_flush")

  // ---------------------------------------------------------------------
  // BatchProcessorSignal::accept_payload
  // ---------------------------------------------------------------------

  /// flushNow abstracts "pending size reached the lower limit".
  def accept_payload(s: State, tracked: bool, flushNow: bool, c: Choice): State =
    val i = s.nextInput
    val s0 = { ...s,
      nextInput: i + 1,
      trackedInputs: if (tracked) s.trackedInputs.union(Set(i)) else s.trackedInputs,
      drainInputs: if (COVERAGE and s.phase == "draining") s.drainInputs.union(Set(i))
                   else s.drainInputs,
    }.ev_if(s.phase == "draining", "accept_during_drain")
    if (tracked and s0.inbound.keys().size() >= INBOUND_CAP)
      answer(s0, i, "nack").ev("inbound_exhausted")
    else
      val wasEmpty = s0.pending.length() == 0
      val s1 = { ...s0,
        inbound: if (tracked) s0.inbound.put(i, 1) else s0.inbound,
        pending: s0.pending.append({ input: i, tracked: tracked }),
      }
      if (flushNow) flush_signal_impl(s1, "size", c)
      else if (wasEmpty) set_arrival(s1, "accept")
      else s1

  // ---------------------------------------------------------------------
  // Actions (one per process() message kind)
  // ---------------------------------------------------------------------

  pure val NO_CHOICE: Choice =
    { batchError: false, tok: Map(), retainTail: false, finishFail: -1 }

  def can_receive: bool = st.phase != "stopped" and not(st.crashed)

  /// PData that stays buffered (below the lower limit).
  action accept_buffer(tracked: bool): bool = all {
    can_receive, st.nextInput < MAX_INPUTS,
    st' = accept_payload(st, tracked, false, NO_CHOICE).ev("act_accept_buffer"),
  }

  /// PData that triggers a size flush.
  action accept_flush(tracked: bool, c: Choice): bool = all {
    can_receive, st.nextInput < MAX_INPUTS,
    st' = accept_payload(st, tracked, true, c).ev("act_accept_flush"),
  }

  /// NodeControlMsg::Wakeup for the live wakeup at its deadline.
  action wakeup(c: Choice): bool = all {
    can_receive, st.liveWakeup,
    st' = flush_signal_impl({ ...st, liveWakeup: false }, "timer", c).ev("act_wakeup"),
  }

  /// D9/F4: the live wakeup arrives before arrival + max_batch_duration,
  /// so flush_signal_impl skips it without re-arming.
  action early_wakeup: bool = all {
    EARLY_WAKEUPS, can_receive, st.liveWakeup, st.pending.length() > 0,
    st' = ev({ ...st, liveWakeup: false, wakeupArmed: false }, "early_wakeup"),
  }

  /// The inbox latches Shutdown with a future deadline and starts draining.
  action begin_drain: bool = all {
    can_receive, st.phase == "running",
    st' = ev({ ...st, phase: "draining", latched: true, liveWakeup: false },
             "drain"),
  }

  /// NodeControlMsg::Shutdown reaches the processor: flush_shutdown. The
  /// inbox drops its channels afterwards, so nothing else is delivered.
  action shutdown(c: Choice): bool = all {
    can_receive,
    st' = { ...flush_signal_impl({ ...st, latched: true, liveWakeup: false },
                                 "shutdown", c),
            phase: "stopped" }.ev("act_shutdown").ev_if(st.phase == "draining", "shutdown_after_drain"),
  }

  /// Downstream Ack or Nack for a tracked output: handle_ack or
  /// handle_nack, handle_response, BatchProcessorSignal::handle.
  action downstream(o: int, ok: bool): bool = all {
    can_receive, st.outbound.keys().contains(o),
    val s1 = { ...st,
      outbound: remove(st.outbound, o),
      outResult: st.outResult.put(o, if (ok) "ack" else "nack"),
    }
    // First downstream Nack for a live input while another of its outputs
    // is still in flight.
    val sib = not(ok) and st.outbound.get(o).foldl(false, (a, i) => a or (st.inbound.keys().contains(i)
                and st.outbound.keys().exists(o2 => o2 != o and has(st.outbound.get(o2), i))))
    st' = handle_partial_responses(s1, ok, st.outbound.get(o))
            .ev(if (ok) "act_downstream_ack" else "act_downstream_nack")
            .ev_if(sib, "nack_sibling_inflight"),
  }

  /// Stutter only once the node has stopped or crashed, so a state before
  /// that with no enabled action shows up as a deadlock.
  action idle = all { st.phase == "stopped" or st.crashed, st' = st }

  action init: bool = st' = {
    phase: "running", latched: false, crashed: false, pending: [],
    inbound: Map(), outbound: Map(), liveWakeup: false, wakeupArmed: false,
    nextInput: 0, nextOutput: 0,
    trackedInputs: Set(), answers: Map(), outputsOf: Map(), outResult: Map(),
    events: Set(), tailInputs: Set(), drainInputs: Set(), failedIn: Set(),
  }

  action step: bool = any {
    nondet tracked = Set(true, false).oneOf()
    accept_buffer(tracked),
    nondet tracked = Set(true, false).oneOf()
    nondet c = choices(st.pending.length() + 1).oneOf()
    accept_flush(tracked, c),
    nondet c = choices(st.pending.length()).filter(c => not(c.retainTail)).oneOf()
    wakeup(c),
    early_wakeup,
    begin_drain,
    nondet c = choices(st.pending.length()).filter(c => not(c.retainTail)).oneOf()
    shutdown(c),
    all {
      st.outbound.keys().size() > 0,
      nondet o = st.outbound.keys().oneOf()
      nondet ok = Set(true, false).oneOf()
      downstream(o, ok),
    },
    idle,
  }

  // ---------------------------------------------------------------------
  // Invariants (spec.md)
  // ---------------------------------------------------------------------

  def pending_names(i: int): bool = st.pending.foldl(false, (a, p) => a or p.input == i)

  def pending_refs(i: int): int =
    st.pending.foldl(0, (a, p) => if (p.tracked and p.input == i) a + 1 else a)

  def outbound_refs(i: int): int =
    st.outbound.keys().fold(0, (a, o) => a + count(st.outbound.get(o), i))

  def outputs_of(i: int): Set[int] =
    if (st.outputsOf.keys().contains(i)) st.outputsOf.get(i) else Set()

  /// S1: at most one answer per input, and only tracked inputs are answered.
  val answered_at_most_once: bool = and {
    st.answers.keys().forall(i => st.answers.get(i).length() <= 1),
    st.answers.keys().subseteq(st.trackedInputs),
  }

  /// S2: an Ack means every output that carried the input's data was Acked
  /// and none of its data is still buffered.
  val ack_only_after_all_outputs_acked: bool =
    st.answers.keys().forall(i => st.answers.get(i) == ["ack"] implies and {
      outputs_of(i).forall(o => st.outResult.get(o) == "ack"),
      not(pending_names(i)),
    })

  /// S2: the first Nack answers at once, so a live input has no Nacked or
  /// dropped output.
  val nack_answers_at_once: bool = st.inbound.keys().forall(i =>
    outputs_of(i).forall(o => not(Set("nack", "dropped").contains(st.outResult.get(o)))))

  /// S3: reference conservation (holds between process() calls).
  val ref_conservation: bool = st.crashed or
    st.inbound.keys().forall(i => and {
      st.inbound.get(i) >= 1,
      st.inbound.get(i) == pending_refs(i) + outbound_refs(i),
    })

  /// S1 corollary: a live slot belongs to an unanswered input.
  val live_means_unanswered: bool =
    st.inbound.keys().forall(i => not(st.answers.keys().contains(i)))

  /// S4: pending entries name distinct inputs, keep their input's tracking,
  /// and a tracked entry belongs to a live input or to one that was already
  /// Nacked (D4).
  val tail_attribution: bool = and {
    st.pending.indices().forall(k =>
      st.pending[k].tracked == st.trackedInputs.contains(st.pending[k].input)),
    st.pending.indices().forall(a => st.pending.indices().forall(b =>
      a == b or st.pending[a].input != st.pending[b].input)),
    st.pending.indices().forall(k => st.pending[k].tracked implies or {
      st.inbound.keys().contains(st.pending[k].input),
      st.answers.get(st.pending[k].input) == ["nack"],
    }),
  }

  /// S5: slot capacity is respected.
  val slot_bounds: bool = and {
    st.inbound.keys().size() <= INBOUND_CAP,
    st.outbound.keys().size() <= OUTBOUND_CAP,
  }

  /// S6: process() never returns Err.
  val no_crash: bool = not(st.crashed)

  /// S6 restricted to F2 (set_arrival inside a size flush). Needs COVERAGE.
  val no_crash_in_flush: bool = not(st.events.contains("crash_flush"))

  /// L1 (state form): once nothing is buffered or in flight before
  /// Shutdown, every tracked input has been answered and no slot is held.
  val no_leak_when_settled: bool =
    (not(st.crashed) and st.phase != "stopped" and st.pending.length() == 0
       and st.outbound.keys().size() == 0)
    implies (st.inbound.keys().size() == 0
             and st.trackedInputs.subseteq(st.answers.keys()))

  /// L2 (state form): while running, buffered data always has a live
  /// wakeup, so a timer flush will come.
  val no_stuck_buffer: bool =
    (st.phase == "running" and not(st.crashed) and st.pending.length() > 0)
    implies st.liveWakeup

  val safety: bool = and {
    answered_at_most_once, ack_only_after_all_outputs_acked, nack_answers_at_once,
    ref_conservation, live_means_unanswered, tail_attribution, slot_bounds,
  }

  val liveness: bool = no_leak_when_settled and no_stuck_buffer

  val all_invariants: bool = safety and liveness and no_crash

  // ---------------------------------------------------------------------
  // Coverage witnesses (quint run --witnesses); see check.sh
  // ---------------------------------------------------------------------

  val w_tail_retained = st.events.contains("tail_retained")
  val w_completed_input = st.events.contains("completed_input")
  val w_split_boundary = st.events.contains("split_boundary")
  val w_batch_error = st.events.contains("batch_error")
  val w_finish_failed = st.events.contains("finish_failed")
  val w_inbound_exhausted = st.events.contains("inbound_exhausted")
  val w_outbound_exhausted = st.events.contains("outbound_exhausted")
  val w_nack_sibling_inflight = st.events.contains("nack_sibling_inflight")
  val w_data_after_nack = st.events.contains("data_after_nack")
  val w_ack_answer = st.events.contains("ack_answer")
  val w_nack_answer = st.events.contains("nack_answer")
  val w_timer_flush = st.events.contains("timer_flush")
  val w_shutdown_flush = st.events.contains("shutdown_flush")
  val w_accept_during_drain = st.events.contains("accept_during_drain")
  val w_set_arrival_latched = st.events.contains("set_arrival_latched")
  val w_crash_accept = st.events.contains("crash_accept")
  val w_crash_flush = st.events.contains("crash_flush")
  // One per action; idle is a stutter and has none.
  val w_act_accept_buffer = st.events.contains("act_accept_buffer")
  val w_act_accept_flush = st.events.contains("act_accept_flush")
  val w_act_wakeup = st.events.contains("act_wakeup")
  val w_early_wakeup = st.events.contains("early_wakeup")
  val w_act_begin_drain = st.events.contains("drain")
  val w_act_shutdown = st.events.contains("act_shutdown")
  val w_act_downstream_ack = st.events.contains("act_downstream_ack")
  val w_act_downstream_nack = st.events.contains("act_downstream_nack")
  val w_tail_flushed_timer = st.events.contains("tail_flushed_timer")
  val w_tail_flushed_size = st.events.contains("tail_flushed_size")
  val w_tail_flushed_shutdown = st.events.contains("tail_flushed_shutdown")
  val w_shutdown_after_drain = st.events.contains("shutdown_after_drain")
  val w_shutdown_flushes_drain_input = st.events.contains("shutdown_flushes_drain_input")
}
