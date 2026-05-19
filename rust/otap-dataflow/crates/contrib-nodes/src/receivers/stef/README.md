# STEF Receiver

The STEF receiver is registered as `urn:otel:receiver:stef`. It accepts
OpenTelemetry metrics over the Collector-compatible STEF gRPC destination
stream and decodes them into OTAP Arrow records for downstream nodes.

This contrib receiver is experimental and is registered whenever the
`otap-df-contrib-nodes` crate is linked into the engine.

## Reference Implementations

Compatibility is validated against the OpenTelemetry Collector Contrib
STEF components:

- [Go STEF receiver][go-stefreceiver]
- [Go STEF exporter][go-stefexporter]

The receiver is primarily interoperable with the Go `stefexporter`: it
uses the same streaming RPC shape, capabilities response, metrics root
struct name, and metrics wire schema expected by that exporter.

## Configuration

The receiver config flattens the shared gRPC server settings and adds one
STEF-specific field. Unknown fields fail validation.

Minimal config:

```yaml
config:
  listening_addr: "0.0.0.0:4320"
```

Supported fields:

- `listening_addr`: socket address to bind, for example
  `"0.0.0.0:4320"`.
- `max_dict_bytes`: maximum advertised STEF dictionary bytes. The
  default is `0`, meaning no advertised limit.
- `request_compression`: accepted inbound compression methods. Supported
  values are `zstd`, `gzip`, `deflate`, a list of those values, or
  `none`. If omitted, all supported methods are accepted.
- Native STEF frame payload compression is decoded from the STEF fixed
  header. The receiver accepts uncompressed STEF and `zstd` STEF frames
  produced by the Go STEF exporter.
- `response_compression`: compression methods allowed for gRPC
  responses. If omitted, responses are uncompressed.
- `max_concurrent_requests`, `max_concurrent_streams`,
  `transport_concurrency_limit`, and `load_shed`: concurrency and
  backpressure controls. The default request concurrency tracks the
  downstream pdata channel capacity.
- `tcp_nodelay`, `tcp_keepalive`, `tcp_keepalive_interval`, and
  `tcp_keepalive_retries`: TCP socket tuning.
- `initial_stream_window_size`, `initial_connection_window_size`,
  `http2_adaptive_window`, `max_frame_size`,
  `http2_keepalive_interval`, and `http2_keepalive_timeout`: HTTP/2
  transport tuning.
- `max_decoding_message_size`: maximum inbound gRPC message size.
- `wait_for_result`: when enabled, the receiver waits for the immediate
  downstream component to acknowledge receipt before responding.
- `timeout`: optional RPC timeout.
- `tls`: optional server TLS or mTLS configuration.

## Current Limits

The Rust STEF receiver is intentionally narrower than the current Go
STEF metrics implementation:

- Signal support is metrics only. Logs and traces are not accepted.
- Metric data point support is limited to numeric `Gauge` and `Sum`
  points. In OTLP terms, counters are represented as monotonic `Sum`
  metrics.
- `Histogram`, `ExponentialHistogram`, and `Summary` points are not
  decoded yet.
- Point values are limited to `int64`, `float64`, and no-recorded-value.
- Non-empty exemplar arrays are not decoded yet.
- Attribute and metadata values are limited to empty, string, bool,
  int, double, and bytes values. Structured array and key-value-list
  values are not supported.

The receiver fails unsupported STEF input explicitly instead of silently
dropping or approximating unsupported data.

## Future Functional Expansion

The next functional milestone is parity with the Go STEF metrics
implementation for OTLP metric families. That means adding receiver
support for:

- Histograms, including count, sum, min, max, explicit bounds, bucket
  counts, aggregation temporality, and no-recorded-value flags.
- Exponential histograms, including scale, zero count, zero threshold,
  positive and negative buckets, min, max, sum, aggregation temporality,
  and no-recorded-value flags.
- Summaries, including count, sum, and quantile values.
- Exemplars on numeric and histogram-like points.
- Structured OTLP `AnyValue` attributes if we decide to preserve full
  OTLP attribute fidelity over STEF.

Any expansion should be validated against the Go `stefexporter` first,
then covered by Rust encode/decode round-trip tests and end-to-end tests
with a Go Collector connected to the OTAP Dataflow engine.

## Design

- Metrics are the explicit scope. Logs and traces are rejected with a
  clear error instead of being silently dropped.
- Compatibility with the Go Collector STEF exporter is the primary
  interoperability requirement.
- STEF decoding targets OTAP Arrow records directly so downstream
  processors receive native OTAP pdata.
- Codec logic stays in the `pdata` crate. This receiver owns gRPC, TLS,
  compression, backpressure, memory-pressure admission, telemetry, and
  ACK/NACK routing.
- Invalid STEF input surfaces as a receiver failure instead of best-effort
  partial conversion.

[go-stefreceiver]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/stefreceiver
[go-stefexporter]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/stefexporter
