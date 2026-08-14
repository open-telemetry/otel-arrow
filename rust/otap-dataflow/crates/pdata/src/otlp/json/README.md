# OTLP JSON serialization

## Purpose

This module serializes OpenTelemetry logs, metrics, and traces from pdata views
into the OTLP JSON Protobuf encoding. It provides one encoding path for every
supported pdata backend instead of first converting each input into an owned
Protobuf message.

The module is a library facility, not an exporter. It writes one compact JSON
document to a caller-provided `std::io::Write`; the caller decides how that
document is buffered, framed, bounded, transported, or stored.

## Public API

The module exposes one function per signal:

- `write_logs_json` accepts a `LogsDataView`.
- `write_metrics_json` accepts a `MetricsView`.
- `write_traces_json` accepts a `TracesView`.

Each function accepts any implementation of its view trait. Existing
implementations include:

- owned Prost-generated OTLP messages;
- raw Protobuf views such as `RawLogsData`, `RawMetricsData`, and
  `RawTraceData`; and
- OTAP Arrow views such as `OtapLogsView`, `OtapMetricsView`, and
  `OtapTracesView`.

For example, an owned message or any other logs view can be encoded into a
reusable buffer:

```rust
use otap_df_pdata::otlp::json::write_logs_json;

let mut document = Vec::new();
write_logs_json(&logs, &mut document)?;
```

The output is a complete JSON document such as `{"resourceLogs":[...]}`.

## When to use it

### File capture and JSON Lines

A file exporter can serialize each batch into a buffer, enforce its own size
limit, append a newline, and then write the completed frame:

```rust
use std::io::Write as _;
use otap_df_pdata::otlp::json::write_logs_json;

let mut document = Vec::new();
write_logs_json(&logs, &mut document)?;

if document.len() > max_document_bytes {
    // Apply the exporter's oversized-document policy.
}

document.push(b'\n');
file.write_all(&document)?;
```

Buffering at this layer prevents a serialization error from leaving an
incomplete JSON frame in the destination. The exporter still owns file
creation, rotation, flushing, synchronization, and recovery policies.

### OTLP/HTTP request bodies

An HTTP exporter can write a signal view to a byte buffer and use that buffer
as an `application/json` request body. Endpoint selection, headers,
compression, retries, timeouts, and response handling remain transport
concerns.

### Diagnostics and interoperability tests

The same encoder can produce readable captures for debugging or compare the
semantic output of owned Protobuf, raw Protobuf, and OTAP Arrow views. Because
all backends use the same serializers, parity tests can detect differences in
their view implementations.

## Encoding behavior

The output follows the OTLP JSON Protobuf mapping, including its deviations
from the standard Protobuf JSON mapping:

- field names use lower camel case;
- 64-bit integers are decimal strings;
- trace and span identifiers are hexadecimal strings;
- enum values are JSON integers;
- other byte fields are base64 strings;
- `NaN`, positive infinity, and negative infinity are strings; and
- default scalar values and empty repeated fields are omitted.

The serializer emits compact JSON without added whitespace.

## Writer and error contract

The signal writers intentionally do not:

- append a newline or any other framing delimiter;
- impose a maximum document size;
- flush the destination writer;
- roll back bytes accepted before an error; or
- make a sequence of writes atomic.

`JsonEncodeError` reports serde encoding failures, including I/O errors from
the destination and invalid UTF-8 exposed by a view. Callers that require a
complete frame before touching the destination should serialize into a
temporary or reusable byte buffer first.

## Performance model

Serialization walks borrowed view iterators and streams fields through serde.
It avoids building a second, owned telemetry message, which is especially
useful for raw Protobuf and OTAP Arrow inputs. Base64 encoding of byte-valued
fields may still allocate.

Callers can reuse a `Vec<u8>` between documents to retain capacity. Whether to
buffer a full document or write directly to a sink is a caller-level tradeoff
between bounded memory, failure isolation, and throughput.

## Coverage and validation limits

The encoder serializes the data exposed by the pdata view traits. Unknown
Protobuf fields and fields absent from those traits cannot be preserved. For
example, `ResourceView` currently does not expose `Resource.entity_refs`, so
`entityRefs` is not emitted.

The validating `try_new` raw view constructors check top-level Protobuf wire
framing, while nested content is interpreted lazily. JSON serialization is not
a semantic OTLP validator: callers remain responsible for constraints such as
identifier widths, timestamp validity, and metric consistency.

When a new OTLP field is added, expose it through the relevant view trait and
all supported backends before adding it to the serializer. Backend-parity tests
should cover the new field.

## Related documentation

- [OTLP JSON Protobuf specification](https://github.com/open-telemetry/opentelemetry-proto/blob/main/docs/specification.md)
- [OTLP pdata overview](../README.md)
- [Pdata view traits](../../../../pdata-views/src/views)
