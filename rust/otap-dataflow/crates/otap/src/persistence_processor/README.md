# Persistence Processor

The Persistence Processor provides crash-resilient durable buffering using
Quiver's write-ahead log (WAL) and segment storage. Data is persisted before
forwarding downstream, enabling recovery after crashes or network outages.

Enable with `--features persistence`.

## Configuration

```yaml
processors:
  persistence:
    # Directory for persistent storage (required)
    path: /var/lib/otap/persistence

    # Maximum disk space (default: 10 GiB)
    retention_size_cap: 10 GiB

    # Policy when size cap is reached (default: backpressure)
    # - backpressure: Block ingestion (no data loss)
    # - drop_oldest: Remove oldest segments (controlled data loss)
    size_cap_policy: backpressure

    # Interval for polling Quiver for bundles (default: 100ms)
    poll_interval: 100ms

    # Maximum time a segment stays open before finalization (default: 1s)
    # Lower values reduce latency but increase I/O overhead
    max_segment_open_duration: 1s
```

## Architecture

Each processor instance (one per CPU core) has its own isolated Quiver engine:

```text
{path}/
├── core_0/
│   ├── wal/
│   └── segments/
├── core_1/
│   ├── wal/
│   └── segments/
└── ...
```

## Dispatch Strategy

**Important**: The dispatch strategy on the incoming edge affects behavior:

| Strategy      | Behavior                               | Recommendation     |
| ------------- | -------------------------------------- | ------------------ |
| `RoundRobin`  | Data distributed across cores          | ✅ **Recommended** |
| `Random`      | Similar to round-robin                 | ✅ OK              |
| `LeastLoaded` | Similar to round-robin                 | ✅ OK              |
| `Broadcast`   | Same data persisted N× (once per core) | ⚠️ **Avoid**       |

Using `Broadcast` on the incoming edge causes:

- N× storage consumption (data duplicated across all cores)
- N× duplicate messages forwarded downstream

For the **outgoing edge** (to exporters), any dispatch strategy is valid.

## Message Flow

1. **Ingest**: Incoming data is written to Quiver's WAL, then ACK sent upstream
2. **Segment Finalization**: When segment reaches size/time threshold, it's
   written to disk
3. **Forward**: Timer tick polls for finalized bundles, sends downstream
4. **ACK/NACK**: On ACK, bundle marked complete; on NACK, deferred for retry
5. **Cleanup**: Fully-consumed segments are deleted to reclaim disk space
