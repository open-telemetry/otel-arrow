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
    destinations:
      - port: primary_export
        primary: true
        timeout: 5s
      - port: backup_export
        fallback_for: primary_export
        timeout: 10s
      - port: analytics_export
```

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

- Wait for all origins to complete
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

## Metrics

| Metric | Description |
|--------|-------------|
| `sent` | Requests dispatched (per incoming PData) |
| `acked` | Requests successfully acked upstream |
| `nacked` | Requests nacked upstream |
| `timed_out` | Destinations that timed out |

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
- **Shutdown**: Inflight requests are dropped (not proactively nacked)
