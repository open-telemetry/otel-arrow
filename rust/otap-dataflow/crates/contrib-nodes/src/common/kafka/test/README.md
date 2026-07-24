# Kafka Test Suite

An in-process Kafka test suite built on
[`rdkafka::mocking::MockCluster`](https://docs.rs/rdkafka/latest/rdkafka/mocking/struct.MockCluster.html)
(librdkafka's built-in mock broker). It lets a test stand up a mock broker,
connect producers/consumers (or, via the node harnesses, the Kafka exporter and
receiver), produce and consume messages, inspect broker state, drive
failure/rebalance scenarios, and assert on metrics -- all without Docker or an
external broker, so the tests run by default in CI.

The suite lives at `crates/contrib-nodes/src/common/kafka/test/` and is the
primary vehicle for the Kafka exporter and receiver integration tests. It is
test-only and never compiled into the shipping library: the modules compile only
when a `kafka-*` feature is enabled.

## Contents

- [When to use it](#when-to-use-it)
- [Threading model (important)](#threading-model-important)
- [Quick start](#quick-start)
- [Cluster and topics](#cluster-and-topics)
- [Producing](#producing)
- [Consuming](#consuming)
- [Consumer-group rebalancing](#consumer-group-rebalancing)
- [Manipulating the broker (fault injection)](#manipulating-the-broker-fault-injection)
- [Inspecting broker state](#inspecting-broker-state)
- [Asserting on messages](#asserting-on-messages)
- [Polling helpers](#polling-helpers)
- [Node harnesses (exporter and receiver)](#node-harnesses-exporter-and-receiver)
- [Asserting on metrics](#asserting-on-metrics)
- [Errors](#errors)
- [Limitations](#limitations)

## When to use it

Use it for integration-style tests that need a real librdkafka client talking to
a broker: produce/consume round-trips, commit and offset behavior, consumer-group
rebalancing, drain/shutdown, and broker fault/recovery scenarios. Pure logic that
does not need a broker (config validation, offset-tracker math, header parsing)
should stay in plain unit tests.

## Threading model (important)

`KafkaTestCluster` wraps a `!Send` `MockCluster` that holds a raw pointer, must
live on its creation thread for the broker's whole lifetime, and tears the broker
down when dropped. Always drive it from a current-thread `LocalSet` using
[`with_cluster`](#cluster-and-topics) or `run_on_local_set`, which own the
`LocalSet`, build the cluster, and keep the mock alive for the duration of the
supplied closure. Clients and node harnesses spawned with `spawn_local` share the
cluster through an `Rc`.

## Quick start

Produce and consume a record directly (no node involved):

```rust,ignore
use crate::common::kafka::test::cluster::KafkaTestCluster;
use crate::common::kafka::test::with_cluster;

#[tokio::test]
async fn round_trips_a_record() {
    with_cluster(
        KafkaTestCluster::builder().topic("it-logs"),
        |cluster| async move {
            let consumer = cluster.consumer().subscribe(&["it-logs"]);
            let producer = cluster.producer().build();
            producer.send("it-logs", b"payload").await.unwrap();
            consumer
                .recv()
                .await
                .assert_topic("it-logs")
                .assert_payload(b"payload");
        },
    )
    .await;
}
```

`run_on_local_set(f)` is a shorthand for `with_cluster` with a default
single-broker cluster and no pre-created topics.

## Cluster and topics

Build the cluster with `KafkaTestCluster::builder()`:

| Builder method | Purpose |
| --- | --- |
| `broker_count(n)` | Number of brokers (default 1). |
| `topic(name)` | Add a single-partition topic (replication 1). |
| `topic_with(name, partitions, replication)` | Add a topic with an explicit layout. |
| `build()` | Create the mock broker and all declared topics. |

On the built `KafkaTestCluster`:

| Method | Purpose |
| --- | --- |
| `bootstrap_servers()` | The `bootstrap.servers` string for wiring clients. |
| `create_topic(name, partitions, replication)` | Create a topic after construction. |
| `producer()` | A `TestProducerBuilder` bound to this cluster. |
| `consumer()` | A `TestConsumerBuilder` bound to this cluster. |
| `faults()` | A `BrokerFaults` handle (fault injection). |
| `inspect()` | A `BrokerInspector` for topology and watermarks. |
| `inspect_group(group)` | A `BrokerInspector` that can also read committed offsets. |
| `mock()` | Raw escape hatch to the underlying `MockCluster`. |

The mock only auto-creates single-partition topics on produce, so any test that
needs more than one partition (or a deterministic layout) must pre-create the
topic via the builder or `create_topic`.

## Producing

`cluster.producer()` returns a `TestProducerBuilder` (`client_id`,
`message_timeout`, raw `set(key, val)`, then `build()`). The default
`message.timeout.ms` is 20s.

`TestProducer` methods:

| Method | Purpose |
| --- | --- |
| `send(topic, payload)` | Produce a single payload. |
| `send_full(SendRecord)` | Produce a fully-specified record. |
| `send_to_partition(topic, partition, payload)` | Produce to a specific partition. |
| `send_n(topic, payloads)` | Produce several payloads in order (panics on failure). |
| `produce_per_partition(topic, partitions, count, payload)` | Produce `count` records to each partition, keyed `k-{partition}-{i}`. |
| `flush(timeout)` | Flush buffered records. |

`SendRecord::new(topic, payload)` builds a record; chain `key(bytes)`,
`partition(i)`, and `header(key, value)` before passing it to `send_full`.
`send`/`send_full`/`send_to_partition` return `Result<(), TestError>` so negative
tests can assert failures; the batch helpers panic on failure.

## Consuming

`cluster.consumer()` returns a `TestConsumerBuilder` (`group_id`,
`auto_offset_reset`, `enable_auto_commit`, `session_timeout`, raw `set`). Finish
with `subscribe(&[topics])` or `assign_partition(topic, partition)`. Defaults:
`auto.offset.reset=earliest`, `enable.auto.commit=false`, `session.timeout.ms=6s`.
An unset `group_id` gets a unique `kafka-test-group-{n}` id to prevent accidental
cross-test group sharing.

`TestConsumer` methods:

| Method | Purpose |
| --- | --- |
| `recv()` | Receive one message (default 30s timeout; panics on timeout). |
| `try_recv(timeout)` | Receive one message, `None` on timeout. |
| `recv_n(n)` | Receive exactly `n` messages. |
| `collect_until_idle(idle)` | Drain until no message arrives within `idle`. |
| `assignment()` | Current `(topic, partition)` assignment. |
| `wait_for_assignment(min_partitions, timeout)` | Drive polling until assigned. |
| `committed_offset(topic, partition)` | Committed offset for this consumer's group. |
| `committed_offsets(&[(topic, partition)])` | Committed offsets for several pairs. |

The free function `committed_offset(brokers, group, topic, partition)` reads a
committed offset with an independent probe client (no live consumer needed).

`TestConsumer` also offers chaining `&Self` assertions: `assert_assignment_count`,
`assert_assigned`, `assert_not_assigned`, `assert_committed_offset`, plus the async
`assert_no_more_messages(idle)` (panics if any record arrives before `idle`).

## Consumer-group rebalancing

Multiple `TestConsumer`s sharing a `group_id` model a consumer group.
`RebalanceTrigger::join(cluster, group, &[topics], timeout)` joins an extra
group member and polls it until it holds an assignment, which deterministically
drives a revoke on the other members. Keep the trigger alive as long as the
revoke must persist; drop it to let partition ownership revert.

## Manipulating the broker (fault injection)

`cluster.faults()` returns a `BrokerFaults` handle. Primitives:

| Method | Purpose |
| --- | --- |
| `broker_down(id)` / `broker_up(id)` | Mark a broker down/up (`-1` = all). |
| `all_brokers_down()` / `all_brokers_up()` | Convenience for all brokers. |
| `round_trip_time(id, duration)` | Inject per-request round-trip latency. |
| `set_partition_leader(topic, partition, Option<broker>)` | Set (or clear) a partition leader. |
| `set_group_coordinator(group, broker)` | Set a group's coordinator broker. |
| `inject_request_errors(api, &[errors])` | Fail a request API with a sequence of errors. |
| `clear_request_errors(api)` | Clear injected request errors. |
| `set_topic_error(topic, err)` | Set a topic-level error. |
| `set_api_version(api, min, max)` | Restrict the advertised API-version range. |

Composite recipes wrap the primitives into intention-revealing operations:

| Method | Purpose |
| --- | --- |
| `restart_broker_reassigning_leader(broker_id, topic, partition, new_leader)` | Take a broker down, move that partition's leadership to `new_leader`, bring it back up. |
| `fail_offset_commits(&[errors])` / `clear_offset_commit_failures()` | Fail/clear consumer offset-commit requests (exercises `offset_commit_errors`). |
| `fail_produce(&[errors])` / `clear_produce_failures()` | Fail/clear produce requests (exercises the exporter transient-nack path). |
| `fail_fetch(&[errors])` / `clear_fetch_failures()` | Fail/clear fetch requests (exercises the receiver `transport_errors` path). |

Note: `broker_down`/`broker_up` do not trigger leader election, so a naive
down/up leaves a partition leaderless. Use `set_partition_leader` manually, or the
`restart_broker_reassigning_leader` recipe, when a restart must keep a partition
served.

## Inspecting broker state

`MockCluster` exposes no getters, so read-only inspection goes through short-lived
client probes. `cluster.inspect()` returns a `BrokerInspector` (topology and
watermarks); `cluster.inspect_group(group)` additionally enables committed-offset
queries.

| Method | Purpose |
| --- | --- |
| `topics()` | All topics and their partition counts (`Vec<TopicInfo>`). |
| `partitions(topic)` | Per-partition topology: leader, replicas, ISR (`Vec<PartitionInfo>`). |
| `topic_exists(topic)` | Whether a topic exists. |
| `watermarks(topic, partition)` | `(low, high)` watermark offsets. |
| `message_count(topic, partition)` | `high - low` for a partition. |
| `committed_offset(topic, partition)` | Committed offset (requires `inspect_group`). |
| `committed_offsets(&[(topic, partition)])` | Committed offsets for several pairs. |
| `probe_timeout(duration)` | Override the probe timeout (default 10s). |

The inspector also exposes chaining `&Self` assertions: `assert_topic_exists`,
`assert_topic_absent`, `assert_partition_count`, `assert_message_count`,
`assert_message_count_at_least`, `assert_high_watermark`, `assert_leader`, and
`assert_committed_offset` (the last requires `inspect_group`). Because
`MockCluster` does not run leader election, `assert_leader` checks the leader the
mock reports (as configured or after an explicit reassignment fault).

## Asserting on messages

`recv`/`recv_n` return `ConsumedMessage`, an owned snapshot with public fields
`topic`, `partition`, `offset`, `key`, `payload`, and `headers`. Assertion methods
return `&Self`, so they chain:

```rust,ignore
let msg = consumer.recv().await;
msg.assert_topic("it-logs")
    .assert_payload(b"payload")
    .assert_key(b"k-0-0")
    .assert_header("x-tenant-id", b"acme")
    .assert_format_otlp();
```

Single-record assertions (all chain, all `&Self`): `assert_topic`, `assert_key`,
`assert_no_key`, `assert_partition`, `assert_offset`, `assert_payload`,
`assert_payload_len`, `assert_no_payload`, `assert_header`, `assert_has_header`,
`assert_no_header`, `assert_format(bytes)`, `assert_format_otlp`,
`assert_format_otap`. `header(key)` and `message_format()` return raw values for
custom assertions.

For a batch of records, the free functions `count_by_topic(&msgs)` and
`count_by_partition(&msgs)` tally a slice directly.

## Polling helpers

`poll_until(timeout, interval, predicate)` and `poll_until_async(timeout,
interval, predicate)` retry a condition until it holds or the timeout elapses,
sleeping `interval` between attempts. Use the async variant when the predicate
must drive `recv()` or query the broker between attempts.

## Node harnesses (exporter and receiver)

`super::node_harness` wraps the Kafka exporter/receiver so a test can drive a
fully-wired node against the mock broker. Each harness takes `&KafkaTestCluster`,
sets `bootstrap.servers` from it, owns the engine wiring plus `LocalSet` spawn and
lifecycle, and exposes intention-revealing handles. `KafkaTopics::logs(topic,
fmt)` / `KafkaTopics::traces(topic, fmt)` describe a per-signal topic layout for
the `start_for` variants.

`KafkaExporterHarness`:

| Method | Purpose |
| --- | --- |
| `start(cluster, cfg)` / `start_for(cluster, topics)` | Start the exporter on the current `LocalSet`. |
| `send_pdata(pdata)` | Send a pdata batch to the exporter. |
| `shutdown(deadline)` | Request a graceful shutdown. |
| `await_stopped()` | Await task completion. |
| `await_terminal_state()` | Await completion and return the final metric snapshots. |

`KafkaReceiverHarness`:

| Method | Purpose |
| --- | --- |
| `start(cluster, cfg)` / `start_with_capture(cluster, cfg, policy)` / `start_for(cluster, topics)` | Start the receiver on the current `LocalSet`. |
| `recv_pdata()` / `try_recv_pdata(timeout)` | Read one decoded pdata batch. |
| `try_recv_runtime(timeout)` | Read one runtime-control message (e.g. `ReceiverDrained`). |
| `ack(pdata)` | Acknowledge consumed pdata so manual-commit offsets advance. |
| `drain(deadline)` | Request a receiver-first ingress drain. |
| `shutdown(deadline)` | Request a graceful shutdown. |
| `await_stopped()` / `await_terminal_state()` | Await task completion (optionally returning final metric snapshots). |

## Asserting on metrics

A node reports its final counters in the `TerminalState` returned at graceful
shutdown. Call `await_terminal_state()` on either harness, then read individual
counters from the snapshots with the shared `node_harness::node_metrics` helpers:

- `metric_value(&snapshot, name)` returns the `u64` value of a field, or `None`.
- `FoldedMetrics` folds several snapshots and exposes cumulative `value(name)` /
  `contains(name)`.

Metric names accept either the Rust field identifier (`offset_commit_errors`) or
the emitted dotted form (`offset.commit.errors`); the helpers normalize `_` to `.`
before lookup. All Kafka node counters are `Counter<u64>`.

Mid-run metric collection is not wired: the node only flushes counters into the
returned `TerminalState` at shutdown, so read metrics after
`await_terminal_state()` rather than while the node is running.

## Errors

Setup/build helpers (cluster/topic/client creation) panic with rich context,
because a setup failure is a test-environment bug. Fallible runtime operations
(`send`, `recv`-style, harness `send_pdata`) return `TestError` so negative tests
can assert failures. `TestError` variants: `ClusterSetup`, `Produce`, `Consume`,
`Timeout`.

## Limitations

- No TLS or SASL handshake: `MockCluster` does not perform real TLS termination
  or SASL authentication, so auth/TLS matrices need a real broker (out of scope
  for this suite).
- No CreateTopics admin API: create topics via the builder or `create_topic`.
- `broker_down`/`broker_up` do not trigger leader election (see the fault-injection
  note above).
- Mid-run metric collection is not wired; metrics are observable only via the
  terminal snapshot at shutdown.
