# OTLP Receiver

The OTLP Receiver ingests telemetry data via the OpenTelemetry Protocol (OTLP)
and forwards it into the OTAP dataflow pipeline. It supports both gRPC (HTTP/2)
and HTTP/1.1 protocols with unified concurrency control.

**Plugin URN (full):** `urn:otel:otlp:receiver`
**Plugin URN (OTel shortcut):** `otlp:receiver`

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
               |             (dual-protocol mode only)         |
               |   +---------------------------------------+   |
               |   |        Global Semaphore               |   |
               |   |   (bounds total across both protos)   |   |
               |   +---------+-----------------+-----------+   |
               |             |                 |               |
        +------v-------------v--+         +----v---------------v-------+
        | gRPC Local Semaphore  |         | HTTP Local Semaphore       |
        | (per-proto)           |         | (per-proto)                |
        +------+----------------+         +---------------+------------+
               |                                          |
               | poll_ready                               | acquire_owned()
               | (backpressure)                           | + timeout
               +--------------------+---------------------+
                                    |
                          +---------v---------+
                          | OtapPdata         |
                          |   context: Context|
                          |   payload: OtlpBytes  | <-- Raw serialized
                          +-----------+-----------+
                                      |
       +------------------------------v-------------------------------+
       |              Shared Infrastructure (gRPC + HTTP)             |
       |  - AckRegistry (wait_for_result subscriptions)               |
       |  - Unified Metrics (requests, acks, nacks, errors)           |
       |  - Control Loop (ACK/NACK routing, telemetry, shutdown)      |
       +------------------------------+-------------------------------+
                                      |
                          +-----------v-----------+
                          | Pipeline (Bounded MPSC)|
                          +-----------------------+
```

## Key Design Principles

### Zero-Deserialization Path

Both protocols keep OTLP payloads in their serialized protobuf form throughout
the receiver. Decoding is deferred to downstream pipeline stages, reducing CPU
overhead and memory allocations on the hot path.

- **gRPC**: Custom `OtlpBytesCodec` extracts the raw bytes without parsing
- **HTTP**: Body is collected and (optionally) decompressed, but not decoded

### Concurrency Control

The receiver supports three deployment modes with different concurrency strategies:

#### Deployment Modes

<!-- markdownlint-disable MD013 -->

| Mode | Configuration | Semaphores Used |
|------|---------------|-----------------||
| **gRPC-only** | Only `protocols.grpc` configured | Local gRPC semaphore only |
| **HTTP-only** | Only `protocols.http` configured | Local HTTP semaphore only |
| **Both protocols** | Both `protocols.grpc` and `protocols.http` | Global + per-protocol local |

<!-- markdownlint-enable MD013 -->

#### Dual-Protocol Mode (Global + Local Semaphores)

When both protocols are enabled, a two-level semaphore design controls admission:

```text
                     +---------------------------+
                     |    Global Semaphore       |
                     | (bounds total inflight)   |
                     +-------------+-------------+
                                   |
              +--------------------+--------------------+
              |                                         |
   +----------v-----------+              +--------------v-----------+
   | gRPC Local Semaphore |              | HTTP Local Semaphore     |
   | poll_ready gating    |              | acquire_owned() + timeout|
   | (backpressure)       |              | (default 30s or timeout) |
   +----------------------+              +--------------------------+
```

Each request must acquire **both** a global permit and a protocol-local permit:

1. **Global semaphore**: Ensures total inflight requests across both protocols
   never exceed downstream channel capacity
2. **Per-protocol semaphore**: Allows independent rate limiting per protocol

This ensures total inflight requests never exceed downstream channel capacity,
regardless of which protocol clients use.

#### Single-Protocol Modes

- **gRPC only:** Uses `GlobalConcurrencyLimitLayer` with early refusal at
  `poll_ready`. No global semaphore overhead.
- **HTTP only:** Uses its own local semaphore with permit timeout (default 30s).
  No global semaphore overhead.

#### Backpressure Behavior

- **gRPC**: Permit acquired in `poll_ready` via Tower layer; backpressure
  propagates to HTTP/2 so new streams are not accepted while saturated.
- **HTTP**: Requests queue for up to `timeout` waiting for a permit; on timeout,
  respond 503 Service Unavailable.

#### Tuning Examples

The `max_concurrent_requests` setting interacts with downstream pipeline capacity
(configured via `policies.channel_capacity.pdata`). Here are
common scenarios:

##### Scenario 1: Full Auto-Tuning (Recommended Default)

```yaml
policies:
  channel_capacity:
      pdata: 100  # Downstream capacity

config:
  protocols:
    grpc:
      max_concurrent_requests: 0   # Auto-tune
    http:
      max_concurrent_requests: 0   # Auto-tune
