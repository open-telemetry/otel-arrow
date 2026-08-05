# Objective

Move the experimental Kafka exporter from its current state toward production-ready.

This is a **high-level tracking issue**. It enumerates the focus areas at a high level; each area is intended to be broken out into its own **sub-issue / subtask** that goes deeper, carries the detailed work, and is closed independently. Progress on the exporter is tracked by the state of those child subtasks.

# Rationale

The exporter has a broad configuration surface, docs, and solid unit coverage. Core runtime paths (topic routing, encoding, partitioning, ACK/NACK reporting, deadline-bounded shutdown flush/purge) are now exercised by in-process integration tests running against `rdkafka::mocking::MockCluster` (librdkafka's built-in mock), so they run in CI by default with no Docker dependency. The mock broker is the primary vehicle for integration testing; several areas (failure/recovery, auth, TLS, backpressure, performance) still need coverage extended on top of it.

The exporter today sends **one message per pdata, awaited synchronously** on a single task -- there is no application-level sending queue, bounded concurrency, or delivery-future pipelining (`exporter.rs:524-530`, module TODO `exporter.rs:6-7`). Retry is delegated to a separate upstream `processor:retry` node rather than handled in-process. Several correctness and security gaps are documented in-code as TODOs (`topic_router.rs:21-22`, `partitioner.rs:93-99`, `producer.rs:587`, `exporter.rs:6-7`). This document makes those explicit so they don't get lost.

# Scope

Each numbered area below is a **child subtask** to be tracked separately. Each lists what to cover at a high level, the relevant **code anchors**, any **known gaps** already visible in the implementation, and its own **acceptance criteria**. The detailed breakdown lives in the corresponding sub-issue; solutions are intentionally left open where non-obvious.

## 1. Security

Constrain dynamic topic routing, prevent uncontrolled topic creation, avoid sensitive-header fingerprinting, and sanitize adversarial values in telemetry.

- [x] **Dynamic topic routing is unconstrained.** A transport-header value can select an arbitrary destination topic with no allowlist, prefix constraint, or ACL check (`topic_router.rs` `TopicRouter::resolve`; priority 1 = header, priority 2 = static config). The only guard is Kafka topic-name syntax validation (`validate_kafka_topic`). (Done: added a per-signal operator allowlist -- `SignalConfig::allowed_topics` and `allowed_topic_prefixes` (exact + prefix, OR semantics) in `config.rs`, enforced in `TopicRouter::resolve` after syntax validation via `SignalConfig::is_dynamic_topic_allowed`; a disallowed header topic returns the new `TopicRoutingError::DisallowedHeaderTopic` and is permanently nacked with no static fallback. Empty lists preserve the previous unrestricted behavior. Covered by topic_router unit tests (`test_allowed_prefix_permits_matching_header_topic`, `test_disallowed_header_topic_is_rejected_without_fallback`, `test_exact_allowlist_permits_listed_header_topic`, `test_exact_or_prefix_allowlist_combined`, `test_no_constraint_allows_any_valid_header_topic`, `test_allowlist_does_not_constrain_static_topic`) and exporter integration tests (`disallowed_dynamic_topic_is_permanently_nacked`, `allowed_dynamic_topic_is_delivered`).)
- [x] **Uncontrolled topic creation.** There is no first-class control over broker-side auto-creation; `allow.auto.create.topics` is only reachable through the `producer_config` escape hatch. Combined with header-driven routing, a client that controls the routing header can cause a broker configured to auto-create to spawn arbitrary topics. (Done: added a first-class `allow_auto_create_topics` field defaulting to `false` (default-deny), always written to the client config in `build_client_config`, and added `allow.auto.create.topics` to `MANAGED_PRODUCER_CONFIG_KEYS` so a passthrough override is superseded and warned. Covered by `build_client_config_defaults_auto_create_to_false`, `build_client_config_allows_auto_create_when_opted_in`, `auto_create_key_is_managed_and_first_class_field_wins`.)
- [x] **Sensitive-header fingerprinting.** `partition_key_from_transport_headers` folds each transport header's **name and value** into a single `Xxh64` hash to derive the partition key. The emitted key is a stable, deterministic 16-char hash (not plaintext), but it makes traffic from a given tenant/token fingerprintable via partition-assignment analysis. (Done/recorded decision: the deterministic non-plaintext-hash tradeoff is accepted and documented in the `partitioner.rs` module doc and the README Security section (co-locating a tenant's data is the feature; disable `partition_by_transport_headers` for null-key round-robin). The "hash, never plaintext" property is pinned by `partition_key_never_contains_raw_header_value_or_name` and `partition_key_is_stable_hash_for_same_sensitive_header`.)
- [x] **Adversarial values in telemetry.** Client- or tenant-controlled values reach logs unsanitized: resolved topic in `kafka.exporter.send.failed`; invalid header topic in `kafka.exporter.topic.invalid_header`; downstream nack reason in `effect_handler.info("Kafka exporter: received Nack - ...")`. (Done: added `sanitize_for_log` (`common/kafka/mod.rs`) which escapes control characters and bounds length to `MAX_SANITIZED_LOG_LEN`, applied at all three sites -- the send-failed topic, the invalid/disallowed-header topic in `TopicRoutingError`, and the nack-reason log. Values stay visible but bounded/escaped. Covered by `sanitize_for_log_passes_through_benign_value`, `sanitize_for_log_escapes_control_characters`, `sanitize_for_log_truncates_overlong_value`. Confirmed SASL/TLS secrets are only set into the rdkafka config and never logged.)

