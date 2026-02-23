# Syslog and CEF Receiver

**URN:** `urn:otel:syslog_cef:receiver`

A high-performance receiver for ingesting syslog messages (RFC 3164 and RFC 5424)
and Common Event Format (CEF) security logs. The receiver automatically detects
the message format and converts them into `OtapPdata` using the efficient
Apache Arrow columnar format for downstream processing.

## Supported Formats

The receiver automatically detects and parses the following message formats:

| Format | Description |
|--------|-------------|
| **RFC 5424** | Modern syslog format with structured data support |
| **RFC 3164** | Traditional BSD syslog format |
| **CEF** | ArcSight Common Event Format for security events |
| **CEF over Syslog** | CEF messages wrapped in RFC 3164 or RFC 5424 headers |

## Configuration

```yaml
receivers:
  syslog_cef:
    protocol:
      tcp:
        listening_addr: "0.0.0.0:514"

        # Optional: TLS configuration (requires "experimental-tls" feature)
        tls:
          cert_file: "/path/to/server.crt"
          key_file: "/path/to/server.key"
          client_ca_file: "/path/to/ca.crt"    # Optional: client cert verification
          handshake_timeout: "10s"             # Optional: default 10s
```

Or for UDP:

```yaml
receivers:
  syslog_cef:
    protocol:
      udp:
        listening_addr: "0.0.0.0:514"
```

Optionally, batching parameters can be tuned to control how messages are
accumulated before being sent downstream. Reducing these values limits the
scope of in-memory data loss at the cost of higher per-batch overhead:

```yaml
receivers:
  syslog_cef:
    protocol:
      tcp:
        listening_addr: "0.0.0.0:514"
    batch:
      flush_timeout_ms: 50    # Flush after 50 ms (default: 100)
      max_size: 25       # Flush after 25 messages (default: 100)
```

### Configuration Options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `protocol` | `object` | Yes | Exactly one of `tcp` or `udp` |
| `protocol.tcp.listening_addr` | `string` | Yes | Socket address (e.g., `"0.0.0.0:514"`) |
| `protocol.tcp.tls` | `object` | No | TLS config for secure TCP (RFC 5425) |
| `protocol.udp.listening_addr` | `string` | Yes | Socket address (e.g., `"0.0.0.0:514"`) |
| `batch` | `object` | No | Batching configuration (see below) |
| `batch.flush_timeout_ms` | `integer` | No | Max ms before flushing a batch (default: `100`) |
| `batch.max_size` | `integer` | No | Max messages per batch (default: `100`) |

## Transport Protocols

### UDP

- Connectionless, fire-and-forget delivery
- Best for high-volume, low-latency scenarios where occasional message loss is
  acceptable
- Each UDP datagram is treated as a single syslog message

### TCP

- Connection-oriented, reliable delivery
- Messages are delimited by newline characters (`\n`)
- Supports multiple concurrent connections
- Each connection is handled independently

### TCP with TLS (RFC 5425)

When the `experimental-tls` feature is enabled, the receiver supports
Syslog over TLS:

- Encrypted transport for sensitive log data
- Optional mutual TLS (mTLS) with client certificate verification
- Configurable handshake timeout

## Message Parsing

### Automatic Format Detection

The parser attempts to identify the message format using the following
strategy (in order):

1. **Pure CEF**: If the message starts with `CEF:`, parse as raw CEF
2. **RFC 5424**: Attempt to parse as RFC 5424. This succeeds if the message has:
   - A valid priority header (e.g., `<34>`)
   - Followed by a numeric version (e.g., `1`)
   - If the message body contains `CEF:`, it's parsed as **CEF over RFC 5424**
3. **RFC 3164 (fallback)**: If RFC 5424 parsing fails, attempt RFC 3164
   parsing. This parser is lenient and will accept almost any input:
   - Messages with valid priority (e.g., `<34>Oct 11 22:14:15 host ...`) are
     **fully parsed** -- `body` is empty, all data in attributes
   - Messages without priority or with malformed headers are **partially
     parsed** -- `body` contains the original raw input
   - If the content contains `CEF:`, it's parsed as **CEF over RFC 3164**

> **Note:** Because RFC 3164 is a fallback parser, even arbitrary text like
> `"Hello World"` will be accepted. For such messages, the raw input is
> preserved in the `body` field to help identify devices sending non-standard
> data.

### RFC 5424 Parsing

RFC 5424 messages follow this structure:

```text
<priority>version timestamp hostname app-name procid msgid [structured-data] msg
```

**Example Input:**

<!-- markdownlint-disable MD013 -->

