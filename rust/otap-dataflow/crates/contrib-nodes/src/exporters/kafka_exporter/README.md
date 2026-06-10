# Kafka Exporter

The Kafka exporter produces OpenTelemetry traces, metrics, and logs to Kafka topics. It is registered under the URN `urn:otel:exporter:kafka` and can be used as an exporter node in a pipeline configuration.

## Configuration Reference

### Top-Level Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
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

Each signal type (`traces`, `metrics`, `logs`) is optional. At least one must be configured. Signals that are omitted will not be exported — if a pdata message arrives for an unconfigured signal, the exporter will permanently nack it (non-retryable).

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `topic` | string | **required** | Kafka topic to produce messages to (static fallback). |
| `encoding` | string | `"otlp_proto"` | Encoding format: `otlp_proto` or `otap_proto`. |
| `topic_from_transport_header` | string | *none* | Transport header name for dynamic topic routing. When set and the header is present, its value overrides `topic`. See [Dynamic Topic Routing](#dynamic-topic-routing). |
| `partition_by_transport_headers` | bool | `false` | Hash all transport headers into a partition key. See [Partitioning](#partitioning). |

```yaml
kafka_exporter:
  type: "urn:otel:exporter:kafka"
  config:
    brokers: "kafka:9092"
    traces:
      topic: "otlp_spans"
      encoding: "otlp_proto"
      topic_from_transport_header: "x_traces_topic"
      partition_by_transport_headers: true
    metrics:
      topic: "otlp_metrics"
      encoding: "otap_proto"
    logs:
      topic: "otlp_logs"
      encoding: "otlp_proto"
      topic_from_transport_header: "x_logs_topic"
```

### Dynamic Topic Routing

Each signal can optionally specify a `topic_from_transport_header` field. When set, the exporter checks the incoming pdata context for a transport header matching the configured normalized name. If the header is present, its value is used as the Kafka destination topic instead of the static `topic` field.

**Priority hierarchy:**

1. Transport header value (if `topic_from_transport_header` is configured and the header is present)
2. Static `topic` from config (fallback)

Each signal type can use a different header key (or none at all), allowing independent dynamic routing per signal. If the header is not present on a particular message, the static `topic` is used as a fallback.

**Example:**

```yaml
traces:
  topic: "otlp_spans"                           # static fallback
  topic_from_transport_header: "x_traces_topic"  # dynamic override (normalized name)
metrics:
  topic: "otlp_metrics"                          # always static (no header configured)
logs:
  topic: "otlp_logs"
  topic_from_transport_header: "x_logs_topic"
```

With this configuration:

- If a traces message arrives with a transport header whose normalized name is `x_traces_topic` (e.g., wire name `X-Traces-Topic`) and value `tenant-a-spans`, it is sent to Kafka topic `tenant-a-spans`.
- If a traces message arrives without that header, it is sent to `otlp_spans`.
- Metrics messages always go to `otlp_metrics` (no dynamic routing configured).
- Logs follow the same pattern as traces, using the `x_logs_topic` header.

**Metrics:** The exporter tracks topic routing decisions with two counters:

- `topic_from_header` — batches where the topic was resolved from a transport header.
- `topic_from_static_config` — batches where the static `topic` was used.

### Partitioning

The exporter provides two controls for Kafka partitioning: a top-level **partitioner strategy** that selects the librdkafka hashing algorithm, and a per-signal **partition key source** that determines what value is fed into that algorithm.

#### Partitioner Strategy

The `partitioning_strategy` field maps directly to librdkafka's `partitioner` configuration property. It controls how partition keys are hashed to partition numbers. The default is `consistent_random`.

| Value | Description |
|-------|-------------|
| `random` | Random distribution. Keys are ignored. |
| `consistent` | CRC32 hash of key. Empty and NULL keys are mapped to a single partition. |
| `consistent_random` | CRC32 hash of key. Empty and NULL keys are randomly partitioned. **(default)** |
| `murmur2` | Java Producer compatible Murmur2 hash of key. NULL keys are mapped to a single partition. |
| `murmur2_random` | Java Producer compatible Murmur2 hash of key. NULL keys are randomly partitioned. Functionally equivalent to the default partitioner in the Java Producer. |
| `fnv1a` | FNV-1a hash of key. NULL keys are mapped to a single partition. |
| `fnv1a_random` | FNV-1a hash of key. NULL keys are randomly partitioned. |

#### Partition by Transport Headers

When `partition_by_transport_headers` is enabled on a signal, the exporter hashes all transport headers from the pdata context (wire name and raw value) into a deterministic partition key. This ensures that requests carrying the same set of transport headers (e.g., same tenant ID, same auth token) are routed to the same Kafka partition.

If no transport headers are present on a message, the partition key is empty, which causes the configured partitioner strategy to apply its NULL/empty-key behavior (round-robin for `*_random` strategies, single-partition for others).

This setting is per-signal — each of `traces`, `metrics`, and `logs` can independently opt in.

**Example:**

```yaml
kafka_exporter:
  type: "urn:otel:exporter:kafka"
  config:
    brokers: "kafka:9092"
    client_id: "my-gateway"
    partitioning_strategy: "murmur2_random"
    traces:
      topic: "otlp_spans"
      partition_by_transport_headers: true
    logs:
      topic: "otlp_logs"
      partition_by_transport_headers: true
    metrics:
      topic: "otlp_metrics"
```

With this configuration, traces and logs are partitioned by transport headers using Murmur2 hashing, while metrics use round-robin (no partition key source configured).

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

```yaml
auth:
  sasl:
    mechanism: AWS_MSK_IAM_OAUTHBEARER
    aws_msk:
      region: "us-east-1"
```

When configured, the exporter uses the AWS MSK IAM SASL Signer to periodically refresh OAuth tokens.

### TLS Configuration

All TLS fields are optional. The configuration mode depends on which fields are provided.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `ca_file` | string | *none* | Path to the CA certificate (PEM). |
| `cert_file` | string | *none* | Path to the client certificate (PEM). Required with `key_file` for mTLS. |
| `key_file` | string | *none* | Path to the client private key (PEM). Required with `cert_file` for mTLS. |
| `key_password` | string | *none* | Password for the client private key. Requires `key_file`. |
| `insecure` | bool | `false` | Disable TLS certificate verification. |

An empty `tls: {}` block enables TLS using the system trust store. Provide only `ca_file` for server verification without client authentication. Add `cert_file` and `key_file` for mutual TLS:

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

### Producer Tuning

Three commonly-configured librdkafka settings are directly exposed:

- **`required_acks`**: Controls durability guarantees. `none` (acks=0) is fire-and-forget; `one` (acks=1, default) waits for the leader; `all` (acks=-1) waits for all in-sync replicas.
- **`max_message_bytes`**: Maximum message size (default: 1,000,000 bytes). Must match the broker's `message.max.bytes`.
- **`linger_ms`**: Artificial delay to accumulate messages into batches (default: 5ms). Higher values improve throughput at the cost of latency.

### Security Protocol Inference

The `security.protocol` is automatically inferred from the TLS and Auth configuration:

| TLS | Auth | Protocol |
|-----|------|----------|
| configured | configured | `SASL_SSL` |
| configured | *none* | `SSL` |
| *none* | AWS MSK IAM | `SASL_SSL` |
| *none* | other SASL | `SASL_PLAINTEXT` |
| *none* | *none* | `PLAINTEXT` |

This logic is shared with the Kafka receiver (via `gateway-common/kafka_util/security.rs`).

### Partitioning

The `partitioning_strategy` field controls the librdkafka partitioner algorithm. Available strategies:

| Strategy | librdkafka value | Description |
|----------|-----------------|-------------|
| `random` | `random` | Random partition assignment |
| `consistent` | `consistent` | CRC32 hash of key |
| `consistent_random` | `consistent_random` | CRC32 hash of key, random when no key (default) |
| `murmur2` | `murmur2` | Murmur2 hash (Java-compatible) |
| `murmur2_random` | `murmur2_random` | Murmur2 hash, random when no key |
| `fnv1a` | `fnv1a` | FNV-1a hash |
| `fnv1a_random` | `fnv1a_random` | FNV-1a hash, random when no key |

When `partition_by_transport_headers: true` is set on a signal, all transport headers from the pdata context are hashed to produce a deterministic partition key.

### Producer Config

The `producer_config` field provides an escape hatch for arbitrary librdkafka producer settings that are not directly exposed as config fields. Settings are applied first; built-in options take precedence on conflict.

```yaml
producer_config:
  "queue.buffering.max.messages": "100000"
  "batch.num.messages": "10000"
```

## Valid Configs

1. `brokers` must be non-empty.
2. `client_id` must be non-empty.
3. At least one signal (`traces`, `metrics`, or `logs`) must be configured.
4. Unknown configuration fields are rejected (`deny_unknown_fields`).

## Full Example

```yaml
nodes:
  kafka_exporter:
    type: "urn:otel:exporter:kafka"
    config:
      brokers: "kafka1:9092,kafka2:9092"
      client_id: "observability-gateway"
      partitioning_strategy: "murmur2_random"
      traces:
        topic: "otlp_spans"
        encoding: "otlp_proto"
        topic_from_transport_header: "x_traces_topic"
        partition_by_transport_headers: true
      metrics:
        topic: "otlp_metrics"
        encoding: "otlp_proto"
        partition_by_transport_headers: true
      logs:
        topic: "otlp_logs"
        encoding: "otlp_proto"
        topic_from_transport_header: "x_logs_topic"
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
