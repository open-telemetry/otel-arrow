# Persistence Processor

The Persistence Processor provides crash-resilient durable buffering using a
write-ahead log (WAL) and segment storage. Data is persisted before forwarding
downstream, enabling recovery after crashes or network outages.

Enable with `--features persistence`.

## Configuration

```yaml
nodes:
  persistence:
    kind: processor
    plugin_urn: "urn:otap:processor:persistence"
    out_ports:
      out_port:
        destinations:
          - exporter
        dispatch_strategy: round_robin
    config:
      # Directory for persistent storage (required)
      path: /var/lib/otap/persistence

      # Maximum disk space (default: 10 GiB)
      retention_size_cap: 10 GiB

      # Maximum age of data to retain (optional, no default)
      # When set, data older than this becomes eligible for removal.
      # Can be combined with retention_size_cap for dual-constraint retention.
      # max_age: 24h

      # Policy when size cap is reached (default: backpressure)
      # - backpressure: Block ingestion (no data loss)
      # - drop_oldest: Remove oldest segments (controlled data loss)
      size_cap_policy: backpressure

      # Interval for polling for available bundles (default: 100ms)
      poll_interval: 100ms

      # Maximum time a segment stays open before finalization (default: 1s)
      # Lower values reduce latency but increase I/O overhead
      max_segment_open_duration: 1s

      # OTLP handling mode (default: pass_through)
      # - pass_through: Store OTLP as opaque binary, very CPU efficient
      # - convert_to_arrow: Convert to Arrow format, enables querying but higher CPU
      otlp_handling: pass_through

      # --- Retry Configuration ---
      # Controls exponential backoff for failed downstream deliveries.
      # Bundles that receive NACK from downstream are retried with backoff.

      # Initial retry delay after first NACK (default: 1s)
      initial_retry_interval: 1s

      # Maximum retry delay cap (default: 30s)
      max_retry_interval: 30s

      # Backoff multiplier (default: 2.0)
      # Each retry: min(initial * multiplier^retry_count, max_interval)
      retry_multiplier: 2.0

      # Maximum bundles in-flight to downstream (default: 1000)
      # Limits concurrent delivery attempts to prevent thundering herd
      # after extended network outages or restarts with large backlogs.
      max_in_flight: 1000
```

## Architecture

Each processor instance (one per CPU core) has its own isolated storage engine:

```text
{path}/
+-- core_0/
|   +-- wal/
|   +-- segments/
+-- core_1/
|   +-- wal/
|   +-- segments/
+-- ...
```

## Dispatch Strategy

**Important**: The dispatch strategy on the incoming edge affects behavior:

| Strategy      | Behavior                               | Recommendation     |
| ------------- | -------------------------------------- | ------------------ |
| `RoundRobin`  | Data distributed across cores          | **Recommended**    |
| `Random`      | Similar to round-robin                 | OK                 |
| `LeastLoaded` | Similar to round-robin                 | OK                 |
| `Broadcast`   | Same data persisted Nx (once per core) | **Avoid**          |

Using `Broadcast` on the incoming edge causes:

- Nx storage consumption (data duplicated across all cores)
- Nx duplicate messages forwarded downstream

For the **outgoing edge** (to exporters), any dispatch strategy is valid.

## Message Flow

1. **Ingest**: Incoming data is written to the WAL, then ACK sent upstream
2. **Segment Finalization**: When segment reaches size/time threshold, it's
   written to disk
3. **Forward**: Timer tick polls for finalized bundles, sends downstream
4. **ACK/NACK**: On ACK, bundle marked complete; on NACK, deferred for retry
5. **Cleanup**: Fully-consumed segments are deleted to reclaim disk space
