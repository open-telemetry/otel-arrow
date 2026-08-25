<!-- markdownlint-disable MD013 -->

# Filelog Receiver Phase 1 Conformance Specification

This document owns the detailed resource-admission models, telemetry semantics,
validation matrix, and normative examples for the [Phase 1 behavioral
specification](filelog-receiver-phase1-spec.md). Moving this reference material
out of the behavioral specification changes review structure, not normative force.

## Resource admission models

The formulas in this section are conservative implementation and test guidance
for the proposed components. Their population bounds and checked-arithmetic
requirements are normative; coefficients and provisional ceilings are not
stable architecture or wire-format constants. They must be revised from
representative measurements when implementation layout changes. They are not
exact RSS, allocator-resident memory, universal throughput, or performance
guarantees.

Every sum, product, cast, and duration conversion uses checked arithmetic.

### Candidate and identity reconciliation

Let `P = 44 + ADVISORY_PATH_STORED_MAX_BYTES = 4140`, covering path kind,
flags, full length, stored length, stored bytes, and full-path digest.

```text
candidate_base =
  (max_pending_candidates + max_open_files) *
  (5 * fingerprint_bytes + 4 * P + 2048)

open_candidate_amplification =
  max_open_files *
  (10 * fingerprint_bytes + 10 * P + 4096)

checkpoint_record_state =
  max_tracked_files *
  (fingerprint_bytes + P + 384)

discovery_tracked_state =
  max_tracked_files * 1024

identity_reconciliation =
  candidate_base
  + open_candidate_amplification
  + checkpoint_record_state
  + discovery_tracked_state
```

`identity_reconciliation` must not exceed 1 GiB.

The coefficients cover simultaneously owned bounded inventories, fingerprint and path
payload copies, update preflight, encoded transaction staging, applied-record scratch,
maps, vectors, locators, allocation metadata, and removal-heavy event state.

### Framer payload

```text
copies = 2 for non-raw preserve_raw
copies = 1 otherwise

framer_peak_payload =
  4 * copies *
    (min(max_line_bytes, max_record_bytes) + max_record_bytes)
  + 16 * copies
  + 16
```

The factor four models old and new vector allocations coexisting during growth. Fixed
terms cover delimiter lookahead and a pending decoded/source unit. Regex program/cache,
decoder objects, allocator metadata, and library overhead remain separate bounded or
measured terms.

### Reader table

```text
reader_table_payload =
  max_tracked_files *
    (fingerprint_bytes + 2 * P + 1024)
  + max_read_bytes_per_turn
```

The per-reader fixed term covers the logical reader, bounded indexes, ready and EOF
scheduling state, lease guard, and collection overhead. The final term is the single
shared source-turn buffer, not one buffer per reader.

### Batch state

One open or retained receiver-wide batch is bounded by:

