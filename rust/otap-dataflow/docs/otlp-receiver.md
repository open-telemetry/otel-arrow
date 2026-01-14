# OTLP Receiver

The OTLP Receiver ingests telemetry data via the OpenTelemetry Protocol (OTLP)
and forwards it into the OTAP dataflow pipeline. It supports both gRPC (HTTP/2)
and HTTP/1.1 protocols with unified concurrency control.

**Plugin URN:** `urn:otel:otlp:receiver`

## Architecture Overview

```text
                    +------------------------------------------+
                    |           OTLP Receiver Node             |
                    +------------------------------------------+
                                       |
               +-----------------------+-----------------------+
               |                                               |
        +------v------+                               +-------v-------+
        |    gRPC     |                               |     HTTP      |
        |   :4317     |                               |    :4318      |
        +------+------+                               +-------+-------+
               |                                               |
               | tonic (HTTP/2)                                | hyper (HTTP/1.1)
               |                                               |
        +------v-----------------+                +------------v-----------+
        | OtlpBytesCodec         |                | HttpHandler            |
        | - Zero-copy framing    |                | - Limited body collect |
        | - No deserialization   |                | - Decompression        |
        +------+-----------------+                +------------+-----------+
               |                                               |
               |   +---------------------------------------+   |
               |   |        Shared Semaphore               |   |
               |   |    (Arc<Semaphore> across both)       |   |
               |   +-------------------+-------------------+   |
               |                       |                       |
               | acquire via           | acquire_owned()       |
               | Tower layer           | + timeout             |
               |                       |                       |
               +----------+------------+------------+----------+
                          |  (permit acquired)     |
                          +------------+-----------+
                                       |
                          +------------v--------------+
                          | OtapPdata                 |
                          |   context: Context        |
                          |   payload: OtlpProtoBytes | <-- Raw serialized
                          +------------+--------------+
                                       |
       +-------------------------------v-------------------------------+
       |              Shared Infrastructure (gRPC + HTTP)              |
       |  - AckRegistry (wait_for_result subscriptions)                |
       |  - Unified Metrics (requests, acks, nacks, errors)            |
       |  - Control Loop (ACK/NACK routing, telemetry, shutdown)       |
       +-------------------------------+-------------------------------+
                                       |
                          +------------v--------------+
                          | Pipeline (Bounded MPSC)   |
                          +---------------------------+
```

## Key Design Principles

### Zero-Deserialization Path

Both protocols keep OTLP payloads in their serialized protobuf form throughout
the receiver. Decoding is deferred to downstream pipeline stages, reducing CPU
overhead and memory allocations on the hot path.

- **gRPC**: Custom `OtlpBytesCodec` extracts the raw bytes without parsing
- **HTTP**: Body is collected and (optionally) decompressed, but not decoded

### Shared Concurrency Control

When both protocols are enabled, a single `tokio::sync::Semaphore`
controls admission:

```text
                     +---------------------------+
                     |   Shared Semaphore        |
                     |   (max_concurrent_requests)|
                     +-------------+-------------+
                                   |
              +--------------------+--------------------+
              |                                         |
   +----------v-----------+              +--------------v-----------+
   | gRPC                 |              | HTTP                     |
   | acquire_owned()      |              | acquire_owned() + timeout|
   | (queues)             |              | (default 30s or timeout) |
   +----------------------+              +--------------------------+
```

This ensures total inflight requests never exceed downstream channel capacity,
regardless of which protocol clients use.

**Protocol-specific behavior:**

- **gRPC only (no HTTP configured):** Uses the original
  `GlobalConcurrencyLimitLayer` with early refusal at `poll_ready`
  when over limit (no queuing on permits).
- **Both protocols:** Shared semaphore; gRPC queues on `acquire_owned().await`
  with no separate permit timeout (the wait is unbounded), but the request-level
  timeout still applies after the permit is acquired. HTTP queues on the shared
  semaphore with a permit timeout (default: `http.timeout`, 30s).
- **HTTP only:** Uses its own semaphore and permit timeout (default: 30s).

### Unified Metrics

A single `MetricSet` tracks receiver activity across both protocols:

- `requests_started` / `requests_completed`
- `acks_received` / `nacks_received`
- `rejected_requests`
- `transport_errors`

## Configuration

### Minimal Configuration (gRPC only)

```yaml
nodes:
  receiver:
    kind: receiver
    plugin_urn: "urn:otel:otlp:receiver"
    config:
      listening_addr: "0.0.0.0:4317"
```

### Full Configuration (gRPC + HTTP)