```

Result:

- Global semaphore: 100 permits (matches downstream)
- gRPC local: 100 permits
- HTTP local: 100 permits
- Total inflight across both: capped at 100 by global semaphore
- Either protocol can use full capacity when the other is idle

##### Scenario 2: Explicit Equal Limits

```yaml
policies:
  channel_capacity:
      pdata: 100

config:
  protocols:
    grpc:
      max_concurrent_requests: 50
    http:
      max_concurrent_requests: 50
```

Result:

- Global semaphore: 100 permits
- gRPC local: 50 permits (capped at 50 even if HTTP is idle)
- HTTP local: 50 permits (capped at 50 even if gRPC is idle)
- Total inflight: max 100, but each protocol limited to 50

##### Scenario 3: Prioritize gRPC Over HTTP

```yaml
policies:
  channel_capacity:
      pdata: 100

config:
  protocols:
    grpc:
      max_concurrent_requests: 80
    http:
      max_concurrent_requests: 20
```

Result:

- Global semaphore: 100 permits
- gRPC can handle up to 80 concurrent requests
- HTTP limited to 20 concurrent requests
- If both saturated: gRPC gets 80, HTTP gets 20

##### Scenario 4: Oversubscribed Limits

```yaml
policies:
  channel_capacity:
      pdata: 100

config:
  protocols:
    grpc:
      max_concurrent_requests: 100
    http:
      max_concurrent_requests: 100
```

Result:

- Global semaphore: 100 permits (still bounds total)
- Each protocol can burst to 100 when alone
- When both active: global semaphore ensures total never exceeds 100
- Requests compete for global permits; neither protocol starves the other
  because both acquire from the same semaphore with equal priority (permits
  are granted in FIFO order as they become available)

##### Scenario 5: Single Protocol (gRPC-only)

```yaml
policies:
  channel_capacity:
      pdata: 100

config:
  protocols:
    grpc:
      max_concurrent_requests: 0   # Auto-tunes to 100
```

Result:

- No global semaphore (single protocol optimization)
- gRPC local: 100 permits
- Lower overhead than dual-protocol mode

##### Scenario 6: Limit Below Downstream Capacity

```yaml
policies:
  channel_capacity:
      pdata: 100

config:
  protocols:
    grpc:
      max_concurrent_requests: 25
    http:
      max_concurrent_requests: 25
```

Result:

- Global semaphore: 100 permits
- Total receiver load capped at 50 (25 + 25)
- Useful when receiver should not saturate downstream, leaving headroom for
  other pipeline inputs

**Key Takeaways:**

<!-- markdownlint-disable MD013 -->

| Setting | Behavior |
|---------|----------|
| `max_concurrent_requests: 0` | Auto-tune to downstream capacity |
| Explicit value < downstream | Hard cap per protocol |
| Explicit value > downstream | Global semaphore still enforces downstream limit |
| Sum of limits < downstream | Receiver never saturates downstream |
| Sum of limits > downstream | Protocols compete for global permits |

<!-- markdownlint-enable MD013 -->

### Unified Metrics

A single `MetricSet` tracks receiver activity across both protocols:

- `requests_started` / `requests_completed`
- `acks_received` / `nacks_received`
- `rejected_requests`
- `transport_errors`

## Configuration

The receiver uses a `protocols` structure to configure gRPC and/or HTTP endpoints.
At least one protocol must be configured.

### Minimal Configuration (gRPC only)

```yaml
nodes:
  receiver:
    type: "urn:otel:otlp:receiver" # or "otlp:receiver"
    config:
      protocols:
        grpc:
          listening_addr: "0.0.0.0:4317"
```

### Minimal Configuration (HTTP only)

```yaml
nodes:
  receiver:
    type: "urn:otel:otlp:receiver"
    config:
      protocols:
        http:
          listening_addr: "0.0.0.0:4318"
