# Kafka Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:kafka` (`urn:otel:receiver:kafka`)
- Feature gate: `kafka-receiver` (also enabled by `contrib-receivers`)
- Stability: experimental

## Overview

The Kafka receiver consumes OpenTelemetry traces, metrics, and logs from
Apache Kafka topics. It supports OTLP and OTAP protobuf encodings,
per-signal topic routing, SASL authentication (PLAIN, SCRAM, AWS MSK IAM),
TLS, manual and automatic offset commit modes, and Kafka message header
extraction into resource attributes or pipeline transport headers.

## Getting Started

The smallest valid configuration needs only one topic and the required
connection fields. All other fields have sensible defaults:

```yaml
type: receiver:kafka
config:
  brokers: "broker-1:9092"
  group_id: "my-consumer-group"
  client_id: "my-client"
  traces:
    topics:
      - "otel-traces"
```

## Configuration

### Top-Level Fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `brokers` | string | **required** | Comma-separated list of Kafka broker addresses. |
| `group_id` | string | **required** | Kafka consumer group ID. |
| `client_id` | string | **required** | Kafka client ID. |
| `group_instance_id` | string | *none* | Static group instance ID for Kafka static membership. On a multi-core pipeline the configured value is automatically suffixed with the pipeline core ID (e.g. `instance-1` becomes `instance-1-0`, `instance-1-1`, ...) so each core is a distinct static member. |
| `auth` | object | *none* | Authentication configuration (see [Authentication](#authentication)). |
| `tls` | object | *none* | TLS configuration for broker connections (see [TLS Configuration](#tls-configuration)). |
| `traces` | object | `{}` | Per-signal config for traces (see [Per-Signal Configuration](#per-signal-configuration)). |
| `metrics` | object | `{}` | Per-signal config for metrics. |
| `logs` | object | `{}` | Per-signal config for logs. |
| `auto_offset_reset` | string | `latest` | Where to start consuming when no committed offset exists. |
| `commit` | object | `{mode: manual}` | Commit configuration (see [Commit Configuration](#commit-configuration)). |
| `session_timeout_ms` | integer | `10000` | Session timeout in milliseconds. |
| `heartbeat_interval_ms` | integer | `3000` | Heartbeat interval in milliseconds. |
| `min_fetch_bytes` | integer | `1` | Minimum number of bytes to fetch. |
| `max_fetch_bytes` | integer | `1048576` | Maximum number of bytes to fetch (1 MB). |
| `max_fetch_wait_ms` | integer | `250` | Maximum time to wait for a fetch response in milliseconds. |
| `max_partition_fetch_bytes` | integer | `1048576` | Maximum bytes per partition per fetch (1 MB). |
| `isolation_level` | string | `read_uncommitted` | Consumer transaction isolation level. |
| `resource_attrs_from_headers` | map | `{}` | Rules for extracting Kafka message headers into resource attributes. |
| `enable_idempotency` | bool | `false` | Skip duplicate messages by offset (manual commit mode only). |
| `rebalance_strategy` | string | *none* | Partition assignment strategy: `range`, `round_robin`, or `cooperative_sticky`. When omitted, librdkafka uses its default (`range,roundrobin`). |
| `message_format_header` | string | `"MessageFormat"` | Kafka header key for per-message format detection. The receiver checks each message for a header with this key and value `otlp` or `otap` to override the per-signal encoding. |
| `debug` | list | *none* | List of librdkafka debug contexts: `generic`, `broker`, `topic`, `metadata`, `feature`, `queue`, `msg`, `protocol`, `cgrp`, `security`, `fetch`, `interceptor`, `plugin`, `consumer`, `admin`, `eos`, `mock`, `assignor`, `conf`, `telemetry`, `all`. |
| `log_level` | string | *none* | Librdkafka log level: `emerg`, `alert`, `critical`, `error`, `warning`, `notice`, `info`, `debug`. When omitted, inferred from the application's log configuration. |
| `consumer_config` | map | `{}` | Additional librdkafka consumer settings as key-value string pairs. |

### Per-Signal Configuration

Each signal type (`traces`, `metrics`, `logs`) accepts a nested configuration:

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `topics` | list | `[]` | Topics to subscribe to. Entries starting with `^` are regex patterns. |
| `exclude_topics` | list | `[]` | Regex patterns for topics to exclude (requires at least one regex in `topics`). |
| `encoding` | string | `otlp_proto` | Default encoding format: `otlp_proto` or `otap_proto`. |

At least one signal must have non-empty `topics` for the receiver to consume any data. Topic names must be **disjoint across signal types** -- the receiver rejects configurations where the same topic appears in more than one signal type.

#### Topic Configuration

Configure topics inside each signal block:

```yaml
config:
  traces:
    topics:
      - "otel-traces-prod"
      - "otel-traces-staging"
      - "^traces-team-.*"
  metrics:
    topics:
      - "otel-metrics"
  logs:
    topics:
      - "otel-logs"
```

#### Regex Topic Patterns

Topic values that start with `^` are treated as regular expressions, enabling pattern-based subscription. For example, `"^traces-.*"` subscribes to all topics matching that pattern. Invalid regex patterns cause a configuration error at startup. Regex patterns can be freely mixed with literal topic names within the same signal.

#### Exclude Topics

When at least one topic in a signal is a regex pattern, you can use `exclude_topics` to filter out specific matches:

```yaml
config:
  traces:
    topics:
      - "^traces-.*"
    exclude_topics:
      - "^traces-test$"
      - "^traces-internal-.*"
```

Each `exclude_topics` entry must be a valid regex and non-empty. The `exclude_topics` field is only allowed when at least one topic in the same signal is a regex pattern.

> **Note:** `exclude_topics` is only valid alongside regex topic subscriptions.
> If all topics in a signal are literal names, specifying `exclude_topics` will
> cause a configuration error at startup.

#### Encoding

Each signal can specify its own encoding format via the `encoding` field:

| Value | Description |
| --- | --- |
| `otlp_proto` | OTLP protobuf encoding (default). |
| `otap_proto` | OTAP Arrow protobuf encoding. |

Encoding can differ per signal:

```yaml
config:
  traces:
    topics: ["traces-prod"]
    encoding: otlp_proto
  metrics:
    topics: ["metrics-prod"]
    encoding: otap_proto
```

Individual Kafka messages can override the per-signal encoding via the `message_format_header` header (defaults to `"MessageFormat"`). The receiver checks each incoming message for a header matching the configured key. If the header is present and its value is `otlp` or `otap`, that encoding is used instead of the per-signal default. If the header is absent or unrecognized, the per-signal encoding is used as a fallback.

### Commit Configuration

The `commit` block controls offset management:

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `mode` | string | `manual` | `manual` (at-least-once, up to a terminal nack -- see [Failure Handling and Retries](#failure-handling-and-retries)) or `auto` (at-most-once). |
| `interval_ms` | integer | *none* | Commit interval in milliseconds. Optional; see mode-specific behavior below. |

#### Offset Management

The receiver supports two offset commit modes controlled by `commit.mode`.

#### Manual Mode (`commit.mode: manual`) -- Default

The receiver tracks each consumed message individually per partition. Offsets are only committed up to the lowest un-acknowledged message, preventing offset skipping when downstream acknowledgements arrive out of order.

The `commit.interval_ms` value controls a periodic safety-net timer for offset commits. If `commit.interval_ms` is not set, the safety-net timer is disabled and offsets are committed purely through ack/nack signals from downstream processing.

```yaml
config:
  commit:
    mode: manual
    interval_ms: 1000
```

#### Failure Handling and Retries

In manual mode the committable offset advances on **both acks and nacks**. A
nack reaching the receiver is treated as a **terminal** outcome: the receiver
advances past the message and commits its offset, so the message is **not**
redelivered.

Retrying transient downstream failures is a separate pipeline concern handled
by the [retry processor](../../../../core-nodes/src/processors/retry_processor/README.md)
(`processor:retry`). To get at-least-once delivery, place a retry
processor in the pipeline between the Kafka receiver and a failure-prone
exporter:

- The retry processor retries **transient** nacks with exponential backoff and
  only forwards a nack to the receiver once retries are exhausted (a "final
  retry") or when the failure is marked permanent.
- While retries are in progress the message's offset stays in-flight at the
  receiver, so it is **not** committed and remains eligible for redelivery on
  restart.
- Only genuinely terminal failures reach the receiver and advance the offset.

See [At-Least-Once with the Retry Processor](#at-least-once-with-the-retry-processor)
for a complete pipeline example.

#### Comparison with the Go Kafka receiver

The OpenTelemetry Collector's Go Kafka receiver bundles offset committing,
message marking, and error backoff into a single component. This receiver splits
those concerns across two pipeline nodes: the receiver handles offset commit and
marking, while transient-failure retry is delegated to the
[retry processor](../../../../core-nodes/src/processors/retry_processor/README.md).

The tables below summarize
the **feature gaps** relative to the Go receiver; the
[offset/error-handling mapping](#offset-and-error-handling) and
[Known gaps](#known-gaps--behavioral-differences) further down cover the
offset-commit and retry differences.

##### Signals

The Go receiver also consumes a `profiles` signal. This receiver supports
`traces`, `metrics`, and `logs` only -- support for `profiles` has not yet been added..

##### Encodings

| Encoding | Go | Here |
| --- | --- | --- |
| `otlp_proto` | yes | yes |
| `otap_proto` (OTAP Arrow) | no | yes |
| `otlp_json` | yes | no |
| `jaeger_proto` / `jaeger_json` (traces) | yes | no |
| `zipkin_proto` / `zipkin_json` / `zipkin_thrift` (traces) | yes | no |
| `raw` / `text` / `text_<encoding>` / `json` (logs) | yes | no |
| `azure_resource_logs` (logs) | yes (deprecated) | no |
| encoding extensions | yes | no |

##### Authentication mechanisms

| Mechanism | Go | Here |
| --- | --- | --- |
| SASL `PLAIN`, `SCRAM-SHA-256`, `SCRAM-SHA-512` | yes | yes |
| SASL `AWS_MSK_IAM_OAUTHBEARER` | yes | yes (requires the build-time `aws` feature) |
| Generic `OAUTHBEARER` (token-source extension) | yes | no |
| Kerberos / GSSAPI | yes | no |

##### Consumer / connection settings

The following Go first-class fields have **no dedicated config field** here.
Where librdkafka exposes an equivalent setting, it can still be set through the
[`consumer_config`](#consumer_config) passthrough.

| Go field | Equivalent here |
| --- | --- |
| `protocol_version` | `consumer_config` passthrough (`api.version.request`, etc.) |
| `resolve_canonical_bootstrap_servers_only` | `consumer_config` passthrough (`client.dns.lookup`) |
| `rack_id` | `consumer_config` passthrough (`client.rack`) |
| `use_leader_epoch` | not exposed |
| `conn_idle_timeout` | `consumer_config` passthrough (`connections.max.idle.ms`) |
| `metadata.full` / `metadata.refresh_interval` / `metadata.retry.*` | `consumer_config` passthrough (`topic.metadata.refresh.*`, `metadata.max.age.ms`) |
| `group_rebalance_strategies` (ordered list + custom balancer extensions) | single `rebalance_strategy` value; a comma-separated list can be set via `consumer_config` (`partition.assignment.strategy`) |

##### Message metadata propagation

The Go receiver injects `kafka.topic`, `kafka.partition`, and `kafka.offset`
(plus all Kafka headers) into each request's metadata/context for downstream
use. This receiver does **not** place those values into the pdata context; the
topic/partition/offset are used internally for offset routing and appear only on
log events. (Header values can still be surfaced via
[`resource_attrs_from_headers`](#header-extraction) or
[transport header capture](#transport-header-capture).)

##### Telemetry differences

The Go receiver exposes an opt-in `telemetry.metrics.kafka_receiver_records_delay`
gauge (consumer lag/delay) and per-metric enable toggles. This receiver emits
always-on counters only (see [Metric Sets](#metric-sets)) -- there is no
consumer-lag/delay gauge, no histograms, and no per-metric toggles.

##### Defaults and required fields

The Go receiver has no required settings (it defaults `brokers`, `group_id`,
`client_id`, and per-signal topics). This receiver **requires** `brokers`,
`group_id`, and `client_id`, and has no default topics -- at least one signal
must declare `topics`.

##### Offset and error handling

The table below maps the Go receiver's error-handling/offset options onto the
equivalent configuration here.

| Go Kafka receiver option | Equivalent here | Notes |
| --- | --- | --- |
| `autocommit.enable: true` | `commit.mode: auto` | At-most-once; offsets committed by the client regardless of downstream outcome. |
| `autocommit.enable: false` | `commit.mode: manual` | At-least-once up to a terminal nack. |
| `autocommit.interval` | `commit.interval_ms` | In `auto` mode forwarded to rdkafka as `auto.commit.interval.ms`; in `manual` mode it drives the safety-net commit timer. |
| `initial_offset: latest` / `earliest` | `auto_offset_reset: latest` / `earliest` | Same semantics. |
| `error_backoff.enabled` | Add a `processor:retry` node to the pipeline | Retry is a separate node, not a receiver field. Omit the node to disable. |
| `error_backoff.initial_interval` | retry processor `initial_interval` | Default `5s`. |
| `error_backoff.max_interval` | retry processor `max_interval` | Default `30s`. |
| `error_backoff.multiplier` | retry processor `multiplier` | Default `1.5`. |
| `error_backoff.max_elapsed_time` | retry processor `max_elapsed_time` | Default `300s` (5m). Set `0` is **not** supported here (must be > 0). |
| `error_backoff.randomization_factor` | *(no equivalent)* | The retry processor backoff has no jitter -- see gaps below. |
| `message_marking.after` | *(no exact equivalent)* | Offsets are tracked at receive time and committed on the downstream ack/nack; there is no "mark only after success" toggle. |
| `message_marking.on_error` / `on_permanent_error` | *(no exact equivalent)* | The receiver commits on **any** terminal nack (it treats a nack as terminal); there is no per-error-kind marking toggle. |
| `group_id`, `client_id`, `session_timeout`, `heartbeat_interval`, `group_rebalance_strategy`, fetch sizes, `tls`, `auth` | Direct receiver fields | See [Consumer Settings](#consumer-settings), [Authentication](#authentication), and [TLS Configuration](#tls-configuration). |

##### Known gaps / behavioral differences

Even with a retry processor in the pipeline, the following behaviors differ from
the Go Kafka receiver:

- **Commit-on-terminal-failure vs. replay from Kafka.** With
  `message_marking.on_error: false`, the Go receiver leaves a failed message's
  offset *unmarked*, so Kafka redelivers it (after a fetch rewind or on the next
  rebalance). This receiver **commits the offset once a terminal nack is
  received** (including after the retry processor exhausts its retries), so the
  message is advanced past at the source rather than replayed from Kafka. The
  retry processor protects against *transient* failures in-process, but it does
  not cause the receiver to hold the offset past a terminal nack.
- **No partition pause on error.** The Go receiver pauses a partition on a
  permanent error (or when no backoff is configured) until the next rebalance.
  This receiver keeps consuming subsequent offsets on the same partition.
- **No backoff jitter.** The retry processor uses exponential backoff without a
  `randomization_factor` (jitter) equivalent.
- **Retry ordering.** The Go receiver retries inline per partition, preserving
  per-partition order at the cost of head-of-line blocking. The retry processor
  retries out-of-band, so a later message can be acked and committed before an
  earlier message that is still being retried.
- **Marking selectivity.** There is no equivalent to `message_marking.after`,
  `on_error`, or `on_permanent_error`; nack handling is uniform.

Out of scope for error handling, but worth noting as general differences: the Go
receiver supports additional encodings (e.g. `jaeger_proto`, `zipkin_json`,
`raw`, `text`, `json`) whereas this receiver supports `otlp_proto` and
`otap_proto`; available authentication mechanisms may also differ.

#### Auto Mode (`commit.mode: auto`)

Offsets are committed automatically by the underlying rdkafka client. The `commit.interval_ms` value is forwarded to rdkafka as `auto.commit.interval.ms`. If `commit.interval_ms` is not set, the property is omitted and librdkafka retains its positive default (5000 ms). `commit.interval_ms`, when set, must be greater than 0.

```yaml
config:
  commit:
    mode: auto
    interval_ms: 5000
```

#### Idempotent Processing

When `enable_idempotency` is `true` and commit mode is `manual`, the receiver checks whether an incoming message offset has already been seen (either currently in-flight or previously committed) and skips duplicates. Defaults to `false`.

### Consumer Settings

#### Connection

The `brokers`, `group_id`, and `client_id` fields are required:

```yaml
config:
  brokers: "broker-1:9092,broker-2:9092"
  group_id: "my-consumer-group"
  client_id: "gateway-instance-1"
```

#### `auto_offset_reset`

Controls where the consumer starts reading when no committed offset exists for the consumer group.

| Value | Description |
| --- | --- |
| `earliest` | Start from the beginning of the topic. |
| `latest` | Start from the end of the topic (default). |
| `error` | Return an error if no offset is found. |

#### `isolation_level`

Controls the transaction isolation level for the consumer.

| Value | Description |
| --- | --- |
| `read_committed` | Read only messages from committed transactions. |
| `read_uncommitted` | Read all messages, including those from uncommitted transactions (default). |

#### `rebalance_strategy`

Controls the partition assignment strategy used during consumer group rebalancing. When omitted, librdkafka uses its default strategy (`range,roundrobin`). Setting this field overrides the default with a single strategy.

| Value | librdkafka Value | Description |
| --- | --- | --- |
| `range` | `range` | Assigns partitions per topic in contiguous ranges. |
| `round_robin` | `roundrobin` | Distributes partitions across consumers evenly. |
| `cooperative_sticky` | `cooperative-sticky` | Minimizes partition movement using cooperative incremental rebalancing. Recommended for large consumer groups. |

```yaml
config:
  rebalance_strategy: cooperative_sticky
```

#### Session and Fetch Tuning

Session management and fetch tuning are first-class fields:

```yaml
config:
  session_timeout_ms: 10000
  heartbeat_interval_ms: 3000
  min_fetch_bytes: 1
  max_fetch_bytes: 1048576
  max_fetch_wait_ms: 250
  max_partition_fetch_bytes: 1048576
```

#### `consumer_config`

A map of additional key-value string pairs passed directly to the underlying librdkafka consumer. This allows tuning any librdkafka setting not exposed as a top-level config field:

```yaml
config:
  consumer_config:
    queued.min.messages: "100000"
```

> **Note:** The built-in configuration fields (`bootstrap.servers`, `group.id`,
> `client.id`, `enable.auto.commit`, `auto.offset.reset`, `isolation.level`,
> `partition.assignment.strategy`, `session.timeout.ms`,
> `heartbeat.interval.ms`, `fetch.min.bytes`, `fetch.max.bytes`,
> `fetch.wait.max.ms`, `max.partition.fetch.bytes`) always take precedence over
> conflicting entries in `consumer_config`. Consumer config entries are applied
> first so built-in options override on conflict.

### Authentication

The receiver supports SASL authentication with the following mechanisms.

#### SASL/PLAIN

```yaml
config:
  auth:
    sasl:
      mechanism: PLAIN
      username: "my-user"
      password: "my-password"
```

#### SASL/SCRAM-SHA-256

```yaml
config:
  auth:
    sasl:
      mechanism: SCRAM-SHA-256
      username: "my-user"
      password: "my-password"
```

#### SASL/SCRAM-SHA-512

```yaml
config:
  auth:
    sasl:
      mechanism: SCRAM-SHA-512
      username: "my-user"
      password: "my-password"
```

#### AWS MSK IAM

> **Note:** AWS MSK IAM authentication requires the `aws` feature to be
> enabled at build time. Without the `aws` feature, the
> `AWS_MSK_IAM_OAUTHBEARER` mechanism is not available and configurations
> using it will be rejected.

```yaml
config:
  auth:
    sasl:
      mechanism: AWS_MSK_IAM_OAUTHBEARER
      aws_msk:
        region: "us-east-1"
```

When configured, the receiver uses the AWS MSK IAM SASL Signer to periodically refresh OAuth tokens.

### TLS Configuration

The `tls` block enables TLS (SSL) encryption for broker connections. All fields are optional -- the configuration mode depends on which fields are provided.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `ca_file` | string | *none* | Path to the CA certificate used to verify the broker's TLS certificate. |
| `cert_file` | string | *none* | Path to the client TLS certificate (PEM). Required with `key_file` for mTLS. |
| `key_file` | string | *none* | Path to the client TLS private key (PEM). Required with `cert_file` for mTLS. |
| `key_password` | string | *none* | Password for the client private key. Requires `key_file`. |
| `insecure` | bool | `false` | If `true`, disables TLS certificate verification. |

#### System Trust Store

An empty `tls` block enables TLS using the operating system's trusted CA certificates:

```yaml
config:
  brokers: "kafka:9093"
  tls: {}
```

#### CA-Only (Server Verification)

Provide only `ca_file` to verify the broker's certificate without client authentication:

```yaml
config:
  brokers: "kafka:9093"
  tls:
    ca_file: "/certs/ca.pem"
```

#### Mutual TLS (mTLS)

Provide `cert_file` and `key_file` for full mutual TLS authentication. `ca_file` is optional (the system trust store is used when omitted):

```yaml
config:
  brokers: "kafka:9093"
  tls:
    ca_file: "/certs/ca.pem"
    cert_file: "/certs/client.pem"
    key_file: "/certs/client-key.pem"
```

If the private key is password-protected, add `key_password`:

```yaml
config:
  brokers: "kafka:9093"
  tls:
    ca_file: "/certs/ca.pem"
    cert_file: "/certs/client.pem"
    key_file: "/certs/client-key.pem"
    key_password: "secret"
```

#### Security Protocol Inference

The receiver automatically infers the `security.protocol` setting based on the combination of `tls` and `auth` configuration:

| TLS | Auth | `security.protocol` |
| --- | --- | --- |
| No | No | `PLAINTEXT` |
| Yes | No | `SSL` |
| No | SASL (AWS MSK IAM) | `SASL_SSL` |
| No | SASL (other) | `SASL_PLAINTEXT` |
| Yes | SASL | `SASL_SSL` |

#### TLS with Authentication

TLS can be combined with SASL authentication. When both are configured, the security protocol is set to `SASL_SSL`:

```yaml
config:
  brokers: "kafka:9093"
  auth:
    sasl:
      mechanism: SCRAM-SHA-256
      username: "my-user"
      password: "my-password"
  tls:
    ca_file: "/certs/ca.pem"
```

### Header Extraction

The `resource_attrs_from_headers` field defines rules for extracting values from Kafka message headers and injecting them as resource attributes on the telemetry payload. This applies uniformly to all signal types (traces, metrics, logs).

Each entry maps a **Kafka header key** to a target resource attribute:

```yaml
resource_attrs_from_headers:
  x-tenant-id:
    key: tenant.id
    value_type: string
  x-priority:
    key: request.priority
    value_type: int
```

In this example, when a Kafka message contains a header named `x-tenant-id`, its value is parsed as a UTF-8 string and inserted as the `tenant.id` resource attribute on every resource in the payload. If a resource attribute with the same key already exists, it is overwritten.

#### Supported Value Types

| `value_type` | Description | Example Value |
| --- | --- | --- |
| `string` | UTF-8 string (no further parsing). | `"production"` |
| `bool` | Boolean. | `"true"` / `"false"` |
| `int` | Signed 64-bit integer. | `"42"` |
| `float` | 64-bit floating-point number. | `"3.14"` |

Header values are always read as raw bytes, decoded as UTF-8, and then parsed according to the specified `value_type`. If UTF-8 decoding or type parsing fails, the attribute is skipped and an error is logged.

### Transport Header Capture

In addition to `resource_attrs_from_headers` (which injects header values into resource attributes), the Kafka receiver supports the framework's **transport header capture policy**. When a `header_capture` policy is configured at the pipeline or node level, the receiver extracts matching Kafka message headers and stores them in the `OtapPdata` context as `TransportHeaders`. These headers flow through the pipeline and can be propagated by exporters using a corresponding `header_propagation` policy.

The two mechanisms are independent and can be used simultaneously:

| Mechanism | Configured via | Stores in | Purpose |
| --- | --- | --- | --- |
| `resource_attrs_from_headers` | Receiver `config` block | Resource attributes in the telemetry payload | Enrich telemetry data with Kafka header values |
| `header_capture` | Pipeline or node-level `transport_headers` policy | `OtapPdata.context.transport_headers` | End-to-end header propagation through the pipeline |

### Validation Rules

The receiver validates the configuration at startup:

1. `brokers`, `client_id`, and `group_id` must be non-empty.
2. `group_instance_id`, when set, must be non-empty.
3. At least one signal must have non-empty `topics`.
4. Topics must be disjoint across signals (no topic in more than one signal type).
5. Regex patterns (topics starting with `^`) must compile.
6. `exclude_topics` only allowed when at least one topic in the same signal is a regex pattern.
7. `exclude_topics` entries must be valid regex and non-empty.
8. `max_fetch_bytes` >= `min_fetch_bytes`.
9. `max_partition_fetch_bytes` > 0.
10. `commit.interval_ms`, when set, must be > 0.
11. `message_format_header` must be non-empty.
12. `resource_attrs_from_headers` keys and their `key` fields must be non-empty.

## Examples

### Multi-Signal with Per-Signal Encoding

```yaml
type: receiver:kafka
config:
  brokers: "broker-1:9092,broker-2:9092"
  group_id: "telemetry-consumers"
  client_id: "gateway-instance-1"
  traces:
    topics:
      - "otel-traces-prod"
      - "otel-traces-staging"
      - "^traces-team-.*"
    exclude_topics:
      - "^traces-team-test$"
    encoding: otlp_proto
  metrics:
    topics:
      - "otel-metrics"
    encoding: otap_proto
  logs:
    topics:
      - "otel-logs"
    encoding: otlp_proto
```

### Full Configuration

A configuration using all available options:

```yaml
type: receiver:kafka
config:
  brokers: "broker-1:9092,broker-2:9092"
  group_id: "telemetry-consumers"
  client_id: "gateway-instance-1"
  group_instance_id: "instance-1"
  auth:
    sasl:
      mechanism: AWS_MSK_IAM_OAUTHBEARER
      aws_msk:
        region: "us-east-1"
  tls:
    ca_file: "/certs/ca.pem"
    cert_file: "/certs/client.pem"
    key_file: "/certs/client-key.pem"
    insecure: false
  traces:
    topics:
      - "otel-traces"
    encoding: otlp_proto
  metrics:
    topics:
      - "otel-metrics"
    encoding: otap_proto
  logs:
    topics:
      - "otel-logs"
    encoding: otlp_proto
  auto_offset_reset: earliest
  commit:
    mode: manual
    interval_ms: 500
  rebalance_strategy: cooperative_sticky
  session_timeout_ms: 10000
  heartbeat_interval_ms: 3000
  min_fetch_bytes: 1
  max_fetch_bytes: 1048576
  max_fetch_wait_ms: 250
  max_partition_fetch_bytes: 1048576
  isolation_level: read_committed
  enable_idempotency: true
  debug:
    - broker
    - security
  log_level: info
  consumer_config:
    queued.min.messages: "100000"
  resource_attrs_from_headers:
    x-tenant-id:
      key: tenant.id
      value_type: string
    x-env:
      key: deployment.environment
      value_type: string
```

### Header Extraction Example

An example focused on enriching telemetry with metadata carried in Kafka headers
(`brokers`, `group_id`, and `client_id` are omitted for brevity):

```yaml
type: receiver:kafka
config:
  traces:
    topics:
      - "otel-traces"
  metrics:
    topics:
      - "otel-metrics"
  logs:
    topics:
      - "otel-logs"
  resource_attrs_from_headers:
    x-tenant-id:
      key: tenant.id
      value_type: string
    x-region:
      key: cloud.region
      value_type: string
    x-sample-rate:
      key: sampling.rate
      value_type: float
    x-priority:
      key: request.priority
      value_type: int
    x-canary:
      key: deployment.canary
      value_type: bool
```

### Transport Header Capture Example

An example using pipeline-level transport header capture alongside the Kafka receiver:

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: ["X-Tenant-Id"]
          store_as: "tenant_id"
        - match_names: ["X-Request-Id"]
    header_propagation:
      default:
        selector: all_captured
        action: propagate

pipelines:
  - receiver:
      type: receiver:kafka
      config:
        traces:
          topics:
            - "otel-traces"
    exporter:
      type: exporter:otlp_grpc
      config:
        endpoint: "collector:4317"
```

In this example, the Kafka receiver captures `X-Tenant-Id` and `X-Request-Id` headers from each Kafka message and attaches them to the pipeline context. The OTLP gRPC exporter then propagates all captured headers onto outbound gRPC requests.

### At-Least-Once with the Retry Processor

These recipes show how to reproduce the Go Kafka receiver's offset/error-handling
behavior using this receiver's `commit` configuration plus, where needed, the
[retry processor](../../../../core-nodes/src/processors/retry_processor/README.md)
(`processor:retry`). See
[Comparison with the Go Kafka receiver](#comparison-with-the-go-kafka-receiver)
for the full option mapping and known gaps.

#### Recipe A -- At-most-once (matches Go `autocommit.enable: true`)

Offsets are committed by the client regardless of downstream outcome; no retry
processor is needed. A message may be lost if the pipeline fails after the
offset is committed.

```yaml
config:
  brokers: "broker-1:9092"
  group_id: "telemetry-consumers"
  client_id: "gateway-instance-1"
  traces:
    topics:
      - "otel-traces"
  commit:
    mode: auto
    interval_ms: 5000   # equivalent to Go autocommit.interval
```

#### Recipe B -- At-least-once with transient retry (matches Go `autocommit: false` + `error_backoff`)

Place a retry processor between the Kafka receiver (in `manual` commit mode) and
the exporter. The retry processor retries transient nacks with exponential
backoff; while retries are in progress the offset stays in-flight and is not
committed, so it remains eligible for redelivery if the process restarts. Only a
terminal failure (retries exhausted or a permanent error) reaches the receiver
and advances the committed offset.

The retry processor fields below map directly to Go's `error_backoff`.

> **Differs from Go:** once retries are exhausted, this receiver **commits** the
> offset (advances past the message) rather than leaving it unmarked for
> redelivery. See [Known gaps](#known-gaps--behavioral-differences).

```yaml
version: otel_dataflow/v1
groups:
  default:
    pipelines:
      main:
        nodes:
          kafka/ingest:
            type: receiver:kafka
            config:
              brokers: "broker-1:9092"
              group_id: "telemetry-consumers"
              client_id: "gateway-instance-1"
              traces:
                topics:
                  - "otel-traces"
              commit:
                mode: manual

          retry:
            type: processor:retry
            config:
              initial_interval: 1s     # Go error_backoff.initial_interval
              max_interval: 30s        # Go error_backoff.max_interval
              max_elapsed_time: 5m     # Go error_backoff.max_elapsed_time
              multiplier: 2.0          # Go error_backoff.multiplier
              # Go error_backoff.randomization_factor has no equivalent (no jitter).

          otlp/export:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://collector:4317"

        connections:
          - from: kafka/ingest
            to: retry
          - from: retry
            to: otlp/export
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.kafka`

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.kafka.messages_received` | `{msg}` | Total messages received from Kafka across all signal types. |
| `receiver.kafka.bytes_received` | `{byte}` | Total payload bytes consumed from Kafka. |
| `receiver.kafka.log_msgs_received` | `{msg}` | Number of log messages received from the Kafka broker. |
| `receiver.kafka.metric_msgs_received` | `{msg}` | Number of metric messages received from the Kafka broker. |
| `receiver.kafka.trace_msgs_received` | `{msg}` | Number of trace messages received from the Kafka broker. |
| `receiver.kafka.acks_received` | `{ack}` | Number of acks received from downstream. |
| `receiver.kafka.nacks_received` | `{nack}` | Number of nacks received from downstream. |
| `receiver.kafka.processing_errors` | `{msg}` | Number of messages that failed processing and were skipped. |
| `receiver.kafka.unmarshal_failed_traces` | `{msg}` | Trace messages that failed to unmarshal. |
| `receiver.kafka.unmarshal_failed_metrics` | `{msg}` | Metric messages that failed to unmarshal. |
| `receiver.kafka.unmarshal_failed_logs` | `{msg}` | Log messages that failed to unmarshal. |
| `receiver.kafka.empty_payloads` | `{msg}` | Messages with empty payload. |
| `receiver.kafka.unknown_topic_errors` | `{error}` | Messages from topics that do not match any configured signal. |
| `receiver.kafka.transport_errors` | `{error}` | Number of Kafka transport errors encountered (non-fatal). |
| `receiver.kafka.offset_commits` | `{commit}` | Number of offset commits performed. |
| `receiver.kafka.offset_commit_errors` | `{error}` | Number of offset commit failures. |
| `receiver.kafka.idempotent_skips` | `{msg}` | Messages skipped due to idempotency check (duplicate detection). |
| `receiver.kafka.topic_id_exhausted` | `{msg}` | Messages dropped because the topic ID space was exhausted (overflow guard). |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `kafka.receiver.consumer_config.override` | `warn` | A `consumer_config` entry was overridden by a built-in configuration field. |
| `kafka.shutdown.commit_failed` | `error` | Final offset commit during shutdown failed. |
| `kafka.commit.failed` | `error` | An offset commit failed (non-fatal; offsets stay tracked and are retried on the next ack/nack/timer-tick). |
| `kafka.message.empty_payload` | `error` | A consumed message had an empty payload. |
| `kafka.message.unknown_topic` | `error` | A consumed message came from a topic not mapped to any signal. |
| `kafka.message.unmarshal_failed` | `error` | A consumed message failed to unmarshal (includes `signal` field: traces, metrics, or logs). |
| `kafka.partition_eof` | `info` | Consumer reached end of a partition. |
| `kafka.transport_error` | `error` | A Kafka transport-level error occurred (non-fatal, consumer continues). |
| `kafka.capture_policy.limits_exceeded` | `error` | Transport header capture exceeded configured limits. |
| `kafka.topic_id.exhausted` | `error` | The topic ID space was exhausted; the message was dropped to avoid a colliding ID. |

## Limits

- This receiver is functional but still pending performance optimization.
- AWS MSK IAM authentication (`AWS_MSK_IAM_OAUTHBEARER`) requires the `aws`
  feature to be enabled at build time.
- The engine supports one periodic timer per node; the commit interval
  safety-net timer uses this slot.
- `consumer_config` entries that conflict with built-in fields are silently
  overridden by the built-in values.
- Idempotent processing (`enable_idempotency`) only applies when commit mode
  is `manual`; the setting is ignored in `auto` mode.
- Compared to the Go Kafka receiver, this receiver does not replay terminally
  failed messages from Kafka (a terminal nack commits the offset), does not pause
  partitions on error, has no backoff jitter, and does not preserve per-partition
  order across retries. See
  [Comparison with the Go Kafka receiver](#comparison-with-the-go-kafka-receiver)
  for details.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Contrib node catalog](../../../README.md)
- [Retry processor](../../../../core-nodes/src/processors/retry_processor/README.md)
