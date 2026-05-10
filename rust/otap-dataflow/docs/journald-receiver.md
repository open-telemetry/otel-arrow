# Journald Receiver Design

<!-- markdownlint-disable MD013 -->

**Status:** Draft
**Tracking issue:** [#2858](https://github.com/open-telemetry/otel-arrow/issues/2858)
**Related epic:** [#2844](https://github.com/open-telemetry/otel-arrow/issues/2844)
**Owner:** @lalitb

## Summary

The journald receiver ingests local `systemd-journald` entries on Linux and
emits OTAP log records. It reads through the `sd-journal` API, not by tailing
`.journal` files and not by execing `journalctl`.

The receiver is a source-specific sibling to the proposed filelog receiver in
[#2844](https://github.com/open-telemetry/otel-arrow/issues/2844). It should
reuse the same architectural principles where they apply:
bounded backpressure, Ack-aware progress, durable checkpoints, partitioned
ownership, and processor-owned semantic processing. It should not depend on
the #2844 filelog assignment extension landing first.

## Core Decisions

| Decision | Choice |
| --- | --- |
| Source API | `sd-journal` via the `systemd` crate or a small internal `libsystemd` FFI wrapper |
| Progress unit | Opaque journald cursor (`__CURSOR`) |
| First implementation | Linux-only, default local system journal, static partition list |
| Delivery model | At-least-once from the last committed cursor |
| Checkpoint advance | Only after downstream Ack and durable checkpoint write |
| Backpressure | Stop calling `sd_journal_next()` when the bounded handoff is full |
| Semantic processing | Kept out of the receiver; processors do normalization, parsing, routing |
| Blocking calls | Isolated on a dedicated worker thread, never on the df-engine async task |
| NUMA | First PR exposes placement hints only; pinning/co-location is future work |

## Journald vs Filelog

Journald is not a normal file tailing source even though journal data is stored
on disk. A filelog receiver owns file discovery, file identity, byte offsets,
line framing, and rotation. A journald receiver owns journal source selection,
`sd-journal` iteration, cursor checkpoints, field extraction, and journal
retention/cursor-loss handling.

```text
filelog:  file identity + byte offset + framing
journald: journal partition + opaque cursor + structured entry
```

## Architecture

```mermaid
flowchart LR
    J["systemd journal<br/>sd-journal API"] --> W["PartitionWorker<br/>blocking thread"]
    W -->|"bounded batches<br/>batch_id + cursor range"| A["Async receiver task<br/>df-engine runtime"]
    A --> P["Processors<br/>parse / enrich / route"]
    P --> E["Exporters"]
    E -->|"Ack / Nack"| A
    A -->|"CommitCursor"| W
    W --> C["Checkpoint file<br/>partition -> cursor"]
```

The first PR uses static partitions from config, normally one partition:
`journal/system`. When #2844 introduces an assignment extension, the receiver
can swap static partitions for assignment events without changing its read,
Ack, or checkpoint model.

## Execution Model

All `sd_journal_*` calls are synchronous. Checkpoint writes also perform
blocking filesystem I/O (`write`, `fsync`, `rename`). None of these run on the
df-engine per-core async pipeline task.

The receiver uses one long-lived blocking worker thread per assigned partition:

- worker owns the `sd_journal*` handle
- async task owns the engine `EffectHandler`, lifecycle state, and Ack tracker
- bounded worker-to-async channel carries completed batches
- bounded async-to-worker channel carries pause/resume/shutdown/commit commands
- no per-record shared lock is required on the hot path

This follows the existing `host_metrics_receiver` pattern for blocking system
calls: use a dedicated worker to cap the blast radius instead of using Tokio's
shared blocking pool.

## Read Loop

```mermaid
flowchart TD
    Start([Start worker]) --> Init["Open journal<br/>apply filters<br/>seek start"]
    Init --> Next[sd_journal_next]
    Next -->|entry| Build["Read fields<br/>get cursor<br/>append to batch"]
    Next -->|no entry| Wait["sd_journal_wait(timeout)"]
    Wait --> Control
    Build --> Ready{Batch ready?}
    Ready -->|no| Next
    Ready -->|yes| Send["try_send batch"]
    Send -->|sent| Next
    Send -->|full| Hold["Hold batch<br/>do not read more"]
    Hold --> Control["Drain control channel"]
    Control -->|commit| Commit["write / fsync / rename"]
    Control -->|pause| Wait
    Control -->|resume| Send
    Control -->|shutdown| Drain["drain and close"]
    Control -->|none| Send
    Commit --> Send
    Drain --> Done([Exit])
```

If a completed batch cannot be handed to the async task, the worker keeps that
batch in memory and does not call `sd_journal_next()` again until the batch is
accepted or shutdown begins. A held batch counts against the partition's
in-flight budget.

Pause and shutdown responsiveness is bounded by `wait_timeout`. The configured
`drain_timeout` should be larger than `wait_timeout`.

## Ack and Checkpoint Model

Each emitted batch carries:

- `batch_id`
- `first_cursor`
- `last_cursor`
- an epoch used to ignore stale completions after rewinds

The async task tracks pending ranges per partition and advances the durable
cursor only through contiguous Acked ranges.

```text
emit range R1, R2, R3
Ack R2 first  -> R2 waits; checkpoint does not move
Ack R1 next   -> commit R1, then R2
Nack R3       -> checkpoint does not move past R2; rewind from committed cursor
```

Checkpoint commit ownership is split deliberately:

- async task decides which cursor should be committed and owns retry/failure
  state
- worker only executes blocking checkpoint I/O and returns success or failure
- in-memory `committed_cursor` advances only after the worker confirms the
  on-disk write succeeded

If there is no committed cursor yet, the receiver rewinds to a frozen initial
anchor captured at partition start. For `start_at: end` on an empty journal,
the first arriving entry becomes the first emitted record; a rewind may
duplicate it, which is acceptable under at-least-once delivery.

## Configuration

Example pipeline configuration:

```yaml
groups:
  default:
    pipelines:
      logs:
        nodes:
          journald:
            type: receiver:journald
            config:
              partitions:
                - id: journal/system

              units: ["nginx.service", "ssh.service"]
              identifiers: []
              priorities: [0, 1, 2, 3, 4, 5, 6, 7]
              # max_priority: info

              start_at: end

              batch:
                max_records: 1024
                max_flush_period: 200ms

              checkpoint:
                directory: "${engine.state_dir}/${engine.instance_id}/journald"
                max_in_flight_batches_per_partition: 1
                on_nack: rewind
                max_consecutive_failures: 5

              wait_timeout: 1s
              drain_timeout: 5s

              transient_error:
                max_retries: 3
                backoff: 100ms
                max_backoff: 5s
                jitter: true
```

`priorities` is an exact-match set. `max_priority` is shorthand expanded by the
receiver into explicit `PRIORITY=N` matches. The default should include all
levels `0..=7`; it should not silently drop debug entries.

Filter changes are not retroactive. If filters are widened after a checkpoint
exists, the receiver resumes from the existing cursor and does not backfill
older entries that now match.

## Field Projection

The receiver performs only mechanical OTLP projection. It preserves native
journal fields as attributes and leaves semantic-convention mapping to
processors.

| OTAP field | Source |
| --- | --- |
| `body` | `MESSAGE`, unset when missing |
| `time_unix_nano` | default `__REALTIME_TIMESTAMP`; `_SOURCE_REALTIME_TIMESTAMP` can be a future option |
| `severity_number` | derived from `PRIORITY` |
| `attributes` | all other native journal fields, key names preserved |
| internal completion state | cursor range and batch id, not emitted as attributes |

Initial severity mapping:

| Journald `PRIORITY` | Meaning | OTel severity |
| --- | --- | --- |
| `0` | emergency | `FATAL4` |
| `1` | alert | `FATAL3` |
| `2` | critical | `FATAL2` |
| `3` | error | `ERROR` |
| `4` | warning | `WARN` |
| `5` | notice | `INFO2` |
| `6` | info | `INFO` |
| `7` | debug | `DEBUG` |

Binary `MESSAGE` handling depends on the current OTAP body/attribute support.
If bytes can be represented directly, preserve bytes. Otherwise encode as
base64 and mark the encoding explicitly; do not lossy-decode.

## Failure Policy

| Case | Behavior |
| --- | --- |
| `sd_journal_open` / permission failure | startup failure; not treated as an empty stream |
| checkpoint missing | apply `start_at` |
| checkpoint corrupt / unknown version | fail closed; operator must remove or migrate it |
| cursor vacuumed | emit `journald.cursor_lost`; apply `start_at` |
| checkpoint commit I/O failure | do not advance in-memory cursor; retry with backoff; fail partition after threshold |
| `sd_journal_get_cursor` failure | discard un-emitted partial batch, reopen, and reseek from committed cursor or initial anchor |
| Nack | do not advance checkpoint; rewind or fail according to config |
| shutdown deadline | abandon pending completions without advancing checkpoint; late completions are ignored and counted |
| duplicate partition in same process | process-local lease rejects the second receiver |
| duplicate across processes | not prevented in v1; use distinct engine instance checkpoint paths |

Worker thread panic fails the partition, releases its process-local lease, and
surfaces an error through the receiver/engine path.

## NUMA and Placement

The first PR does not pin threads or guarantee NUMA-local reads. It only exposes
placement metadata so a future scheduler or #2844 assignment extension can act
on it.

Linux discovery should be best-effort:

```text
journal directory -> backing device -> /sys/block/<dev>/device/numa_node
```

If the journal is on tmpfs, overlayfs, a bind mount, or a device that cannot be
resolved, the NUMA node is reported as unknown.

Future goal:

```text
journal storage NUMA node -> partition worker thread -> same-node pipeline
```

## Implementation Scope

First PR:

- `crates/core-nodes/src/receivers/journald_receiver/`
- `urn:otel:receiver:journald`
- Linux gated behind a `journald` Cargo feature
- real `SdJournalReader` plus fake reader for tests
- static partition list from config
- local cursor checkpoint file per partition
- dedicated worker thread and bounded channels
- contiguous-Ack tracker with default `max_in_flight_batches_per_partition = 1`

Not in first PR:

- #2844 assignment extension integration
- multi-namespace discovery
- NUMA pinning or scheduler co-location
- `journalctl` fallback
- semantic-convention normalization processor
- offline `.journal` file ingestion

## Tests

Use a fake journal reader for most tests and a Linux-only smoke test for real
`sd-journal` behavior.

Required unit coverage:

- config validation and priority expansion
- field projection and severity mapping
- contiguous Ack tracker, out-of-order Acks, Nack rewind
- initial-anchor behavior before the first committed cursor
- corrupt checkpoint load and checkpoint commit failure
- malformed entry fields and cursor-get failure
- backpressure: held batch stops further `sd_journal_next()` calls
- duplicate partition lease and lease release on failure
- shutdown deadline and late completion handling
- worker panic path

Linux smoke coverage:

- read entries injected with `sd_journal_send`
- source-side matches for units, identifiers, and priorities
- restart from committed cursor
- live tailing through `sd_journal_wait`

## Relationship to #2844

The journald receiver should stay source-specific. It should not be folded into
the filelog receiver because it checkpoints by journal cursor, not by file
identity and byte offset.

Reusable concepts with #2844:

- partition-oriented ownership
- Ack-aware checkpoint advancement
- checkpoint envelope conventions
- placement hints for future NUMA-aware assignment
- handoff/drain contracts once an assignment extension exists

The local v1 checkpoint envelope is provisional. If #2844 later freezes a
shared checkpoint format, this receiver may need a one-time migration.

## References

- [`sd-journal` API](https://www.freedesktop.org/software/systemd/man/sd-journal.html)
- [Journal file format](https://systemd.io/JOURNAL_FILE_FORMAT/)
- [Journal export format](https://systemd.io/JOURNAL_EXPORT_FORMATS/)
- [Native journal protocol](https://systemd.io/JOURNAL_NATIVE_PROTOCOL/)
- [Go contrib journaldreceiver](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/receiver/journaldreceiver)
- [`systemd` Rust crate](https://crates.io/crates/systemd)
- [`tracing-journald`](https://docs.rs/tracing-journald/latest/tracing_journald/)
