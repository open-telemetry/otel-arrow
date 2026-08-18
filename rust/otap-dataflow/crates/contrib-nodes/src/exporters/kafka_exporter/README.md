# Kafka Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:kafka` (`urn:otel:exporter:kafka`)
- Feature gate: `kafka-exporter` (also enabled by `contrib-exporters`)
- Stability: Experimental (pending performance optimization)

## Overview

The Kafka exporter produces OpenTelemetry traces, metrics, and logs to
Apache Kafka topics. It supports OTLP and OTAP protobuf encodings,
per-signal topic and encoding configuration, dynamic topic routing via
transport headers, SASL authentication (PLAIN, SCRAM, AWS MSK IAM), TLS,
configurable partitioning strategies, and producer tuning knobs.

## Getting Started

The smallest valid configuration needs only the required connection fields
and at least one signal. All other fields have sensible defaults:

```yaml
type: exporter:kafka
config:
  brokers: "broker-1:9092"
  client_id: "my-client"
  traces:
    topic: "otel-traces"
```

## Configuration

### Top-Level Fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `brokers` | string | **required** | Comma-separated list of Kafka broker addresses. |
| `client_id` | string | **required** | Kafka client ID sent to brokers for tracking. |
| `traces` | object | *none* | Per-signal config for traces (see [Per-Signal Configuration](#per-signal-configuration)). |
| `metrics` | object | *none* | Per-signal config for metrics. |
| `logs` | object | *none* | Per-signal config for logs. |
| `timeout_ms` | integer | `5000` | Request timeout in milliseconds (`message.timeout.ms`). Must be in the range `1` to `30000`. `0` is rejected because librdkafka interprets it as an infinite delivery timeout, which would let a broker outage block the exporter from shutting down. |
| `compression` | string | *none* | Compression type: `gzip`, `snappy`, `lz4`, or `zstd`. |
| `required_acks` | string | `"one"` | Required broker acks: `none` (0), `one` (1), or `all` (-1). |
| `max_message_bytes` | integer | `1000000` | Maximum message size in bytes (`message.max.bytes`). |
| `linger_ms` | integer | `5` | Artificial delay in ms before sending a batch (`linger.ms`). |
| `max_in_flight` | integer | `10` | Maximum number of Kafka deliveries kept in flight concurrently before the exporter stops accepting new pdata. `10` (the default) pipelines deliveries for throughput. Must be in the range `1` to `100000` (the librdkafka default producer queue depth); larger values are rejected. See [Backpressure and concurrency](#backpressure-and-concurrency). |
| `auth` | object | *none* | Authentication configuration (see [Authentication](#authentication)). |
| `tls` | object | *none* | TLS configuration (see [TLS Configuration](#tls-configuration)). |
| `partitioning_strategy` | string | `"consistent_random"` | Librdkafka partitioner algorithm. See [Partitioning](#partitioning). |
| `allow_auto_create_topics` | bool | `true` | Whether the broker may auto-create topics this exporter produces to (`allow.auto.create.topics`). Defaults to `true`. Set to `false` for default-deny (recommended when routing by a client-controlled header). See [Security](#security). |
| `producer_config` | map | `{}` | Additional librdkafka producer settings as key-value string pairs. |
| `message_format_header` | string | `"MessageFormat"` | Kafka header key for the message format indicator. Each outgoing message includes a header with this key and value `otlp` or `otap`, allowing consumers to detect the encoding. |
| `debug` | list | *none* | List of librdkafka debug contexts: `generic`, `broker`, `topic`, `metadata`, `feature`, `queue`, `msg`, `protocol`, `cgrp`, `security`, `fetch`, `interceptor`, `plugin`, `consumer`, `admin`, `eos`, `mock`, `assignor`, `conf`, `telemetry`, `all`. |
| `log_level` | string | *none* | Librdkafka log level: `emerg`, `alert`, `critical`, `error`, `warning`, `notice`, `info`, `debug`. When omitted, inferred from the application's log configuration. |

### Per-Signal Configuration

Each signal type (`traces`, `metrics`, `logs`) is optional. At least one
must be configured. Signals that are omitted will not be exported -- if a
pdata message arrives for an unconfigured signal, the exporter will
permanently nack it (non-retryable).

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `topic` | string | **required** | Kafka topic to produce messages to (static fallback). |
| `encoding` | string | `"otlp_proto"` | Encoding format: `otlp_proto` or `otap_proto`. |
| `topic_from_transport_header` | string | *none* | Transport header name for dynamic topic routing. When set and the header is present with a valid topic, its value overrides `topic`; if the header is absent the static `topic` is used, and if present but invalid the batch is permanently nacked. See [Dynamic Topic Routing](#dynamic-topic-routing). |
| `partition_by_transport_headers` | bool | `false` | Serialize all transport headers into a Kafka record key. See [Partitioning](#partitioning). |
| `allowed_topics` | list of strings | *empty* | Operator allowlist of exact topic names permitted for header-supplied (dynamic) routing. Empty means no exact-match constraint. See [Security](#security). |
| `allowed_topics_regex` | list of strings | *empty* | Operator allowlist of regex patterns permitted for header-supplied (dynamic) routing. Each pattern must match the whole topic (anchored, not a substring); entries must be valid standalone regular expressions (validated at config time). Empty means no regex constraint. See [Security](#security). |

### Dynamic Topic Routing

Each signal can optionally specify a `topic_from_transport_header` field.
When set, the exporter checks the incoming pdata context for a transport
header matching the configured transport header name. If the header is present,
its value is used as the Kafka destination topic instead of the static
`topic` field.

**Priority hierarchy:**

1. Transport header value (if `topic_from_transport_header` is configured
   and the header is present)
2. Static `topic` from config (fallback)

Each signal type can use a different header key (or none at all), allowing
independent dynamic routing per signal. If the header is not present on a
particular message, the static `topic` is used as a fallback.

The configured `topic_from_transport_header` value is lowercased during config
validation to match how captured transport header names are normalized on
ingress (lowercase, dashes preserved). For example, `X-Target-Topic` is
matched as `x-target-topic`. If a capture policy stores a header under a custom
`store_as` name, set this value to that stored name.

If a transport header *is* present but supplies an invalid Kafka topic name,
the batch is **permanently nacked** rather than silently routed to the static
`topic`. This avoids misdelivering data that explicitly requested a different
(but unusable) destination.

### Security

Because a client-controlled transport header can influence routing and
partitioning, the exporter provides operator controls for the trust boundary.

#### Constraining dynamic topic routing

A header-supplied topic (`topic_from_transport_header`) is always validated for
Kafka topic-name syntax. In addition, each signal may declare an operator
allowlist so a client cannot direct data to an arbitrary topic:

- `allowed_topics`: exact topic names permitted for header routing.
- `allowed_topics_regex`: regex patterns permitted for header routing. Each
  pattern must match the **whole** topic (it is anchored as `\A(?:<pattern>)\z`),
  so a prefix/suffix pattern cannot be satisfied by a substring of a
  client-supplied topic. Because this is an authorization boundary, each entry
  must be a valid **standalone** regular expression: a pattern is compiled on its
  own before being anchored, which rejects a pattern crafted to balance its
  parentheses against the anchoring wrapper (and thereby drop the whole-topic
  anchors). Patterns are validated at config time and compiled once at exporter
  construction (and on reconfigure); an invalid pattern is a configuration error
  caught at startup.

When either list is non-empty, a header-supplied topic must exactly match the
`allowed_topics` list or fully match an `allowed_topics_regex` pattern; otherwise
the batch is **permanently nacked** (non-retryable) and is not routed to the
static `topic`. When both lists are empty (the default), dynamic routing is
unrestricted (backwards compatible). The allowlist constrains only the
header-supplied path -- the static per-signal `topic` is operator-controlled and
is never subject to it.

#### Topic auto-creation

`allow_auto_create_topics` defaults to `true` (matching the Go Collector Kafka
exporter's `allow_auto_topic_creation`) and is always written to the librdkafka
client config (`allow.auto.create.topics`). The key is managed: setting it
through the `producer_config` escape hatch is overridden by the first-class
field and reported via the `kafka.exporter.producer_config.overridden_key`
warning.

Security: combined with header-driven routing
(`topic_from_transport_header`), leaving auto-creation enabled lets a
client-controlled routing header cause the broker to spawn arbitrary topics.
Operators who route by header (or otherwise want default-deny) should set
`allow_auto_create_topics: false`.

#### Partition-key fingerprinting

When `partition_by_transport_headers` is enabled, the record key is a
deterministic 16-character hash of the transport header names and values -- never
the plaintext value -- so tenant IDs / auth tokens are not exposed in the record
key. The accepted tradeoff is that a given tenant/token produces a *stable* key,
which makes its traffic fingerprintable via partition-assignment analysis; this
is intentional (co-locating a tenant's data is the feature). Leave
`partition_by_transport_headers` disabled (the default) for null-key
round-robin partitioning.

### Authentication

The exporter supports SASL authentication with the following mechanisms:

#### SASL/PLAIN

```yaml
auth:
  sasl:
    mechanism: PLAIN
    username: "my-user"
    password: "my-password"
```

#### SASL/SCRAM-SHA-256

```yaml
auth:
  sasl:
    mechanism: SCRAM-SHA-256
    username: "my-user"
    password: "my-password"
```

#### SASL/SCRAM-SHA-512

```yaml
auth:
  sasl:
    mechanism: SCRAM-SHA-512
    username: "my-user"
    password: "my-password"
```

#### AWS MSK IAM

Requires the `aws` feature to be enabled at build time.

```yaml
auth:
  sasl:
    mechanism: AWS_MSK_IAM_OAUTHBEARER
    aws_msk:
      region: "us-east-1"
```

When configured, the exporter uses the AWS MSK IAM SASL Signer to
periodically refresh OAuth tokens.

### TLS Configuration

All TLS fields are optional. The configuration mode depends on which fields
are provided.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `ca_file` | string | *none* | Path to the CA certificate (PEM). |
| `cert_file` | string | *none* | Path to the client certificate (PEM). Required with `key_file` for mTLS. |
| `key_file` | string | *none* | Path to the client private key (PEM). Required with `cert_file` for mTLS. |
| `key_password` | string | *none* | Password for the client private key. Requires `key_file`. |
| `insecure` | bool | `false` | Disable TLS certificate verification. |

An empty `tls: {}` block enables TLS using the system trust store. Provide
only `ca_file` for server verification without client authentication. Add
`cert_file` and `key_file` for mutual TLS:

```yaml
# System trust store
tls: {}

# CA-only (server verification)
tls:
  ca_file: "/certs/ca.pem"

# Mutual TLS
tls:
  ca_file: "/certs/ca.pem"
  cert_file: "/certs/client.pem"
  key_file: "/certs/client-key.pem"
```

The `security.protocol` is automatically inferred from the TLS and Auth
configuration:

| TLS | Auth | Protocol |
| --- | --- | --- |
| configured | configured | `SASL_SSL` |
| configured | *none* | `SSL` |
| *none* | AWS MSK IAM | `SASL_SSL` |
| *none* | other SASL | `SASL_PLAINTEXT` |
| *none* | *none* | `PLAINTEXT` |

### Partitioning

The exporter provides two controls for Kafka partitioning: a top-level
**partitioner strategy** that selects the librdkafka hashing algorithm, and
a per-signal **partition key source** that determines what value is fed into
that algorithm.

#### Partitioner Strategy

The `partitioning_strategy` field maps directly to librdkafka's
`partitioner` configuration property. It controls how partition keys are
hashed to partition numbers. The default is `consistent_random`.

| Value | Description |
| --- | --- |
| `random` | Random distribution. Keys are ignored. |
| `consistent` | CRC32 hash of key. Empty and NULL keys are mapped to a single partition. |
| `consistent_random` | CRC32 hash of key. Empty and NULL keys are randomly partitioned. **(default)** |
| `murmur2` | Java Producer compatible Murmur2 hash of key. NULL keys are mapped to a single partition. |
| `murmur2_random` | Java Producer compatible Murmur2 hash of key. NULL keys are randomly partitioned. |
| `fnv1a` | FNV-1a hash of key. NULL keys are mapped to a single partition. |
| `fnv1a_random` | FNV-1a hash of key. NULL keys are randomly partitioned. |

#### Partition by Transport Headers

When `partition_by_transport_headers` is enabled on a signal, the exporter
hashes the request's transport headers to derive the Kafka record key, so
requests carrying the same headers (e.g. same tenant ID) are routed to the same
partition. This setting is per-signal -- each of `traces`, `metrics`, and `logs`
can independently opt in.

### Producer Tuning

Three commonly-configured librdkafka settings are directly exposed:

- **`required_acks`**: Controls durability guarantees. `none` (acks=0) is
  fire-and-forget; `one` (acks=1, default) waits for the leader; `all`
  (acks=-1) waits for all in-sync replicas.
- **`max_message_bytes`**: Maximum message size (default: 1,000,000 bytes).
  Must match the broker's `message.max.bytes`.
- **`linger_ms`**: Artificial delay to accumulate messages into batches
  (default: 5ms). Higher values improve throughput at the cost of latency.

### Producer Config Escape Hatch

The `producer_config` field provides an escape hatch for arbitrary
librdkafka producer settings that are not directly exposed as config fields.
Settings are applied first; built-in options take precedence on conflict.

```yaml
producer_config:
  "queue.buffering.max.messages": "100000"
  "batch.num.messages": "10000"
```

### Backpressure and concurrency

The exporter encodes and enqueues each accepted pdata to librdkafka and then
tracks the delivery in a bounded in-flight set. The `max_in_flight` config caps
how many deliveries may be outstanding at once:

- **`max_in_flight > 1` (default `10`).** Deliveries are pipelined for higher
  throughput. When the in-flight set is full the exporter stops accepting new
  pdata and only drains completions, so in-flight memory stays bounded and
  backpressure propagates upstream.

### Live Reconfiguration

The exporter accepts live configuration changes at runtime (via a `Config`
control message). Reconfiguration builds a new librdkafka producer from the
incoming config, performs a bounded drain (flush, then purge) of the old
producer, and then swaps in the new producer, config, and compiled
dynamic-routing allowlists. If the new config fails to deserialize/validate or
the new producer fails to build, the change is logged and ignored and the
current producer keeps running.

Live reconfiguration is currently **experimental** and does not yet provide two
guarantees. Both are tracked in the live-reconfiguration issue
([#ISSUE](https://github.com/open-telemetry/otel-arrow/issues/3768)):

- **In-flight data can cross configurations.** Control messages (including the
  reconfiguration message) and telemetry data travel on separate channels, and
  control messages are processed with priority. Telemetry the exporter already
  accepted *before* the config change can therefore still be waiting in its
  inbox and be processed *after* the producer and config are swapped. Those
  records are then sent using the **new** topic, credentials, or tenant rather
  than the configuration that was in effect when they were accepted. There is no
  ordered cutover barrier that applies the new config only after all preceding
  data has been sent.
- **The swap can briefly block the pipeline.** The old producer is flushed and
  retired synchronously, so a slow or unavailable broker can stall normal
  processing and backpressure for up to the configured flush timeout
  (`timeout_ms`) instead of letting the pipeline keep making progress.

Until these are addressed, avoid live reconfiguration changes that alter the
destination topic, credentials, or tenant while data is in flight if
cross-configuration delivery would be unsafe for your deployment. Prefer draining
the exporter (or restarting the node) for such changes.

### Comparison with the Go Kafka exporter

The OpenTelemetry Collector's Go Kafka exporter bundles a synchronous producer
with a built-in `sending_queue` (queueing/batching) and `retry_on_failure`
(exponential backoff) in a single component. This exporter delegates
transient-failure retry to the separate
[retry processor](../../../../core-nodes/src/processors/retry_processor/README.md)
(`processor:retry`) placed upstream of the exporter.

This exporter targets the OTAP dataflow engine and intentionally supports a
narrower feature set than the upstream Go exporter. The tables below summarize
the **feature gaps**; the
[Error handling and the retry processor](#error-handling-and-the-retry-processor)
subsection explains how that node closes the `retry_on_failure` gap.

#### Signals

The Go exporter also exports a `profiles` signal. This exporter supports
`traces`, `metrics`, and `logs` only -- there is no `profiles` signal.

#### Encodings differences

| Encoding | Go | Here |
| --- | --- | --- |
| `otlp_proto` | yes | yes |
| `otap_proto` (OTAP Arrow) | no | yes |
| `otlp_json` | yes | no |
| `jaeger_proto` / `jaeger_json` (traces) | yes | no |
| `zipkin_proto` / `zipkin_json` (traces) | yes | no |
| `raw` (logs) | yes | no |
| encoding extensions | yes | no |

#### Authentication mechanisms

| Mechanism | Go | Here |
| --- | --- | --- |
| SASL `PLAIN`, `SCRAM-SHA-256`, `SCRAM-SHA-512` | yes | yes |
| SASL `AWS_MSK_IAM_OAUTHBEARER` | yes | yes (requires the build-time `aws` feature) |
| Generic `OAUTHBEARER` (token-source extension) | yes | no |
| Kerberos / GSSAPI | yes | no |
| SASL protocol `version` (0/1) | yes | no |

#### Destination topic and message key

| Capability | Go | Here |
| --- | --- | --- |
| Static per-signal `topic` | yes | yes |
| Topic from transport header | no (uses context topic / attribute) | yes (per-signal `topic_from_transport_header`) |
| `topic_from_metadata_key` (arbitrary request-metadata key) | yes | no |
| `topic_from_attribute` (resource attribute) | yes | no |
| `message_key_from_metadata_key` | yes | no |
| `partition_traces_by_id` | yes | no (planned) |
| `partition_*_by_resource_attributes` | yes | no |
| `partition_logs_by_trace_id` | yes | no |
| Partition key from hashed transport headers | no | yes (`partition_by_transport_headers`) |

#### Partitioning model

The Go exporter selects a `record_partitioner` (`sticky_key` with a
`sarama_compat`/`murmur2` hasher, `round_robin`, `least_backup`, or a custom
`extension`). This exporter instead selects a librdkafka partitioner via
[`partitioning_strategy`](#partitioning) (`random`, `consistent`,
`consistent_random`, `murmur2[_random]`, `fnv1a[_random]`). There is **no**
`least_backup` strategy and **no** custom partitioner extension here.

#### Header differences

The Go exporter supports `record_headers` (static headers written on every
record) and `include_metadata_keys` (propagate request-metadata values as Kafka
record headers). This exporter has **neither** as config fields: the only
always-written header is the encoding indicator (`message_format_header`), and
transport-header propagation onto Kafka records is driven by a pipeline-level
`header_propagation` policy rather than an exporter config field.

#### Producer and connection settings

The following Go first-class fields have **no dedicated config field** here.
Where librdkafka exposes an equivalent setting, it can still be set through the
[`producer_config`](#producer-config-escape-hatch) passthrough.

| Go field | Equivalent here |
| --- | --- |
| `compression_params.level` | `producer_config` (`compression.level`) |
| `max_broker_write_bytes` | `producer_config` |
| `flush_max_messages` | `producer_config` (`batch.num.messages`, `queue.buffering.max.messages`) |
| `protocol_version` | `producer_config` (`api.version.request`, etc.) |
| `resolve_canonical_bootstrap_servers_only` | `producer_config` (`client.dns.lookup`) |
| `conn_idle_timeout` | `producer_config` (`connections.max.idle.ms`) |
| `metadata.refresh_interval` | `producer_config` (`topic.metadata.refresh.interval.ms`) |

The Go `timeout`, `compression`, `producer.required_acks`,
`producer.max_message_bytes`, `producer.linger`, `allow_auto_topic_creation`,
and `client_id` settings have direct fields here (`timeout_ms`, `compression`,
`required_acks`, `max_message_bytes`, `linger_ms`, `allow_auto_create_topics`,
`client_id`); see
[Producer Tuning](#producer-tuning), [Authentication](#authentication), and
[TLS Configuration](#tls-configuration).

#### Sending queue and batching

The Go exporter has a `sending_queue` (with `enabled`, `num_consumers`, and
`queue_size`) and processes batches. This exporter sends **one message per pdata,
awaited synchronously** -- there is no application-level sending queue,
`num_consumers`, or in-memory queue backpressure. Batching and lingering are
delegated to librdkafka via `linger_ms` and any
[`producer_config`](#producer-config-escape-hatch) queue knobs (e.g.
`queue.buffering.max.messages`, `batch.num.messages`).

#### Error handling and the retry processor

The Go exporter retries failed exports internally via `retry_on_failure`. This
exporter has **no internal retry loop** (beyond librdkafka's queue-full retry).
Instead, on a Kafka **send failure** it emits a **transient (retryable) nack**;
a [retry processor](../../../../core-nodes/src/processors/retry_processor/README.md)
placed **upstream** of the exporter catches that nack and retries the batch with
exponential backoff, only forwarding it onward once retries are exhausted or the
failure is permanent.

Not every failure is retryable. The exporter emits a **permanent** (non-retryable)
nack -- which the retry processor forwards immediately without retrying -- for:

- an **encoding failure** (the payload cannot be serialized);
- a pdata message for an **unconfigured signal type**;
- an **invalid dynamic topic** supplied via a transport header (see
  [Dynamic Topic Routing](#dynamic-topic-routing)); and
- a **non-retryable send error** returned by the Kafka producer -- a record that
  exceeds the size limit (`message.max.bytes`), a malformed/invalid record, an
  authorization failure, or an unsupported request. All other send failures
  (timeouts, an unavailable broker/leader, network errors, or a full producer
  queue) remain **transient** and are retried.

The retry processor's backoff fields map onto Go's `retry_on_failure`:

| Go exporter option | Equivalent here | Notes |
| --- | --- | --- |
| `retry_on_failure.enabled` | Add a `processor:retry` node upstream of the exporter | Retry is a separate node, not an exporter field. Omit the node to disable. |
| `retry_on_failure.initial_interval` | retry processor `initial_interval` | Default `5s`. |
| `retry_on_failure.max_interval` | retry processor `max_interval` | Default `30s`. |
| `retry_on_failure.multiplier` | retry processor `multiplier` | Default `1.5`. |
| `retry_on_failure.max_elapsed_time` | retry processor `max_elapsed_time` | Default `300s` (5m); must be > 0. |
| `retry_on_failure.randomization_factor` | *(no equivalent)* | The retry processor backoff has no jitter -- see [Known gaps](#known-gaps--behavioral-differences). |

Example pipeline placing a retry processor in front of the Kafka exporter so
transient send failures are retried with backoff:

```yaml
version: otel_dataflow/v1
groups:
  default:
    pipelines:
      main:
        nodes:
          otlp/ingest:
            type: receiver:otlp

          retry:
            type: processor:retry
            config:
              initial_interval: 1s     # Go retry_on_failure.initial_interval
              max_interval: 30s        # Go retry_on_failure.max_interval
              max_elapsed_time: 5m     # Go retry_on_failure.max_elapsed_time
              multiplier: 2.0          # Go retry_on_failure.multiplier
              # Go retry_on_failure.randomization_factor has no equivalent (no jitter).

          kafka/export:
            type: exporter:kafka
            config:
              brokers: "broker-1:9092"
              client_id: "gateway-instance-1"
              traces:
                topic: "otlp_spans"

        connections:
          - from: otlp/ingest
            to: retry
          - from: retry
            to: kafka/export
```

##### Known gaps / behavioral differences

The table below maps the Go exporter's remaining error-handling behavior onto
the equivalent here, assuming this exporter runs with an upstream
`processor:retry` node.

| Go exporter option | Equivalent here | Notes |
| --- | --- | --- |
| `retry_on_failure.randomization_factor` | *(no equivalent)* | The retry processor backoff has no jitter. |
| `sending_queue` (`queue_size`, `num_consumers`) | `max_in_flight` (bounded delivery pipelining) | `max_in_flight` bounds concurrent in-flight deliveries and propagates backpressure upstream; there is still no separate application-level queue with persistent storage. See [Backpressure and concurrency](#backpressure-and-concurrency). |
| `sending_queue` persistent storage | Add a `processor:durable_buffer` node | Retry/queue state is in-memory; add a durable buffer node for cross-restart durability. |
| *(in-line per-export retry ordering)* | *(no equivalent)* | The retry processor retries out-of-band, so a later batch may be sent and acked before an earlier batch still being retried. |
| *(drop after retries exhausted)* | Final nack forwarded upstream | After `max_elapsed_time` the retry processor forwards a final nack; data is dropped at the source. No dead-letter queue. |
| *(encoding failure / unconfigured signal / invalid dynamic topic)* | Permanent nack (not retried) | These are non-retryable; the retry processor forwards them immediately. No dead-letter queue. |

Beyond error handling, this exporter also supports fewer encodings, auth
mechanisms, and routing/partitioning options -- see the tables above.

### Validation Rules

1. `brokers` must be non-empty.
2. `client_id` must be non-empty.
3. At least one signal (`traces`, `metrics`, or `logs`) must be configured.
4. Unknown configuration fields are rejected (`deny_unknown_fields`).
5. Each signal's `topic` and every entry in `allowed_topics` must be a
   syntactically valid Kafka topic name; every `allowed_topics_regex` entry must
   be a valid regular expression.

## Examples

### Multi-Signal with Dynamic Topic Routing

```yaml
type: exporter:kafka
config:
  brokers: "kafka:9092"
  client_id: "my-gateway"
  traces:
    topic: "otlp_spans"
    encoding: "otlp_proto"
    topic_from_transport_header: "x-traces-topic"
    partition_by_transport_headers: true
  metrics:
    topic: "otlp_metrics"
    encoding: "otap_proto"
  logs:
    topic: "otlp_logs"
    encoding: "otlp_proto"
    topic_from_transport_header: "x-logs-topic"
```

### Full Configuration

```yaml
type: exporter:kafka
config:
  brokers: "kafka1:9092,kafka2:9092"
  client_id: "observability-gateway"
  partitioning_strategy: "murmur2_random"
  traces:
    topic: "otlp_spans"
    encoding: "otlp_proto"
    topic_from_transport_header: "x-traces-topic"
    partition_by_transport_headers: true
  metrics:
    topic: "otlp_metrics"
    encoding: "otlp_proto"
    partition_by_transport_headers: true
  logs:
    topic: "otlp_logs"
    encoding: "otlp_proto"
    topic_from_transport_header: "x-logs-topic"
    partition_by_transport_headers: true
  timeout_ms: 5000
  compression: "zstd"
  required_acks: "all"
  max_message_bytes: 1000000
  linger_ms: 5
  auth:
    sasl:
      mechanism: "AWS_MSK_IAM_OAUTHBEARER"
      aws_msk:
        region: "us-east-1"
  tls:
    ca_file: "/certs/ca.pem"
    cert_file: "/certs/client.pem"
    key_file: "/certs/client-key.pem"
    insecure: false
  producer_config:
    "queue.buffering.max.messages": "100000"
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

Recording several measurements for one export attempt is intentional. Each
recording updates a bounded in-process aggregate; it does not synchronously send
a separate request to the telemetry backend. Together, the measurements answer
different operational questions:

- `exporter.exports.messages`: Is the exporter succeeding?
- `exporter.kafka.failures.messages`: Why is an export failing?
- `exporter.kafka.operations.duration`: Is encoding or Kafka delivery slow?
- `exporter.exports.duration`: What end-to-end latency does the pipeline
  experience?
- `exporter.kafka.exports.bytes`: Are encoded messages approaching Kafka size
  limits, or do wire bytes correlate with failures?
- `exporter.kafka.routing.messages`: Is dynamic topic routing being used as
  expected?

#### `exporter.exports`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.exports.messages` | `{message}` | `signal`, `outcome` | Pdata messages whose Kafka export reached a terminal outcome. |
| `exporter.exports.duration` | `s` | `signal`, `outcome` | Time from dequeuing PData through the terminal Kafka delivery result, including routing and encoding but excluding Ack/Nack notification. |

`signal` is one of `traces`, `metrics`, or `logs`. The Kafka exporter emits the
terminal `outcome` values `success` and `failure`. Duration uses a bounded
exponential histogram.

#### `exporter.kafka.exports`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.kafka.exports.bytes` | `By` | `signal`, `outcome` | Encoded Kafka payload bytes for attempts that reached the producer. |

Wire bytes use a bounded exponential histogram and are absent for attempts that
fail before encoding completes.

#### `exporter.kafka.operations`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.kafka.operations.duration` | `s` | `signal`, `operation`, `outcome` | Time spent encoding or awaiting Kafka delivery. |

`operation` is `encoding` or `delivery`; `outcome` is `success` or `failure`.
This separates local serialization cost from producer queueing and broker
acknowledgement latency.

#### `exporter.kafka.failures`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.kafka.failures.messages` | `{message}` | `signal`, `error.type` | Failed export attempts classified by actionable reason. |

`error.type` is one of `unconfigured_signal`, `invalid_topic`, `encoding`,
`queue_full`, `timeout`, `message_too_large`, `authentication`, `authorization`,
`unknown_topic_or_partition`, `insufficient_replicas`, `transport`, or `other`.

#### `exporter.kafka.routing`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.kafka.routing.messages` | `{message}` | `signal`, `topic.source` | Messages routed by a transport header or static configuration. |

`topic.source` is `header` or `static_config`. Destination topic names are not
metric attributes, which keeps cardinality bounded for dynamic tenant routing.

### Legacy Metric Migration

| Legacy metric | Replacement |
| --- | --- |
| `exporter.pdata.exports.messages` | `exporter.exports.messages`, preserving the `signal` and `outcome` attributes. |
| `exporter.kafka.exports.messages` | `exporter.exports.messages`, preserving the `signal` and `outcome` attributes. |
| `exporter.kafka.topic_from_header` | `exporter.kafka.routing.messages{topic.source="header"}`, now also partitioned by `signal`. |
| `exporter.kafka.topic_from_static_config` | `exporter.kafka.routing.messages{topic.source="static_config"}`, now also partitioned by `signal`. |
| `exporter.kafka.acks_received` | Removed. An exporter is a terminal node, so downstream acknowledgement controls are not an export outcome. |
| `exporter.kafka.nacks_received` | Removed. Use `exporter.exports.messages{outcome="failure"}` for terminal export failures. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `kafka.exporter.producer_config.overridden_key` | `warn` | A `producer_config` key is also managed by a first-class setting and may be overwritten. |
| `kafka.exporter.signal.unconfigured` | `warn` | Pdata arrived for a signal without exporter configuration and was permanently nacked. |
| `kafka.exporter.topic.invalid_header` | `warn` | A transport header supplied an invalid destination topic and the message was permanently nacked. |
| `kafka.exporter.encode.failed` | `error` | Pdata encoding failed and the message was permanently nacked. |
| `kafka.exporter.send.failed` | `warn` | Kafka delivery failed and the message was nacked for upstream retry handling. |
| `kafka.exporter.shutdown.flush_failed` | `warn` | Shutdown flushing failed or timed out; queued and in-flight messages were purged. |
| `kafka.exporter.producer.poll_thread_join_failed` | `warn` | The producer polling thread could not be joined during teardown. |

## Limits

- AWS MSK IAM authentication (`AWS_MSK_IAM_OAUTHBEARER`) requires the `aws`
  feature to be enabled at build time.
- `producer_config` entries that conflict with built-in fields are silently
  overridden by the built-in values.
- The exporter uses a custom `FutureProducer` with a 1-second polling
  interval as a workaround for high idle CPU utilization in the upstream
  rdkafka implementation.
- Resource attribute-based partitioning is not yet implemented.
- Live reconfiguration is experimental: data accepted before a config change may
  be delivered using the new topic/credentials/tenant, and the producer swap can
  briefly block the pipeline. See
  [Live Reconfiguration](#live-reconfiguration).
- Compared to the Go Kafka exporter, this exporter delegates retry to an
  upstream `processor:retry` node (no built-in `retry_on_failure`), has no
  application-level sending queue, supports fewer encodings/auth mechanisms, and
  offers fewer topic-routing/partitioning options. See
  [Comparison with the Go Kafka exporter](#comparison-with-the-go-kafka-exporter)
  for details.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Contrib node catalog](../../../README.md)