```text
<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su 12345 ID47 [exampleSDID@32473 iut="3" eventSource="Application"] 'su root' failed for lonvick on /dev/pts/8
```

<!-- markdownlint-enable MD013 -->

**Resulting LogRecord:**

| Field | Value |
|-------|-------|
| `timestamp` | `2003-10-11T22:14:15.003Z` |
| `severity_number` | `18` (ERROR2 - maps from syslog severity 2/Critical) |
| `severity_text` | `ERROR2` |
| `body` | *(null - fully parsed)* |

| Attribute | Value |
|-----------|-------|
| `syslog.version` | `1` |
| `syslog.facility` | `4` |
| `syslog.severity` | `2` |
| `syslog.host_name` | `mymachine.example.com` |
| `syslog.app_name` | `su` |
| `syslog.process_id` | `12345` |
| `syslog.process_id_str` | `12345` |
| `syslog.msg_id` | `ID47` |
| `syslog.structured_data` | `[exampleSDID@32473 iut="3" ...]` |
| `syslog.message` | `'su root' failed for lonvick on /dev/pts/8` |

### RFC 3164 Parsing

RFC 3164 messages follow this structure:

```text
<priority>timestamp hostname tag: content
```

**Example Input:**

```text
<34>Oct 11 22:14:15 mymachine su[1234]: 'su root' failed for lonvick
```

**Resulting LogRecord:**

| Field | Value |
|-------|-------|
| `timestamp` | `<current_year>-10-11T22:14:15` (local timezone) |
| `severity_number` | `18` (ERROR2) |
| `severity_text` | `ERROR2` |
| `body` | *(null - fully parsed)* |

| Attribute | Value |
|-----------|-------|
| `syslog.facility` | `4` |
| `syslog.severity` | `2` |
| `syslog.host_name` | `mymachine` |
| `syslog.tag` | `su[1234]` |
| `syslog.app_name` | `su` |
| `syslog.process_id` | `1234` |
| `syslog.content` | `'su root' failed for lonvick` |

> **Note:** RFC 3164 timestamps don't include the year, so the receiver
> assumes the current year.

### CEF Parsing

CEF messages follow this structure:

```text
CEF:Version|Device Vendor|Device Product|Device Version|Device Event Class ID|Name|Severity|[Extension]
```

See the [CEF Implementation Standard][cef-spec] for full details.

**Example Input:**

<!-- markdownlint-disable MD013 -->

```text
CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232
```

<!-- markdownlint-enable MD013 -->

**Resulting LogRecord:**

| Field | Value |
|-------|-------|
| `timestamp` | `0` *(not available in CEF format)* |
| `observed_time` | *(set to current time when batch is built)* |
| `severity_number` | `0` (UNSPECIFIED - CEF has its own severity) |
| `severity_text` | `UNSPECIFIED` |
| `body` | *(empty - fully parsed)* |

| Attribute | Value |
|-----------|-------|
| `cef.version` | `0` |
| `cef.device_vendor` | `Security` |
| `cef.device_product` | `threatmanager` |
| `cef.device_version` | `1.0` |
| `cef.device_event_class_id` | `100` |
| `cef.name` | `worm successfully stopped` |
| `cef.severity` | `10` |
| `src` | `10.0.0.1` |
| `dst` | `2.1.2.2` |
| `spt` | `1232` |

[cef-spec]: https://www.microfocus.com/documentation/arcsight/arcsight-smartconnectors-8.3/cef-implementation-standard/Content/CEF/Chapter%201%20What%20is%20CEF.htm

### CEF over Syslog

When CEF is transported over syslog, both the syslog header and CEF content
are parsed:

**Example Input:**

<!-- markdownlint-disable MD013 -->

```text
<34>1 2003-10-11T22:14:15.003Z myhost app 5678 - - CEF:0|Security|IDS|1.0|100|Attack detected|8|src=10.0.0.1 dst=192.168.1.1 spt=12345
```

<!-- markdownlint-enable MD013 -->

**Resulting LogRecord:**

| Field | Value |
|-------|-------|
| `timestamp` | `2003-10-11T22:14:15.003Z` |
| `severity_number` | `18` (ERROR2 - from syslog severity 2/Critical) |
| `severity_text` | `ERROR2` |
| `body` | *(empty - fully parsed)* |

| Attribute | Value |
|-----------|-------|
| `syslog.version` | `1` |
| `syslog.facility` | `4` |
| `syslog.severity` | `2` |
| `syslog.host_name` | `myhost` |
| `syslog.app_name` | `app` |
| `syslog.process_id` | `5678` |
| `syslog.process_id_str` | `5678` |
| `cef.version` | `0` |
| `cef.device_vendor` | `Security` |
| `cef.device_product` | `IDS` |
| `cef.device_version` | `1.0` |
| `cef.device_event_class_id` | `100` |
| `cef.name` | `Attack detected` |
| `cef.severity` | `8` |
| `src` | `10.0.0.1` |
| `dst` | `192.168.1.1` |
| `spt` | `12345` |

