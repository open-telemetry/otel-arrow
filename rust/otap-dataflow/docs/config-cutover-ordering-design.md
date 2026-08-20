# Design: Strict config-cutover ordering via an InboxCore Config-latch

Status: Proposal (design only, not yet implemented). Requesting review before
any engine code is written.

## Problem

Live reconfiguration delivers `NodeControlMsg::Config` on a node's control
channel. Control and pdata travel on two physically separate channels, and the
inbox prefers control over pdata (bounded by a fairness burst). As a result, a
`Config` can be observed and applied by a node *before* pdata that was accepted
into the node's pdata channel earlier. For the Kafka exporter this means a batch
accepted before the `Config` can be routed/encoded/sent under the new
configuration (topic, credentials, tenant), violating the intended invariant:

> Any pdata accepted before a `Config` must be handled under the configuration
> in effect when it was accepted. The new configuration must take effect only
> for pdata accepted after the `Config`.

Today the Kafka exporter binds a batch to a generation at *dequeue* time, not at
*accept* time, so buffered-but-not-yet-dequeued pdata crossing a `Config` is
handled under the new generation. (This is documented in the existing test
`reconfigure_drains_pipelined_in_flight_before_swap`, which asserts no data loss
but explicitly does not assert pre-config routing.)

## Why a node-local drain cannot be strict, and why a cross-node watermark is too costly

- `Config` and pdata come from different senders (the pipeline controller vs the
  upstream data node) on different channels, with no cross-channel happens-before
  relationship. Observing `Config` says nothing about pdata-channel contents.
- The pdata channel is bounded; an upstream batch can be committed (parked in a
  `SendFuture` awaiting capacity) but not yet in the buffer. A "drain until
  empty" therefore cannot be proven to capture every pre-config batch.
- A truly strict, controller-free guarantee requires an in-band marker on the
  pdata channel that survives transit through every intervening processor and is
  reconciled across fan-in. That is a pervasive protocol change: the shared
  `OtapPdata`/`OtapPayload` type, every processor's `process()`, a new
  `OutputRouter` broadcast primitive, and new fan-in per-source alignment state.
  Rejected as disproportionate for a capability that currently has no production
  Config-delivery path.

## Proposed approach: mirror the existing Shutdown-drain latch for Config

`InboxCore` already implements an ordering latch for `Shutdown`
(`crates/engine/src/message.rs`): it stores the `Shutdown` in `pending_shutdown`
and releases it only after the buffered pdata backlog has drained
(`shutdown_drain_complete`). We propose an analogous latch for `Config`: hold the
`Config` and release it to the node only after the pdata that was buffered when
the `Config` arrived has been delivered to the node.

This confines the change to `crates/engine/src/message.rs` (`InboxCore`) plus the
control-delivery path, and requires no changes to the shared pdata type, other
processors, the `OutputRouter`, or fan-in nodes.

### Guarantee provided

- Strict *at the reconfigured node*, for all pdata that was already buffered in
  the node's pdata channel when the `Config` became observable: such pdata is
  delivered to the node before the `Config`, so the node applies the new config
  only after handling it.
- NOT strict against the backpressure race in isolation: a batch committed by
  upstream but still parked awaiting channel capacity is not yet buffered, so it
  is not covered. Strictness for that case requires pairing the latch with
  controller-driven upstream quiescence (drain/pause upstream, then send
  `Config`). This limitation must be stated explicitly wherever the guarantee is
  documented.

### Key difference from the Shutdown latch

`shutdown_drain_complete` gates on `pdata_rx.is_closed() && pdata_rx.is_empty()`.
A live `Config` does not close the pdata channel, so `is_closed()` cannot be
reused. The Config-latch needs a different "pre-Config pdata drained" condition
that does not depend on closure. Two candidate definitions:

1. Buffer-snapshot count: at latch time record the buffered depth N (if the
   bounded MPSC can report it) and release `Config` after delivering N pdata
   items. Precise for buffered items; ignores the backpressure race.
2. Drain-to-currently-empty: deliver pdata until `pdata_rx.is_empty()` observed
   once, then release `Config`. Matches "currently buffered"; also subject to the
   backpressure race.

Both are equivalent modulo the backpressure race. Recommendation: option 2
(drain-to-empty), since it avoids depending on an exact `len()` from the channel
abstraction and composes with the existing empty-probe logic.

## Sketch of the InboxCore changes

- Add latch state alongside the shutdown latch:
  - `pending_config: Option<NodeControlMsg<PData>>`
  - a boolean/marker indicating a Config drain is in progress.
- On observing a `Config` in the control channel while none is pending: store it
  in `pending_config` and enter a Config-drain mode instead of returning it.
- In Config-drain mode, prefer delivering buffered pdata to the node; when the
  pdata buffer is observed empty (option 2), release `pending_config` to the node
  and exit Config-drain mode.
- Reuse the `CONTROL_BURST_LIMIT` fairness handling so a control burst cannot
  starve the pre-Config pdata being drained (mirror the shutdown-drain arm).
- Interaction with the shutdown latch: if `Shutdown` arrives while a Config drain
  is in progress, shutdown takes precedence (shutdown is terminal); define and
  test this ordering.
- Interaction with other control messages (Ack/Nack/CollectTelemetry/TimerTick):
  these must continue to flow during a Config drain, exactly as they do during a
  shutdown drain.

## Delivery path

No production path currently delivers `Config` to a running node
(`RuntimeCtrlMsgManager` never sends `Config`; live reconfiguration rebuilds the
pipeline). The single targeted API is `RuntimePipeline::send_node_control_message`
-> node `send_control_msg`. The design integrates with that path; the latch lives
below it in `InboxCore`, so any future Config delivery inherits the ordering.

## Test strategy

- InboxCore unit tests (mirroring the existing shutdown-drain tests): buffer K
  pdata, then send `Config`; assert all K pdata are delivered before the `Config`.
- Fan-in / burst fairness: interleave control (Ack/Nack) with buffered pdata and
  assert the pre-Config pdata is not starved and `Config` is still released after
  the buffer empties.
- Shutdown-during-Config-drain precedence test.
- Kafka exporter integration: with the latch in place, a pre-Config batch routes
  to the old topic (the strict version of
  `reconfigure_drains_pipelined_in_flight_before_swap`).

## Blast radius

- Confined to `crates/engine/src/message.rs` (`InboxCore`) plus the targeted
  control-delivery path.
- No change to `OtapPdata`/`OtapPayload`, other nodes, `OutputRouter`, or fan-in
  handling.
- Behavior change is opt-in in practice: only nodes that receive `Config` while
  running are affected, and no production path sends `Config` today.

## Open questions for review

1. Is the node-local strict guarantee (plus controller-driven upstream quiesce
   for the backpressure race) sufficient, or is the cross-node watermark required
   despite its cost?
2. Option 1 (buffer-snapshot count) vs option 2 (drain-to-empty) for the drain
   condition?
3. Should the latch be unconditional for all nodes, or gated per node-kind (e.g.
   only exporters), to avoid changing Config timing for processors that treat
   Config as an immediate event?
