# Objective

Move the experimental Kafka receiver from its current state toward production-ready.

This is a **high-level tracking issue**. It enumerates the focus areas at a high level; each area is intended to be broken out into its own **sub-issue / subtask** that goes deeper, carries the detailed work, and is closed independently. Progress on the receiver is tracked by the state of those child subtasks.

# Rationale

The receiver has a broad configuration surface, docs, and solid unit coverage. Core runtime paths (commit, rebalance, drain, routing, header extraction) are now exercised by in-process integration tests running against `rdkafka::mocking::MockCluster` (librdkafka's built-in mock), so they run in CI by default with no Docker dependency. The mock broker is the primary vehicle for integration testing; several areas (failure/recovery, auth, TLS, performance) still need coverage extended on top of it. Several correctness gaps are also documented in-code as TODOs. This document makes those explicit so they don't get lost.

# Scope

Each numbered area below is a **child subtask** to be tracked separately. Each lists what to cover at a high level, the relevant **code anchors**, any **known gaps** already visible in the implementation, and its own **acceptance criteria**. The detailed breakdown lives in the corresponding sub-issue; solutions are intentionally left open where non-obvious.

## 1. Offset guarantees

Validate at-least-once (manual, default) and at-most-once (auto) semantics.

- [x] Manual commit watermark logic -- only the lowest un-acked offset is committed, so out-of-order acks cannot skip offsets (`offset_tracker.rs` -- `PartitionTracker::committable_offset`, `OffsetTracker::committable_tpl`).
  - Completed by test `out_of_order_acks_commit_only_lowest_contiguous` in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: acks a partition's records out of order and asserts the committed offset never advances past the un-acked lowest offset, then jumps to the full count once that offset is acked.
- [ ] Steady-state commits are **async** and non-blocking (`receiver.rs:426` `commit_offsets`); broker outcome is observed later via `commit_callback` (`rebalance.rs:697`). Validate that a commit failure is surfaced (`offset_commit_errors`) and does not silently advance state.
- [x] Out-of-order ACK/NACK across many in-flight records per partition.
  - Completed by tests in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: `out_of_order_acks_commit_only_lowest_contiguous` validates the multi-partition case -- a 2-partition topic with 3 in-flight records per partition, where each delivered record is correlated back to its `(partition, offset)` via the stamped calldata (`source_route` + `decode_calldata`), partition 0 receives out-of-order ACKs and partition 1 receives out-of-order terminal NACKs (offsets 1,2 first, lowest offset 0 withheld), and each partition's watermark is asserted to hold at the gap and then advance to the full per-partition count only once offset 0 is acked/nacked (proving partition-scoped tracking and that ACK and terminal NACK advance the watermark identically). `terminal_nack_advances_offset_past_message` additionally covers the in-flight/uncommitted-then-advance window for the terminal NACK path.
- [x] Poison / undecodable messages advance past without stalling the partition (`receiver.rs:925`), and do so without violating the late-ack guard.
  - Completed by test `poison_message_advances_without_stalling_partition` in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: an undecodable OTAP-Arrow record between two good records is counted as a processing/unmarshal error, is not forwarded, and the committed offset advances past it while the surrounding good records are still delivered.
- [ ] Commit failures, consumer restarts, and process crashes result in correct re-delivery (no data loss under at-least-once, bounded duplication only).
- [x] Auto-commit mode: confirm the tracker/rebalance paths are truly no-ops and librdkafka owns offsets (`rebalance.rs` -- auto_commit short-circuits).
  - Completed by test `auto_commit_mode_lets_librdkafka_own_offsets` in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: an auto-commit (`CommitMode::Auto`) receiver consumes every produced record but never acks, and the broker-side committed offset still advances to the full record count purely from librdkafka's periodic auto-commit; the receiver's `acks_received` and `offset_commit_errors` stay 0, proving the manual tracker/rebalance-commit paths are inert and librdkafka owns offsets. Complements the existing `purge_revoked_partitions_is_noop_under_auto_commit` / `reconcile_is_noop_under_auto_commit` unit tests with an end-to-end broker-side proof.
- [ ] Retry interaction with a `processor:retry` node in the pipeline: the message offset stays in-flight (uncommitted) while retries are in progress and advances only once a nack is final/permanent (`receiver.rs:747`).

**Retry support is out-of-process, by design.** A `Nack` reaching the receiver is a **terminal** outcome that advances past the message (`receiver.rs:747`, `nacks_received` counter); transient retry is delegated to the separate `processor:retry` node placed between the receiver and a failure-prone exporter (see the receiver `README.md`, "Failure Handling and Retries"). The work here is to **confirm** that path end-to-end via mock-broker integration testing rather than to add retry logic to the receiver.

**Acceptance criteria:**

- [ ] At-least-once (manual) and at-most-once (auto) semantics validated, including out-of-order acks and poison messages.
- [ ] At-least-once confirmed end-to-end with a `processor:retry` node via integration testing: the offset is held during retries and advances only on an exhausted/permanent failure -- `receiver.rs:747`.

## 2. Consumer-group rebalancing

- [x] All three assignment strategies: `range`, `roundrobin`, `cooperative-sticky` (`config.rs:86`, `RebalanceStrategy`).
  - Completed by tests in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: `rebalance_cooperative_sticky_retains_owned_partitions` covers `cooperative-sticky`; `rebalance_strategy_range_assigns_and_commits` and `rebalance_strategy_roundrobin_assigns_and_commits` each run two same-group receivers (configured via `with_rebalance_strategy`) against a 2-partition topic and assert the group distributes both partitions across the two members (`partitions_assigned` sum covers the topic) and commits every produced record with no loss or double-commit (broker-side `message_count` retention + per-partition committed offset equal to the produced count).
- [ ] Adding/removing consumers; partitions returning to the same consumer keep their generation (`rebalance.rs` -- `AssignedPartitions`, generation allocator).
- [ ] Incremental (cooperative-sticky) reassignments handled correctly via the full-`assignment()` query with additive merge fallback (`rebalance.rs:495` `merge_assignment`, `rebalance.rs:541` `handle_assign`).
- [ ] Commit-before-revoke ordering under rebalance is sync and scoped to the revoked partitions only (`rebalance.rs:373` `handle_revoke`, `rebalance.rs:391`).
- [ ] Generation/ownership guards prevent stale acks from committing revoked partitions (`receiver.rs:596` stale-generation guard, `receiver.rs:609` late-ack guard, `acks_for_revoked_partition` counter).
- [ ] In-flight records crossing an assignment change.

**Known gap -- in-flight records on revoked partitions are not drained or interrupted** (documented design choice, `rebalance.rs:29-35`). The receiver relies on re-delivery under at-least-once. Validate the resulting duplication is bounded and correct, and decide whether any partition-scoped draining is warranted.

**Acceptance criteria:**

- [ ] All three assignment strategies validated, including incremental (cooperative-sticky) reassignments and stale-ack guards.
  - Assignment-strategy coverage is complete: `range` and `roundrobin` via `rebalance_strategy_range_assigns_and_commits` / `rebalance_strategy_roundrobin_assigns_and_commits`, and `cooperative-sticky` (incremental reassignment) via `rebalance_cooperative_sticky_retains_owned_partitions`; stale-ack guards are covered by `stale_revocation_preserves_reassigned_partition_state` and the `acks_for_revoked_partition` late-ack guard. Left unchecked pending full sub-issue triage of the remaining rebalance scenarios.
- [ ] In-flight-drain-on-revoke behavior triaged with a recorded decision, and the resulting duplication shown to be bounded -- `rebalance.rs:29-35`.

## 3. Lifecycle (drain & shutdown)

- [ ] `DrainIngress` (receiver-first): unsubscribe -> stop polling -> final commit -> `notify_receiver_drained` -> await `Shutdown` (`receiver.rs:700`).
- [ ] `Shutdown`: final commit -> unsubscribe -> cancel telemetry -> terminal state (`receiver.rs:684`).
- [ ] Drain/shutdown **under sustained traffic**.
- [ ] Bounded shutdown deadline. Note: drain currently does **not** wait for in-flight downstream acks (`receiver.rs:713`); confirm this is acceptable or add a bounded wait.
- [ ] Final commits succeed (or fail observably) at shutdown.
- [ ] Broker unavailable during shutdown does not hang the pipeline.

**Acceptance criteria:**

- [ ] Drain and shutdown validated under sustained traffic, with a bounded deadline and no pipeline hang when the broker is unavailable.
- [ ] The in-flight-ack wait behavior at drain confirmed acceptable or a bounded wait added -- `receiver.rs:713`.

## 4. Failure recovery

- [ ] Broker restarts and leader elections.
- [ ] Network interruptions / partitions.
- [ ] Authentication failures (including token refresh for AWS MSK OAUTHBEARER).
- [ ] Commit timeouts.
- [x] Prolonged broker outages -- verify reconnect/backoff behavior and that `transport_errors` (currently non-fatal, loop continues) is the right contract (`receiver.rs`, transport-error arm of `run_receive_loop`).
  - Completed by test `transport_error_is_non_fatal_and_recovers` in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: injects a run of `Fetch` errors, asserts no delivery while the fault is active, then clears it and asserts the same receive loop resumes delivering (proving the transport-error arm is non-fatal / log-and-continue). Note: librdkafka retries the injected fetch errors internally, so the `transport_errors` counter is observed as 0 through the mock and is asserted best-effort; broker restarts, leader election, network partitions, auth/token-refresh, and commit timeouts remain future work.

**Acceptance criteria:**

- [ ] Broker restarts, network interruptions, auth failures (incl. token refresh), commit timeouts, and prolonged outages validated via the mock broker suite, with reconnect/backoff behavior confirmed.

## 5. Backpressure & performance

- [ ] Throughput and latency benchmarks (per encoding, per signal, at 1 and N partitions); micro-bench decode cost per format to size any offload knobs.
- [ ] Slow downstream consumers -- verify backpressure propagates and broker operations **never stall the pipeline core**.
- [ ] High partition counts.
- [ ] Bounded memory usage under load.

**Known gaps & throughput direction.** The receiver is a single-task poll -> decode -> send -> (sync commit) loop on one core thread, so CPU-bound decode blocks the next poll. The directions below should stay opt-in with behavior-preserving defaults and must keep the at-least-once guarantee -- the `BTreeSet` offset tracker commits only the lowest un-acked offset, so out-of-order completion is already safe.

- Heavy decode runs inline on the receive thread and is flagged as a risk/TODO (`receiver.rs:5-7`). Moving CPU-bound decode onto a dedicated worker thread with `Send` channels -- the `journald_receiver` pattern -- would parallelize it without touching the `!Send` runtime state, with fetch gated on the in-flight count.
- Commit is synchronous and per-watermark-advance on the hot loop (`commit_offsets` `receiver.rs:425`, `advance_offset_and_commit`). Switching to async, tick-batched commits would keep the loop moving while retaining **sync** commits at the rebalance and shutdown boundaries.
- Per-message allocations come from `topic().to_owned()` and multi-pass header parsing. Resolving a `Copy` `topic_id` via the topic registry, pre-computing the header->attribute key map once, and resolving format + capture in a single header pass would remove them.
- `pdata_consumer` cannot be reused and is recreated per OTAP message (`receiver.rs:202`, tracked as issue #1669) -- a hot-path allocation cost for OTAP traffic; a serial decode worker could hold and reuse one consumer.
- librdkafka prefetch depth (`queued.min.messages` / `queued.max.messages.kbytes`) is only reachable via the `consumer_config` escape hatch. First-class knobs that pair with the decode in-flight bound would make it tunable.

**Acceptance criteria:**

- [ ] Performance characteristics documented via benchmarks, with confirmation that broker operations cannot stall the pipeline core and memory stays bounded under load.

## 6. Routing & payload correctness

- [x] All supported encodings: `OtlpProto` (default, zero-copy) and `OtapProto` (Arrow), plus per-message header override via the `MessageFormat` header (`receiver.rs:115` `detect_message_format`).
  - Completed by tests in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: `test_kafka_receiver_traces`/`_metrics`/`_logs` (OtlpProto) and `test_kafka_receiver_traces_otap`/`_metrics_otap`/`_logs_otap` (OtapProto) validate both encodings per signal; `test_kafka_receiver_message_format_header_overrides_signal_default` proves a per-message `MessageFormat: otap` header overrides an `OtlpProto` per-signal default (decodes via the OTAP path).
- [x] Malformed payloads produce the correct per-signal error metric and log without crashing the loop (`errors.rs`, `unmarshal_failed_*`, `empty_payloads`).
  - Completed by test `poison_message_advances_without_stalling_partition` in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: a malformed traces payload increments `processing_errors` and `unmarshal_failed_traces` (the correct per-signal counter) and the loop keeps running and delivering surrounding good records.
- [x] Topic regex (`^`-prefixed) and `exclude_topics` matching (`receiver.rs:64-114`), including disjoint-topic validation across signals (`config.rs` `TryFrom`).
  - Completed by test `topic_regex_and_exclude_topics_subscription_matching` in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: a traces signal configured with a `^`-prefixed include regex plus an `exclude_topics` pattern is run against three topics -- one matching the include and not the exclude, one matching both (carved out), and one matching neither. The included non-excluded topic is delivered and decoded as traces (`trace_msgs_received == 1`); the excluded topic (which the include regex matches, so librdkafka subscribes and polls it) is rejected by the receiver-side `matches_any_exclude` guard and counted in `unknown_topic_errors` rather than delivered; the unrelated topic is never delivered. Protects `matches_any_topic` (regex include) and `matches_any_exclude` (regex exclude).
- [x] Header extraction into resource attributes for both OTLP and OTAP (`headers.rs`, `resource_attrs_from_headers`).
  - Completed by tests `test_kafka_receiver_traces_header_extraction` (OTLP) and `test_kafka_receiver_traces_header_extraction_otap` (OTAP) in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: both map a Kafka header into a `tenant.id` resource attribute and assert it lands on every resource and not on spans.
- [x] Mixed-signal configurations (traces/metrics/logs on distinct topics).
  - Completed by test `mixed_signal_distinct_topics_route_correctly` in `crates/contrib-nodes/src/receivers/kafka_receiver/receiver.rs`: a single receiver configured with traces, metrics, and logs each on its own distinct topic receives one record per signal and each is routed to the correct signal decoder based solely on its arrival topic (traces topic -> `ExportTracesRequest`, metrics topic -> `ExportMetricsRequest`, logs topic -> `ExportLogsRequest`), each appearing exactly once with a lossless payload round-trip and no cross-signal misrouting. Protects the per-topic signal dispatch chain in `run_receive_loop`.

**Acceptance criteria:**

- [ ] All supported encodings, topic matching, and header extraction validated across mixed-signal configurations.

## 7. Compatibility

- [ ] Representative Kafka broker versions.
- [ ] Linux / macOS / Windows builds (the in-process `MockCluster` integration suite runs on all three without Docker).
- [ ] TLS: `ca_file` / `cert_file` / `key_file` / `key_password` / `insecure`, mTLS and server-only (`src/common/kafka/security.rs`).
- [ ] SASL mechanisms: `PLAIN`, `SCRAM-SHA-256`, `SCRAM-SHA-512`, `AWS_MSK_IAM_OAUTHBEARER` (feature `aws`) (`src/common/kafka/auth.rs`).

**Acceptance criteria:**

- [ ] Representative broker versions, all three OS targets, and the TLS/SASL matrix validated.

## 8. Operational visibility

- [ ] Verify metrics accurately distinguish: expected filtering, transient failures, rebalances, commit failures, and data-processing errors (`metrics.rs`, 21 counters).
- [ ] Error reporting/logging granularity matches the above categories.

**Known gap -- no gauges.** Every metric is a `Counter<u64>`; there are no gauges for consumer lag, end-to-end latency, in-flight record count, or queue depth (`metrics.rs`). Propose adding the operationally important gauges (lag is the most requested for Kafka consumers).

**Acceptance criteria:**

- [ ] Metric/logging categories verified to distinguish filtering, transient failures, rebalances, commit failures, and processing errors.
- [ ] Operational metrics gaps addressed or explicitly deferred, at minimum a recorded consumer-lag gauge decision -- `metrics.rs`.

## Cross-cutting: integration test suite

Integration testing is done primarily against `rdkafka::mocking::MockCluster` (librdkafka's built-in mock), which runs **in-process** and in CI by default with no Docker dependency. The suite already covers traces/metrics/logs for both encodings, header extraction, capture policies, and end-to-end rebalance/drain scenarios, including multi-consumer rebalancing (`rebalance_single_consumer_assigns_and_commits`, `rebalance_revoke_commits_before_reassign`, `rebalance_cooperative_sticky_retains_owned_partitions`, `rebalance_revoke_then_reassign_preserves_new_records`, `drain_ingress_stops_polling_and_notifies_drained`).

- [ ] #3539

**Acceptance criteria:**

- [ ] The mock broker suite stays green in CI and is extended to cover the offset/rebalance/failure/shutdown scenarios above.