**Acceptance criteria:**

- [x] A routing-constraint mechanism (allowlist / prefix / regex / ACL) is implemented or a decision to defer is recorded. (Done: per-signal exact + prefix allowlist implemented in `config.rs` / `topic_router.rs`; regex intentionally not added to avoid a regex/ReDoS dependency.)
- [x] Uncontrolled-topic-creation posture is decided (gated or explicitly documented). (Done: first-class `allow_auto_create_topics`, default-deny, managed key.)
- [x] Sensitive-header fingerprinting risk in the partition-key derivation is triaged with a recorded decision. (Done: accepted tradeoff documented in `partitioner.rs` + README; non-plaintext property test-pinned.)
- [x] Adversarial values (topic, nack reason) in logs are bounded/sanitized or the exposure is explicitly accepted. (Done: `sanitize_for_log` applied at all three log sites.)

## 2. Shutdown & live reconfiguration

Verify deadline-aware cancellation, draining, flushing, producer-thread termination, and replacement while deliveries are in flight.

- [x] **Shutdown path** (`exporter.rs:548-560`): info log -> cancel the periodic-telemetry timer -> `drain_and_flush(deadline, ...)` -> return `TerminalState::new(deadline, [self.metrics.snapshot()])`. (Done: exercised by the `shutdown_flushes_buffered_records` and new `shutdown_flushes_under_sustained_traffic` integration tests in `exporter.rs`, which drive the full Shutdown arm and assert the terminal flush delivers all buffered records.)
- [x] **Deadline-bounded drain/flush** (`drain_and_flush`, `exporter.rs:473-498`): `flush_timeout = deadline - now` saturating to `Duration::ZERO` (`483-485`); `producer.flush(flush_timeout)` (`487`); **only on flush failure/timeout** -> `otel_warn!("kafka.exporter.shutdown.flush_failed", ...)` (`488-491`) then `producer.purge(queue().inflight())` (`495-496`). (Done: validated by the new `shutdown_honors_deadline_when_broker_unavailable` test in `exporter.rs`, which points the exporter at an unroutable broker, buffers a record, and asserts the deadline-bounded flush+purge returns well within a generous outer timeout instead of hanging; purge-on-error path is thereby exercised.)
- [x] **Producer-thread termination.** The custom `Drop` (`producer.rs:203-224`) sets `should_stop = true` and **joins** the 1-second poll thread (`producer.rs:212-214`); on join failure -> `otel_warn!("kafka.exporter.producer.poll_thread_join_failed", ...)` (`producer.rs:216-219`). **`Drop` does not flush or purge** -- draining is entirely the responsibility of `drain_and_flush` before the exporter returns. (Done: poll-thread teardown is now exercised on two paths -- at shutdown by the shutdown tests, and mid-run by the live-reconfigure build-and-swap in `KafkaExporter::reconfigure` (`exporter.rs`), where dropping the old producer joins its poll thread; covered by `reconfigure_switches_topic` and `reconfigure_flushes_inflight_before_swap`.)
- [x] **Drain relies on engine receiver-first ordering.** There is no explicit `DrainIngress` handling in the loop; unrecognized control messages fall through the `Control(_)` catch-all (`exporter.rs:561-563`) with a comment noting ingress is already closed by the engine's receiver-first drain (`exporter.rs:552-555`). (Done: `shutdown_honors_deadline_when_broker_unavailable` confirms a broker-unavailable shutdown cannot hang past the deadline, and `shutdown_flushes_under_sustained_traffic` confirms the drain holds under sustained traffic.)