```yaml
nodes:
  receiver:
    kind: receiver
    plugin_urn: "urn:otel:otlp:receiver"
    config:
      # ---------------------------------------------------------
      # gRPC Settings (flattened at root level)
      # ---------------------------------------------------------
      listening_addr: "0.0.0.0:4317"

      # Concurrency: 0 = auto-tune to downstream channel capacity
      max_concurrent_requests: 0

      # Wait for downstream ACK before responding to client
      wait_for_result: false

      # Request timeout (humantime format)
      timeout: "30s"

      # Compression methods accepted for requests
      # Default: [zstd, gzip, deflate]
      request_compression:
        - zstd
        - gzip

      # Compression methods for responses (default: none)
      response_compression: []

      # TCP socket tuning
      tcp_nodelay: true
      tcp_keepalive: "45s"
      tcp_keepalive_interval: "15s"
      tcp_keepalive_retries: 5

      # HTTP/2 tuning
      max_concurrent_streams: 100
      initial_stream_window_size: "8MiB"
      initial_connection_window_size: "24MiB"
      http2_adaptive_window: false
      max_frame_size: "16KiB"
      http2_keepalive_interval: "30s"
      http2_keepalive_timeout: "10s"

      # Message size limit
      max_decoding_message_size: "4MiB"

      # Load shedding behavior
      load_shed: true

      # ---------------------------------------------------------
      # HTTP Settings (nested under 'http' key)
      # ---------------------------------------------------------
      http:
        listening_addr: "0.0.0.0:4318"

        # Concurrency: 0 = share gRPC semaphore capacity
        max_concurrent_requests: 0

        wait_for_result: false
        timeout: "30s"

        # Body size limit (wire + decompressed)
        max_request_body_size: "4MiB"

        # Accept gzip/deflate/zstd compressed request bodies
        accept_compressed_requests: true

        # TCP socket tuning
        tcp_nodelay: true
        tcp_keepalive: "45s"
        tcp_keepalive_interval: "15s"
        tcp_keepalive_retries: 5
```

## Protocol Details

<!-- markdownlint-disable MD013 -->

### OTLP/gRPC

| Aspect | Details |
|--------|---------|
| Default Port | 4317 |
| Transport | HTTP/2 via tonic |
| Endpoints | Standard gRPC service methods |
| Compression | zstd, gzip, deflate (configurable) |
| Concurrency | `GlobalConcurrencyLimitLayer` when HTTP is disabled; shared semaphore when HTTP is enabled |
| Backpressure | gRPC-only: early refusal at `poll_ready`; dual-protocol: queues on shared semaphore (no permit timeout) |

### OTLP/HTTP

| Aspect | Details |
|--------|---------|
| Default Port | 4318 |
| Transport | HTTP/1.1 via hyper |
| Endpoints | `POST /v1/logs`, `/v1/metrics`, `/v1/traces` |
| Content-Type | `application/x-protobuf` (JSON not supported) |
| Compression | gzip, deflate, zstd via `Content-Encoding` |
| Concurrency | Direct semaphore acquisition with timeout (shared when HTTP+gRPC) |
| Backpressure | Waits up to `timeout` for permit, then 503 |

#### HTTP Endpoints

```text
POST /v1/logs    -> ExportLogsServiceRequest    -> ExportLogsServiceResponse
POST /v1/metrics -> ExportMetricsServiceRequest -> ExportMetricsServiceResponse
POST /v1/traces  -> ExportTraceServiceRequest   -> ExportTraceServiceResponse
```

#### HTTP Response Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad Request (body too large, decompression failed) |
| 404 | Unknown endpoint |
| 405 | Method not allowed (only POST supported) |
| 415 | Unsupported media type (only protobuf) |
| 500 | Internal error |
| 503 | Service unavailable (at capacity or pipeline error) |

<!-- markdownlint-enable MD013 -->

## Security Considerations

### Body Size Limits

The `max_request_body_size` setting enforces limits on BOTH wire size and
decompressed size, providing defense-in-depth against:

1. **Bandwidth abuse**: Large payloads rejected during collection
2. **Decompression bombs**: Payloads that decompress to excessive sizes

Example with `max_request_body_size: 4MiB`:

- 5 MiB compressed payload -> Rejected (wire size exceeds limit)
- 2 MiB compressed -> 10 MiB decompressed -> Rejected
- 2 MiB compressed -> 3 MiB decompressed -> Accepted

### Timeout Protection

Both protocols enforce timeouts to mitigate slow-client (Slowloris-style) DoS:

- **gRPC**: Server-level timeout via tonic
- **HTTP**: Per-request timeout wrapping the entire handler, plus permit
  acquisition timeout (permit wait is included in the overall request timeout)

### Concurrency Limits

When both protocols are enabled, the shared semaphore bounds total inflight
requests across gRPC and HTTP. When at capacity:

- **gRPC (dual-protocol):** Requests queue on `acquire_owned().await`
  (no permit timeout); request-level `timeout` applies after permit acquisition.
- **HTTP (dual-protocol):** Requests queue for up to `timeout` (default 30s)
  waiting for a permit; on timeout, respond 503.

When only gRPC is enabled, the original `GlobalConcurrencyLimitLayer` applies:
excess requests are refused at `poll_ready` rather than queued.

## TLS Support

TLS is available via the `experimental-tls` feature flag:

```yaml
config:
  # gRPC TLS
  tls:
    cert_file: "/path/to/cert.pem"
    key_file: "/path/to/key.pem"
    # Optional client CA for mTLS
    client_ca_file: "/path/to/ca.pem"
    handshake_timeout: "10s"

  http:
    listening_addr: "0.0.0.0:4318"
    # HTTP TLS (independent from gRPC)
    tls:
      cert_file: "/path/to/cert.pem"
      key_file: "/path/to/key.pem"
```

## Wait-for-Result Mode

When `wait_for_result: true`, the receiver waits for an ACK/NACK from the
immediate downstream component before responding to the client:

```text
Client -> Receiver -> Pipeline -> [Downstream] -> ACK/NACK -> Receiver -> Client
```

This provides delivery confirmation but does NOT guarantee end-to-end delivery
to the final exporter destination.

## Implementation Notes

### Why No JSON Support?

The HTTP receiver only accepts `application/x-protobuf`. JSON support would
require full deserialization, breaking the zero-copy optimization that makes
this receiver efficient. To preserve the zero-deserialization path and lower
CPU/memory cost, JSON is intentionally unsupported. If JSON is needed, an
upstream proxy can perform the conversion.

### Shared vs Per-Protocol Concurrency

The `max_concurrent_requests` in the HTTP config is used only when the HTTP
server runs standalone. When HTTP is configured alongside gRPC (the typical
case), both protocols share the gRPC semaphore to ensure unified backpressure.

### Service Cloning Pattern

The `SharedConcurrencyLayer` clones the inner gRPC service per request. This
is safe because the OTLP service implementations:

1. Have stateless `poll_ready` (always returns `Ready`)
2. Use `Arc`-based internal state sharing

See [shared concurrency implementation][shared-concurrency] for detailed
documentation on service compatibility requirements.

### Thread-per-Core vs `Send` Tax

- **Execution model today:** Each pipeline runs on a single-threaded Tokio runtime
  (`new_current_thread` + `LocalSet`), so scheduling remains thread-per-core.
- **Why `Send` is required:** The gRPC path uses tonic, which requires `Send`
  futures/services. Shared metrics/state are therefore `Arc`-backed, and the HTTP
  path shares that state and uses `tokio::spawn` via `TaskTracker`, which also
  imposes `Send`.
- **Trade-off:** Even though tasks stay on one OS thread, the shared `Send`
  bounds mean we pay the `Arc`/atomic tax instead of using `Rc`/`!Send` types.
- **Potential improvement:** An HTTP-only `!Send` path (local task tracker +
  `spawn_local`, `Rc` metrics) would drop the `Send` tax, but would require
  separating HTTP state from the tonic-driven `Send` path.

### Memory Usage & Future Improvements

The current HTTP implementation uses a "buffer-then-decompress" strategy:

1. Collect the full compressed body (up to `max_request_body_size`).
2. Decompress into a separate buffer (also limited by `max_request_body_size`).

**Impact:** Peak memory usage per request is roughly
`compressed_size + decompressed_size`, bounded by `2 * max_request_body_size`.
For the default 4MiB limit, this means ~8MiB peak per concurrent request.

**Future Optimization:** A streaming decompression implementation (wrapping the
`Incoming` body stream directly) could reduce this to just `decompressed_size`,
avoiding the double buffering. This is a potential optimization if memory
constraints become tighter or default body limits need to increase
significantly.

### Graceful Shutdown

The receiver implements a robust shutdown mechanism to ensure no in-flight data
is lost when the process terminates:

1. **Stop Accepting:** The listener loop is broken immediately, stopping new
   connections.
2. **Drain Idle:** HTTP/1.1 Keep-Alive connections are signaled to close after
   their current request completes.
3. **Wait for Active:** A `TaskTracker` waits for all currently executing
   request handlers to finish (e.g., waiting for pipeline ACKs).
4. **Timeout:** A hard deadline (default 30s) prevents the server from hanging
   indefinitely if a handler stalls. The HTTP drain timeout is tied to the HTTP
   request timeout (default 30s) and coded in sync with it; if the default
   changes in code, update this doc accordingly.

[shared-concurrency]: ../crates/otap/src/shared_concurrency.rs
