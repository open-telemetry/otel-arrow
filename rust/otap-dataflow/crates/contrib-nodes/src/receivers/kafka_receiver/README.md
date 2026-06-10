# Kafka Receiver

The Kafka receiver consumes OpenTelemetry traces, metrics, and logs from Kafka topics. It is registered under the URN `urn:otel:receiver:kafka` and can be used as a receiver node in a pipeline configuration.

## Configuration Reference

### Top-Level Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `brokers` | string | **required** | Comma-separated list of Kafka broker addresses. |
| `group_id` | string | **required** | Kafka consumer group ID. |
| `client_id` | string | **required** | Kafka client ID. |
| `group_instance_id` | string | *none* | Static group instance ID for Kafka static membership. |
| `auth` | object | *none* | Authentication configuration (see [Authentication](#authentication)). |
| `tls` | object | *none* | TLS configuration for broker connections (see [TLS Configuration](#tls-configuration)). |
| `traces` | object | `{}` | Per-signal config for traces (see [Per-Signal Configuration](#per-signal-configuration)). |
| `metrics` | object | `{}` | Per-signal config for metrics. |
| `logs` | object | `{}` | Per-signal config for logs. |
| `auto_offset_reset` | string | `latest` | Where to start consuming when no committed offset exists. |
| `commit` | object | `{mode: manual, interval_ms: 1000}` | Commit configuration (see [Commit Configuration](#commit-configuration)). |
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
|-------|------|---------|-------------|
| `topics` | list | `[]` | Topics to subscribe to. Entries starting with `^` are regex patterns. |
| `exclude_topics` | list | `[]` | Regex patterns for topics to exclude (requires at least one regex in `topics`). |
| `encoding` | string | `otlp_proto` | Default encoding format: `otlp_proto` or `otap_proto`. |

At least one signal must have non-empty `topics` for the receiver to consume any data. Topic names must be **disjoint across signal types** -- the receiver rejects configurations where the same topic appears in more than one signal type.

#### Topic Configuration

Configure topics inside each signal block:

```yaml
kafka:
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
kafka:
  traces:
    topics:
      - "^traces-.*"
    exclude_topics:
      - "^traces-test$"
      - "^traces-internal-.*"
```

Each `exclude_topics` entry must be a valid regex and non-empty. The `exclude_topics` field is only allowed when at least one topic in the same signal is a regex pattern (from Go receiver validation).

> **Note:** `exclude_topics` is only valid alongside regex topic subscriptions.
> If all topics in a signal are literal names, specifying `exclude_topics` will
> cause a configuration error at startup.

#### Encoding

Each signal can specify its own encoding format via the `encoding` field:

| Value | Description |
|-------|-------------|
| `otlp_proto` | OTLP protobuf encoding (default). |
| `otap_proto` | OTAP Arrow protobuf encoding. |

Encoding can differ per signal:

```yaml
kafka:
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
|-------|------|---------|-------------|
| `mode` | string | `manual` | `manual` (at-least-once) or `auto` (at-most-once). |
| `interval_ms` | integer | `1000` | Commit interval in milliseconds. |

#### Offset Management

The receiver supports two offset commit modes controlled by `commit.mode`.

#### Manual Mode (`commit.mode: manual`) -- Default

The receiver tracks each consumed message individually per partition. Offsets are only committed up to the lowest un-acknowledged message, preventing offset skipping when downstream acknowledgements arrive out of order.

The `commit.interval_ms` value controls a periodic safety-net timer for offset commits, if `commit.interval_ms` isn't set the safety-net timer is disabled

```yaml
kafka:
  commit:
    mode: manual
    interval_ms: 1000
```

### Auto Mode (`commit.mode: auto`)

Offsets are committed automatically by the underlying rdkafka client. The `commit.interval_ms` value is forwarded to rdkafka as `auto.commit.interval.ms`. If `commit.interval_ms` isn't set, it defaults to 0ms interval

```yaml
kafka:
  commit:
    mode: auto
    interval_ms: 5000
```

#### Idempotent Processing

When `enable_idempotency` is `true` and commit mode is `manual`, the receiver checks whether an incoming message offset has already been seen (either currently in-flight or previously committed) and skips duplicates. Defaults to `false`.

## Consumer Settings

### Connection

The `brokers`, `group_id`, and `client_id` fields are required:

```yaml
kafka:
  brokers: "broker-1:9092,broker-2:9092"
  group_id: "my-consumer-group"
  client_id: "gateway-instance-1"
```

### `auto_offset_reset`

Controls where the consumer starts reading when no committed offset exists for the consumer group.

| Value | Description |
|-------|-------------|
| `earliest` | Start from the beginning of the topic. |
| `latest` | Start from the end of the topic (default). |
| `error` | Return an error if no offset is found. |

### `isolation_level`

Controls the transaction isolation level for the consumer.

| Value | Description |
|-------|-------------|
| `read_committed` | Read only messages from committed transactions (default). |
| `read_uncommitted` | Read all messages, including those from uncommitted transactions. |

### `rebalance_strategy`

Controls the partition assignment strategy used during consumer group rebalancing. When omitted, librdkafka uses its default strategy (`range,roundrobin`). Setting this field overrides the default with a single strategy.

| Value | librdkafka Value | Description |
|-------|------------------|-------------|
| `range` | `range` | Assigns partitions per topic in contiguous ranges. |
| `round_robin` | `roundrobin` | Distributes partitions across consumers evenly. |
| `cooperative_sticky` | `cooperative-sticky` | Minimizes partition movement using cooperative incremental rebalancing. Recommended for large consumer groups. |

```yaml
kafka:
  rebalance_strategy: cooperative_sticky
```

### Session and Fetch Tuning

Session management and fetch tuning are first-class fields. Previously these could only be set via `consumer_config`:

```yaml
kafka:
  session_timeout_ms: 10000
  heartbeat_interval_ms: 3000
  min_fetch_bytes: 1
  max_fetch_bytes: 1048576
  max_fetch_wait_ms: 250
  max_partition_fetch_bytes: 1048576
```

### `consumer_config`

A map of additional key-value string pairs passed directly to the underlying librdkafka consumer. This allows tuning any librdkafka setting not exposed as a top-level config field:

```yaml
consumer_config:
  queued.min.messages: "100000"
```

> **Note:** The built-in configuration fields (`bootstrap.servers`, `group.id`, `client.id`, `enable.auto.commit`, `auto.offset.reset`, `isolation.level`, `partition.assignment.strategy`, `session.timeout.ms`, `heartbeat.interval.ms`, `fetch.min.bytes`, `fetch.max.bytes`, `fetch.wait.max.ms`, `max.partition.fetch.bytes`) always take precedence over conflicting entries in `consumer_config`. Consumer config entries are applied first so built-in options override on conflict.

## Authentication

The receiver supports SASL authentication with the following mechanisms.

### SASL/PLAIN

```yaml
kafka:
  auth:
    sasl:
      mechanism: PLAIN
      username: "my-user"
      password: "my-password"
```

### SASL/SCRAM-SHA-256

```yaml
kafka:
  auth:
    sasl:
      mechanism: SCRAM-SHA-256
      username: "my-user"
      password: "my-password"
```

### SASL/SCRAM-SHA-512

```yaml
kafka:
  auth:
    sasl:
      mechanism: SCRAM-SHA-512
      username: "my-user"
      password: "my-password"
```

### AWS MSK IAM

```yaml
kafka:
  auth:
    sasl:
      mechanism: AWS_MSK_IAM_OAUTHBEARER
      aws_msk:
        region: "us-east-1"
```

When configured, the receiver uses the AWS MSK IAM SASL Signer to periodically refresh OAuth tokens.

## TLS Configuration

The `tls` block enables TLS (SSL) encryption for broker connections. All fields are optional -- the configuration mode depends on which fields are provided.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `ca_file` | string | *none* | Path to the CA certificate used to verify the broker's TLS certificate. |
| `cert_file` | string | *none* | Path to the client TLS certificate (PEM). Required with `key_file` for mTLS. |
| `key_file` | string | *none* | Path to the client TLS private key (PEM). Required with `cert_file` for mTLS. |
| `key_password` | string | *none* | Password for the client private key. Requires `key_file`. |
| `insecure` | bool | `false` | If `true`, disables TLS certificate verification. |

### System Trust Store

An empty `tls` block enables TLS using the operating system's trusted CA certificates:

```yaml
kafka:
  brokers: "kafka:9093"
  tls: {}
```

### CA-Only (Server Verification)

Provide only `ca_file` to verify the broker's certificate without client authentication:

```yaml
kafka:
  brokers: "kafka:9093"
  tls:
    ca_file: "/certs/ca.pem"
```

### Mutual TLS (mTLS)

Provide `cert_file` and `key_file` for full mutual TLS authentication. `ca_file` is optional (the system trust store is used when omitted):

```yaml
kafka:
  brokers: "kafka:9093"
  tls:
    ca_file: "/certs/ca.pem"
    cert_file: "/certs/client.pem"
    key_file: "/certs/client-key.pem"
```

If the private key is password-protected, add `key_password`:

```yaml
kafka:
  brokers: "kafka:9093"
  tls:
    ca_file: "/certs/ca.pem"
    cert_file: "/certs/client.pem"
    key_file: "/certs/client-key.pem"
    key_password: "secret"
```

### Security Protocol Inference

The receiver automatically infers the `security.protocol` setting based on the combination of `tls` and `auth` configuration:

| TLS | Auth | `security.protocol` |
|-----|------|---------------------|
| No  | No   | `PLAINTEXT` |
| Yes | No   | `SSL` |
| No  | SASL (AWS MSK IAM) | `SASL_SSL` |
| No  | SASL (other) | `SASL_PLAINTEXT` |
| Yes | SASL | `SASL_SSL` |

### TLS with Authentication

TLS can be combined with SASL authentication. When both are configured, the security protocol is set to `SASL_SSL`:

```yaml
kafka:
  brokers: "kafka:9093"
  auth:
    sasl:
      mechanism: SCRAM-SHA-256
      username: "my-user"
      password: "my-password"
  tls:
    ca_file: "/certs/ca.pem"
```

## Header Extraction

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

### Supported Value Types

| `value_type` | Description | Example Value |
|--------------|-------------|---------------|
| `string` | UTF-8 string (no further parsing). | `"production"` |
| `bool` | Boolean. | `"true"` / `"false"` |
| `int` | Signed 64-bit integer. | `"42"` |
| `float` | 64-bit floating-point number. | `"3.14"` |

Header values are always read as raw bytes, decoded as UTF-8, and then parsed according to the specified `value_type`. If UTF-8 decoding or type parsing fails, the attribute is skipped and an error is logged.

## Transport Header Capture

In addition to `resource_attrs_from_headers` (which injects header values into resource attributes), the Kafka receiver supports the framework's **transport header capture policy**. When a `header_capture` policy is configured at the pipeline or node level, the receiver extracts matching Kafka message headers and stores them in the `OtapPdata` context as `TransportHeaders`. These headers flow through the pipeline and can be propagated by exporters using a corresponding `header_propagation` policy.

The two mechanisms are independent and can be used simultaneously:

| Mechanism | Configured via | Stores in | Purpose |
|-----------|---------------|-----------|---------|
| `resource_attrs_from_headers` | Receiver `config` block | Resource attributes in the telemetry payload | Enrich telemetry data with Kafka header values |
| `header_capture` | Pipeline or node-level `transport_headers` policy | `OtapPdata.context.transport_headers` | End-to-end header propagation through the pipeline |

### Example: Pipeline-Level Transport Headers Policy

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
      type: "urn:otel:receiver:kafka"
      config:
        traces:
          topics:
            - "otel-traces"
    exporter:
      type: "urn:otel:exporter:otlp"
      config:
        endpoint: "collector:4317"
```

In this example, the Kafka receiver captures `X-Tenant-Id` and `X-Request-Id` headers from each Kafka message and attaches them to the pipeline context. The OTLP exporter then propagates all captured headers onto outbound gRPC requests.

## Validation Rules

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
10. `commit.interval_ms` > 0.

## Examples

### Minimal Configuration

The smallest valid configuration needs only one topic. All other fields have sensible defaults:

```yaml
receiver:
  type: "urn:otel:receiver:kafka"
  config:
    traces:
      topics:
        - "otel-traces"
```

### Multi-Signal with Per-Signal Encoding

```yaml
receiver:
  type: "urn:otel:receiver:kafka"
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
receiver:
  type: "urn:otel:receiver:kafka"
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

### Configuration with Header Extraction

An example focused on enriching telemetry with metadata carried in Kafka headers:

```yaml
receiver:
  type: "urn:otel:receiver:kafka"
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