**Live reconfiguration is now handled (build-and-swap).** The engine defines `NodeControlMsg::Config { config }` for pushing a new configuration to a running node (`control.rs:261-264`). The exporter previously ignored it via the `Control(_)` catch-all; it now handles it in a dedicated event-loop arm that calls the new `KafkaExporter::reconfigure` method (`exporter.rs`). Reconfiguration deserializes/validates the incoming JSON into a `KafkaExporterConfig`, builds a replacement librdkafka producer via the shared `KafkaExporter::build_producer` helper (extracted from `KafkaExporter::new` so the AWS-gated client-context selection lives in one place), performs a **bounded drain of the old producer** (flush bounded by the old config's `timeout_ms`, then purge of anything still queued so a slow/unavailable broker cannot stall the swap), and only then swaps in the new producer and config. Records already in flight get one bounded final chance to deliver before the swap. Reconfiguration is best-effort: an invalid config or a producer-build failure is logged (`kafka.exporter.reconfigure_error`) and the existing producer keeps running, matching the warn-and-keep posture of sibling nodes (condense-attributes and retry processors).

**Acceptance criteria:**

- [x] Deadline-aware cancellation, drain, flush, and poll-thread termination validated under sustained traffic, with a bounded deadline and no pipeline hang when the broker is unavailable -- `exporter.rs:473-498`, `producer.rs:203-224`. (Done: covered by `shutdown_flushes_under_sustained_traffic` (sustained traffic) and `shutdown_honors_deadline_when_broker_unavailable` (bounded deadline, unavailable broker) in `exporter.rs`.)
- [x] `Config`-based live reconfiguration is supported (build-and-swap with a bounded in-flight drain) or explicitly deferred with a recorded decision -- `exporter.rs:561-563`, `control.rs:261-264`. (Done: implemented in `KafkaExporter::reconfigure` + the `NodeControlMsg::Config` event-loop arm in `exporter.rs`; covered by `reconfigure_switches_topic`, `reconfigure_flushes_inflight_before_swap`, and `reconfigure_with_invalid_config_keeps_running`.)

## 3. Retry correctness

Validate integration with the Retry Processor, including transient/permanent error classification, retry exhaustion, and duplicate-delivery behavior.

- [ ] **The exporter has no internal retry loop** (beyond librdkafka's queue-full retry, `producer.rs:416-446`, which sleeps 100ms on `QueueFull` at `producer.rs:423`). Transient retry is delegated to a separate `processor:retry` node placed **upstream** of the exporter (`README.md:355-363`, example wiring `README.md:414-418`).
- [ ] **Error classification** (validate each path emits the correct nack kind):
  - Kafka **send failure** -> **transient** nack `reporter.nack(...)` (`exporter.rs:441-464`, `inc_failed` at `442`);
  - **encoding failure** -> **permanent** nack (`exporter.rs:405-407`);
  - **unconfigured signal type** -> **permanent** nack (`exporter.rs:354-356`);
  - **invalid dynamic topic** from a header -> **permanent** nack (`exporter.rs:369-373`, `TopicRoutingError`).
  Permanent vs. transient is realized via `NackMsg::new_permanent` vs. `NackMsg::new` in `EffectHandlerReporter` (`exporter.rs:118-136`).
- [ ] **Retry exhaustion.** After `max_elapsed_time`, the retry processor forwards a final nack; data is dropped at the source with no dead-letter queue (`README.md:433-434`).
- [ ] **Duplicate delivery / ordering.** Because the retry processor retries out-of-band, a later batch may be sent and acked before an earlier batch still being retried (`README.md:432`); a send that timed out or failed after the broker persisted it can also produce a duplicate on retry. Characterize the resulting duplication and ordering and confirm it is bounded.

**Retry support is out-of-process, by design.** A `Nack` classification is the exporter's terminal contribution; the actual retry/backoff lives in the separate `processor:retry` node (`README.md:355-363`). The work here is to **confirm** that path end-to-end via mock-broker integration testing rather than to add retry logic to the exporter.

**Acceptance criteria:**

- [ ] Transient vs. permanent classification validated end-to-end with an upstream `processor:retry` node -- `exporter.rs:354-356`, `369-373`, `405-407`, `441-464`.
- [ ] Retry-exhaustion (final nack, drop-at-source) and duplicate-delivery/ordering behavior characterized and shown to be bounded -- `README.md:432-434`.

## 4. Backpressure & resource bounds

Exercise bounded concurrency, queue saturation, prolonged broker outages, and memory behavior under sustained load.

- [ ] **Serial, inline send.** The event loop handles one pdata per iteration and awaits it to full delivery before the next `recv()` (`exporter.rs:524-530`); there is no `FuturesUnordered`, bounded concurrency, or delivery-future pipelining. This is flagged in-code: `exporter.rs:6-7` -- `//! ToDo: Currently only handles one kafka message add a time we should` / `//! improve the throughput by handling delivery futures`.
- [ ] **No application-level sending queue.** There is no `sending_queue` / `num_consumers` / in-memory queue backpressure (`README.md:345-353`, gap table `README.md:430`); batching, lingering, and buffering are delegated to librdkafka via `linger_ms` (`config.rs:464`) and `producer_config` queue knobs (e.g. `queue.buffering.max.messages`, `batch.num.messages`), which are only reachable through the escape hatch.
- [ ] **Queue saturation.** librdkafka `QueueFull` triggers an in-producer retry with a 100ms sleep (`producer.rs:416-446`); confirm this interacts sanely with the serial loop and the pipeline's own backpressure rather than unbounded stalling.
- [ ] **Prolonged broker outage.** With bounded `message.timeout.ms` (see section 8), each send fails within the deadline and becomes a transient nack; validate memory stays bounded (no unbounded in-flight accumulation) across a sustained outage.

**Acceptance criteria:**

- [ ] Behavior under queue saturation, prolonged broker outage, and sustained load is documented; memory stays bounded; backpressure propagation to the pipeline is confirmed -- `exporter.rs:524-530`, `producer.rs:416-446`.
- [ ] The bounded-concurrency / delivery-future direction is triaged with a recorded decision (behavior-preserving default) -- `exporter.rs:6-7`.

## 5. Delivery semantics

Test ACK/NACK propagation, ordering, partitioning, timeouts, and potentially-persisted messages across broker and network failures.

- [x] **Delivery is awaited via the delivery callback.** `ExporterFutureProducer::send` enqueues the record and awaits a oneshot resolved from the librdkafka delivery callback (`producer.rs:395-447`; callback -> `tx.send(...)` at `producer.rs:574-588`, with `producer.rs:587` -- `let _ = tx.send(owned_delivery_result); // TODO: handle error`). So each send resolves on final delivery/failure, not on enqueue. (Done: `send_success_reports_ack` and `send_failure_reports_transient_nack` in `exporter.rs` assert the outcome is decided by the resolved delivery -- an ack only after a successful callback and a transient nack only after a failure callback -- rather than on enqueue.)
- [x] **ACK/NACK propagation.** Success -> `inc_exported` + `reporter.ack(...)` (`exporter.rs:427-440`); failure -> `inc_failed` + transient `reporter.nack(...)` (`exporter.rs:441-464`). Downstream Ack/Nack observed by the loop update `acks_received` / `nacks_received` (`exporter.rs:537-547`). (Done: classification asserted at unit level via `export_once` + `RecordingReporter` in `send_success_reports_ack` (one ack, no nacks) and `send_failure_reports_transient_nack` (one transient nack, no permanent, no ack); the broker-backed counters are asserted via the terminal `logs.exported`/`logs.failed` in `delivery_success_increments_exported` and `produce_failure_increments_failed`.)
- [x] **Timeouts.** Send timeout is `Duration::from_millis(self.config.timeout_ms())` (`exporter.rs:425`), mapped to librdkafka `message.timeout.ms` (`config.rs:454`) and validated to `(0, 30_000]` (see section 8) so the await is always bounded. (Done: `send_times_out_within_bound_when_broker_unavailable` in `exporter.rs` points the send at an unroutable broker with a short `timeout_ms` and asserts the delivery await resolves as a failure well within a generous outer bound, i.e. it never hangs.)
- [x] **Partitioning.** Partition key is derived from transport headers when enabled (`partitioner.rs:80-91`) and applied to the record only when present (`exporter.rs:416-419`); the librdkafka partitioner strategy is configurable (`config.rs:788-825`, applied at `config.rs:467`). (Done: `same_partition_key_maps_to_stable_partition` sends many same-key records to a 4-partition topic and asserts they all land on one partition with the documented header-derived key (key-to-partition stability); `null_key_distributes_evenly_across_partitions` asserts keyless records reach every partition in a near-even split (round-robin).)
- [x] **Potentially-persisted messages.** A send that fails or times out after the broker has already persisted the record yields a transient nack and a possible duplicate on retry; validate this across broker restarts, leader elections, and network partitions. (Done/characterized: `recovers_across_broker_restart_and_leader_reassignment` exercises a broker restart with explicit leader reassignment and asserts no accepted record is lost and post-restart records are delivered; `produce_failure_is_bounded_and_not_persisted_on_mock` characterizes the produce-failure outcome as a bounded transient failure. NOTE: the true persisted-despite-reported-failure duplicate window (record persisted before an ack lost to a network partition) cannot be reproduced by the in-process `MockCluster` and requires a real broker; this is documented in the test.)

**Acceptance criteria:**

- [x] ACK/NACK propagation, ordering, partitioning, and timeout behavior validated, including duplicate/at-least-once behavior for messages persisted despite a reported failure, across broker and network failures -- `exporter.rs:425-464`, `producer.rs:395-447`, `partitioner.rs:80-91`. (Done: ACK/NACK -> `send_success_reports_ack`, `send_failure_reports_transient_nack`, `delivery_success_increments_exported`, `produce_failure_increments_failed`; ordering -> `preserves_per_partition_order` (monotonic per-partition offsets in send order); partitioning -> `same_partition_key_maps_to_stable_partition`, `null_key_distributes_evenly_across_partitions`; timeouts -> `send_times_out_within_bound_when_broker_unavailable`; broker/leader failures -> `recovers_across_broker_restart_and_leader_reassignment`. The persisted-despite-failure duplicate window is characterized via `produce_failure_is_bounded_and_not_persisted_on_mock` with a documented mock limitation requiring a real broker for a true duplicate.)

## 6. Kafka integration

Add broker-backed success and failure tests covering supported encodings, acknowledgements, compression, TLS, SASL, and AWS MSK IAM.

- [ ] **Encodings.** `OtlpProto` via `encode_to_otlp_bytes` (`encoder.rs:39-50`) and `OtapProto` via `encode_to_batch_arrow_record_bytes` (`encoder.rs:63-77`); the message-format header (`MessageFormat`, values `otlp`/`otap`) is always written (`exporter.rs:285-293`, constants `common/kafka/mod.rs:361-375`). (`OtlpJson` is not implemented -- `common/kafka/mod.rs:207-209`.)
- [ ] **Acknowledgements.** `required_acks` = `None`/`One`/`All` mapped to `request.required.acks` `0`/`1`/`-1` (`config.rs:138-162`, applied at `config.rs:462`).
- [ ] **Compression.** `gzip`/`snappy`/`lz4`/`zstd` mapped to `compression.type` (`config.rs:755-779`, applied at `config.rs:458`); `snappy`/`lz4` are noted as not end-to-end tested (`config.rs:753-754`).
- [ ] **TLS.** `ca_file` / `cert_file` / `key_file` / `key_password` / `insecure`, mTLS and server-only, mapped to `ssl.*` properties (`common/kafka/security.rs:119-136`; validation `common/kafka/mod.rs:166-198`).
- [ ] **SASL.** `PLAIN`, `SCRAM-SHA-256`, `SCRAM-SHA-512`, and `AWS_MSK_IAM_OAUTHBEARER` (`common/kafka/auth.rs:14-32`); protocol selection in `resolve_security_protocol` (`common/kafka/security.rs:45-58`).
- [ ] **AWS MSK IAM.** OAUTHBEARER token generation and refresh via `generate_oauth_token` (spawns a std thread with a current-thread Tokio runtime to `block_on(generate_auth_token(region))`, `common/kafka/aws.rs:37-60`); `ENABLE_REFRESH_OAUTH_TOKEN = true` (`common/kafka/aws.rs:34`, `77`). Requires building with the `aws` feature (see section 8).

**Acceptance criteria:**

- [ ] Broker-backed success and failure tests cover all supported encodings, the acks matrix, compression codecs, the TLS/SASL matrix, and AWS MSK IAM token refresh -- primarily via `MockCluster`, with a real broker where the mock cannot exercise the behavior.

## 7. Telemetry

Verify metric names, units and record counts, final shutdown snapshots, and safe structured events.

- [ ] **Metric set `exporter.kafka`, all `Counter<u64>`** (`metrics.rs:25-58`): `logs_exported` / `logs_failed`, `metrics_exported` / `metrics_failed`, `traces_exported` / `traces_failed`, `acks_received`, `nacks_received`, `topic_from_header`, `topic_from_static_config`. **There are no gauges or up/down counters.**
- [ ] **Known gap -- unit vs. count semantics.** The `*_exported` / `*_failed` metrics carry per-record units (`{log}`, `{datapoint}`, `{span}`) and the README describes them as "Number of ... records" (`README.md:521-526`), but the code increments **by 1 per pdata/batch** regardless of how many records the batch contains (`inc_exported` / `inc_failed` call `.inc()`, `metrics.rs:62-77`). Similarly `acks_received` / `nacks_received` are per batch. Reconcile the units/descriptions with the actual counting (either count records or relabel as batches).
- [ ] **Final shutdown snapshot.** The terminal state carries `self.metrics.snapshot()` (`exporter.rs:559`); confirm the snapshot reflects all activity up to shutdown.
- [ ] **Safe structured events.** The node currently emits no structured events (`README.md:532-534`); the `otel_*` / `effect_handler.info` log sites are enumerated in section 1. Ensure any events/logs are bounded and free of sensitive or adversarial content.
- [ ] **Operational gauges gap.** No gauges exist for in-flight record count, producer queue depth, or end-to-end latency (`metrics.rs`). Propose adding the operationally important gauges or record a decision to defer.

**Acceptance criteria:**

- [ ] Metric names, units, and record-vs-batch counting are reconciled and verified -- `metrics.rs:62-77`, `README.md:521-526`.
- [ ] The final shutdown snapshot is verified and structured events/logs are confirmed safe -- `exporter.rs:559`, section 1 log sites.
- [ ] Operational-gauge gaps are addressed or explicitly deferred with a recorded decision -- `metrics.rs`.

## 8. Configuration & packaging

Validate timeout limits, producer escape-hatch interactions, feature gates, and optional dependency isolation.

- [ ] **Timeout limits.** `timeout_ms` is validated to reject `0` (which maps to librdkafka's infinite `message.timeout.ms`) and any value `> MAX_TIMEOUT_MS` (`config.rs:533-544`, `MAX_TIMEOUT_MS = 30_000` at `config.rs:722`); it maps to `message.timeout.ms` (`config.rs:454`).
- [ ] **Producer escape-hatch interactions.** `producer_config` is applied first so built-in fields override conflicting keys (`build_client_config`, `config.rs:441-489`, order at `447-485`); overridden keys emit `otel_warn!("kafka.exporter.producer_config.overridden_key", ...)` (`exporter.rs:197-204`) using the managed-key list `MANAGED_PRODUCER_CONFIG_KEYS` (`config.rs:21-40`, checked via `overridden_producer_config_keys`, `config.rs:699-706`). `debug` and `log_level` are applied last to override the escape hatch (`config.rs:479-485`). Validate the precedence and the warning surface.
- [ ] **Feature gates & optional-dependency isolation.** `kafka-exporter` pulls `dep:rdkafka`, `dep:futures-channel`, `dep:futures-util`, `dep:xxhash-rust`, `dep:hex` (`Cargo.toml:81-87`); `aws` pulls `dep:aws-config`, `dep:aws-msk-iam-sasl-signer` (`Cargo.toml:88-91`). `aws` is **not** auto-enabled by `kafka-exporter`, so AWS MSK IAM requires building with both. Validate builds with `kafka-exporter` alone, `kafka-exporter + aws`, and neither, and that AWS-only types are correctly gated (e.g. `SaslMechanism::AwsMskIamOauthbearer`, `build_aws_msk_context` defense-in-depth at `common/kafka/security.rs:95-107`).

**Acceptance criteria:**

- [ ] Timeout bounds, escape-hatch override precedence + warning, and validation rules are covered by tests -- `config.rs:441-544`, `exporter.rs:197-204`.
- [ ] Feature-gated builds (`kafka-exporter` alone, with `aws`, and disabled) compile and isolate optional dependencies correctly -- `Cargo.toml:81-91`.

## 9. Performance & scalability

Benchmark throughput, latency, polling overhead, and multi-core behavior with slow and unavailable brokers.

- [ ] **Throughput / latency benchmarks** per encoding and per signal, at 1 and N partitions; the serial single-message send loop (`exporter.rs:524-530`) is the current ceiling and should be measured directly.
- [ ] **Polling overhead.** The custom producer runs a dedicated poll thread at a **1-second** interval as a workaround for high idle CPU in upstream rdkafka (`producer.rs:88-98`, rationale `README.md:542-544`); measure its idle and under-load overhead and confirm the interval is appropriate.
- [ ] **Slow / unavailable brokers.** Measure behavior and pipeline-core impact when the broker is slow or unavailable (ties to section 4 backpressure and section 2 shutdown); confirm broker operations never stall the pipeline core beyond the configured deadline.
- [ ] **Multi-core behavior.** Characterize scaling across cores/instances (the exporter node itself is single-task and `!Send`-bound to its thread).

**Acceptance criteria:**

- [ ] Performance characteristics are documented via benchmarks (throughput, latency, poll overhead, multi-core), with slow/unavailable-broker behavior confirmed not to stall the pipeline core, and the concurrency direction from section 4 quantified -- `exporter.rs:524-530`, `producer.rs:88-98`.

## Cross-cutting: integration test suite

Integration testing is done primarily against `rdkafka::mocking::MockCluster` (librdkafka's built-in mock), which runs **in-process** and in CI by default with no Docker dependency. The exporter drives a fully-wired node through the engine's `ExporterWrapper` (control channel, pdata channel, effect handler) and then consumes the produced records back from the mock broker to assert on topic, payload bytes, message-format header, and partition key. The current suite covers: per-signal OTLP happy path (traces/metrics/logs), OTAP encoding (`otap` format header + `BatchArrowRecords` decode), dynamic topic routing from a transport header, partition-key derivation from transport headers, graceful shutdown flushing buffered records, deadline-bounded shutdown against an unavailable broker, shutdown flushing under sustained traffic, live reconfiguration via `Config` build-and-swap, and delivery semantics (ack/nack propagation and classification, timeout-boundedness, key-to-partition stability, null-key round-robin, per-partition ordering, and recovery across a broker restart / leader reassignment) (`exports_logs_otlp_to_mock_broker`, `exports_traces_otlp_to_mock_broker`, `exports_metrics_otlp_to_mock_broker`, `exports_logs_otap_sets_otap_format_header`, `routes_to_topic_from_transport_header`, `sets_partition_key_from_transport_headers`, `shutdown_flushes_buffered_records`, `shutdown_honors_deadline_when_broker_unavailable`, `shutdown_flushes_under_sustained_traffic`, `reconfigure_switches_topic`, `reconfigure_flushes_inflight_before_swap`, `reconfigure_with_invalid_config_keeps_running`, `send_success_reports_ack`, `send_failure_reports_transient_nack`, `delivery_success_increments_exported`, `produce_failure_increments_failed`, `send_times_out_within_bound_when_broker_unavailable`, `same_partition_key_maps_to_stable_partition`, `null_key_distributes_evenly_across_partitions`, `preserves_per_partition_order`, `recovers_across_broker_restart_and_leader_reassignment`, `produce_failure_is_bounded_and_not_persisted_on_mock`, `disallowed_dynamic_topic_is_permanently_nacked`, `allowed_dynamic_topic_is_delivered`).

- [ ] Extend coverage on top of the mock broker for the security-constraint, shutdown/reconfigure, retry-processor, backpressure, delivery-semantics, auth/TLS, and telemetry scenarios called out above.

**Acceptance criteria:**

- [ ] The mock broker suite stays green in CI and is extended to cover the security, shutdown/reconfiguration, retry, backpressure, delivery-semantics, auth/TLS, and telemetry scenarios above, with a real broker used only where the mock cannot exercise the behavior.