```

### Full Configuration (gRPC + HTTP)

```yaml
nodes:
  receiver:
    type: "urn:otel:otlp:receiver" # or "otlp:receiver"
    config:
      protocols:
        # ---------------------------------------------------------
        # gRPC Settings
        # ---------------------------------------------------------
        grpc:
          listening_addr: "0.0.0.0:4317"

          # --- Request/Response behavior ---
          wait_for_result: false        # Wait for downstream ACK before responding
          timeout: "30s"                # Timeout for gRPC requests

          # --- Concurrency limits ---
          # 0 = auto-tune to downstream channel capacity
          max_concurrent_requests: 0
          max_concurrent_streams: null  # HTTP/2 streams per connection (null = auto)
          transport_concurrency_limit: null  # Per-connection limit (null = auto)
          load_shed: true               # Fast reject when overloaded

          # --- Compression ---
          request_compression:          # Methods accepted for requests
            - zstd
            - gzip
            - deflate
          response_compression: []      # Methods for responses (empty = none)

          # --- TCP socket tuning ---
          tcp_nodelay: true
          tcp_keepalive: "45s"
          tcp_keepalive_interval: "15s"
          tcp_keepalive_retries: 5

          # --- HTTP/2 tuning ---
          initial_stream_window_size: "8MiB"
          initial_connection_window_size: "24MiB"
          http2_adaptive_window: false
          max_frame_size: "16KiB"
          max_decoding_message_size: "4MiB"
          http2_keepalive_interval: "30s"
          http2_keepalive_timeout: "10s"

          # --- TLS configuration ---
          # tls:
          #   cert_file: "/path/to/server.crt"
          #   key_file: "/path/to/server.key"
          #   client_ca_file: "/path/to/client-ca.crt"  # For mTLS
          #   include_system_ca_certs_pool: false
          #   handshake_timeout: "10s"
          #   reload_interval: "5m"

        # ---------------------------------------------------------
        # HTTP Settings
        # ---------------------------------------------------------
        http:
          listening_addr: "0.0.0.0:4318"

          # --- Request/Response behavior ---
          wait_for_result: false        # Wait for downstream ACK before responding
          timeout: "30s"                # Request processing timeout
          max_request_body_size: "4MiB" # Max body size (wire + decompressed)
          accept_compressed_requests: true  # Accept gzip/deflate/zstd bodies

          # --- Concurrency limits ---
          # 0 = auto-tune to downstream channel capacity
          max_concurrent_requests: 0

          # --- TCP socket tuning ---
          tcp_nodelay: true
          tcp_keepalive: "45s"
          tcp_keepalive_interval: "15s"
          tcp_keepalive_retries: 5

          # --- TLS configuration ---
          # tls:
          #   cert_file: "/path/to/server.crt"
          #   key_file: "/path/to/server.key"
          #   client_ca_file: "/path/to/client-ca.crt"  # For mTLS
          #   include_system_ca_certs_pool: false
          #   handshake_timeout: "10s"
          #   reload_interval: "5m"
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
| Concurrency | Local semaphore only (gRPC-only mode); global + local (dual-protocol mode) |
| Backpressure | `poll_ready` gating; backpressure propagates to HTTP/2 |

### OTLP/HTTP

| Aspect | Details |
|--------|---------|
| Default Port | 4318 |
| Transport | HTTP/1.1 via hyper |
| Endpoints | `POST /v1/logs`, `/v1/metrics`, `/v1/traces` |
| Content-Type | `application/x-protobuf` (JSON not supported) |
| Compression | gzip, deflate, zstd via `Content-Encoding` |
| Concurrency | Local semaphore only (HTTP-only mode); global + local (dual-protocol mode) |
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

The receiver enforces concurrency limits to prevent overwhelming downstream
components. The behavior depends on deployment mode:

**Dual-protocol mode (gRPC + HTTP):**

- Global semaphore bounds total inflight requests across both protocols
- Per-protocol semaphores provide additional per-protocol limits
- **gRPC**: Permit acquired in `poll_ready`; backpressure propagates to HTTP/2
- **HTTP**: Requests queue for up to `timeout` (default 30s); 503 on timeout

**Single-protocol modes:**

- **gRPC only**: `GlobalConcurrencyLimitLayer` applies; excess requests refused
  at `poll_ready` rather than queued
- **HTTP only**: Local semaphore with permit timeout (default 30s)

When `max_concurrent_requests: 0` (the default), the limit is auto-tuned to
match downstream channel capacity.

## TLS Support

TLS is available via the `experimental-tls` feature flag. Each protocol has its
own independent TLS configuration:

```yaml
config:
  protocols:
    grpc:
      listening_addr: "0.0.0.0:4317"
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        # Optional: Client CA for mutual TLS (mTLS)
        client_ca_file: "/path/to/client-ca.crt"
        # Optional: Include system CA certificates
        include_system_ca_certs_pool: false
        # Optional: TLS handshake timeout
        handshake_timeout: "10s"
        # Optional: Certificate reload interval
        reload_interval: "5m"

    http:
      listening_addr: "0.0.0.0:4318"
      # HTTP TLS is independent from gRPC TLS
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        client_ca_file: "/path/to/client-ca.crt"  # For mTLS
        include_system_ca_certs_pool: false
        handshake_timeout: "10s"
        reload_interval: "5m"
```

You can use TLS on one protocol while keeping the other plaintext, or use
different certificates for each protocol.

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

### Global vs Per-Protocol Concurrency

The receiver implements a two-level concurrency control strategy:

**Dual-protocol mode (gRPC + HTTP):**

Each request must acquire two permits:

1. **Global permit**: Bounds total inflight requests across both protocols,
   derived from downstream channel capacity
2. **Protocol-local permit**: Independent per-protocol limit from
   `max_concurrent_requests`

This design allows:

- Total receiver load to be bounded by downstream capacity
- Independent tuning of each protocol's limit
- Either protocol to use full capacity when the other is idle

**Single-protocol modes (gRPC-only or HTTP-only):**

Only the protocol-local semaphore is used. The global semaphore is not created,
avoiding unnecessary overhead.

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
