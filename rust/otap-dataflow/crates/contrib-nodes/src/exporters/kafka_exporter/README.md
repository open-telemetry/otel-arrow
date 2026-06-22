# Kafka Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:kafka` (`urn:otel:exporter:kafka`)
- Feature gate: `kafka-exporter` (also enabled by `contrib-exporters`)
- Stability: Experimental

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
| `timeout_ms` | integer | `5000` | Request timeout in milliseconds (`message.timeout.ms`). |
| `compression` | string | *none* | Compression type: `gzip`, `snappy`, `lz4`, or `zstd`. |
| `required_acks` | string | `"one"` | Required broker acks: `none` (0), `one` (1), or `all` (-1). |
| `max_message_bytes` | integer | `1000000` | Maximum message size in bytes (`message.max.bytes`). |
| `linger_ms` | integer | `5` | Artificial delay in ms before sending a batch (`linger.ms`). |
| `auth` | object | *none* | Authentication configuration (see [Authentication](#authentication)). |
| `tls` | object | *none* | TLS configuration (see [TLS Configuration](#tls-configuration)). |
| `partitioning_strategy` | string | `"consistent_random"` | Librdkafka partitioner algorithm. See [Partitioning](#partitioning). |
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

### Validation Rules

1. `brokers` must be non-empty.
2. `client_id` must be non-empty.
3. At least one signal (`traces`, `metrics`, or `logs`) must be configured.
4. Unknown configuration fields are rejected (`deny_unknown_fields`).

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

#### `exporter.kafka`

| Metric | Unit | Description |
| --- | --- | --- |
| `exporter.kafka.logs_exported` | `{log}` | Number of log records successfully exported to Kafka. |
| `exporter.kafka.logs_failed` | `{log}` | Number of log records that failed to export to Kafka. |
| `exporter.kafka.metrics_exported` | `{datapoint}` | Number of metric data points successfully exported to Kafka. |
| `exporter.kafka.metrics_failed` | `{datapoint}` | Number of metric data points that failed to export to Kafka. |
| `exporter.kafka.traces_exported` | `{span}` | Number of trace spans successfully exported to Kafka. |
| `exporter.kafka.traces_failed` | `{span}` | Number of trace spans that failed to export to Kafka. |
| `exporter.kafka.acks_received` | `{batch}` | Number of acks received from downstream. |
| `exporter.kafka.nacks_received` | `{batch}` | Number of nacks received from downstream. |
| `exporter.kafka.topic_from_header` | `{batch}` | Batches where topic was resolved from a transport header. |
| `exporter.kafka.topic_from_static_config` | `{batch}` | Batches where topic was resolved from static per-signal config. |

### Events

This node does not emit structured events.

## Limits

- AWS MSK IAM authentication (`AWS_MSK_IAM_OAUTHBEARER`) requires the `aws`
  feature to be enabled at build time.
- `producer_config` entries that conflict with built-in fields are silently
  overridden by the built-in values.
- The exporter uses a custom `FutureProducer` with a 1-second polling
  interval as a workaround for high idle CPU utilization in the upstream
  rdkafka implementation.
- Resource attribute-based partitioning is not yet implemented.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Contrib node catalog](../../../README.md)
