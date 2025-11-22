# OTel Receiver Architecture

## Overview and Objectives

This module implements an experimental OpenTelemetry (OTEL) receiver that speaks:

- OTAP Arrow over gRPC (streaming logs, metrics, traces)
- OTLP protobuf over gRPC (unary Export endpoints)

It runs directly on top of `h2` without tonic, is designed to live in a thread-per-core engine, and integrates with the OTAP dataflow engine (pipeline, admission control, telemetry, acks).

Primary objectives:

1. Support OTLP and OTAP Arrow gRPC with minimal dependencies and tight control over performance.
2. Fit a single threaded, per core runtime (no `Send + Sync` futures, `Rc` instead of `Arc`).
3. Keep the hot path free of locks and shared atomics, and minimize heap allocations.
4. Provide explicit backpressure and bounded resource usage:
   - Connection and stream admission control.
   - Bounded in flight requests and ack slots.
   - Bounded frame sizes and decompression output.

## Design Principles

- **Single threaded by design**: All request handling for a given core stays on that core.
- **Explicit backpressure**:
  - `Admitter` gates TCP connections and h2 streams.
  - `AckRegistry` is a bounded slab that limits outstanding wait slots.
  - `StatusStream` enforces per connection `max_in_flight` limits.
  - HTTP/2 flow control credits are returned only for bytes actually consumed.
- **No locks on the hot path**:
  - Router, ack registries, encoder pools, decompressor, ... all live in single threaded context.
  - Sharing uses `Rc` and `RefCell` only within that context.
- **Minimal allocations and reuse of buffers**:
  - `GrpcStreamingBody` reuses a decompression scratch buffer.
  - `ResponseEncoderPool` reuses encoders and compression buffers.
  - `AckRegistry` preallocates all slots and uses an intrusive free list.
- **Fail fast on protocol or resource violations**
  - Strict gRPC header validation.
  - Rejection of unsupported compression codes.
  - Bounded decoded message size with clear `RESOURCE_EXHAUSTED`.
  - Handshake and request timeouts.
  - Well defined error paths for admission failures and pipeline errors.

## Architecture and Main Components

### Top level receiver (otel_receiver.rs)

- A **control loop** for control messages (shutdown, metrics collection, Ack/Nack routing).
- A **data plane loop** `run_grpc_server` to handle incoming connections.

This split isolates cluster control from request serving and ensures clean shutdown behavior.

### Data plane: `run_grpc_server` and `handle_tcp_conn`

- `run_grpc_server`:
  - Accept new TCP connections.
  - One task per TCP connections (`spawn_local`).
  - Applies admission via `Admitter::try_admit_connection`
  - On cancellation or listener close, drains all connection tasks and exits.

- `handle_tcp_conn`:
  - Performs the HTTP/2 handshake with configured timeout.
  - Applies admission via `tcp_conn_guard.try_open_stream`.
  - Manages HTTP2 keep-alive.
  - Limits number of in-flight streams.

This is the main HTTP/2 server loop per TCP connection.

### Request routing: `GrpcRequestRouter`

- Ack registries per signal (optional, only if `wait_for_result`).
- `max_in_flight_per_connection`: per connection limit of in flight batches that wait for ack.
- Parses and negotiates gRPC encoding.
- Handles compression and timeout.
- Switches on `request.uri().path()`:
  - OTAP Arrow streaming services (`ARROW_*`).
  - OTLP unary Export services (`OTLP_*`).
  - Unknown path: respond with `UNIMPLEMENTED`.
- `serve_otap_stream<T>` (bidirectional OTAP Arrow endpoints).
- `serve_otlp_unary` (OTLP endpoints).

### Ack tracking: `AckRegistry` and Ack flow

- Maintains fixed-size slab of `AckSlot`s:
  - Each slot has generation counter and state.
  - States:
    - `Free { next_free }` for intrusive free list.
    - `Waiting(WaitingSlot)` with waker and outcome.
- Allocation:
  - `allocate` pops from `head_free` list and moves slot into `Waiting`.
  - O(1) with no heap allocation.

### Batch status streaming: `StatusStream`

Only used for OTAP.

## Bounded resources and backpressure

- **Connection and stream counts** through `Admitter`:
  - Hard caps on number of TCP connections and per connection h2 streams.
  - Rejection or soft drop behavior when the system is overloaded.
- **Ack wait slots**:
  - `AckRegistry` has a fixed size slab of slots.
  - On exhaustion, new work is rejected with an overloaded status.
- **Per connection in flight batches**:
  - `StatusStream` uses `max_in_flight_per_connection` to prevent a single connection from consuming all ack slots.
- **HTTP/2 flow control**:
  - `GrpcStreamingBody` tracks unacknowledged bytes and calls `release_capacity` only when bytes are actually consumed.
  - Protects both ends from window exhaustion or accidental frame drops.
- **Frame and decompression limits**:
  - `validate_frame_len` enforces `max_decoding_message_size`.
  - Decompression paths check for output exceeding this limit and fail with `RESOURCE_EXHAUSTED`.

## Timeout and keepalive guarantees

- `RequestTimeout`:
  - Enforces maximum idle time per request or per stream.
  - Resets timer on each successful item or poll.
  - Maps timeouts to `DEADLINE_EXCEEDED` status.
- `Http2Keepalive`:
  - Sends HTTP/2 PING only when connection is idle and a configured interval has elapsed.
  - Uses timeout to fail the connection if PONG is not received.
  - Prevents silent half open connections.

## Error handling

- Protocol errors (wrong content type, unsupported compression, invalid headers) are surfaced as gRPC errors with log messages.
- Transport errors in the h2 layer result in connection closure and logged errors.
- Pipeline send errors:
  - Cancel or complete any outstanding ack tokens.
  - Close stream or respond unary error, depending on path.

## StatusStream and in flight set

- At most `max_in_flight` concurrent `AckWaitFuture` per stream.
- `fill_inflight` performs at most `max_in_flight` iterations per call, each constant time except for:
  - Network reads in `next_message` (underlying cost is dominated by I/O).
  - Pipeline send, which may yield.
- `InFlightSet::poll_next`
  Delegates to `FuturesUnordered`, which is amortized O(1) per future over the lifetime of the stream.

## Configurable Parameters

- Network and HTTP/2:
  - `listening_addr`
  - `max_concurrent_streams` (per connection)
  - `initial_stream_window_size`
  - `initial_connection_window_size`
  - `max_frame_size`
  - `http2_handshake_timeout`
  - `http2_keepalive_interval`
  - `http2_keepalive_timeout`

- gRPC behavior:
  - `max_concurrent_requests` (used to size encoder pool and ack registries).
  - `request_compression_methods()` (server allowed request encodings).
  - `response_compression_methods()` (server allowed response encodings).
  - `timeout` (per request idle timeout in `RequestTimeout`).
  - `max_decoding_message_size` (per frame decoded size limit; default 64 MiB).

- Admission control:
  - `Admitter::new(100000, max_concurrent_streams or 100, 100000)`
    Currently hard coded connection and backlog limits, with a note to make them tunable.

- Ack registries:
  - Size per signal is `settings.max_concurrent_requests`.

- Per connection in flight limit:
  - Derived by `per_connection_limit(&settings)` from gRPC settings.

## Known Limitations and TODOs

- Better metrics and logging/tracing
- Unix Domain Socket
- Snappy compression (supported by the Go Collector)
- OpenZL exploration
