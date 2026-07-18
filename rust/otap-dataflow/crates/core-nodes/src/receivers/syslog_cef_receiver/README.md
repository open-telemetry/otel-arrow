# Syslog and CEF Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:syslog_cef` (`urn:otel:receiver:syslog_cef`)
- Feature gate: Default
- Stability: Experimental

## Overview

A high-performance receiver for ingesting syslog messages (RFC 3164 and RFC 5424)
and Common Event Format (CEF) security logs. The receiver automatically detects
the message format and converts them into `OtapPdata` using the efficient
Apache Arrow columnar format for downstream processing.

## Getting Started

Start with a TCP listener:

```yaml
type: receiver:syslog_cef
config:
  protocol:
    tcp:
      listening_addr: "0.0.0.0:514"
```

## Supported Formats

The receiver automatically detects and parses the following message formats:

| Format | Description |
| ----- | ---------- |
| **RFC 5424** | Modern syslog format with structured data support |
| **RFC 3164** | Traditional BSD syslog format |
| **CEF** | ArcSight Common Event Format for security events |
| **CEF over Syslog** | CEF messages wrapped in RFC 3164 or RFC 5424 headers |

## Configuration

```yaml
type: receiver:syslog_cef
config:
  protocol:
    tcp:
      listening_addr: "0.0.0.0:514"

      # Optional: TLS configuration
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        client_ca_file: "/path/to/ca.crt"    # Optional client verification
        handshake_timeout: "10s"             # Optional, default 10s
```

Or for UDP:

```yaml
type: receiver:syslog_cef
config:
  protocol:
    udp:
      listening_addr: "0.0.0.0:514"
```

Optionally, batching parameters can be tuned to control how messages are
accumulated before being sent downstream. Reducing these values limits the
scope of in-memory data loss at the cost of higher per-batch overhead:

```yaml
type: receiver:syslog_cef
config:
  protocol:
    tcp:
      listening_addr: "0.0.0.0:514"
  batch:
    max_batch_duration_ms: 50    # Flush after 50 ms (default: 100)
    max_size: 25                 # Flush after 25 messages (default: 100)
```

Exactly one of `protocol.tcp` or `protocol.udp` must be configured.
`protocol.*.listening_addr` is required for the selected transport.
`protocol.tcp.tls` enables secure TCP (RFC 5425). `batch.max_batch_duration_ms`
defaults to `100`, and `batch.max_size` defaults to `100`.

The receiver supports pressure-aware `messages/second` rate limiting when the
engine-level memory limiter is configured. UDP counts one datagram as one
message. TCP counts one newline-framed line as one message. Over-limit UDP
datagrams are dropped; over-limit TCP messages are dropped while the connection
remains open. TCP rate-limit drops are silent because plain syslog TCP has no
per-message acknowledgement or retry hint.

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

The receiver supports
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

### Detected Input Format (`input.format`)

Every log record emitted by this receiver includes an `input.format` attribute
indicating the format that was detected by the auto-detection logic above.
This allows downstream processors to filter, route, or transform records based
on the originating format without re-inspecting the content.

| `input.format` Value | Description |
| -------------------- | ----------- |
| `rfc5424` | RFC 5424 syslog message |
| `rfc3164` | RFC 3164 (BSD) syslog message |
| `cef` | Raw CEF message (no syslog header) |
| `cef_rfc5424` | CEF message wrapped in an RFC 5424 syslog header |
| `cef_rfc3164` | CEF message wrapped in an RFC 3164 syslog header |

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
| ----- | ------- |
| `timestamp` | `2003-10-11T22:14:15.003Z` |
| `severity_number` | `18` (ERROR2 - maps from syslog severity 2/Critical) |
| `severity_text` | `ERROR2` |
| `body` | *(null - fully parsed)* |

| Attribute | Value |
| --------- | ----- |
| `input.format` | `rfc5424` |
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
| ----- | ----- |
| `timestamp` | `<current_year>-10-11T22:14:15` (local timezone) |
| `severity_number` | `18` (ERROR2) |
| `severity_text` | `ERROR2` |
| `body` | *(null - fully parsed)* |

| Attribute | Value |
| --------- | ----- |
| `input.format` | `rfc3164` |
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
| ----- | ----- |
| `timestamp` | `0` *(not available in CEF format)* |
| `observed_time` | *(set to current time when batch is built)* |
| `severity_number` | `0` (UNSPECIFIED - CEF has its own severity) |
| `severity_text` | `UNSPECIFIED` |
| `body` | *(empty - fully parsed)* |