- `batch.max_records`;
- `batch.max_bytes` under the shared logical-size function;
- 4,096 distinct file deltas and the
  [checkpoint-format-defined](filelog-checkpoint-format.md#maximum-encoded-lengths-summary)
  `MAX_PROGRESS_TX_FRAME_BYTES`;
- one first-record deadline;
- one correlation tuple and retry state; and
- bounded Arrow/library overhead measured separately.

Retained and outgoing batch values may share Arrow buffers. The model does not claim
that cloning has zero overhead.

At most one carry-over may coexist with the retained batch. It adds at most one
record's `batch.max_bytes` logical projection, exact bounded attributes/ranges,
one progress delta, and one post-frame decoder/framer resume state. Aggregate
admission includes this simultaneous retained-batch-plus-carry-over peak; it
does not assume the carry-over shares mutable source or batch storage.

### Checkpoint recovery

Durable artifact maxima are derived from:

- `checkpoint.compact_after_bytes`;
- `limits.max_tracked_files`; and
- `identity.fingerprint_bytes`.

The store derives:

- maximum snapshot bytes;
- maximum WAL bytes;
- maximum transaction bytes; and
- maximum snapshot-record bytes.

The derived maximum transaction bytes must be at least the
[checkpoint-format](filelog-checkpoint-format.md#maximum-encoded-lengths-summary)
`MAX_PROGRESS_TX_FRAME_BYTES` so every valid maximum-size Ack/drop progress
transaction is encodable and recoverable.

Each artifact is bounded by a fixed 1 GiB ceiling. The conservative recovery working
set is the larger of:

```text
snapshot phase =
  4 * maximum snapshot bytes

WAL phase =
  3 * maximum snapshot bytes
  + maximum WAL bytes
  + 4 * maximum transaction bytes
```

The combined recovery model also must not exceed 1 GiB. Recovery validates declared
counts and lengths before allocation, drops the snapshot input buffer before loading
the WAL, and decodes/applies one transaction at a time.

The maximum size written by a store is compatible with the maximum size the same
configuration reads. An append or compaction that would exceed the bound is refused
before in-memory authoritative state advances.

### Aggregate admission

Phase 1 admission combines candidate/identity, reader, framer, batch,
checkpoint, regex, decoder, channel, lease, Arrow, and fixed worker state into one
coherent admission decision without double-counting shared terms.

Phase 1 requires this integrated model and representative measurement before claiming
a complete per-instance RSS ceiling.

## Telemetry and health events

### Cardinality contract

The proposed metric-set scope is `receiver.filelog`; it is not an approved stable
public instrument name.

Metric labels are bounded and use fixed dimensions such as reason, policy, result, or
rotation type. Paths, `file_id`, runtime locators, fragment IDs, checkpoint IDs supplied
by users, and raw error strings are never metric labels.

Detailed identity and path context appears only in bounded, sampled, and rate-limited
health events. Event payload values are bounded. Repeated failures cannot create an
unbounded event queue or log flood.

### Metric inventory

| Semantic handle, not a stable instrument name | Required semantics |
| --- | --- |
| `records_emitted` | OTAP records/fragments emitted |
| `source_bytes_read` | Original source bytes read |
| `body_bytes_emitted` | Logical body bytes emitted |
| `batches_emitted` | Initial batch sends |
| `batches_acked` | Matching terminal Acks |
| `batches_nacked` | Matching Nacks by bounded class |
| `batches_resent` | Retry sends |
| `retry_attempts` | Resend attempts |
| `retry_exhausted` | Retained batches exhausting budget |
| `stale_completions` | Ignored stale Ack/Nack completions |
| `ack_membership_failures` | Required publication rejected for zero/unready/unsafe Ack membership |
| `records_dropped_on_nack` | Explicit `drop_and_continue` loss |
| `checkpoint_transactions` | Logical WAL transactions by operation class |
| `checkpoint_persists` | Successful append/sync/publication operations |
| `checkpoint_failures` | Store failures by bounded operation class |
| `checkpoint_duration` | Append/sync latency |
| `checkpoint_sync_delay` | Ack-to-required-sync delay |
| `wal_bytes` | Current bounded WAL size |
| `wal_transactions` | Current transaction count |
| `compaction_duration` | Synchronous compaction latency |
| `recovery_duration` | Bounded namespace recovery latency |
| `files_discovered` | Stable candidate observations |
| `files_tracked` | Current durable tracked population |
| `files_open` | Current resident tail handles |
| `files_quarantined` | Current and entered quarantine by bounded reason |
| `identity_resets` | New identity due to mismatch/ambiguity |
| `runtime_lease_wait` | Time waiting for local locator ownership |
| `local_locator_conflicts` | Rejected duplicate local ownership by bounded result |
| `namespace_lock_wait` | Time waiting for checkpoint namespace |
| `rotations` | Move/create finalizations by bounded outcome |
| `copytruncate_detected` | Observable truncation detections |
| `descriptor_evictions` | Completed resident-handle evictions |
| `descriptor_reopen_failures` | Revalidation/reopen failures by reason |
| `descriptor_budget_warnings` | Startup descriptor-budget warnings by bounded result |
| `carry_over_records` | Records retained across one prior in-flight batch because final projected size did not fit |
| `eof_reprobes` | Scheduled EOF source probes |
| `environmental_reprobes` | Bounded transient retries by operation and error class |
| `read_paused_time` | Source-read pause due to backpressure/in-flight batch |
| `read_turns` | Source turns and bytes by bounded outcome |
| `discovery_scan_duration` | Reconciliation duration |
| `discovery_incomplete` | Incomplete passes by bounded reason |
| `candidate_pending_depth` | Current retained pending population |
| `candidate_oldest_age` | Age of oldest retained pending candidate |
| `candidate_overflow` | Matches not retained due to capacity |
| `candidate_overflow_passes` | Reconciliation passes with overflow |
| `candidate_admission_stall` | Time since admission while overflow persists |
| `pattern_not_matched` | Start-mode fallback physical lines |
| `decode_errors` | Malformed units by configured policy and result |
| `decode_replacements` | Replacement units emitted |
| `records_split` | Logical records producing fragments |
| `fragments_emitted` | Split fragments |
| `records_truncated` | Truncated logical records |
| `truncated_source_bytes` | Source bytes intentionally discarded |
| `multiline_flushes` | Flushes by end/start, line, byte, timeout, rotation, or drain reason |
| `partial_bytes_pending_drain` | Recoverable uncommitted partial bytes at drain |
| `terminal_unterminated_records` | D17 records emitted because confirmed permanent EOF established the terminal boundary |
| `framing_profile_incompatible` | Per-file stored profile version/digest mismatch |
| `advisory_path_truncated` | Native advisory paths represented by bounded suffix and full-path digest |
| `quarantine_resets` | Administrative action by bounded action |
| `tracked_capacity_saturation` | Time or events at tracked-file capacity |
| `open_capacity_saturation` | Time or events at resident-handle capacity |

The semantic inventory and cardinality constraints above are normative. Exact
instrument names, kinds, units, aggregation, and stability guarantees require a
separate telemetry review. The descriptive handles in this table are not a public
compatibility contract, and an implementation does not claim complete observability
until that review defines the instrument details.

### Health-event inventory

Rate-limited operator-visible events cover:

- invalid or rejected configuration;
- include patterns apparently covering engine output;
- incomplete reconciliation and its reason;
- candidate overflow and prolonged admission stall;
- descriptor-budget incompatibility and environmental open/reprobe backoff;
- discovery traversal/evidence failure;
- exclusion revocation;
- identity ambiguity, mismatch, and reset policy;
- framing-profile incompatibility;
- local locator ownership conflict;
- runtime lease contention or integrity failure;
- namespace ownership wait and timeout;
- file open, read, permission, and sharing failures;
- descriptor-evicted identity becoming impossible to reopen after removal;
- decode preserve/replace/fail outcomes;
- pattern fallback;
- multiline line, byte, and timeout flush;
- split, truncate, and discarded-byte outcomes;
- carry-over retention and shutdown without carry-over progress;
- advisory-path truncation;
- pending partial bytes at drain;
- D17 terminal-unterminated emission or decode-fail quarantine;
- move/create finalization and late-write limitation;
- copytruncate detection and selected policy;
- durable quarantine and explicit reset action;
- quarantine administration unavailable in an initial delivery;
- checkpoint append, sync, recovery, corruption, compaction, publication, and cleanup
  failures;
- retry, permanent Nack, exhaustion, and explicit drop;
- zero or unsafe required Ack membership;
- stale completion;
- drain timeout and forced shutdown; and
- worker failure or blocked-worker lifecycle detachment.

## Validation matrix

The following tests are required evidence for conformance. Test names and
benchmark hardware are intentionally not normative. Linux cases gate the
initial Phase 1 release. macOS and Windows cases gate enabling those platforms,
while their semantic and format definitions remain normative from version 1.

| Area | Scenario | Required observation |
| --- | --- | --- |
| Configuration | Empty includes | Rejected before startup |
| Configuration | Unknown field | Rejected |
| Configuration | Pattern-count/length/aggregate boundary | Exact bound accepted; next value rejected |
| Configuration | Direct checkpoint self-include | Rejected |
| Configuration | Reconciliation interval outside `100ms..=24h` | Rejected |
| Configuration | Reconciliation jitter outside `0..=25` or jitter arithmetic overflow | Rejected |
| Configuration | EOF reprobe interval outside `10ms..=1h` | Rejected |
| Configuration | Either interval causing clock overflow | Rejected or terminal before wrapped deadline |
| Configuration | Nonzero `force_flush_period >= rotation.rotate_wait` | Rejected before startup |
| Configuration | Both multiline patterns | Rejected |
| Configuration | Unsupported regex construct/profile | Rejected |
| Configuration | Framing bound exactly minimum | Accepted |
| Configuration | Framing bound below encoding minimum | Rejected |
| Configuration | Record plus attributes equals batch bound | Accepted |
| Configuration | Record plus attributes exceeds batch bound | Rejected |
| Configuration | `max_tracked_files > u32::MAX` | Rejected |
| Configuration | Memory formula equals ceiling | Accepted if representable |
| Configuration | Formula exceeds/overflows ceiling | Rejected with actionable knobs |
| Discovery | New growing file | Admitted without waiting for close |
| Discovery | Exclude overlaps include | Excluded |
| Discovery | Lexical alias maps to excluded target | Excluded |
| Discovery | `follow_symlinks: false` final symlink | Not admitted |
| Discovery | `follow_symlinks: true` allowed target | Admitted once |
| Discovery | FIFO, socket, directory, or device candidate | Rejected without blocking the discovery thread |
| Discovery | Path target substituted between check and open | Opened handle fails policy/type/identity validation; not admitted |
| Discovery | Symlink directory cycle | Bounded, incomplete pass |
| Discovery | Hardlink/overlapping glob | One locator candidate/reader |
| Discovery | Traversal error | No false removal |
| Discovery | Evidence changes between probes | Incomplete; no unsafe inheritance |
| Discovery | Candidate overflow | Bounded state and later varying opportunity |
| Discovery | Stable-order overflow | No permanent traversal-order exclusion mechanism |
| Discovery | Full tracked table with reconnect | Candidate reaches identity matching |
| Discovery | Full tracked table with new file | Bounded deferral |
| Discovery | Cancellation during large tree | Observed between bounded units |
| Discovery | Kernel-blocked operation | Async task does not synchronously forever-join |
| Discovery | Old unrelated candidate exactly at age threshold | Eligible |
| Discovery | Old unrelated candidate beyond age threshold | Present but not admitted; no checkpoint record |
| Discovery | Ignored candidate receives later append | Later pass may admit it |
| Discovery | Recovered/quarantined/rotation-replacement file is old | Age filter does not hide or exclude it |
| Discovery | Wall clock rolls back or mtime is future | Age zero; no exclusion |
| Identity | Two live equal fingerprints | Distinct `file_id`s |
| Identity | Short or full equal fingerprint at changed locator | New identity; mismatch policy chooses anchor; no progress transfer |
| Identity | Copied file on another device/volume with equal fingerprint | New identity; default beginning permits duplicates rather than skipping copied bytes |
| Identity | Exact locator and valid prefix | Recovery permitted |
| Identity | Exact locator and mismatched prefix | Atomic superseded-record removal/new registration; no old offset inheritance or duplicate Active locator claim |
| Identity | Committed offset beyond size | No old offset inheritance |
| Identity | Growing evidence | Same `file_id`, evidence extends |
| Identity | Stored framing-profile mismatch | Per-file fail closed; no new identity or `skip_to_end` |
| Identity | Locator/inode reused after `RotatedFinalized` | New identity; finalized state never reconnects |
| Identity | `start_at: beginning` new file | Durable offset zero before read |
| Identity | `start_at: end` new file | Durable handle-derived EOF before read |
| Identity | Existing checkpoint with different `start_at` | Checkpoint wins |
| Identity | Same-locator quarantine restart | Quarantine reconnects unchanged |
| Identity | Replacement at quarantined path | New identity, no inherited quarantine |
| Identity | Duplicate runtime lease request | One reader; bounded wait/failure |
| Identity | Descriptor closes temporarily | Lease remains held |
| Reader | More tracked than open files | Resident handles remain bounded |
| Reader | Receiver FD budget exceeds process soft limit | Startup rejected before source open |
| Reader | Receiver FD budget exceeds warning threshold | Bounded startup warning; no aggregate process-ownership claim |
| Reader | `EMFILE` or `ENFILE` | Bounded backoff/admission pause; no quarantine |
| Reader | Hot and cold ready files | Round-robin bounded turns |
| Reader | Many EOF files | Deadline reprobe, no busy loop |
| Reader | Turn hits source-byte bound | Stops and yields |
| Reader | Descriptor eviction with partial state | Discard and rewind to applied progress |
| Reader | Reopen same identity | Exact revalidation and deterministic reread |
| Reader | Reopen mismatch | No read under old identity |
| Reader | Removed resident identity | Descriptor pinned through finalization |
| Reader | Removed evicted identity | Honest reopen/late-write failure |
| Decoding | UTF-8/ASCII/UTF-16LE/UTF-16BE/raw | Expected body and source offsets |
| Decoding | Matching initial BOM | Stripped body; frame owns BOM |
| Decoding | Conflicting BOM | Configured decode-error policy |
| Decoding | BOM away from stream start | Ordinary content/error semantics |
| Decoding | NUL in every encoding | Preserved as data |
| Decoding | Split source unit across reads | No premature emit/commit |
| Decoding | Malformed unit after complete record | Complete record emitted first |
| Decoding | Preserve-raw malformed unsplit | Exact framed source bytes |
| Decoding | Preserve-raw later malformed split | All fragments exact bytes |
| Decoding | Replace | Ordered replacement and count |
| Decoding | Fail | Durable quarantine without overtaking prior records |
| Framing | `a\n` | Body `a`; frame advances through LF |
| Framing | `a\r\n` | Body retains CR |
| Framing | Empty line | Empty body; LF progress |
| Framing | Multiline | Internal LFs retained |
| Framing | Start-pattern fallback | Newline record and bounded counter |
| Framing | Next start match | Previous record emits; match starts next |
| Framing | End match | Matching line included |
| Framing | Body exactly at line/record bound | Fits |
| Framing | Oversize physical line after multiline buffer | Earlier buffer emits first |
| Framing | Multiline overflow trigger | Deterministic boundary at trigger-line LF |
| Framing | Split crosses Ack boundary | Durable continuation reproduces next fragment |
| Framing | Resume continuation at empty EOF | No fabricated fragment |
| Framing | Truncate malformed discarded tail | Malformation counted |
| Framing | Truncate tail malformed under decode `fail` | Same-record prefix not emitted; quarantine before malformed unit |
| Framing | EOF then new byte before deadline | Deadline canceled before framing |
| Framing | True idle EOF deadline | Timeout record and clean resume after Ack |
| Framing | Idle deadline with incomplete UTF-8/UTF-16/BOM under preserve_raw | Complete pending source range emitted byte-for-byte |
| Framing | Idle deadline with incomplete UTF-8/UTF-16/BOM under replace | Decoded prefix plus one exact-range replacement |
| Framing | Idle deadline with incomplete unit under fail | Earlier records resolve, then quarantine without malformed-unit progress |
| Framing | Permanent EOF, framing-eligible incomplete unit under preserve/replace | Exact bytes or one replacement emitted only inside the eligible record |
| Framing | Confirmed permanent EOF with a nonempty pending frame | Emit with terminal-unterminated evidence; progress remains Ack-gated |
| Framing | Confirmed permanent EOF with malformed input under decode-fail | Durable quarantine; no progress over the malformed source unit |
| Framing | Permanent EOF under decode `fail` | Durable quarantine; prior complete records resolve first |
| Framing | Live EOF with incomplete unit or record | No terminal policy and no fabricated completion |
| Framing | Drain with partial bytes | Pending, uncommitted |
| OTAP | Raw and decoded records | Bytes/text body respectively |
| OTAP | Record ready time | `observed_time_unix_nano` set |
| OTAP | Timestamp/severity/JSON | Not interpreted by receiver |
| OTAP | `u16` record boundary | 65,535 accepted; overflow impossible |
| OTAP | Untruncated losslessly textual path | Registered `log.file.path`/`name` emitted |
| OTAP | Truncated or non-text native path | No misleading registered path; bounded project-native evidence and truncation marker |
| OTAP | Split fragment | Exact project-experimental ID/index/finality/body/frame attributes |
| Batch | Final projected size crosses byte bound after framing | Exact bounded record retained as sole carry-over; no reread |
| Batch | Multiple records same file | Contiguous delta coalesces |
| Batch | Delta gap or epoch mismatch | Rejected |
| Batch | 4,096 distinct file deltas | One atomic progress transaction remains encodable |
| Batch | Scheduler selects a 4,097th distinct file | Current batch seals before any source read for that file |
| Batch | Copytruncate occurs while prior batch waits | Already-framed carry-over remains exact and seeds next batch without source reread |
| Batch | Direct shutdown with carry-over | Memory released without progress; restart depends on surviving source |
| Ack | Matching current attempt | Atomic durable delta application |
| Ack topology | Required broadcast with automatic propagation and `all` aggregation | One aggregate Ack only after every required eligible subscriber Acks |
| Ack topology | Zero ready required subscribers | Bounded backpressure or explicit non-success; never Ack |
| Ack topology | Subscriber becomes ready after membership snapshot | Not added retroactively to that attempt |
| Ack topology | Required subscriber Nack or disappearance | One aggregate Nack under the engine topic contract |
| Ack topology | Missing Ack declaration, readiness, automatic propagation, or all-required aggregation | Graph rejected by engine topology validation |
| Ack topology | Retry after partial subscriber success | Same publication may redeliver to the previously successful subscriber |
| Ack | Duplicate/late/old attempt | Ignored and counted |
| Ack | Prior file epoch | Cannot advance replacement |
| Ack | Delta set over transaction operation bound | Entire Ack rejected before any advance |
| Transition ordering | D17 terminal emission with existing file delta | Stop, seal, resolve old-state delta, then emit and Ack-gate the terminal frame |
| Transition ordering | Decode-fail quarantine with existing file delta | Stop, seal, resolve old-state delta, apply progress, then quarantine |
| Transition ordering | Truncation with retained old epoch | Current attempt remains valid until terminal; reset follows progress |
| Transition ordering | Exclusion with existing delta | Resolve delta, then revoke while preserving `Active` checkpoint state |
| Transition ordering | Administrative action while receiver owns delta | Exclusive tool cannot proceed until receiver releases ownership with no delta |
| Nack | Retryable | Same retained batch, bounded backoff, no reread |
| Nack | Permanent default | Terminal without progress |
| Nack | `drop_and_continue` | Explicit loss and durable atomic advance |
| Nack | Exhaustion | Configured terminal policy |
| Checkpoint | Crash before registration sync | File was never eligible to read |
| Checkpoint | Crash after send before Ack | Reconstruct and duplicate if bytes survive |
| Checkpoint | Crash after Ack before delayed sync | Duplicate window only |
| Checkpoint | Torn final transaction | Only exact format-defined tail discarded |
| Checkpoint | Incomplete fixed transaction header | Torn tail |
| Checkpoint | Valid fixed header with incomplete body | Torn tail |
| Checkpoint | Upward/downward corrupted body length | Header redundancy/CRC corruption; never torn |
| Checkpoint | Complete final bad header/frame checksum | Fail closed |
| Checkpoint | Corruption before tail | Fail closed |
| Checkpoint | Unknown version/operation | Fail closed |
| Checkpoint | Snapshot unreachable lifecycle/epoch/resume/path state | `InvalidSnapshotState` before WAL replay |
| Checkpoint | Snapshot or WAL namespace digest differs from selected ID or its peer | Distinct namespace-mismatch error before applying records |
| Checkpoint | Valid `CURRENT` names missing generation file | Distinct missing-authoritative-generation error; no fallback |
| Checkpoint | Valid `CURRENT` names unreadable or incomplete authoritative generation | Distinct fail-closed recovery error; no fallback |
| Checkpoint | Genuinely absent namespace | Exact first-generation publish order and sync before read |
| Checkpoint | Artifacts exist without valid `CURRENT` | Repair only recognized interrupted first publication; otherwise fail closed |
| Checkpoint | WAL append writes no bytes | Retry from known boundary |
| Checkpoint | WAL append is partial | Validate, truncate/sync exact torn suffix, then retry |
| Checkpoint | WAL append result is ambiguous | Accept complete valid expected sequence, repair exact torn suffix, otherwise fail |
| Checkpoint | WAL append succeeds and sync fails | Validate transaction and retry sync without duplicate append |
| Checkpoint | Compaction fault before publication | Previous generation authoritative |
| Checkpoint | Cleanup interruption | Resumable, bounded artifacts |
| Checkpoint | Cleanup sees current generation among retired candidates | Files named by `CURRENT` are retained |
| Checkpoint | Retention age without runtime absence | Not removed |
| Checkpoint | Quarantined retention candidate | Not removed |
| Checkpoint | Administrative removal wrong namespace | Fail closed |
| Checkpoint | Administrative removal wrong namespace and absent `file_id` | Namespace mismatch before absent idempotency |
| Checkpoint | Quarantine administration available | Exclusive, audited inspect/reset/remove path operates without manual byte edits |
| Checkpoint | Build can quarantine but has no administration mechanism | Phase 1 conformance rejected |
| Checkpoint | `keep_failed` exact stored resume/epoch/offset | Audit operation appended; operational state byte-identical |
| Checkpoint | `keep_failed` attempts resume/epoch/offset change | Fail closed with no state change |
| Rotation | POSIX rename/create | Old and replacement read independently |
| Rotation | Replacement receives bytes before first reconciliation | Recognized replacement starts at zero regardless of `start_at` |
| Rotation | POSIX unlink with resident descriptor | Old reads through wait/finalization |
| Rotation | Windows compatible rename/delete-pending | Old handle continues |
| Rotation | Windows temporary incompatible sharing | Bounded environmental backoff, no quarantine or skipped progress |
| Rotation | Late write before wait expiry | Wait resets; bytes read |
| Rotation | Late write after finalization | Documented possible miss |
| Rotation | Final record remains in open or retained batch | Descriptor stays pinned; no finalizing transaction yet |
| Rotation | Matching Ack reaches final source frontier | Ack transaction may finalize; sync precedes descriptor release |
| Rotation | Zero-delta finalization | Permitted only after all finalization preconditions; sync precedes release |
| Rotation | Retryable Nack | Batch and descriptor retained; no finalization |
| Rotation | Permanent Nack under `fail` | Receiver terminal; identity remains unfinalized |
| Rotation | `drop_and_continue` final delta | Explicit-loss transaction applies and syncs before finalization/release |
| Rotation | Drain with unresolved rotated source | No finalization; drain reports failure/timeout without progress |
| Rotation | Pinned descriptors consume all open slots | Bounded backpressure/admission refusal; no deadline-based noncapture |
| Rotation | D17 pending terminal bytes under preserve/replace | Marked terminal emission resolves before `RotatedFinalized` |
| Rotation | D17 malformed terminal bytes under decode-fail | Quarantine transition, never `RotatedFinalized` |
| Truncation | Size below committed offset | Detected |
| Truncation | Fingerprint mismatch | Detected |
| Truncation | Truncate/regrow between probes | No lossless claim |
| Truncation | `fail` | Durable quarantine |
| Truncation | `read_new` | Durable epoch reset before new read |
| Backpressure | Downstream full | Reads pause; memory stays bounded |
| Backpressure | Drain arrives during blocked send | Control interrupts send |
| Lifecycle | Normal drain | Ack/persist/sync/release/notify |
| Lifecycle | Drain timeout with unacked batch | No progress; replay possible |
| Lifecycle | Clean drain receives no Shutdown | Cleanup still completes |
| Lifecycle | Direct Shutdown | Immediate forced path handled |
| Lifecycle | Worker blocked in kernel | Engine wait bounded; no false thread-interruption claim |
| Failure | Per-file read error | Other eligible files continue |
| Failure | `EAGAIN`, temporary permission/sharing, retryable I/O, or mount outage | Bounded environmental reprobe; no durable quarantine |
| Failure | Source-side `ENOSPC` | Bounded environmental reprobe; no durable quarantine |
| Failure | Checkpoint-side `ENOSPC` | Receiver-level store failure budget, then terminal |
| Failure | Store failure | Receiver-wide bounded retry then terminal |
| Telemetry | Many unique paths/file IDs | No metric-cardinality growth |
| Telemetry | Repeated identical failure | Event sampling/rate limit bounds output |
| Platform | Unix directory-sync fault | Publication guarantee validated |
| Platform | Unix link following enabled/disabled | Follow policy selects open primitive; handle must be regular |
| Platform | Windows reparse following enabled/disabled | `OPEN_REPARSE_POINT` only on non-following path; handle must be regular |
| Platform | Windows directory-sync absence | Limitation reported; no equal power-loss claim |
| Platform | Long Windows path/UNC | Publication and advisory path remain valid |

## Normative examples

### Example 1: LF and CR

Source bytes:

```text
61 0d 0a
```

The text body is `a\r`. The body range owns bytes 0 through 2. The frame range owns
bytes 0 through 3. Acked progress advances to 3.

### Example 2: Empty line

Source is one LF at offset 40. The receiver emits an empty text body with empty body
range `[40, 40)` and frame range `[40, 41)`. Matching Ack can advance to 41.

### Example 3: Initial BOM

A UTF-8 stream begins with a matching BOM followed by `x\n`. The body is `x`. The body
range begins after the BOM. The frame range begins at source offset zero and ends after
the LF. Reopen from the committed end does not probe for another BOM.

### Example 4: Internal multiline LF

Physical lines `BEGIN\n`, `value\n`, and `END\n` use end pattern `^END$`. The body is:

```text
BEGIN
value
END
```

Internal LFs remain. The final LF is omitted from the body and included in progress.

### Example 5: Start-pattern fallback

Before the first `^START` match, `noise\n` emits as a newline-framed record. It is not
held indefinitely. The fallback counter increments.

### Example 6: Next start

With start pattern `^20\d\d-`, line `2026-a\n` begins a buffer. `detail\n` joins it.
Line `2026-b\n` completes the first record before that line and begins the second.

### Example 7: Exact bound

If a decoded body is exactly `max_record_bytes`, it fits. The receiver does not split or
truncate it merely because equality was reached.

### Example 8: Deterministic multiline overflow

A buffered multiline record has 90 bytes and the bound is 100. The next complete
physical line has 20 body bytes and LF. Including it would exceed the bound. The
deterministic record ends at that line's LF. Split reconstructs that finite record;
truncate discards only through that LF. Both restart in initial multiline state.

### Example 9: Preserve-raw split

An early clean UTF-8 prefix fills a fragment. A later source unit is malformed before
the bound-terminated record ends. Because preserve_raw was configured, the early
fragment was emitted as exact source bytes prospectively. Every fragment concatenates
to the exact body source bytes.

### Example 10: Later failure ordering

Source contains `ok\n` followed by malformed UTF-8 under `fail`. `ok` is made
emit-ready first. The receiver stops reads and resolves `ok` through terminal
Ack/Nack policy before quarantining at the malformed unit. The failure never
suppresses or overtakes the earlier complete record or invalidates its current
attempt.

### Example 11: EOF-gated timeout cancellation

A partial record reaches EOF and arms a 500 ms deadline. At 400 ms a new source byte is
read. The deadline is canceled before that byte enters framing. A timeout cannot emit
the old prefix separately at 500 ms.

### Example 12: Continuation at idle EOF

A nonfinal fragment was Acked with continuation index 2. After restart, the file is
still exactly at the committed source boundary and EOF. The receiver emits nothing. It
does not create an empty final fragment.

### Example 13: Descriptor eviction

A reader has committed offset 100 and speculative decoded state through 140. Its
descriptor is evicted. The speculative state is discarded and the logical reader
rewinds to 100. Reopen validates identity and rereads source bytes from 100.

### Example 14: Equal fingerprints

Two live empty files have distinct locators and equal zero-length provisional
fingerprints. They receive different `file_id`s. Neither inherits an old record through
fingerprint-only matching.

### Example 15: Incomplete discovery

A directory traversal fails after observing one of two previously tracked files. The
pass is incomplete. The unseen file is not removed, and observed fingerprints are not
treated as complete-population unique.

### Example 16: Capacity reconnect

The durable table is full. A candidate has an exact locator matching an existing record.
It still reaches identity resolution and reconnects without consuming a new tracked
slot.

### Example 17: Overflow opportunity

Pending capacity is full and later candidates are unretained. The next reconciliation
changes the bounded selection opportunity. A previously overflowed stable candidate can
be selected. No finite admission deadline is promised.

### Example 18: Move/create

`app.log` locator A is renamed to `app.log.1`; a new `app.log` locator B appears.
Resident A remains pinned and is read through EOF plus `rotate_wait`. B receives its own
identity at offset zero and never inherits A's offset, even when
`start_at: end` is configured.

### Example 19: Evicted rotation

A's descriptor was evicted before unlink. Discovery proves disappearance, but the
unlinked identity cannot be reopened by path. The receiver reports the late-write
capture limitation and does not pretend A reached a lossless EOF.

### Example 20: Copytruncate gap

The application appends bytes after a rotation tool copies the file but before the tool
truncates it. The receiver does not observe those bytes before destruction. No
checkpoint rule can recover them. The receiver reports detectable truncation if later
evidence exposes it, but does not claim capture.

### Example 21: Truncate epoch guard

Batch attempt 1 contains old epoch 7 when truncation is detected. The receiver
stops reads but does not increment the epoch. Attempt 1 remains current until
its aggregate Ack/Nack policy is terminal and any authorized old-stream
progress is applied. Only then does `read_new` persist epoch 8 and offset zero.
A duplicate completion for terminal attempt 1 arriving afterward is stale and
cannot advance epoch 8.

### Example 22: Retry without reread

Batch 10 attempt 1 receives retryable Nack. The receiver waits bounded backoff and sends
the retained batch as attempt 2. It does not reopen or reread any file. A late Ack for
attempt 1 is ignored.

### Example 23: Atomic distinct-delta bound

An open batch already carries deltas for 4,096 distinct files. A record from a
4,097th file is ready. Distinct-delta preflight seals the existing batch
before a source turn or buffered framing resumes, so no record from the
4,097th file is framed or discarded. The existing batch remains one atomic
progress transaction. Ack preflight would reject the entire transaction rather
than partially apply it if this construction invariant were violated.

### Example 24: Crash duplicate

Committed offset is 100. The receiver emits bytes through 200 and downstream accepts
them. The process crashes before progress is synced. Restart resumes at 100 if source
bytes survive and can re-emit them. This is a duplicate window, not authorization to
skip to 200.

### Example 25: Quarantine replacement

Locator A at `app.log` is quarantined. A is removed and locator B appears at the same
path. B never inherits A's quarantine or offset. A's durable quarantine remains for
administration; B is a new candidate.

### Example 26: Retention veto

A durable active record is older than the retention interval but still has a runtime
lease or appears in the retained batch. It is not eligible for removal. Age alone
cannot authorize deletion.

### Example 27: Drain under backpressure

A downstream send is blocked. `DrainIngress` arrives. Biased control cancels the send
wait, stops reads, and applies bounded drain behavior. If the retained batch remains
unacked at deadline, its offsets do not advance.

### Example 28: Direct Shutdown

The engine sends `Shutdown` without a prior drain. The receiver handles it immediately,
cancels workers and retry waits, and leaves all unacknowledged progress unchanged.

### Example 29: D17 terminal emission

A rotated UTF-8 file ends with a nonempty unterminated newline record. Idle
flush is disabled. After the same handle remains at EOF through `rotate_wait`,
D17 makes the pending frame terminally eligible. The receiver emits it with
terminal-unterminated evidence and advances only after aggregate Ack. It never
reports the frame as LF-completed. If malformed input encounters
`on_decode_error: fail`, the identity is quarantined at its last applied offset.

### Example 30: Aggregate broadcast completion

Batch 12 attempt 1 is published to required subscribers A and B. A Acks and B
Nacks. The engine returns one aggregate Nack; filelog retains and retries the
batch. Attempt 2 may reach A again even though A previously succeeded. Only an
aggregate Ack from an attempt on which all required eligible subscribers Ack
can authorize progress.

### Example 31: UTF-8 idle flush with an incomplete scalar

At stream offset zero, source bytes are:

```text
61 62 63 e2
```

With `force_flush_period: 500ms`, expiry establishes frame range `[0, 4)`.
Under `preserve_raw`, the body is the exact four bytes and body range `[0, 4)`.
Under `replace`, the body is text `abc\u{fffd}`; the replacement owns source
range `[3, 4)`, while body and frame ranges are `[0, 4)`. Under `fail`, no part
of this logical record emits and quarantine preserves committed offset zero.
Any emitted variant advances to 4 only after aggregate Ack and checkpoint
application.

With `force_flush_period: 0s`, mere live EOF emits nothing. At confirmed
permanent rotation EOF, D17 establishes terminal eligibility. `preserve_raw`
emits the four exact bytes with terminal-unterminated and malformed-tail
evidence, advancing to 4 only after aggregate Ack and checkpoint application.

### Example 32: UTF-16 idle flush with an incomplete surrogate pair

UTF-16LE bytes:

```text
61 00 62 00 00 d8
```

contain `ab` followed by an unmatched high surrogate in `[4, 6)`. After the
enabled idle deadline, `preserve_raw` emits all six exact source bytes;
`replace` emits `ab\u{fffd}` with the replacement owning `[4, 6)`; and `fail`
quarantines without advancing through the surrogate. The complete eligible
frame range is `[0, 6)`.

### Example 33: Idle flush with an unresolved BOM probe

A UTF-8 stream begins with only:

```text
ef bb
```

After a 500 ms enabled idle deadline, `preserve_raw` emits the exact two bytes
and `replace` emits one replacement owning `[0, 2)`. `fail` quarantines at
offset zero. With idle flush disabled, neither decode policy nor live EOF
creates a record. Confirmed permanent rotation EOF creates a terminally
eligible frame; `preserve_raw` or `replace` emits it with
terminal-unterminated evidence, while `fail` quarantines without advancing.

### Example 34: Decode fail outranks oversize truncate

With UTF-8, `max_line_bytes: 4`, `max_record_bytes: 4`,
`max_log_size_behavior: truncate`, and `on_decode_error: fail`, source is:

```text
61 62 63 64 65 e2 0a
```

The prospective bounded prefix is `[0, 4)` (`abcd`), the valid discarded tail
is `[4, 5)` (`e`), the malformed unit is `[5, 6)` (`e2`), and terminal LF is
`[6, 7)`. The receiver scans through the deterministic frame boundary before
making the prefix emit-ready. Decode fail suppresses this same-record prefix
and quarantines at committed offset zero; no body or frame range is committed.
Under a non-failing decode policy, the truncated body range would be `[0, 4)`
and its complete frame range `[0, 7)`.

### Example 35: Carry-over survives mutable source change

A fully framed record cannot fit the remaining byte budget of a nonempty open
batch. The receiver retains its exact body, attributes, ranges, epoch,
post-frame resume, and delta as the sole carry-over, then sends the prior
batch. The source is copytruncated while that batch waits. After the prior
batch resolves, the next batch is seeded from retained carry-over bytes rather
than rereading the changed file.

### Example 36: Recognized rotation replacement ignores `start_at: end`

Locator A moves from `app.log` while A remains in rotation handling. Locator B
is created at `app.log` and receives bytes before the next reconciliation.
That ordered transition recognizes B as the replacement. B receives a new
`file_id`, clean framing, and offset zero even though `start_at: end` is
configured, so the pre-reconciliation replacement bytes remain eligible.
