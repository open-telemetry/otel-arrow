# Fan-out Processor

The fan-out processor clones incoming PData to multiple downstream destinations
with configurable delivery modes, ack policies, and fallback routing.

## Configuration

```yaml
processor:
  urn: urn:otel:fanout:processor
  config:
    mode: parallel          # or "sequential"
    await_ack: primary      # or "all" or "none"
    timeout_check_interval: 200ms
    max_inflight: 10000     # maximum in-flight messages (0 = unlimited)
    destinations:
      - port: primary_export
        primary: true
        timeout: 5s
      - port: backup_export
        fallback_for: primary_export
        timeout: 10s
      - port: analytics_export
```

> **Future improvement**: The `destinations` config could be merged with the node's
> `outputs` mechanism to reduce redundancy. This would allow fanout-specific fields
> (primary, timeout, fallback_for) to be specified directly on output ports rather
> than duplicating port names in both places.

## Delivery Modes

### Parallel (default)

Send to all destinations simultaneously:

```text
                         +-----------------+
                         |     FANOUT      |
    PData -------------->|                 +------> Dest A (primary)
                         |  Clone & Send   +------> Dest B
                         |  Simultaneously +------> Dest C
                         +-----------------+
```

### Sequential

Send one-by-one, advancing only after receiving an ack:

```text
    PData --> FANOUT --> Dest A --> [wait] --> Dest B --> [wait] --> Dest C
```

- If a destination nacks and has a fallback, the fallback is tried first
- If a destination nacks without fallback, the entire request fails

## Ack Policies (`await_ack`)

### `none` (Fire-and-Forget)

```text
    Upstream              FANOUT                    Downstream
        |                    |
        |  PData             |
        +------------------->|----> Send to all
        |                    |
        |<--- Ack (immediate)|      (outcomes ignored)
```

- Ack upstream immediately after dispatch
- Downstream acks/nacks are ignored
- Inflight cleared immediately; no tracking

### `primary` (default)

```text
    Upstream              FANOUT                    Downstream
        |                    |
        |  PData             |
        +------------------->|----> Dest A (primary)
        |                    |----> Dest B (secondary)
        |                    |
        |                    |<---- Nack B (ignored)
        |                    |<---- Ack A
        |<--- Ack -----------|
```

- Only the primary destination's outcome matters
- Secondary acks/nacks are ignored
- Upstream ack when primary (or its fallback) acks
- Upstream nack when primary (and all fallbacks) fail

### `all`

```text
    Upstream              FANOUT                    Downstream
        |                    |
        |  PData             |
        +------------------->|----> Dest A
        |                    |----> Dest B
        |                    |
        |                    |<---- Ack A (wait for B)
        |                    |<---- Ack B (all complete)
        |<--- Ack -----------|
```

- Wait for all non-fallback destinations to complete
- **Fail-fast**: If ANY destination nacks (without fallback), upstream nacks

## Fallback Routing

Destinations can declare `fallback_for: <port>` to act as a backup:

```yaml
destinations:
  - port: primary
    primary: true
    timeout: 5s
  - port: backup
    fallback_for: primary
```

### Behavior

```text
    FANOUT --> Primary --> [Nack or Timeout]
                   |
                   +---> Trigger Backup --> [Ack] --> Upstream Ack
```

- Fallbacks are **not sent initially** (held in reserve)
- Triggered on nack OR timeout of the origin
- Fallback's outcome becomes the origin's outcome

### Chained Fallbacks

Fallbacks can be chained (A -> B -> C):

```yaml
destinations:
  - port: a
    primary: true
  - port: b
    fallback_for: a
  - port: c
    fallback_for: b
```

```text
    A nacks --> B triggered --> B nacks --> C triggered --> C acks --> Ack
```

- Cycles are detected and rejected at config validation
- If the final fallback fails, upstream nacks
- **Fallbacks are only triggered on failure**: When an origin succeeds, its fallback(s)
  are marked as skipped and will not be dispatched (even in sequential mode)

## Timeouts

Per-destination timeouts trigger fallback (or nack if no fallback):

```yaml
destinations:
  - port: primary
    timeout: 5s      # Timeout triggers fallback
  - port: backup
    fallback_for: primary
    timeout: 10s     # Backup has its own timeout
```

- Timeouts are checked on periodic `TimerTick` (default interval: 200ms)
- Timeout is treated as a failure (same as nack)
- **Timeouts are terminal**: Once an endpoint times out, late acks/nacks from that
  endpoint are ignored. The fallback's outcome determines the result, even if the
  original destination eventually responds successfully.

## Backpressure

The fanout processor propagates backpressure upstream through multiple mechanisms:

### 1. Ack/Nack Propagation

When `await_ack` is `primary` or `all`, the processor withholds upstream acks until
downstream destinations respond:

```text
Upstream --> FANOUT --> Downstream
                |
         waits for ack/nack
                |
         then acks/nacks upstream
```

- **`await_ack: primary`**: Fanout does not ack upstream until primary
  destination responds
- **`await_ack: all`**: Fanout does not ack upstream until all destinations
  respond
- **`await_ack: none`**: Upstream acked immediately (fire-and-forget)

Note: Withholding acks does not prevent upstream from sending more messages -
upstream can continue sending until `max_inflight` or channel capacity limits are reached.

### 2. Max Inflight Limit

