# STEF Exporter

The STEF exporter is registered as `urn:otel:exporter:stef`. It sends
OpenTelemetry metrics to a Collector-compatible STEF destination service
over gRPC.

## Reference Implementations

Compatibility is validated against the OpenTelemetry Collector Contrib
STEF components:

- [Go STEF receiver][go-stefreceiver]
- [Go STEF exporter][go-stefexporter]

The exporter is primarily interoperable with the Go `stefreceiver`: it
sends the same first message, metrics root struct name, metrics wire
schema, STEF chunks, and end-of-chunk marker expected by that receiver.

## Configuration

The exporter config flattens the shared gRPC client settings. Unknown
fields fail validation.

Minimal config:

```yaml
config:
  grpc_endpoint: "http://127.0.0.1:4320"
```

Supported fields:

- `grpc_endpoint`: required target endpoint, for example
  `"http://127.0.0.1:4320"` or `"https://collector.example:4320"`.
- `compression`: outbound request compression. Supported values are
  `zstd`, `gzip`, and `deflate`. The legacy alias
  `compression_method` is also accepted.
- `concurrency_limit`: maximum in-flight transport requests.
- `connect_timeout`: TCP connection timeout.
- `tcp_nodelay`, `tcp_keepalive`, `tcp_keepalive_interval`, and
  `tcp_keepalive_retries`: TCP socket tuning.
- `initial_stream_window_size`, `initial_connection_window_size`,
  `http2_adaptive_window`, `http2_keepalive_interval`,
  `http2_keepalive_timeout`, and `keep_alive_while_idle`: HTTP/2
  transport tuning.
- `timeout`: optional RPC timeout.
- `tls`: optional client TLS or mTLS configuration.
- `buffer_size`: optional internal Tower buffer size.
- `proxy`: optional proxy configuration. If omitted, standard proxy
  environment variables are honored.

## Current Limits

The Rust STEF exporter is intentionally narrower than the current Go STEF
metrics implementation:

- Signal support is metrics only. Logs and traces are rejected.
- Metric data point support is limited to numeric `Gauge` and `Sum`
  points. In OTLP terms, counters are represented as monotonic `Sum`
  metrics.
- `Histogram`, `ExponentialHistogram`, and `Summary` points are not
  encoded yet.
- Point values are limited to `int64`, `float64`, and no-recorded-value.
- Exemplars are not encoded yet.
- Attribute and metadata values are limited to empty, string, bool,
  int, double, and bytes values. Structured array and key-value-list
  values are not supported. The OTLP view path reports them as
  unsupported; the direct OTAP path does not preserve map or slice
  values today.

Unsupported metric families or values fail the export explicitly instead
of being silently dropped or approximated.

## Future Functional Expansion

The next functional milestone is parity with the Go STEF metrics
implementation for OTLP metric families. That means adding exporter
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

Any expansion should preserve compatibility with the Go `stefreceiver`,
the OTLP metrics view path, and the direct OTAP record path. It should be
validated with semantic comparison against the current Rust subset,
Go Collector interoperability tests, and end-to-end benchmarks.

## Design

- Metrics are the explicit scope. Logs and traces are rejected with a
  clear error instead of being silently dropped.
- Compatibility with the Go Collector STEF receiver is the primary
  interoperability requirement.
- For `OtlpBytes(ExportMetricsRequest)`, the exporter traverses
  serialized OTLP bytes through the OTLP metrics view framework and
  encodes STEF from that view.
- For `OtapArrowRecords`, the exporter encodes STEF from the direct OTAP
  record view.
- The production OTLP bytes to STEF path uses the shared metrics view
  abstraction. It avoids full generated-message materialization while
  staying close to the existing OTLP support and keeping validation
  straightforward.
- Codec logic stays in the `pdata` crate. This exporter owns gRPC, TLS,
  compression, telemetry, and ACK/NACK routing through the original OTAP
  context.
- Invalid STEF or unsupported OTLP values surface as explicit export
  failures instead of best-effort partial conversion.

## Future Optimization Directions

We prototyped a more specialized raw OTLP protobuf to STEF encoder in an
isolated worktree. The prototype bypassed the generic metrics view layer
and decoded serialized OTLP protobuf bytes directly into the STEF column
encoders. On the 50k point benchmark fixture, this reduced OTLP bytes to
STEF encode cost by roughly 17% compared with the committed view-based
path.

That prototype is not the production path for now. It adds more
specialized parsing code and should only be brought back if the
additional complexity is justified by end-to-end pipeline performance.
If we revisit this direction, the most useful options are:

- Add a raw OTLP metrics protobuf to STEF encoder behind a narrow API,
  while keeping the view-based encoder as the reference implementation
  during validation.
- Fuse raw number data point parsing with attribute traversal so each
  point is scanned once.
- Fuse raw `AnyValue` parsing with STEF value encoding, materializing
  state only when needed for subsequent delta comparisons.
- Use SIMD UTF-8 validation for field-level string checks, but preserve
  per-field validation semantics. Do not validate multiple independent
  OTLP strings as one continuous UTF-8 stream.
- Avoid repeated UTF-8 checks where the source is already an Arrow
  string or another trusted `&str`.
- Pre-size hot STEF column buffers when the record count or reasonable
  fixture-derived estimates are available.

The experiment also ruled out a few options for the current fixture:

- A byte-keyed STEF string dictionary regressed the raw encoder.
- A last-hit cache in `StringEncoder` did not provide a measurable win.
- Multi-entry UTF-8 validation is not semantically safe for independent
  OTLP string fields without extra boundary handling.

Any future raw encoder work should include:

- Compatibility validation against the Go STEF receiver and exporter.
- Byte-for-byte or semantic comparison with the view-based Rust encoder.
- Focused microbenchmarks for OTLP bytes to STEF, OTAP to STEF, and
  STEF to OTAP.
- End-to-end benchmarks with a Go Collector connected to the OTAP
  Dataflow engine over STEF.

[go-stefreceiver]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/stefreceiver
[go-stefexporter]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/stefexporter