| Attribute | Value |
| --------- | ----- |
| `input.format` | `cef` |
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
| ----- | ----- |
| `timestamp` | `2003-10-11T22:14:15.003Z` |
| `severity_number` | `18` (ERROR2 - from syslog severity 2/Critical) |
| `severity_text` | `ERROR2` |
| `body` | *(empty - fully parsed)* |

| Attribute | Value |
| --------- | ----- |
| `input.format` | `cef_rfc5424` |
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
| --------------- | ----------- | ---------------------- | ------------------- |
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
+-------------------------------------------------------------------+
|                         Batching Logic                            |
+-------------------------------------------------------------------+
|                                                                   |
|   Messages arrive  -->  ArrowRecordsBuilder  -->  Batch sent      |
|                              |                                    |
|   Flush conditions:          |                                    |
|   +- Size: max_size messages +------------------------------->    |
|   +- Time: max_batch_duration_ms  +-------------------------->    |
|                                                                   |
+-------------------------------------------------------------------+
```

A batch is flushed when either:

- **`batch.max_size`** messages have accumulated (default: 100), or
- **`batch.max_batch_duration_ms`** milliseconds have elapsed since the last flush
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

## Telemetry

<!-- markdownlint-disable MD013 -->

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.syslog_cef`

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.syslog_cef.received_logs_forwarded` | `{item}` | Number of log records successfully forwarded downstream. |
| `receiver.syslog_cef.received_logs_invalid` | `{item}` | Number of log records rejected because their payload is zero-length. |
| `receiver.syslog_cef.received_logs_truncated` | `{item}` | Number of log records whose raw message exceeded `MAX_MESSAGE_SIZE` and were truncated before parsing. For TCP, truncation is detected precisely when a newline-delimited message exceeds the size limit. For UDP, it is a heuristic - a datagram that fills the entire receive buffer is assumed truncated, though a message exactly `MAX_MESSAGE_SIZE` bytes would also trigger this. |
| `receiver.syslog_cef.received_logs_forward_failed` | `{item}` | Number of log records refused by downstream (backpressure/unavailable) |
| `receiver.syslog_cef.received_logs_total` | `{item}` | Total number of log records observed at the socket before parsing. |
| `receiver.syslog_cef.tcp_connections_active` | `{conn}` | Number of active TCP connections. |
| `receiver.syslog_cef.tls_handshake_failures` | `{error}` | Number of TLS handshake failures. |
| `receiver.syslog_cef.received_logs_rejected_memory_pressure` | `{item}` | Number of log records dropped due to process-wide memory pressure. |
| `receiver.syslog_cef.received_logs_refused_rate_limit` | `{item}` | Number of log records refused by message-rate throttling. |
| `receiver.syslog_cef.received_logs_would_refuse_rate_limit` | `{item}` | Number of log records that would be refused by observe-only message-rate throttling. |
| `receiver.syslog_cef.tcp_connections_rejected_memory_pressure` | `{conn}` | Number of TCP connections rejected or closed due to process-wide memory pressure. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `syslog_cef_receiver.start` | `info` | Receiver startup with protocol and listening address. |
| `syslog_cef_receiver.tls_enabled` | `info` | TLS was enabled for the TCP listener. |
| `syslog_cef_receiver.drain_ingress.timeout` | `warn` | Ingress drain timed out while connection tasks were still active. |
| `syslog_cef_receiver.tls.handshake.success` | `debug` | TLS handshake completed for an incoming connection. |
| `syslog_cef_receiver.tls.handshake.failed` | `warn` | TLS handshake failed and the connection was closed. |
| `syslog_cef_receiver.arrow_records.build_failed` | `warn` | Arrow records could not be built from a parsed batch; the batch was dropped. |
| `syslog_cef_receiver.memory_pressure.disconnect` | `warn` | A TCP connection was closed because process-wide memory pressure was active. |

<!-- markdownlint-enable MD013 -->

## Feature Flags

| Feature | Description |
| ------- | ----------- |
| Built-in TLS support | Enables TLS support for secure TCP connections |

## Examples

See [Configuration](#configuration).

## Limits

- Exactly one protocol variant, `tcp` or `udp`, is configured per receiver.
- UDP payloads are bounded by datagram size.
- TCP buffers are flushed by batch size, batch duration, connection close, or
  shutdown.
- Partially parsed messages can still be emitted with parser metadata.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Parsing behavior](syslog-parsing-behavior.md)
- [Telemetry](telemetry.md)
- [Core node catalog](../../../README.md)