For tracked modes (`primary`/`all`), the `max_inflight` setting (default: 10,000)
provides the explicit bound on concurrent requests:

| `await_ack` | Overflow Mechanism | Behavior |
|-------------|-------------------|----------|
| `primary`/`all` | `max_inflight` | Nack when limit reached |
| `none` | Channel capacity | Block until downstream drains |

> **Note**: `max_inflight` only applies to tracked modes.
> Fire-and-forget (`await_ack: none`) has no inflight tracking and relies
> solely on channel backpressure.

When the limit is reached:

```text
Upstream                    FANOUT
    |                          |
    |-- PData ---------------->|  inflight.len() >= max_inflight?
    |                          |           |
    |<-- NACK "limit exceeded"-|<----------+ YES
    |                          |
    |  (upstream can retry     |
    |   or apply its policy)   |
```

### 3. Bounded Channels

All sends go through bounded async channels. Even in fire-and-forget mode
(`await_ack: none`), if a downstream channel is full, the send will `await`
until space is available:

```text
FANOUT --> [bounded channel] --> Downstream
              |
         if full, await
         (blocks processor)
```

### Summary

| Layer | Mechanism | Effect |
|-------|-----------|--------|
| Semantic | Ack/nack propagation | Upstream waits for downstream outcome |
| State | `max_inflight` limit | Nacks when too many requests pending |
| Transport | Bounded channels | Blocks when downstream channel full |

Set `max_inflight: 0` for unlimited tracking (not recommended for production).

## Metrics

| Metric | Description |
|--------|-------------|
| `sent` | Requests dispatched (per incoming PData) |
| `acked` | Requests acked upstream (after await_ack/fallback) |
| `nacked` | Requests nacked upstream (after await_ack/fallback) |
| `timed_out` | Destinations that timed out |
| `rejected_max_inflight` | Requests rejected due to max_inflight limit |

> **Note**: `acked` and `nacked` reflect *request-level* outcomes after
> applying the `await_ack` policy and fallback logic - not per-destination
> results. For per-destination metrics, use channel-level metrics.

## Cloning and Mutability

Each destination receives an independent **clone** of the incoming PData.

### Clone Cost

Cloning is cheap for both data formats:

| Format | Storage | Clone Cost |
|--------|---------|------------|
| OTLP (OtlpProtoBytes) | `bytes::Bytes` | O(1) refcount bump |
| OTAP (OtapArrowRecords) | `Arc` per column | O(columns) refcount bumps |

No telemetry data is deep-copied during fan-out. Only reference counts are
incremented.

### Mutability

- Downstream processors and exporters may mutate their copy freely
- Mutations do not affect other destinations
- Underlying buffers are copy-on-write (deep copy only when modified)

### Context Stack

Each clone gets its own Context (subscription stack) for independent ack/nack
routing per destination. The context is small (typically 1-3 frames) and is
copied during clone.

## Quick Reference

| Mode | await_ack | Behavior |
|------|-----------|----------|
| parallel | none | Send all, ack immediately, ignore outcomes |
| parallel | primary | Send all, wait for primary (or fallback) |
| parallel | all | Send all, wait for all, fail-fast on nack |
| sequential | primary | Send one-by-one, complete when primary done |
| sequential | all | Send one-by-one, all must complete in order |

## Edge Cases

- **Duplicate ports**: Rejected at config validation
- **Multiple primaries**: Rejected; exactly one primary allowed
- **Primary as fallback**: Rejected; primary cannot have `fallback_for`
- **Unknown fallback target**: Rejected; `fallback_for` must reference port
- **Fallback cycles**: Detected and rejected at config validation
- **Fallback with `await_ack: none`**: Rejected; fire-and-forget ignores fallbacks
- **Timeout with `await_ack: none`**: Rejected; fire-and-forget doesn't track responses
- **Max inflight exceeded**: New messages nacked with backpressure
  (does not apply to `await_ack: none`)
- **Shutdown**: Inflight state is dropped; no nacks sent to upstream.
  Upstream will not receive notification for in-progress requests.

## Performance Optimizations

The processor uses optimized fast paths for common configurations to minimize
memory allocations per request:

| Configuration | Fast Path | State Per Request |
|---------------|-----------|-------------------|
| `await_ack: none` | Fire-and-forget | **None** (zero tracking) |
| `parallel` + `primary` (no fallback/timeout) | Slim primary | Minimal map |
| All other configs | Full | Complete endpoint tracking |

> **Note**: For tracked modes (`primary`/`all`), internal state is bounded by
> `max_inflight` (default: 10,000). When the limit is reached, new requests
> are nacked. See [Backpressure](#backpressure).

### Fire-and-Forget (`await_ack: none`)

- Bypasses all inflight state allocation
- Clones and sends to each destination immediately
- Acks upstream without waiting for downstream
- No subscriptions to downstream control messages

### Slim Primary Path

Eligible when: `mode: parallel`, `await_ack: primary`, no `fallback_for`, no `timeout`

- Uses a tiny map instead of per-endpoint state
- Ignores non-primary acks/nacks
- Forwards upstream immediately on primary ack/nack

### Full Path

Required for: `sequential` mode, `await_ack: all`, any fallback, any timeout

- Tracks per-endpoint status, timeouts, and fallback chains
- Coordinates ordering (sequential) or completion (await_all)
- Handles failover to backup destinations