### Severity Mapping

Syslog severity levels are mapped to OpenTelemetry severity numbers per the
[OTel Logs Data Model][otel-severity]:

| Syslog Severity | Syslog Name | OTel Severity Number | OTel Severity Text |
|-----------------|-------------|----------------------|--------------------|
| 0 | Emergency | 21 | FATAL |
| 1 | Alert | 19 | ERROR3 |
| 2 | Critical | 18 | ERROR2 |
| 3 | Error | 17 | ERROR |
| 4 | Warning | 13 | WARN |
| 5 | Notice | 10 | INFO2 |
| 6 | Informational | 9 | INFO |
| 7 | Debug | 5 | DEBUG |

[otel-severity]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.47.0/specification/logs/data-model-appendix.md#appendix-b-severitynumber-example-mappings

### Handling Partially Parsed Messages

For messages that cannot be fully parsed (e.g., missing priority header in
RFC 3164):

- **Fully parsed**: `body` is empty; all data is in attributes
- **Partially parsed**: `body` contains the original raw input for debugging

This helps identify devices sending malformed syslog data.

## Implementation Design

### Local Receiver Architecture

This receiver is implemented as a **local receiver** (non-`Send`) and it runs
on a single-threaded async runtime using `tokio::task::spawn_local`.
This design choice provides:

- **Efficient memory access**: No synchronization overhead for shared state
- **Simpler state management**: Metrics and buffers use `Rc<RefCell<...>>`
  instead of `Arc<Mutex<...>>`
- **Predictable performance**: No cross-thread coordination latency

For TCP, each connection spawns a separate local task, allowing concurrent
connection handling within the same thread.

### Batching Strategy

To optimize throughput and reduce per-message overhead, the receiver batches
messages into Apache Arrow record batches before sending downstream:

```text
+-------------------------------------------------------------+
|                    Batching Logic                           |
+-------------------------------------------------------------+
|                                                             |
|   Messages arrive  -->  ArrowRecordsBuilder  -->  Batch     |
|                              |                     sent     |
|                              |                              |
|   Flush conditions:          |                              |
|   +- Size: max_size messages +-------------------------->   |
|   +- Time: flush_timeout_ms  |                              |
|                                                             |
+-------------------------------------------------------------+
```

A batch is flushed when either:

- **`batch.max_size`** messages have accumulated (default: 100), or
- **`batch.flush_timeout_ms`** milliseconds have elapsed since the last flush
  (default: 100 ms), or
- The connection closes (TCP) or the receiver shuts down

Both thresholds are optionally configurable via the `batch` section (see
[Configuration](#configuration)). Lowering these values reduces the window
of in-memory data that could be lost on a crash, at the expense of more
frequent (and smaller) downstream sends.

This batching strategy balances:

- **Latency**: Configurable max delay for low-volume streams
- **Throughput**: Amortized overhead for high-volume streams
- **Memory**: Bounded buffer size prevents unbounded growth

### Arrow Columnar Format

The receiver converts syslog messages directly into Apache Arrow columnar
format, which provides:

- **Efficient compression**: Columnar layout enables better compression ratios
- **Zero-copy processing**: Downstream processors can operate on Arrow buffers
  directly
- **Vectorized operations**: Enables SIMD-optimized batch processing

## Telemetry Metrics

The receiver exposes the following internal metrics:

| Metric | Type | Description |
|--------|------|-------------|
| `received_logs_total` | Counter | Total logs observed at socket |
| `received_logs_forwarded` | Counter | Logs successfully sent downstream |
| `received_logs_invalid` | Counter | Logs that failed to parse |
| `received_logs_forward_failed` | Counter | Logs refused by downstream |
| `tcp_connections_active` | UpDownCounter | Active TCP connections |
| `tls_handshake_failures` | Counter | TLS handshake failures (TLS only) |

## Example Pipeline Configuration

```yaml
receivers:
  syslog_cef:
    protocol:
      tcp:
        listening_addr: "0.0.0.0:1514"

processors:
  batch:
    timeout: 1s
    send_batch_size: 1000

exporters:
  otlp:
    endpoint: "otel-collector:4317"

pipelines:
  logs:
    receivers: [syslog_cef]
    processors: [batch]
    exporters: [otlp]
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `experimental-tls` | Enables TLS support for secure TCP connections |
