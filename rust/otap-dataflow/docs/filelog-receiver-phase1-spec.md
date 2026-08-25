<!-- markdownlint-disable MD013 -->

# Filelog Receiver Phase 1 Behavioral Specification

Status: Proposed normative Phase 1 contract

This document specifies the exact runtime behavior and state transitions of the
Phase 1 OTAP filelog receiver.

The companion documents divide normative ownership as follows:

| Document | Normative ownership |
| --- | --- |
| [Filelog Receiver Design](filelog-receiver.md) | Architecture, scope, guarantees, decisions, and tradeoffs |
| This document | Exact Phase 1 runtime behavior and state transitions |
| [Filelog Receiver Checkpoint Format](filelog-checkpoint-format.md) | Exact durable byte format and replay representation |

The architecture remains authoritative for the system boundaries and accepted
compromises. This specification refines those decisions; it does not override them.
The checkpoint-format specification defines durable bytes, magic values, versions,
field widths and ordering, checksum coverage, transaction framing, and golden vectors.
This document intentionally does not duplicate those definitions.

Requirements in this behavioral specification are written as direct declarative
statements. The checkpoint-format specification states its own normative-keyword
convention.

## Terminology

| Term | Meaning |
| --- | --- |
| Aggregate completion | The single Ack or Nack the engine/topic runtime returns to filelog for one `(batch_id, attempt)` after applying required-subscriber fan-out semantics. |
| Advisory path | A bounded, reversible, platform-native representation of a last-known path. It is metadata, not identity. |
| Body source range | The half-open source-byte range whose bytes were transformed into the emitted body. |
| Candidate | An eligible opened regular file presented to identity resolution with stable evidence. |
| Candidate inventory | The bounded population and multiplicity evidence used by one reconciliation pass. |
| Clean resume | Durable framing state in which the next complete source unit begins a new logical record. |
| Applied progress | An Ack-authorized transaction successfully appended to the WAL and applied atomically to the live checkpoint state; it may be newer than the durable frontier. |
| Committed offset | The source-byte offset in live applied checkpoint state; it becomes crash-durable only when covered by the durable frontier. |
| Complete inventory | A reconciliation result that can prove both presence and absence and can establish fingerprint multiplicity over its bounded population. |
| Continuation | Durable split-fragment state containing only the original record start and next fragment index. |
| Downstream Ack | The matching aggregate completion that authorizes, but does not itself apply or sync, one retained batch's progress transaction. |
| Durable frontier | The latest applied checkpoint progress guaranteed to survive by a successful required filesystem sync; recovery may also replay a later complete valid WAL prefix that survived. |
| EOF reprobe | A scheduled handle read for an already admitted reader at temporary EOF; it does not traverse directories or reconcile discovery. |
| Exact locator | The platform runtime locator obtained from an opened file handle. |
| File epoch | A monotonic generation of one `file_id` stream. A truncate reset or administrative reset increments it. |
| `file_id` | An opaque durable logical identity used as the checkpoint key. |
| Fingerprint | Bounded raw source bytes used as matching evidence. It is not a unique identity. |
| Frame source range | The half-open source-byte range owned by a framed result, including framing bytes omitted from the body. |
| Full reconciliation | A bounded traversal that refreshes discovery evidence and may establish a complete inventory. |
| Incomplete inventory | A reconciliation result that cannot safely prove all absence or uniqueness facts. |
| Logical record | A newline-framed record, multiline record, or deterministic bound-terminated record before oversize projection. |
| Matched path | The lexical path that satisfied an include glob. |
| Open batch | The one receiver-wide mutable OTAP batch under construction. |
| Pending candidate | Retained candidate evidence waiting for an admission opportunity. |
| Physical line | Decoded content ending at decoded U+000A, or raw content ending at byte `0x0a`; the delimiter is not part of line content. |
| Quarantine | Durable per-identity failure state that prevents reading until an explicit administrative action. |
| Resolved target | The canonical target opened after alias and symlink policy is applied. |
| Retained batch | The one sealed receiver-wide batch held for Ack, Nack, or resend. |
| Runtime lease | Process-local ownership of one exact locator. It survives descriptor closure. |
| Runtime locator | POSIX device/inode or Windows volume/file-ID evidence obtained from an opened handle. |
| Source turn | One bounded read opportunity for one scheduled file. |
| Stable evidence | Two handle-based observations whose locator agrees, whose size does not shrink, and whose later fingerprint extends the earlier fingerprint. |

Unless stated otherwise, source ranges are half-open ranges `[start, end)` and all
offsets count original source bytes.

## Phase 1 scope

Phase 1 runs one receiver instance for the source pipeline. The factory rejects a
pipeline configured with more than one core for this receiver. The instance uses:

- one discovery OS thread;
- one read/checkpoint OS thread;
- one async engine task on the pipeline core;
- one bounded discovery-to-worker channel;
- one worker-to-async batch handoff slot;
- one bounded async-to-worker command channel;
- one checkpoint namespace lock; and
- process-local runtime leases for live locators.

Phase 1 provides:

- periodic local discovery;
- bounded candidate admission and identity resolution;
- durable logical identity;
- bounded source reading, decoding, and framing;
- deterministic newline and multiline behavior;
- one receiver-wide open or retained batch;
- one engine-aggregated completion per batch attempt, matching-Ack-gated
  applied progress, and a filesystem-synced durable frontier;
- bounded Nack retry;
- restart recovery when checkpoint state and required source bytes survive;
- durable quarantine and audited administrative recovery;
- ordinary move/create rotation; and
- best-effort detection and handling of copytruncate.

Phase 1 preserves ordering within each file. It defines no ordering across files.
Delivery is at least once after emission. Retry and a crash after downstream acceptance
but before durable progress can produce duplicates.

Phase 1 does not provide:

- distributed ownership or fencing;
- multiple concurrent receiver instances for one source namespace;
- durable spooling of emitted OTAP batches;
- recovery when required source bytes no longer exist;
- lossless copytruncate capture;
- proof that no late writer exists after `rotate_wait`;
- lossless ready-before-ownership live rollout;
- network-filesystem identity guarantees;
- read-once/delete, archives, compressed streams, or header-content skipping;
- built-in timestamp, JSON, severity, trace, enrichment, filtering, or routing
  semantics; or
- universal latency, throughput, RSS, or allocator guarantees.

The receiver decides which source bytes form a record. Processors decide what the
record means. Exporters decide how the record is represented and delivered.
The engine/topic runtime owns completion aggregation across required fan-out
subscribers.

## Proposed configuration contract

The schema in this section is proposed. It is not yet a compatibility promise. Unknown
fields are rejected. Each component factory validates only its own configuration. In
particular, receiver validation never reads, interprets, or validates timestamp-
processor configuration.

### Complete proposed schema

```yaml
receivers:
  filelog:
    urn: "urn:otel:receiver:filelog"
    config:
      include: ["/var/log/app/*.log"]
      exclude: []
      recursive: true
      follow_symlinks: false
      max_recursion_depth: 64
      start_at: end
      discovery:
        reconcile_interval: 5s
        reconcile_jitter_percent: 10
      reader:
        eof_reprobe_interval: 250ms
      ignore_older_than: 0s
      identity:
        fingerprint_bytes: 1000
        ignored_header_bytes: 0
        on_recovery_mismatch: beginning
      encoding: utf-8
      on_decode_error: preserve_raw
      framing:
        max_line_bytes: 1MiB
        max_record_bytes: 1MiB
        max_log_size_behavior: split
        force_flush_period: 500ms
        multiline:
          regex_profile: re2-v1
          line_start_pattern: null
          line_end_pattern: null
        max_multiline_lines: 500
      metadata:
        include_file_record_offset: false
        include_file_record_number: false
      limits:
        max_tracked_files: 10000
        max_pending_candidates: 10000
        max_open_files: 512
        max_read_bytes_per_turn: 128KiB
      batch:
        max_records: 1024
        max_bytes: 8MiB
        max_flush_period: 1s
      rotation:
        rotate_wait: 5s
        on_truncate: fail
      checkpoint:
        id: app-logs
        sync_interval: 0s
        compact_after_bytes: 64MiB
        compact_after_transactions: 10000
        retention: 7d
        ownership_timeout: 30s
        max_consecutive_failures: 5
      retry:
        max_attempts: 8
        initial_backoff: 100ms
        max_backoff: 5s
      on_nack: fail
      drain_timeout: 10s
```

### Fields, defaults, and variants

| Field | Default | Accepted values or meaning |
| --- | --- | --- |
| `include` | None | Required nonempty list of nonempty path globs |
| `exclude` | `[]` | Path globs; exclusion wins over inclusion |
| `recursive` | `true` | Whether traversal descends below each include root |
| `follow_symlinks` | `false` | Whether eligible descendant and final symlinks or reparse points are followed |
| `max_recursion_depth` | `64` | Integer in `1..=1024` |
| `start_at` | `end` | `beginning` or `end`; durable state wins |
| `discovery.reconcile_interval` | `5s` | Reviewable initial full-reconciliation delay in `100ms..=24h`; not a discovery-latency guarantee |
| `discovery.reconcile_jitter_percent` | `10` | Integer percentage in `0..=25`, independently sampled for each completed pass |
| `reader.eof_reprobe_interval` | `250ms` | Reviewable initial tail reprobe interval in `10ms..=1h`; does not trigger traversal |
| `ignore_older_than` | `0s` | Zero disables the admission-age filter |
| `identity.fingerprint_bytes` | `1000` | Raw evidence window in `16..=65535` bytes |
| `identity.ignored_header_bytes` | `0` | Prefix bytes skipped before evidence, at most `u32::MAX` |
| `identity.on_recovery_mismatch` | `beginning` | `beginning`, `skip_to_end`, or `fail` |
| `encoding` | `utf-8` | `utf-8`, `ascii`, `utf-16le`, `utf-16be`, or `raw` |
| `on_decode_error` | `preserve_raw` | `preserve_raw`, `replace`, or `fail` |
| `framing.max_line_bytes` | `1MiB` | Nonzero physical-line body bound |
| `framing.max_record_bytes` | `1MiB` | Nonzero logical-record body bound |
| `framing.max_log_size_behavior` | `split` | `split` or `truncate` |
| `framing.force_flush_period` | `500ms` | Zero disables idle partial flush |
| `framing.multiline.regex_profile` | `re2-v1` | The sole Phase 1 executable profile |
| `framing.multiline.line_start_pattern` | `null` | Optional start boundary |
| `framing.multiline.line_end_pattern` | `null` | Optional end boundary |
| `framing.max_multiline_lines` | `500` | Nonzero physical-line count bound |
| `metadata.include_file_record_offset` | `false` | Include optional source offset metadata |
| `metadata.include_file_record_number` | `false` | Include optional process-local record number metadata |
| `limits.max_tracked_files` | `10000` | Nonzero durable identity population |
| `limits.max_pending_candidates` | `10000` | Nonzero retained pending population |
| `limits.max_open_files` | `512` | Nonzero resident tail-handle population |
| `limits.max_read_bytes_per_turn` | `128KiB` | Nonzero source-byte turn bound |
| `batch.max_records` | `1024` | `1..=65535` |
| `batch.max_bytes` | `8MiB` | Nonzero logical batch-size bound |
| `batch.max_flush_period` | `1s` | Nonzero first-record batch deadline |
| `rotation.rotate_wait` | `5s` | Nonzero post-EOF inactivity interval |
| `rotation.on_truncate` | `fail` | `fail` or `read_new` |
| `checkpoint.id` | Derived logical receiver key | Optional stable namespace name of 1 to 255 ASCII bytes; set explicitly to preserve continuity across logical-key renames |
| `checkpoint.sync_interval` | `0s` | Zero syncs every Ack transaction before release; nonzero batches filesystem sync after release |
| `checkpoint.compact_after_bytes` | `64MiB` | Nonzero WAL compaction threshold |
| `checkpoint.compact_after_transactions` | `10000` | Nonzero transaction threshold |
| `checkpoint.retention` | `7d` | Zero retains eligible inactive records indefinitely |
| `checkpoint.ownership_timeout` | `30s` | Nonzero bounded ownership wait |
| `checkpoint.max_consecutive_failures` | `5` | Nonzero store-failure budget |
| `retry.max_attempts` | `8` | Nonzero total sends, including the first |
| `retry.initial_backoff` | `100ms` | Nonzero initial retry delay |
| `retry.max_backoff` | `5s` | Retry delay ceiling |
| `on_nack` | `fail` | `fail` or `drop_and_continue` |
| `drain_timeout` | `10s` | Nonzero receiver drain budget |

The reconciliation and EOF-reprobe defaults are reviewable initial values.
They are neither universal latency targets nor performance guarantees.

When `checkpoint.id` is omitted, derive it as:

```text
"auto-" + lowercase_hex(
  SHA-256(
    UTF-8("otel-arrow-filelog-checkpoint-id-v1\0") ||
    u32_be(len(UTF-8(pipeline_group_id))) || UTF-8(pipeline_group_id) ||
    u32_be(len(UTF-8(pipeline_id)))       || UTF-8(pipeline_id)       ||
    u32_be(len(UTF-8(node_id)))           || UTF-8(node_id)
  )
)
```

The three configured IDs are the complete input set. Renaming the pipeline group,
pipeline, or receiver node ID changes the default. A receiver component name, component
URN, deployment generation, CPU count, core ID, and runtime instance ID do not. Each
encoded input length must fit `u32`; otherwise validation fails.

Continuity across any logical-key rename requires an explicit ID from initial
deployment, an explicit ID equal to the current effective derived value before the
rename, or an explicit namespace migration. Startup opens only the selected ID. If it
does not exist, the receiver creates an empty namespace, emits a bounded health event,
and applies `start_at` to newly registered files. It never searches sibling namespaces.
Changing the ID therefore selects different existing state or an empty namespace and
can cause replay or intentional `start_at` exclusion.

### Configuration variants

UTF-16 input selects an explicit byte order:

```yaml
encoding: utf-16le
on_decode_error: preserve_raw
```

Raw input keeps bytes and uses byte-newline framing:

```yaml
encoding: raw
```

Start-pattern multiline treats each matching line as the start of a new record:

```yaml
framing:
  multiline:
    regex_profile: re2-v1
    line_start_pattern: '^\d{4}-\d{2}-\d{2}T'
```

End-pattern multiline includes the matching line in the completed record:

```yaml
framing:
  multiline:
    regex_profile: re2-v1
    line_end_pattern: '^END request$'
```

An operator can explicitly accept reset after detectable truncation:

```yaml
rotation:
  on_truncate: read_new
```

### Structural and relationship validation

Validation completes before threads start or a checkpoint namespace is opened.
Arithmetic used for counts, byte sums, products, deadlines, and conversions is checked.
Overflow rejects configuration; values are never wrapped or saturated.

The following relationships are enforced:

1. `include` is present and nonempty. No include or exclude entry is empty.
2. Include plus exclude contains at most 1,024 patterns.
3. Each pattern contains at most 4,096 UTF-8 bytes.
4. Aggregate include-plus-exclude pattern text contains at most 1 MiB.
5. Include and exclude globs compile once during validation.
6. An include resolving directly to the receiver checkpoint namespace is rejected.
7. `max_recursion_depth` is in `1..=1024`.
8. `discovery.reconcile_interval` is in `100ms..=24h`.
9. `discovery.reconcile_jitter_percent` is an integer in `0..=25`.
10. `reader.eof_reprobe_interval` is in `10ms..=1h`.
11. Jitter multiplication, interval conversion, and every resulting deadline are
    representable with checked arithmetic.
12. `identity.fingerprint_bytes` is in `16..=65535`.
13. `identity.ignored_header_bytes` is at most `u32::MAX`.
14. `ignored_header_bytes + fingerprint_bytes` is representable as `u64`.
15. Exactly zero or one multiline boundary pattern is configured.
16. `re2-v1` is the only accepted regex profile.
17. A multiline pattern contains at most 4,096 bytes.
18. Counted repetition bounds do not exceed 1,000.
19. Each compiled matcher has a 10 MiB program-size limit.
20. Each matcher has a 2 MiB lazy-DFA cache limit.
21. Text patterns compile for validated decoded UTF-8.
22. Raw patterns compile in non-Unicode byte mode.
23. Backreferences, look-around, Unicode properties, unsupported set operations,
    unsupported flags, and other non-RE2 constructs are rejected.
24. `framing.max_line_bytes` and `max_record_bytes` fit the target `usize`.
25. The minimum framing bound is one byte for raw and ASCII/fail.
26. The minimum is three bytes for ASCII with preserve_raw or replace.
27. The minimum is four bytes for UTF-8 and UTF-16 under every decode policy.
28. `framing.max_multiline_lines` is nonzero.
29. A nonzero `force_flush_period` resolves exactly to whole milliseconds and fits
    `u64` milliseconds.
30. `limits.max_tracked_files`, `max_pending_candidates`, `max_open_files`, and
    `max_read_bytes_per_turn` are nonzero.
31. `limits.max_open_files <= limits.max_tracked_files`.
32. Candidate-population and aggregate reader-count sums fit `usize`.
33. `batch.max_records` is in `1..=65535`.
34. `batch.max_bytes` fits `usize`.
35. The hard distinct-file progress-delta limit is 4,096 and the maximum
    Ack/drop transaction size derived from it is representable.
36. `batch.max_flush_period`, `rotation.rotate_wait`, `retry.initial_backoff`,
    `checkpoint.ownership_timeout`, and `drain_timeout` are nonzero.
37. `retry.max_attempts` is nonzero.
38. `retry.max_backoff >= retry.initial_backoff`.
39. Both checkpoint compaction thresholds and the consecutive-failure budget are
    nonzero.
40. Every UTF-8 input to the derived checkpoint-ID recipe has a length representable
    as `u32`.
41. `checkpoint.id` is nonempty after defaulting.
42. `checkpoint.id` is neither `.` nor `..`.
43. `checkpoint.id` contains only ASCII alphanumerics, `_`, `-`, and `.`.
44. Its ASCII byte length is at most 255, and those exact bytes are its namespace path
    component.
45. Both configured framing bounds plus worst-case enabled attributes fit within
    `batch.max_bytes`.
46. Identity reconciliation, framer, reader, and checkpoint recovery admission models
    are representable with checked arithmetic and remain within their fixed ceilings.

Separately, engine topology validation rejects a filelog path that requires
Ack-gated progress across broadcast subscribers unless it provides automatic
Ack propagation and all-required-subscriber aggregation. That graph-level
validation is not performed by the receiver factory.

The logical size of a record is:

```text
body bytes
+ every projected attribute-key byte
+ every projected attribute-value byte
+ 128 bytes conservative fixed record overhead
```

Configuration reserves worst-case bounded path encodings and policy-specific
attributes. The same logical size function governs runtime batch admission. This is not
an Arrow allocation or wire-size measurement.

### Configuration changes and resumable state

The checkpoint stores a versioned compatibility digest covering identity evidence and
all inputs that affect record boundaries or deterministic replay. A mismatch against
resumable state fails closed and requires an explicit migration or reset.

Changing these values is therefore not an ordinary live reload:

- fingerprint profile or evidence window;
- ignored header bytes;
- encoding;
- decode-error policy;
- multiline mode, profile, or source pattern;
- physical-line or logical-record bounds;
- oversize policy;
- multiline line limit; or
- idle flush period.

Shrinking tracked-file, fingerprint, or WAL bounds below an existing namespace's
validated durable population fails recovery closed. The receiver does not silently
truncate durable state.

## Discovery reconciliation

### Compilation and path semantics

Discovery receives validated, precompiled include and exclude matchers. It does not
recompile patterns per scan or per path.

On Unix, `/` is the separator. Backslash escapes glob metacharacters and is removed
when deriving a literal traversal prefix. On Windows, both platform path behavior and
the glob compiler treat backslash as a separator rather than as a glob escape.

Each include has:

- a configured lexical pattern;
- a fixed lexical traversal prefix;
- a canonicalized traversal root; and
- mappings needed to preserve operator-visible aliases.

Lexical matching determines whether the configured path matches. Opening and identity
evidence use the resolved target. Exclusion is evaluated against:

- the lexical matched path;
- the canonical resolved target; and
- the target mapped through each canonicalized exclusion root.

An alias therefore cannot bypass an exclusion merely because canonicalization changes
its prefix. Exclusion always wins over inclusion.

The receiver unconditionally excludes its resolved checkpoint namespace. Direct
inclusion of that namespace is a configuration error. Apparent inclusion of the
engine's own output produces a bounded warning.

### Reconciliation schedule

At most one full reconciliation pass runs at a time. After a pass completes,
the discovery worker schedules the next pass from that completion time:

```text
base_ns = discovery.reconcile_interval in nanoseconds
spread_ns = floor(base_ns * discovery.reconcile_jitter_percent / 100)
next_delay_ns is selected in [base_ns - spread_ns, base_ns + spread_ns]
```

All arithmetic is checked. The selection is independently varied per pass,
need not be persisted or cryptographically random, and is exactly `base_ns`
when jitter is zero. Jitter reduces synchronized scans; it does not create a
latency guarantee. An explicit startup pass and a bounded reconciliation
request may run earlier, but neither permits overlapping traversals.

### Symlinks, reparse points, hardlinks, and cycles

With `follow_symlinks: false`:

- an alias in the fixed include prefix is followed to reach the configured root;
- descendant directory symlinks or reparse points are not traversed;
- a final symlink or reparse point is not accepted as a regular-file candidate; and
- handle validation prevents a path-check/open substitution race.

With `follow_symlinks: true`:

- eligible links may be followed;
- resolved-target exclusions still apply;
- directory cycles are detected by runtime locator;
- the traversal stack and ancestor-locator population are bounded by
  `max_recursion_depth`; and
- a cycle or exhausted traversal bound makes the inventory incomplete.

Overlapping globs, hardlinks, aliases, and rename-visible paths that resolve to the same
live runtime locator produce one candidate identity. Path is updated as advisory
metadata. Equal fingerprints do not deduplicate distinct locators.

`follow_symlinks` controls symbolic-link and reparse-point traversal, not
hardlinks. A hardlink is another eligible name for the same opened file and is
deduplicated by runtime locator. Whether an untrusted principal can create a
hardlink to another readable file depends on filesystem, OS hardlink
protections, ownership, directory permissions, and collector privilege. The
receiver does not authenticate the provenance of a hardlinked source.

### Candidate evidence

Candidate probing must not block indefinitely. Pre-open path metadata may reject
an obviously ineligible entry but never establishes acceptance. Discovery opens
the candidate using the selected link policy and validates both type and identity
from that opened handle. FIFO, socket, device, directory, and every other
non-regular-file handle are rejected without entering identity resolution. A
path-check/open substitution that changes the selected target is not accepted.

On Unix, each probe uses a nonblocking, close-on-exec opening strategy.
When `follow_symlinks: false`, it also uses `O_NOFOLLOW`, `openat2` resolution
constraints, or an equivalent non-following primitive. When link following is
enabled, it resolves and opens according to that policy without
unconditionally applying `O_NOFOLLOW`. `fstat` or an equivalent handle query
must prove that the opened object is a regular file before any content read.

On Windows, probing uses reparse-point behavior appropriate to the selected
follow policy. `FILE_FLAG_OPEN_REPARSE_POINT` applies to the non-following path
and is not unconditional. Handle type, attributes, reparse state, volume, and
file ID are queried from the opened object; only an eligible regular file
continues.

Identity evidence is derived from the validated handle, not from pre-open path
metadata.

Candidate stability uses two bounded observations:

1. Open and observe the regular file.
2. Close the first transient probe.
3. Open and observe it again.
4. Require equal runtime locators.
5. Require the second observed size not to be smaller.
6. Require the second fingerprint evidence to extend the first.

A replacement, truncation, path-resolution change, evidence read failure, or unstable
observation makes the pass incomplete. It is not converted into false uniqueness or
absence evidence.

Discovery uses at most one transient probe handle beyond `max_open_files`. It does not
retain one probe per candidate.

Watched directories are a source trust boundary distinct from checkpoint state.
Application-writable directories can supply adversarial names, links, churn, and
contents. Least-privilege read access, nonblocking probes, bounded traversal,
buffers, records, and deterministic failure classification limit resource
impact; they do not authenticate source content.

### Complete and incomplete inventories

A complete inventory visited every relevant bounded traversal unit and produced stable
evidence for every fact needed to reconcile retained state.

The following conditions make a pass incomplete:

- traversal error;
- directory open or enumeration error;
- path-resolution error;
- symlink or reparse-point cycle;
- depth exhaustion;
- cancellation;
- unstable locator, size, or fingerprint evidence;
- reappearance of a locator still awaiting finalization;
- inability to retain required multiplicity evidence; or
- bounded-state overflow that prevents proving the complete candidate population.

Candidate overflow alone does not suppress removal when traversal still visits and
marks every retained locator. If overflow also prevents the pass from proving required
presence, absence, or multiplicity facts, the affected inventory is incomplete.

An incomplete pass:

- emits no removal based only on non-observation;
- does not evict unseen pending evidence;
- does not admit stale pending evidence as though it were current;
- disables fingerprint-only checkpoint inheritance;
- keeps valid exact-locator matching available; and
- records bounded health and telemetry evidence.

Only a later complete pass can prove disappearance or complete-population uniqueness.

### Bounded populations

| Population | Bound |
| --- | --- |
| Traversal stack and ancestor locators | Incremental state bounded by `max_recursion_depth` |
| Current directory entry and match state | One bounded traversal unit |
| Candidate transition batch | At most `max_open_files` observed/updated transitions plus bounded tracked removals |
| Discovery event channel | One bounded batch |
| Pending candidates | `max_pending_candidates` |
| Candidate identity-resolution batch | At most `max_open_files` |
| Durable tracked identities | `max_tracked_files` |
| Resident tail handles | `max_open_files` |
| Transient discovery probes | One beyond the resident-handle pool |

The scanner never materializes the full filesystem match set. It keeps generation
markers only for bounded tracked identities and retained pending candidates, while
visiting other matches incrementally.

### Admission and fairness

Retained pending candidates are ordered by oldest discovery time. Strict oldest-first
ordering applies only to that retained population.

When pending capacity is full, additional matches are counted and reported but are not
retained indefinitely. Periodic reconciliation makes stable overflow candidates
eligible again. A generation-varying selection keyed by runtime locator, or an
equivalent bounded mechanism, varies their admission opportunity across passes.

The receiver guarantees recurring opportunity, not a finite per-candidate or global
starvation bound for candidates whose arrival state could not be retained. Telemetry
reports pending depth, oldest retained age, overflow, overflowing passes, and time since
the last successful admission while overflow continues.

Identity resolution still receives a bounded candidate subset when
`max_tracked_files` is exhausted. A candidate may reconnect an existing durable record,
which consumes no new tracked slot. Only after identity resolution proves that a new
`file_id` is required may the candidate be deferred back to bounded pending state.

With retention disabled, tracked capacity can remain exhausted until configuration is
raised or an operator explicitly removes state. Quarantined records also consume
capacity and are not ordinary retention candidates.

### Reconciliation transitions

A complete pass produces one ordered transition stream:

| Transition | Preconditions | Effect |
| --- | --- | --- |
| Observed | Eligible stable locator is not retained or active | Present candidate to identity resolution |
| Updated | Retained locator remains eligible but path or mutable metadata changed | Update bounded advisory evidence |
| Excluded | Active locator now matches an exclusion | Revoke at the next complete record boundary |
| Disappeared | Complete pass proves locator no longer has an eligible name | Enter rotation/removal handling |

The implementation-facing transition must preserve the reason distinction between
exclusion revocation and disappearance. A reasonless `Removed` event is insufficient
because exclusion requires prompt revocation while disappearance can require descriptor-
dependent late-write capture and rotation finalization.

Transition order for one locator is preserved. A stale removal cannot overtake a later
observation. Reappearance before the old logical reader and lease finalize is blocked
and makes the pass incomplete. Reappearance after finalization is a fresh observation.

Cancellation is checked between bounded directory entries, path resolutions, evidence
observations, and channel handoff waits. A filesystem operation already blocked in the
kernel may not be interruptible. The async lifecycle never synchronously waits forever
for the discovery thread to join.

## Identity and local ownership

### Evidence roles

| Value | Role | Explicit non-role |
| --- | --- | --- |
| `file_id` | Opaque durable checkpoint key and future partition input | Not derived from path, locator, fingerprint, CPU, or generation |
| Runtime locator | Open-reader key, discovery dedup key, and runtime-lease key | Not permanent identity |
| Fingerprint | Guarded restart/reopen matching evidence | Not a unique key |
| Advisory path | Reversible bounded diagnostics and recovery context | Not identity |
| File epoch | Guards progress against stream reset | Not distributed fencing |

`file_id` is created from operating-system randomness, checked against the loaded
table, and durably registered before reading. It never changes because a path changes,
a fingerprint grows, or a runtime locator is refreshed.

Advisory paths retain native representation without lossy conversion. Unix stores
original native path bytes. Windows stores native UTF-16 code units reversibly. The
checkpoint-format specification owns the exact durable representation and bound.

### Matching hierarchy

Identity resolution uses this order:

1. A live in-process runtime lease wins and prevents a second reader.
2. A durable exact runtime locator is considered only with successful fingerprint-
   prefix validation and `committed_offset <= current_size`.
3. A full-window fingerprint-only match is considered only when the reconciliation
   inventory is complete.
4. Fingerprint-only inheritance requires uniqueness across the complete bounded live
   candidate population and eligible durable records.
5. Otherwise the candidate receives a new `file_id` or follows the explicit recovery-
   mismatch failure policy.

Fingerprint-only matching is disabled for short/provisional fingerprints and for an
incomplete inventory. Exact-locator matching remains available when its direct evidence
validates.

Two live locators with equal fingerprints are distinct files and receive distinct
logical identities. Cross-device or cross-volume copy/unlink is a new file. Fingerprint
equality never merges independent live streams.

Fingerprint evidence for a growing file may extend under the same `file_id`. An update
replaces only matching evidence. It never rekeys progress or changes lifecycle state.

### Ambiguity and mismatch

The following conditions never inherit an old offset:

- multiple eligible durable records share the same full fingerprint;
- multiple live candidates share it;
- the candidate has only a short fingerprint;
- the inventory is incomplete;
- an exact-locator fingerprint prefix fails;
- committed progress exceeds current size;
- evidence changed while it was observed; or
- framing/identity compatibility does not match.

The candidate becomes a new logical identity and applies
`identity.on_recovery_mismatch`:

| Policy | Initial durable progress |
| --- | --- |
| `beginning` | Offset zero, clean resume |
| `skip_to_end` | Current EOF from the validated handle, clean resume; explicit intentional loss |
| `fail` | Durable quarantine pending operator action |

The receiver records an identity-reset signal. It does not silently guess.

### `start_at` and durable state

`start_at` applies only when no durable identity is recovered.

For `beginning`, registration stores offset zero and clean framing state.

For `end`, the receiver obtains EOF from the validated opened handle and durably stores
that offset as the initial anchor. Bytes before the anchor are intentionally excluded
and need no downstream Ack. Bytes appended after the anchor use normal Ack-gated
progress. Because discovery is periodic, bytes written before the file's first
successful observation are also before this anchor. This is the intentional
initial-exclusion window selected by `start_at: end`, not accidental data loss.
It is the initial default because a newly configured tailer otherwise replays
an unbounded and operationally surprising historical backlog; operators that
require existing bytes select `beginning`.

Recovered durable state always wins over `start_at`.

### Quarantine reconnection

A quarantined record reconnects only through the same exact runtime locator with valid
same-identity evidence. It remains quarantined; general recovery-mismatch policy does
not release or replace it.

A different locator at the same path never inherits the old quarantine. It is evaluated
as a new candidate.

Quarantine evidence is immutable:

- lifecycle state does not change through ordinary metadata updates;
- the recorded quarantine locator does not change;
- reason, observed size, quarantine epoch, and quarantine time do not change; and
- only bounded last-seen and advisory-path metadata may be refreshed.

Configuration reload never releases quarantine.

### Namespace ownership and runtime leases

The receiver acquires an exclusive advisory lock for the stable checkpoint namespace
before loading or mutating it. Acquisition retries are bounded by
`checkpoint.ownership_timeout`. Timeout is terminal.

One process-local runtime lease exists for each active locator. A lease:

- is acquired before a logical reader starts;
- is bounded by configured tracked populations;
- contains no telemetry payload or durable progress;
- survives temporary descriptor closure and reopen;
- is released on finalization, exclusion revocation, normal drain, receiver failure,
  or panic unwinding; and
- fails closed on registry poisoning, corruption, or inconsistent release.

Lease-map critical sections contain only bounded map operations. No filesystem I/O,
sleep, retry, or channel wait occurs while the registry lock is held. Async pipeline
tasks do not block on that lock.

If another filelog node already owns the exact locator, the requesting node
does not start a second reader and emits a bounded, rate-limited, high-severity
`filelog.local_locator_conflict` health event with the two logical node
identities and bounded locator/path evidence. Static overlapping-glob detection
is not required: globs can overlap without resolving to the same file, and
aliases or replacement can create runtime conflicts that static text cannot
prove.

The namespace lock and runtime leases prevent overlapping local readers in one engine
process. They provide no distributed fencing. Separate processes, independent state
directories, and unreliable network-filesystem advisory locks are outside the Phase 1
ownership guarantee.

## Reader scheduling and descriptor resources

### Logical reader table

Admission creates a logical reader after identity resolution, lease acquisition, and
durable registration or recovery. The logical reader may exist without a resident file
handle.

`max_tracked_files` bounds logical identity state. `max_open_files` independently bounds
resident tail handles. Discovery may hold only the separately bounded transient probe.

The receiver-local descriptor budget is:

```text
filelog_fd_budget =
  limits.max_open_files
  + 1 transient candidate probe
  + 8 checkpoint/namespace descriptors
```

The checkpoint allowance covers the namespace directory and lock, active
snapshot/WAL, `CURRENT`, and bounded temporary publication descriptors; the
store must close intermediate handles before exceeding it. Startup rejects a
receiver-local budget above the process soft `RLIMIT_NOFILE` or platform
equivalent and emits a warning when it consumes more than 80 percent of that
limit. This is not an aggregate process admission claim: other nodes,
libraries, and concurrent startup can consume the remaining descriptors.

At runtime, `EMFILE` and `ENFILE` pause new opens and admissions, retain logical
state, and use the bounded environmental backoff defined under
[Failure containment](#failure-containment). They never directly quarantine an
identity.

### Source turns

Ready logical readers are scheduled round-robin. One source turn:

- selects one reader;
- owns the one shared source-turn buffer;
- performs at most one outstanding source read;
- reads at most `max_read_bytes_per_turn` original source bytes;
- returns the shared allocation to the scheduler; and
- yields to control, batch, lifecycle, or another reader decision.

The bound counts source bytes, not decoded UTF-8 size. A hot file cannot consume an
unbounded turn. Round-robin provides recurring turns among ready admitted readers, but
it does not remove receiver-wide Ack head-of-line blocking.

### EOF scheduling

A temporary EOF reader is moved to an EOF deadline set. It is re-probed at
`reader.eof_reprobe_interval` unless an earlier framing, rotation, batch, retry,
checkpoint, or lifecycle deadline applies. The reprobe reads only that reader's
validated handle; it does not run discovery or traverse a directory. EOF readers
are not continuously requeued and cannot spin.

The configured interval must be representable when added to the scheduling clock.
Deadline overflow is a configuration or terminal scheduling error; it is never wrapped.

### Descriptor eviction

When a ready logical reader needs a descriptor and all resident slots are occupied, the
least-recently-served eligible resident reader is selected.

Eviction is a handshake:

1. Stop scheduling the victim.
2. Ensure it owns no outstanding read turn.
3. Preserve only applied checkpoint progress and framing resume.
4. Discard decoder, physical-line, multiline, provisional record, and speculative
   batch state derived after applied progress.
5. Rewind its in-memory frontier to applied progress.
6. Close the descriptor.
7. Keep its runtime lease.
8. Make the slot available.

Reopen obtains a new handle, validates it as a regular file, validates the exact
locator and fingerprint evidence, seeks to the latest live applied progress, and
reconstructs all later state from source bytes. Restart recovery validates the
authoritative snapshot, replays every complete valid WAL transaction present, and seeks
to the resulting recovered progress. The filesystem-synced durable frontier is only
the guaranteed recovery floor.

If those bytes disappeared, ordinary restart/reopen recovery cannot reconstruct them.

### Removal and pinned descriptors

A disappeared reader whose descriptor is resident keeps that handle pinned while
rotation finalization is possible. It is not an eviction victim. On POSIX this permits
reading an unlinked inode. On Windows it permits compatible rename/delete-pending
continuity.

If the descriptor was evicted before the path disappeared, the receiver cannot portably
reopen that unlinked or delete-pending identity by path. It reports a
high-severity capture limitation, keeps the durable record unfinalized, and
does not claim late-write capture. Phase 1 has no implicit deadline-based loss
policy for this case.

Pinned removed handles consume `max_open_files` slots. Waiting present readers cannot
evict them, and they remain subject to bounded scheduling and finalization deadlines.

### Batch-seal rewind

The open batch may refuse a record because adding it would exceed record, byte,
distinct-delta, or deadline bounds. On seal:

- the refused record is not retained across the in-flight window;
- decoder/framer state beyond the sealed batch is discarded;
- each file represented by a batch delta rewinds to that delta's final offset;
- every other speculative reader rewinds to its latest applied checkpoint progress;
- open descriptors may remain resident; and
- reconstruction resumes only after terminal processing of the retained batch.

No source read occurs while a batch is in flight.

## Source decoding and framing

### Offset domain and pipeline order

Every source offset counts original bytes. The processing order is:

1. read bounded source bytes;
2. decode source units according to configured encoding;
3. identify physical lines;
4. apply newline or multiline framing;
5. apply deterministic line/record bounds;
6. construct an OTAP record or fragment; and
7. attach a progress delta owning the complete frame range.

Text newline and regex framing always follow decoding. Raw mode performs no decoding.

### Encoding behavior

| Encoding | Source units | Physical-line delimiter | Emitted clean body |
| --- | --- | --- | --- |
| `utf-8` | Valid UTF-8 scalars | Decoded U+000A | Text |
| `ascii` | Bytes `0x00..=0x7f` | Decoded U+000A | Text |
| `utf-16le` | Little-endian UTF-16 code units and surrogate pairs | Decoded U+000A | Text |
| `utf-16be` | Big-endian UTF-16 code units and surrogate pairs | Decoded U+000A | Text |
| `raw` | Individual bytes | Source byte `0x0a` | Bytes |

Encoding is never inferred from content.

A matching BOM is recognized only at offset zero of a new stream. It is validated and
stripped from the body. It remains owned by the first frame source range. Detectable
truncate reset begins a new stream at offset zero and repeats BOM handling.

A conflicting BOM follows `on_decode_error`; it never switches encoding. Raw mode does
not recognize or strip a BOM.

NUL is ordinary data in every encoding. It is neither EOF nor a record boundary.

### Source-unit boundaries

The decoder and framer never split, commit, truncate, or end a fragment inside:

- one UTF-8 scalar;
- one UTF-16 code unit;
- one UTF-16 surrogate pair;
- an unresolved BOM probe;
- one malformed source unit; or
- the exact source bytes associated with one decoded replacement unit.

An incomplete source unit remains uncommittable until completed or handled by an
explicit terminal policy.

### Permanent EOF and terminal framing

Live EOF is temporary. It never applies terminal decode policy, completes an
unterminated record, or proves that a writer is gone. In Phase 1, only rotation
finalization may request permanent-EOF handling, after the same resident handle has
observed EOF through `rotate_wait`. The wait is an inactivity heuristic, not writer
fencing.

D17 is evaluated in two ordered stages.

**Stage 1 -- framing eligibility.** Permanent EOF does not add a framing rule.
An unterminated terminal record is eligible only if one of these already
configured rules completed it according to that rule's ordinary semantics:

- a physical-line LF, multiline start/end match, multiline line-count bound,
  or deterministic line/record byte bound already established a boundary; or
- `framing.force_flush_period` is nonzero and its EOF-gated idle deadline
  actually expired without intervening source activity.

The first category normally completes before permanent EOF; it is listed to
make clear that already-complete records remain eligible. Phase 1 has no
separate generic terminal `emit` mode. `rotate_wait`, `preserve_raw`, and
`replace` do not satisfy or bypass newline, start-pattern, end-pattern,
multiline, or idle-flush semantics.

**Stage 2 -- decode and terminal outcome.**

| Condition | Required permanent-EOF behavior |
| --- | --- |
| Framing made the record eligible and `on_decode_error: preserve_raw` applies to an incomplete UTF-8 scalar, UTF-16 code unit or surrogate pair, or BOM probe | Emit the exact malformed source bytes in the eligible framed result and mark bounded malformed evidence |
| Framing made the record eligible and `on_decode_error: replace` applies | Emit one replacement owning the exact incomplete source range in the eligible framed result |
| Framing made the record eligible and `on_decode_error: fail` applies | Durably quarantine without advancing over the malformed source unit |
| No framing rule made all pending bytes eligible | Durably quarantine the identity without advancing across any pending byte, regardless of `preserve_raw` or `replace` |

Earlier complete records are emitted first and must resolve through aggregate
Ack/Nack policy before quarantine is applied. An emitted preserve or replacement
result advances only after matching aggregate Ack, atomic WAL application, and
the configured sync policy. D17 quarantine preserves the last applied offset
and framing resume, records bounded terminal evidence, and is synced before the
descriptor and lease are released.

Silent discard, silent commit, fabricated record completion, a
finalize-with-explicit-noncapture fallback, and treating ordinary live EOF as
permanent are prohibited.

### LF, CR, and ranges

Decoded U+000A is the sole text physical-line delimiter. Raw byte `0x0a` is the sole raw
delimiter.

For a terminal LF:

- the LF is excluded from the emitted body;
- the LF is included in frame progress;
- a preceding CR is retained as ordinary body data; and
- an empty line emits an empty body but advances through the LF.

For example, UTF-8 source `a\r\n` emits text body `a\r`. Its body source range owns
`a\r`; its frame source range additionally owns `\n`.

Multiline joins retain internal LF characters between physical lines. Those internal
LFs are body data and belong to both body and frame ranges. Only the final physical
line's terminal LF is omitted from the body.

The body source range can begin after the frame source range when an initial BOM was
stripped. The frame range can end after the body range for a terminal LF or discarded
truncated tail.

### Malformed input ordering

Decode policy is applied in source order:

| Policy | Behavior |
| --- | --- |
| `preserve_raw` | Emit the complete framed source slice as bytes and mark/count malformed evidence |
| `replace` | Emit decoded text with replacement, mark/count loss |
| `fail` | Quarantine the file at the earliest malformed unit not already preceded by an emit-ready record |

An earlier complete record is emitted before a decode failure caused by later source
bytes. Later errors never overtake earlier complete records.

Under `preserve_raw`, an open-ended split sequence may emit before a later malformed
unit is observed. Every fragment in that sequence therefore uses exact source bytes,
including fragments that were clean when emitted. Concatenating fragment bodies
reconstructs the deterministic bound-terminated body.

Prospective split sizing under `preserve_raw` uses:

```text
max(decoded UTF-8 body bytes, exact source body bytes)
```

This ensures that a later representation change to bytes cannot make an earlier
fragment exceed the configured body bound or become unreconstructable.

A clean unsplit record may remain text. A clean prefix emitted under explicit truncate
policy may remain text. Malformed units in a discarded truncate tail are still decoded
enough to detect and count them, even though their evidence is not emitted.

### Newline framing

With no multiline pattern, each complete physical line is one logical record. Its body
excludes the final LF, and its progress includes it.

A trailing partial line remains buffered. It is emitted only by:

- EOF-gated idle flush;
- deterministic oversize split/truncate behavior.

Permanent rotation EOF does not add another emission rule. If neither rule
above made the pending line eligible, D17 quarantines without advancing it.

Drain without an enabled and satisfied idle flush does not redefine the partial line as
complete.

### Start-pattern multiline

Start-pattern mode begins in `seeking`.

In `seeking`:

- a matching physical line starts a multiline buffer;
- a complete nonmatching line is emitted under newline framing; and
- each such fallback increments bounded `pattern_not_matched` telemetry.

In `buffering`:

- a nonmatching line is appended with its internal LF;
- a matching line completes the previous record immediately before the matching line;
- the matching line begins the next record;
- an end-of-line-count bound completes the current record;
- a deterministic byte bound completes the current bound-terminated record; and
- an EOF-gated idle deadline can complete the current record with timeout reason.

After timeout or line-count completion, the state returns to `seeking`.

The receiver never permanently disables a valid pattern because no match has yet been
observed.

### End-pattern multiline

End-pattern mode begins buffering with the first physical line.

- Each nonmatching complete line remains buffered with its internal LF.
- A matching line is included and completes the record.
- Line count, deterministic byte bound, or EOF-gated idle timeout completes the record.
- The next physical line begins a new buffer.

### Executable `re2-v1` subset

`re2-v1` provides a bounded linear-time executable profile:

- RE2 inline flags `i`, `m`, `s`, and `U` are accepted.
- Perl classes `\d`, `\s`, `\w`, their negations, and word boundaries use RE2's
  ASCII semantics.
- Counted repetition limits are at most 1,000.
- Text patterns run over validated decoded UTF-8.
- Raw patterns use non-Unicode byte semantics, including exact byte escapes.
- Backreferences and look-around are rejected.
- Rust-only `u`, `R`, and `x` flags are rejected.
- Unicode property escapes are rejected.
- Nested or set-operation character classes outside the profile are rejected.
- Constructs unsupported by the selected linear-time engine are rejected.
- Program and cache bounds are enforced at compile time.

The agent compiles defensively as well. A defensive mismatch fails the affected data
source. It never silently falls back to different regex semantics.

### Physical-line and logical-record bounds

`max_line_bytes` bounds one decoded physical-line body. `max_record_bytes` bounds the
emitted logical-record body. In raw mode the values count source body bytes. In text
mode they count emitted UTF-8 body bytes, with the preserve_raw prospective rule above.

A body exactly equal to a bound fits.

When a physical line crosses `max_line_bytes`:

1. Any earlier buffered multiline content is emitted first with reason
   `oversize_line_boundary`.
2. The oversize physical line becomes a self-contained logical record.
3. Regex matching is not attempted over that unbounded line.
4. `split` or `truncate` scans to its physical-line LF in bounded units.
5. Multiline state returns to its initial state after the line.

No unbounded physical-line buffer is created.

When adding a complete physical line would make a multiline body exceed
`max_record_bytes`, the deterministic logical boundary is the LF of that first
overflow-triggering line. Regex and line-count decisions are suppressed until that
line ends. The framer then resets multiline state.

For a body exactly equal to `max_record_bytes`, retaining another internal separator
would overflow. The current body is emitted as a clean forced boundary before any later
source unit is decoded. The next physical line begins from initial multiline state.

This boundary rule ensures restart needs no hidden regex or line-count state.

### Split behavior

`split` emits every body byte of the deterministic bound-terminated record in source
order. Each fragment:

- fits `max_record_bytes`;
- ends only at a safe source-unit boundary;
- has a stable correlation ID;
- has a zero-based fragment index;
- states whether it is final;
- carries exact body and frame source semantics;
- can participate in a batch ending before the sequence is complete; and
- reconstructs the bound-terminated record, not a hypothetical unlimited multiline
  record.

The durable continuation contains only:

```text
record_start_offset
next_fragment_index
```

It contains no hidden line count, regex state, buffered body, BOM probe, or decoder
partial unit. A nonfinal fragment records a continuation only after its batch is Acked
and its transaction is appended and applied. The continuation becomes crash-durable
when covered by filesystem sync.

The fragment ID is stable across retry and restart and is derived from the durable
identity, file epoch, and record start. It is the lowercase 64-character hexadecimal
encoding of the full SHA-256 digest over:

```text
UTF-8("otel-arrow-filelog-fragment-v1\0") ||
file_id as its exact 16 opaque bytes ||
file_epoch as u32 big-endian ||
record_start_offset as u64 big-endian
```

The checkpoint-format document owns no fragment-ID byte layout; this value is emitted
metadata, not checkpoint encoding.

Fragment index overflow fails before emitting a nonfinal fragment whose successor
cannot be represented. A final fragment at the maximum representable index is allowed.

A resumed continuation with no newly observed source bytes remains pending. It cannot
fabricate an empty final fragment.

Fragment attributes are experimental project attributes, not registered OpenTelemetry
semantic conventions. Standardization requires an explicit migration. An implementation
does not silently rename them to a future standard.

### Truncate behavior

`truncate` emits the largest safe prefix within the body bound and marks it truncated.
It discards through the same deterministic logical boundary that `split` would have
used:

- through the physical-line LF for an oversize line; or
- through the overflow-triggering physical-line LF for multiline.

Discarded source bytes count toward frame progress only after the truncated record is
Acked. They are included in discarded-byte and malformed-unit telemetry. The receiver
never advances over them merely because they were read.

### Idle partial flush

`force_flush_period: 0s` disables idle partial flush.

A nonzero deadline is armed only after a source read observes EOF while a nonempty
partial record is pending. It is measured from the most recent relevant physical-line
or source activity.

When a later read observes any new source byte:

1. Cancel the old idle deadline.
2. Feed the new bytes to decoding.
3. Continue framing.
4. Arm a new deadline only after EOF is observed again.

An expired EOF-gated deadline emits the current partial record with a timeout reason.
Its ending source offset becomes eligible for a clean resume after Ack. Later appended
bytes begin a new record. This is an explicit slow-writer split risk.

### Rotation and drain framing

When idle flush is enabled, rotation and drain may use the same reason-marked flush
only after its EOF-gated condition is satisfied.

Without such a flush, recoverable drain bytes remain pending and uncommitted. They are
reported, not counted as dropped.

At rotation finalization, any pending incomplete unit, unresolved BOM probe, or
unterminated framed record follows
[Permanent EOF and terminal framing](#permanent-eof-and-terminal-framing).
If no configured framing rule permits emission, D17 quarantines the identity
without advancing. Drain remains restartable and never invokes this permanent
outcome merely because its deadline expires.

## OTAP output

### Record content

Each logical record or split fragment becomes one OTAP log record.

The receiver sets:

- `body` to decoded text or exact bytes according to decoding and oversize policy;
- `observed_time_unix_nano` when the framed result becomes ready;
- required bounded file provenance;
- optional bounded source metadata when configured; and
- experimental reason, truncation, decoding, or fragment metadata when applicable.

The receiver leaves unset:

- `time_unix_nano`;
- severity number and text;
- trace and span correlation;
- parsed JSON or structured fields;
- host identity;
- destination-specific fields; and
- customer-specific enrichment, filtering, and routing decisions.

Processors own those semantic transformations. A timestamp-processor failure does not
change record framing or source progress.

### Provenance

Every record carries registered `log.file.path` and `log.file.name` provenance derived
from bounded advisory evidence. A non-UTF-8 native path uses a reversible bounded
representation and an encoding discriminator rather than lossy substitution.

Record offset and record number are optional and off by default. The architecture
describes optional resolved-path provenance, but no configuration key for enabling it
has been approved. This specification does not add one. Fragment source ranges are
required when fragments are emitted.

The semantic requirements are:

- record offset means the first original source byte represented by the body;
- body and frame ranges remain distinguishable internally;
- fragment source ranges identify original source bytes represented by that fragment;
- opaque `file_id` is not exposed by default; and
- host identity comes from resource detection or enrichment.

Public attribute names for general body/frame source ranges are not yet approved.
This specification does not invent them. Any proposed names for resolved path, record
offset, and fragment ranges remain experimental until semantic-convention review
resolves the public surface.

The public compatibility status of optional record number remains unresolved. When
enabled in Phase 1, it uses a zero-based, process-local counter per `file_id` and file
epoch. It survives descriptor eviction, resets on process restart or epoch change, and
is omitted from every split fragment. A successfully appended unsplit record emits and
consumes one number. Fragment zero consumes one number without emitting it;
continuation fragments neither emit nor consume one. These semantics are not yet a
stable public promise.

### Batch construction

The receiver uses one logical sizing function for build-time admission and runtime
batching. A record larger than `batch.max_bytes` is rejected as an invariant violation;
configuration must have made that state unreachable through framing and attribute
bounds.

The open batch seals before adding a record that would exceed:

- `batch.max_records`;
- `batch.max_bytes`;
- `MAX_DISTINCT_FILE_DELTAS_PER_BATCH = 4096`;
- the first-record `batch.max_flush_period` deadline; or
- the OTAP `u16` log-record ID space.

`batch.max_records` has a hard maximum of 65,535. Record IDs never wrap.

The first record arms the batch deadline. Later records do not extend it. A sparse
nonempty batch flushes when the deadline expires.

Progress deltas for multiple records from one file coalesce only when contiguous,
monotonic, and in the same file epoch. A gap, overlap, regression, or epoch change is
terminal rather than silently merged. A record for a 4,097th distinct `file_id`
is refused from the current open batch, the batch seals, and that record is
reconstructed for the next batch after terminal processing.

An Ack or `drop_and_continue` transaction contains exactly one
`update_progress` operation per distinct-file delta and no unrelated operation.
Version 1 reserves zero additional operations in that transaction, so:

```text
MAX_DISTINCT_FILE_DELTAS_PER_BATCH
  = WAL_MAX_OPS_PER_TX - ACK_TX_RESERVED_OPS
  = 4096 - 0
  = 4096
```

The [checkpoint-format specification](filelog-checkpoint-format.md#maximum-encoded-lengths-summary)
derives the exact
`UPDATE_PROGRESS_MAX_OP_FRAME_BYTES` and `MAX_PROGRESS_TX_FRAME_BYTES`
constants from this 4,096-operation limit. Configuration and recovery admission
must accommodate that maximum. Seal-time enforcement is the primary guard;
Ack preflight repeats both operation-count and encoded-size checks. One valid
retained batch can always be applied in one atomic transaction and is never
split into partially successful progress.

There is one receiver-wide open batch while reading. Once sealed, it becomes the one
retained in-flight batch. No new open batch is populated until that retained batch
reaches a terminal completion and the corresponding progress policy completes.

This is a correctness-first Phase 1 decision, not a throughput guarantee. For a
steady stream of full batches, one useful illustrative relationship is:

```text
ideal full-batch ceiling ~= batch record count / downstream Ack round-trip time
```

Actual throughput can be lower because of `batch.max_bytes`,
`batch.max_flush_period`, record sizes, processor time, exporter latency,
retries, filesystem and checkpoint work, and source distribution. The formula
is neither a benchmark nor a production limit. Multiple in-flight batches are
Phase 2 work and require explicit contiguous-progress, ordering,
Nack-in-the-middle, retry, memory, and drain contracts.

## Ack, Nack, and checkpoint timing

### Correlation

Each sealed logical batch has a stable `batch_id` and a send `attempt`.

Before every send or resend, the async task subscribes for Ack/Nack completion and
places `(batch_id, attempt)` in opaque engine `CallData`. The engine transports that
opaque data; filelog owns its meaning and validation.

The engine/topic runtime returns exactly one aggregate completion for that tuple.
It owns required-subscriber membership and Ack/Nack aggregation; filelog does
not maintain a per-destination completion set. For broadcast fan-out:

- aggregate Ack is produced only when every required subscriber eligible for
  that publication Acks;
- a required subscriber's Nack or disappearance produces aggregate Nack
  according to the engine topic contract; and
- retrying the publication may redeliver to subscribers that already
  succeeded, consistent with at-least-once delivery.

A required broadcast path must provide behavior equivalent to:

```yaml
ack_propagation:
  mode: auto
broadcast_ack_mode: all
```

The names above describe the required engine semantics, not receiver fields.
The receiver factory does not inspect cross-pipeline topic configuration.
Engine topology validation rejects a graph that claims Ack-gated progress
across required broadcast destinations without automatic propagation and
all-required-subscriber aggregation. No default topic behavior is assumed to
satisfy this requirement.

A completion can mutate receiver state only when:

- its batch ID equals the current retained batch;
- its attempt equals the currently subscribed attempt;
- the retained batch is still awaiting completion; and
- every progress delta still passes its file-epoch precondition.

Duplicate, late, superseded-attempt, and post-reset completions are stale. They are
ignored and counted. They never mutate a replacement stream.

### Ack

Matching Ack authorizes the retained batch's delta set for checkpoint application. It
does not itself append a WAL transaction, apply logical progress, advance the durable
frontier, release the batch, or resume source reading.

The worker:

1. Validates every delta against current `file_id`, expected offset, and file epoch.
2. Validates the entire transaction operation count and encoded-size bound.
3. Rejects the complete Ack atomically if the delta set exceeds the WAL transaction
   operation limit.
4. Never divides an overlarge Ack into partially successful progress transactions.
5. Appends one atomic logical progress transaction for all deltas.
6. Applies that transaction all-or-nothing to live checkpoint state.
7. Syncs according to the checkpoint policy.
8. Releases the retained batch at the release point defined below.
9. Reconstructs readers and resumes source scheduling only after release.

No retained batch is released and no source scheduling resumes if preflight, WAL
append, or logical application fails. With a zero sync interval, sync failure also
blocks release. Store failure never permits partial release.

### Sync interval

With `checkpoint.sync_interval: 0s`, every Ack transaction is appended, applied, and
filesystem-synced before the batch is released. Source scheduling resumes only after
that sync succeeds.

With a nonzero interval:

- an Acked transaction must append and apply successfully before release;
- the retained batch is then released and source scheduling resumes without waiting
  for filesystem sync;
- the applied progress newer than the last successful sync is not yet at the durable
  frontier;
- the worker drives the next-sync deadline even while every source is idle;
- the interval may widen the crash-duplicate window;
- ordered later sync covers all previously applied progress; and
- drain syncs all outstanding applied progress before releasing namespace ownership.

A crash after aggregate downstream Ack and release but before filesystem sync may recover only
the guaranteed durable frontier and replay the Acked data. It may instead replay a
later complete, valid, Ack-authorized WAL prefix that survived even though its sync was
not guaranteed. A structurally complete corrupted transaction still fails closed.
Delayed sync never permits recovery beyond validated Ack-authorized progress or
skipping unacknowledged source data.

### Retryable Nack

Retryable Nack classes schedule bounded exponential backoff:

```text
delay(attempt) = min(initial_backoff * 2^(attempt - 1), max_backoff)
```

Every multiplication and deadline addition is checked. The delay never wraps or
becomes a zero-delay unbounded loop.

After the delay:

1. The async task requests `Resend`.
2. The worker returns the same retained logical batch.
3. The async task increments the attempt.
4. It subscribes before sending.
5. It sends the retained batch without rereading any source.

Retry state is bounded by one retained batch and the configured attempt counter.
In a broadcast topology, this resend can duplicate delivery at a required
subscriber that Acked the prior attempt before another subscriber caused the
aggregate Nack.

### Permanent Nack and retry exhaustion

Permanent Nack, closed route, node shutdown completion, and exhausted retry budget apply
`on_nack`:

| Policy | Result |
| --- | --- |
| `fail` | Receiver terminates without advancing the retained batch |
| `drop_and_continue` | Receiver records explicit loss, applies the same atomic delta set, and releases according to the configured sync policy |

`drop_and_continue` is an intentional data-loss policy. It is counted and produces an
operator-visible event.

Drain or direct Shutdown interrupts retry backoff. Uncommitted progress remains
unchanged.

### Receiver-wide coupling

One receiver-wide batch intentionally creates:

- Ack-latency coupling across files;
- no source reads while completion is pending;
- receiver-wide backoff on retryable Nack;
- receiver-wide terminal failure on default permanent-Nack policy;
- checkpoint transaction coupling across unrelated files; and
- receiver-wide drain delay.

Round-robin source turns do not provide post-emission failure isolation.

## Checkpoint semantic contract

This section defines logical operations and their timing. The
[checkpoint-format specification](filelog-checkpoint-format.md) alone defines how they
are encoded, framed, checksummed, published, and replayed as bytes.

### Durable states

| State | Meaning |
| --- | --- |
| Absent | No durable record exists |
| Active | Identity can be reconciled and read |
| Rotated finalized | Old identity completed its rotation-finalization policy |
| Quarantined | Reading is blocked until explicit administration |

Every mutation names `file_id` and carries expected state and epoch evidence. Stale,
conflicting, impossible, unknown, or overflowing transitions fail closed.

### Logical operations

| Operation | Preconditions | Effect and timing |
| --- | --- | --- |
| `register_file` | `file_id` absent, or exactly identical idempotent replay | Create active identity, initial offset, clean resume, evidence, and metadata; durable before read |
| `update_progress` | Active; expected offset and epoch match | Atomically append and apply Acked offset and framing resume; may finalize rotation; sync follows policy |
| `reset_after_truncate` | Active; expected epoch matches; `read_new` selected | Increment epoch, reset offset and framing; sync before reading replacement stream |
| `update_fingerprint` | Active; expected evidence and epoch match | Extend/replace guarded evidence without changing identity or progress |
| `update_metadata` | Active or quarantined | Update allowed advisory fields; quarantine evidence remains immutable |
| `quarantine_file` | Active at expected epoch, or identical idempotent replay | Enter durable quarantine; sync before reporting durable state |
| `reset_quarantined_file` | Matching quarantined record | Apply `reset_to_beginning`, `reset_to_end`, or `keep_failed`; sync before release/report |
| `remove_file` | Exact matching prior state and epoch | Ordinary removal for vetted active/finalized records, or audited administrative removal |

### Registration

Registration contains the initial identity evidence, initial offset, epoch, compatible
framing profile, clean resume, active lifecycle state, and bounded advisory metadata.

Newly generated IDs are checked for collision. A conflicting existing ID fails closed.
A reconciliation batch may register multiple new identities in one atomic transaction,
but none is read until the complete registration transaction is durable.

### Progress

Progress is monotonic within one file epoch. Offset and framing resume advance
atomically. An ordinary Ack cannot change file epoch. A finalizing progress update may
transition Active to Rotated finalized only after all included source progress is Acked.
Applied progress controls live reading. Crash recovery starts from the authoritative
snapshot and replays every complete valid WAL transaction present in sequence. The
filesystem-synced durable frontier is the guaranteed replay floor, not a cap on a later
valid prefix that survived.

### Truncate reset

`reset_after_truncate` is the only non-administrative operation that increments file
epoch. It records observed truncation and explicit `read_new` intent, resets to source
offset zero with clean resume, and is synced before replacement bytes are read.

Stale Ack deltas from the prior epoch fail their guard and cannot advance the new
stream.

### Fingerprint and metadata updates

Fingerprint growth changes matching evidence only. The expected old evidence must
match. Evidence cannot shrink or conflict silently.

Active metadata may update locator, path, and last-seen time. Quarantined metadata may
update bounded advisory path and last-seen time only. Its locator and failure evidence
remain immutable.

### Quarantine and administrative recovery

Quarantine is persisted and synced before the receiver reports it as durable. Restart
therefore cannot bypass the condition.

Administrative actions require exclusive namespace ownership, exact
`checkpoint.id`, exact `file_id`, matching quarantine epoch, and a bounded nonempty
audit reason.

| Action | Effect |
| --- | --- |
| `reset_to_beginning` | Increment epoch, offset zero, clean resume, Active |
| `reset_to_end` | Validate current same-locator stream EOF, increment epoch, store that offset, clean resume, Active |
| `keep_failed` | Preserve quarantine, epoch, offset, framing state, locator, and evidence exactly |

A configuration change is not an administrative action.

Administrative removal names the exact namespace and `file_id`, matches prior state and
epoch, and carries an audit reason. It can remove quarantined state. Ordinary retention
cannot.

This specification defines operation semantics, not a complete administrative CLI or
API. An operable Phase 1 release with durable quarantine must provide a
separately reviewed engine administrative interface or offline tool that can
inspect quarantine and invoke these actions. The interface must stop or exclude
the receiver, acquire exclusive namespace ownership, use supported checkpoint
operations rather than edit bytes, validate exact namespace, `file_id`, state,
and epoch, and record the required audit reason.

Normal receiver configuration never releases quarantine or rewrites incompatible
framing state. An incompatible profile requires a separately reviewed versioned
migration, or audited administrative removal followed by ordinary registration and
its configured `start_at` behavior.

If an initial delivery omits that interface or tool, reset, administrative
removal, and migration are unavailable. In that mode durable quarantine may be
entered only by the bounded deterministic set defined under
[Failure containment](#failure-containment): decode `fail`, truncation `fail`,
D17 terminal-framing failure, recovery mismatch under `fail`, and another
per-file integrity rule only if this specification adds it explicitly. The
operator-visible startup and health status must prominently disclose that
quarantine cannot be released. The receiver still fails closed and preserves
the affected state. Selecting a different `checkpoint.id` opens another
namespace; it is not a reset of the blocked state.

### Retention

Retention is evaluated during bounded compaction. Persisted age is necessary but never
sufficient.

The runtime supplies an explicitly vetted removal set. A record is eligible only when
it has been absent for the retention interval from:

- complete discovery evidence;
- logical reader state;
- resident descriptors;
- runtime leases;
- pending candidates;
- current open-batch deltas;
- the retained in-flight batch; and
- rotation-finalization state.

Quarantined records are never ordinary retention candidates. A large forward wall-clock
jump can make old inactive records age-eligible; the runtime-vetted absence checks still
apply. Returning after removal can cause duplicate ingestion or `start_at: end`
exclusion, which is the explicit retention tradeoff.

### Publication, compaction, cleanup, and recovery

Compaction constructs one complete new authoritative generation before publishing it.
The previously authoritative generation remains recoverable until publication
completes. Cleanup cannot make an incomplete generation authoritative and is resumable
after interruption.

Recognized generations and temporary artifacts are bounded. A store with pending
retired-generation cleanup does not start another compaction until cleanup succeeds.
Cleanup rereads and validates `CURRENT` under exclusive namespace ownership and
never deletes either file belonging to the generation it currently names.

Recovery:

1. Reads and validates `CURRENT` and selects exactly the named authoritative
   generation.
2. Derives the expected namespace digest from the exact selected
   `checkpoint.id` bytes using the checkpoint-format recipe.
3. Opens both named generation files. A missing, unreadable, or incomplete
   authoritative snapshot or WAL fails closed with its distinct recovery error;
   recovery never chooses another generation by modification time.
4. Validates header version, bounds, integrity, generation, and namespace digest
   before parsing a snapshot record or WAL transaction.
5. Requires snapshot and WAL namespace digests to equal both the expected digest
   and one another. Any mismatch is a distinct namespace-mismatch error.
6. Loads a bounded snapshot.
7. Replays complete transactions in strict sequence.
8. Applies each transaction atomically.
9. Discards only the exact structurally incomplete final transaction defined by the
   checkpoint-format specification.
10. Fails closed on every other corruption, invalid length, checksum failure, unknown
   version or operation, sequence error, impossible transition, or non-tail damage.

Recovery never guesses from modification time when authority is ambiguous. Unknown
format or framing-profile versions require explicit migration or reset.

### Checkpoint trust boundary

`engine.state_dir` and the checkpoint namespace are trusted host-local state. The
operator must prevent writes by untrusted principals and apply least-privilege local
file and directory permissions. Namespace and artifact access validates opened
objects and rejects symlink, reparse-point, non-regular-file, and replacement
substitution rather than trusting path text alone.

CRC and framing digests detect accidental corruption or incompatibility; they do not
authenticate state or protect against hostile replacement. Advisory paths, locators,
fingerprints, and failure evidence in health events may be sensitive. Their values are
bounded and emitted only through appropriately protected, sampled, and rate-limited
engine telemetry.

## Rotation and truncation

### Move/create

Suppose the active path has locator A. Rotation renames A and creates a replacement at
the original path with locator B.

The receiver:

1. Keeps A's logical identity, lease, descriptor, epoch, and progress.
2. Updates A's advisory path when evidence permits.
3. Independently discovers B.
4. Gives B a new identity unless B validly reconnects separate durable state.
5. Reads A through its retained descriptor.
6. Reads B under its own source scheduling and progress.
7. Never gives B A's offset.

Same-filesystem or same-volume rename continuity comes from the locator. Cross-device or
cross-volume copy/unlink is a new identity.

### Late writes and finalization

A removed resident handle remains pinned. After each EOF, `rotate_wait` begins or
restarts. New bytes cancel the wait before framing continues.

The read/checkpoint worker is the sole originator of an
`update_progress(finalize = 0x01)` transaction. It may originate that operation
only when every precondition below holds:

1. A complete reconciliation established disappearance and the logical reader
   entered rotation finalization with its resident handle pinned.
2. The checkpoint record is `Active` at the exact expected epoch and applied
   committed offset.
3. The same validated handle observed EOF continuously through `rotate_wait`;
   any new byte canceled and restarted the wait.
4. Normal framing and any explicitly enabled, satisfied idle-flush rule have
   emitted every eligible record.
5. D17 found no ineligible pending encoded unit, BOM probe, physical line, or
   multiline record; such bytes require quarantine instead.
6. The identity owns no unresolved source delta in the open batch or retained
   batch and no completion or retry remains pending for such a delta.
7. Its final applied committed offset equals the source frontier eligible for
   finalization and its durable framing resume is `Clean`.

When the final record is in the retained batch, the matching Ack transaction may
carry its `update_progress` with `finalize = 0x01`. If the committed offset is
already current, a zero-delta finalization operation with
`new_committed_offset == expected_committed_offset` is permitted. In either
case, the finalization transaction is filesystem-synced before the descriptor
is closed and the runtime lease is released, even when ordinary Ack progress
uses a nonzero sync interval.

The ordering around non-Ack outcomes is:

- an open batch containing this identity is sealed and resolved before
  finalization;
- a retained batch keeps the identity and descriptor pinned;
- retryable Nack retains the same batch and cannot finalize;
- permanent Nack or retry exhaustion under `fail` leaves the identity
  unfinalized and terminates the receiver;
- `drop_and_continue` may finalize only after its explicit-loss progress
  transaction is atomically applied and synced;
- drain may complete finalization only if these same preconditions and
  completion steps finish within its deadline; otherwise the durable record
  remains `Active` and the drain reports the unresolved rotated source; and
- D17 quarantine is synced as `Quarantined`, closes the descriptor, releases
  the lease, and never also writes a finalizing progress operation.

`rotate_wait` is an inactivity heuristic, not writer fencing. Writes after finalization
can be missed.

If A's descriptor was already evicted before unlink or delete-pending state, portable
reopen is impossible. The receiver reports the limitation and does not claim
late-write capture or silently finalize the identity. The record remains
`Active` until an explicit deterministic failure rule or audited
administrative action applies.

Pinned removed handles consume `max_open_files` slots and are never eviction
victims. Descriptor pressure pauses or refuses new descriptor admission and
uses bounded environmental backoff. Phase 1 defines no implicit
deadline-triggered loss policy: `rotate_wait`, drain timeout, or capacity
pressure cannot close a pinned handle and convert unresolved source bytes into
noncapture. A future configured loss policy would require separate review.

### POSIX move/create

POSIX locators use device and inode evidence obtained from the handle. An unlinked file
remains readable through a retained descriptor. Rename or unlink alone does not revoke
the reader.

The receiver does not rely on path reopen after unlink. It never treats inode identity
as permanent across deletion and reuse.

### Windows move/create

Windows locators use volume serial plus 128-bit file-ID evidence from the handle. The
receiver requests compatible read/write/delete sharing so ordinary rotation can rename
or delete-pend while the handle remains open.

Compatible name removal does not close the reader. Incompatible writer sharing can
prevent open, rename, or continued access. A temporary sharing violation uses
bounded environmental backoff and does not quarantine the identity. The exact
platform error is reported, and no unread bytes are checkpointed.

### Copytruncate

Copytruncate has an unavoidable observation gap. Bytes appended after the copy but
before truncation may exist in neither the copied file nor the truncated original by the
time the receiver observes it. Truncate and regrow between observations may look like a
normal append.

No platform receives a lossless copytruncate claim.

Observable truncation includes:

- current size below committed offset;
- current size below an uncommitted reader frontier;
- fingerprint-prefix mismatch; or
- stable evidence proving stream replacement or reset.

The receiver never advances over unacknowledged bytes.

### `on_truncate: fail`

The receiver stops the file, persists and syncs quarantine for the exact `file_id`, and
then reports a high-severity condition. Existing retained batch data can still receive
its matching completion, but a crash may make destroyed source bytes unreconstructable.

Restart and configuration reload preserve quarantine.

### `on_truncate: read_new`

The receiver:

1. Stops old-epoch reads.
2. Invalidates speculative decoder/framer state.
3. Increments file epoch with checked arithmetic.
4. Persists and syncs `reset_after_truncate`.
5. Resets offset to zero and framing to clean.
6. Treats offset zero as a new stream for BOM handling.
7. Emits a high-severity intentional-reset event.
8. Reads replacement bytes only after durable reset.

An old-epoch completion cannot advance replacement progress.

## Backpressure, cancellation, and lifecycle

### Backpressure

Every queue, map, descriptor pool, candidate population, reader, source turn, decoder,
framer, batch, delta set, retry, and checkpoint transaction is bounded.

When downstream send blocks:

- the async task races the send against incoming control;
- control has biased priority;
- the worker eventually fills the single handoff and stops source reads;
- unread bytes remain in source files; and
- no hidden unbounded queue absorbs overload.

A closed route is terminal or follows explicit permanent-Nack policy. It is not retried
forever.

### Lifecycle states

| State | Required behavior |
| --- | --- |
| Starting | Construct bounded channels and workers; do not read |
| Waiting for ownership | Report alive, acquire namespace lock with bounded retry; do not load/read |
| Recovering | Load and validate durable state; fail closed |
| Reconciling | Produce initial complete or incomplete inventory |
| Running | Admit, read, frame, batch, emit, and checkpoint |
| Draining | Stop new discovery/admission/reads; finish bounded delivery and sync |
| Forced shutdown | Cancel workers and stop without advancing uncommitted progress |
| Terminal | Release local resources possible without unbounded wait; report outcome |

Controller `Ready` means the component task started. It is not proof that filelog owns
the namespace, completed recovery, reconciled sources, or is actively collecting.
Phase 1 has no engine readiness signal for that stronger condition.

### Startup

Startup order is:

1. Parse and validate configuration.
2. Create bounded channels and fixed workers.
3. Enter waiting-for-ownership.
4. Acquire the checkpoint namespace lock.
5. Recover durable state fail-closed.
6. Start initial reconciliation.
7. Resolve identities.
8. Acquire runtime leases.
9. Durably register new identities.
10. Start source scheduling.

No source byte is read before registration or recovery establishes durable progress.

### Control responsiveness

The async task polls lifecycle control with biased priority over worker handoff and
downstream completion. A blocked downstream send is directly raced with newly arriving
control, not only with a precomputed deadline.

Workers receive cooperative cancellation independently of bounded command-channel
capacity. They check it between bounded work units.

The engine cannot guarantee interruption of a stuck kernel filesystem call. It also
does not supply a bounded worker-thread join facility. The async lifecycle therefore
never performs a synchronous forever-join. It can bound its own wait and surface a
terminal condition, but it does not claim to bound an uninterruptible kernel call or
the lifetime of a stuck OS thread.

### Normal drain

On `DrainIngress` while downstream remains live:

1. Stop discovery and new admissions.
2. Capture each reader's current provisional source frontier and stop ordinary tail
   scheduling and advancement beyond it.
3. Cancel pending descriptor acquisition.
4. Permit bounded rereads only to reconstruct provisional bytes up to the captured
   frontiers.
5. Apply a partial-record flush only when the existing EOF-gated idle-flush condition
   is already satisfied.
6. Flush a nonempty open batch.
7. Await terminal completion of the retained batch.
8. Apply Ack/Nack policy.
9. Sync all outstanding applied progress to the durable frontier.
10. Measure and report every recoverable uncommitted byte as pending, including an
    incomplete source unit or unresolved BOM probe.
11. Complete an already-eligible rotation finalization only under the full
    finalization preconditions; drain does not shorten `rotate_wait` or invoke
    permanent EOF for an ordinary live reader.
12. Rewind uncommitted in-memory tails to durable progress for cleanup.
13. Close descriptors that have no unresolved rotated source.
14. Release runtime leases.
15. Release namespace ownership.
16. Notify the engine that the receiver drained.
17. Exit.

The effective deadline is the earlier of the engine drain deadline and
`drain_timeout`. Drain does not read bytes appended after its captured frontiers and
does not synthesize a completion or Ack at the deadline. Expiry leaves unacknowledged
offsets unchanged. If a pinned rotated descriptor still owns unresolved bytes,
the receiver reports drain failure/timeout rather than successful finalization.
A forced process exit may then close the OS handle, but it does not advance or
mark those bytes captured. Drain-only rewind permits restart reconstruction
when the source remains reopenable; it never invokes D17 for a live identity.

A cleanly drained receiver normally does not receive `Shutdown`. Cleanup never waits
for it.

### Direct or forced Shutdown

Direct `Shutdown` is always handled, even when no prior drain command arrived. The
receiver:

- stops reads and emission attempts immediately;
- cancels retry backoff;
- does not advance unacknowledged progress;
- requests cooperative worker shutdown;
- releases resources that can be released without an unbounded wait; and
- exits according to the forced-shutdown path.

### Source readiness limitation

Slow or blocked discovery delays new sources but does not stop already admitted readers
unless shared receiver-wide delivery or checkpoint state is blocked. A source is not
considered ready merely because the controller task is ready.

## Failure containment

Failure classification is normative and based on the failing operation and
evidence, not merely an OS error number:

| Class and examples | Containment | Required result |
| --- | --- | --- |
| Record policy: oversize under split/truncate; malformed input under preserve_raw/replace | Record | Apply configured bounded record policy and telemetry |
| Transient descriptor pressure: `EMFILE`, `ENFILE` | Receiver-local admission | Pause new opens, preserve state, and use bounded environmental backoff; never quarantine |
| Transient source/environment: `EAGAIN`, source-side `ENOSPC`, temporary permission or sharing violation, retryable I/O, temporary filesystem or mount unavailability | File, traversal root, or probe operation | Preserve progress and candidate/reader state, reprobe with bounded environmental backoff, and keep other eligible sources running; never directly quarantine |
| Deterministic decode failure under `on_decode_error: fail` | File | Durable quarantine at the earliest failing source unit |
| Deterministic truncation under `on_truncate: fail` | File | Durable quarantine |
| D17 terminal framing ineligible for emission | File | Durable quarantine without advancing pending bytes |
| Deterministic identity or recovery mismatch under `identity.on_recovery_mismatch: fail` | File | Durable quarantine |
| Another per-file integrity failure | File | Quarantine only if an explicit normative rule in this specification names it; otherwise fail or retry in its documented domain |
| Ambiguous identity under a non-fail mismatch policy | File | New identity and configured initial anchor; never inherit uncertain progress |
| Runtime lease timeout | File | Do not start a duplicate reader |
| Retryable Nack | Receiver-wide batch | Retain and retry within delivery bounds |
| Permanent Nack or exhaustion | Receiver | Apply `on_nack`; default terminal |
| Checkpoint append/sync/compaction failure, including checkpoint-side `ENOSPC` | Receiver | Bounded consecutive store failures, then terminal; never per-file quarantine |
| Structurally complete checkpoint CRC failure, namespace mismatch, impossible transition, or other checkpoint/namespace corruption | Receiver startup or receiver | Fail closed; never quarantine one file and never recover an automatic WAL prefix |
| Namespace lock timeout | Receiver startup | Terminal without reading |
| Lease-registry integrity failure | Receiver | Fail closed |
| Worker failure | Receiver | Terminal; do not invent progress |
| Downstream closure | Receiver | Terminal or explicit permanent-Nack policy |

Environmental retries use one bounded state entry per affected locator,
traversal root, or receiver-global descriptor condition:

```text
delay(failure_count) =
  min(250ms * 2^(min(failure_count - 1, 7)), 30s)
```

Arithmetic and deadline addition are checked. Success clears the failure count.
At the 30-second ceiling, reprobe may continue at that bounded rate while the
condition remains environmental; repeated failure alone does not reclassify it
as deterministic quarantine.

Per-file quarantine preserves actionable reason, operation, locator/evidence context,
and source error chain in bounded state and rate-limited health events. It never turns a
file failure into silent progress.

## Resource admission models

The formulas in this section are conservative logical admission models for the
proposed components. They are not exact RSS, allocator-resident memory, universal
throughput, or performance guarantees.

Every sum, product, cast, and duration conversion uses checked arithmetic.

### Candidate and identity reconciliation

Let `P = ADVISORY_PATH_MAX_BYTES`.

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

## Platform requirements

### Common behavior

Linux, macOS, and Windows provide the same logical contracts for:

- candidate probes that do not block indefinitely;
- handle-derived regular-file evidence;
- opaque durable identity;
- exact-locator guarded recovery;
- bounded advisory paths;
- descriptor residency and reopen validation;
- per-file ordering;
- move/create replacement separation;
- copytruncate limitations;
- Ack-gated progress;
- fail-closed recovery; and
- lifecycle cancellation.

Platform-specific APIs may differ, but they do not change logical identity or progress.

### Linux

Linux uses nonblocking, close-on-exec probes and handle-derived device and inode
locator evidence. Non-following opens reject final symlinks when
`follow_symlinks: false`; following mode resolves and validates the target
instead of applying `O_NOFOLLOW` unconditionally. `fstat` or equivalent must
prove a regular file. Open unlinked files remain readable through resident descriptors.
Checkpoint publication uses same-directory atomic rename and file and directory sync.

Validation covers FIFO and device candidates, rename, unlink, late writes, inode
reuse guards, both symlink policies, cycle handling, permission failures, torn
writes, and directory-sync fault points.

### macOS

macOS uses nonblocking, close-on-exec probes and handle-derived device and
inode evidence with the same logical regular-file and selected-follow-policy
rules as Linux. APFS and platform path behavior receive native validation
rather than being assumed from Linux results.

Validation covers rename/unlink continuity, symlink aliases, native path bytes,
descriptor eviction, checkpoint replacement, and crash-recovery fault points.

### Windows

Windows uses volume serial plus 128-bit file-ID evidence obtained from the opened
handle. This is required for ReFS correctness. The receiver applies
`FILE_FLAG_OPEN_REPARSE_POINT` only for the non-following path, follows links
according to the enabled policy otherwise, and validates regular-file type and
reparse state from the opened handle. It preserves native UTF-16 advisory paths.

Tail handles request compatible read, write, and delete sharing. Writers that deny
shared read access remain unsupported without a separately scoped capture mechanism.

Checkpoint publication can use `ReplaceFileW` for replacement and
`MoveFileExW` with write-through semantics for first publication. Handles are closed
before replacement. Absolute extended-length paths preserve long drive and UNC forms.
Errors that may have changed one or both names make the live store unusable until reopen
re-establishes authority.

Rust standard filesystem APIs do not provide the same directory-handle sync used by
Unix in this design. After syncing the temporary file, Windows directory sync is a
documented no-op. Atomic replacement remains required, but power-loss durability of the
directory entry relies on filesystem metadata journaling.

The design does not claim equal Windows crash-durability evidence until platform
power-cut or equivalent fault testing demonstrates it. Ordinary CI rename and fault-
injection tests are necessary but not sufficient for that stronger claim.

Windows validation covers:

- non-regular candidates and both reparse-point follow policies;
- NTFS and ReFS identity where available;
- compatible rename;
- delete-pending/name removal;
- late writes;
- incompatible sharing;
- first checkpoint publication;
- replacement of existing authority;
- long paths and UNC forms;
- interrupted compaction and cleanup;
- namespace-lock contention; and
- reopen after ambiguous publication errors.

### Unsupported filesystem assumptions

Network shares and filesystems with weak, unstable, or nonlocal locator and advisory-
lock behavior require separate contracts. Phase 1 does not infer their correctness from
local Linux, macOS, or Windows validation.

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
| `terminal_framing_quarantines` | D17 quarantine because pending terminal bytes were not framing-eligible |
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
- local locator ownership conflict;
- runtime lease contention or integrity failure;
- namespace ownership wait and timeout;
- file open, read, permission, and sharing failures;
- descriptor-evicted identity becoming impossible to reopen after removal;
- decode preserve/replace/fail outcomes;
- pattern fallback;
- multiline line, byte, and timeout flush;
- split, truncate, and discarded-byte outcomes;
- pending partial bytes at drain;
- D17 terminal-framing quarantine;
- move/create finalization and late-write limitation;
- copytruncate detection and selected policy;
- durable quarantine and explicit reset action;
- quarantine administration unavailable in an initial delivery;
- checkpoint append, sync, recovery, corruption, compaction, publication, and cleanup
  failures;
- retry, permanent Nack, exhaustion, and explicit drop;
- stale completion;
- drain timeout and forced shutdown; and
- worker failure or blocked-worker lifecycle detachment.

## Validation matrix

The following tests are required evidence for conformance. Test names and benchmark
hardware are intentionally not normative.

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
| Configuration | Both multiline patterns | Rejected |
| Configuration | Unsupported regex construct/profile | Rejected |
| Configuration | Framing bound exactly minimum | Accepted |
| Configuration | Framing bound below encoding minimum | Rejected |
| Configuration | Record plus attributes equals batch bound | Accepted |
| Configuration | Record plus attributes exceeds batch bound | Rejected |
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
| Identity | Two live equal fingerprints | Distinct `file_id`s |
| Identity | Short fingerprint | No fingerprint-only inheritance |
| Identity | Unique full fingerprint, complete inventory | Guarded recovery permitted |
| Identity | Unique-looking fingerprint, incomplete inventory | Fingerprint-only recovery disabled |
| Identity | Exact locator and valid prefix | Recovery permitted |
| Identity | Exact locator and mismatched prefix | No old offset inheritance |
| Identity | Committed offset beyond size | No old offset inheritance |
| Identity | Growing evidence | Same `file_id`, evidence extends |
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
| Framing | EOF then new byte before deadline | Deadline canceled before framing |
| Framing | True idle EOF deadline | Timeout record and clean resume after Ack |
| Framing | Permanent EOF, framing-eligible incomplete unit under preserve/replace | Exact bytes or one replacement emitted only inside the eligible record |
| Framing | Permanent EOF without an eligible terminal record | D17 durable quarantine; no progress over pending bytes |
| Framing | Permanent EOF under decode `fail` | Durable quarantine; prior complete records resolve first |
| Framing | Live EOF with incomplete unit or record | No terminal policy and no fabricated completion |
| Framing | Drain with partial bytes | Pending, uncommitted |
| OTAP | Raw and decoded records | Bytes/text body respectively |
| OTAP | Record ready time | `observed_time_unix_nano` set |
| OTAP | Timestamp/severity/JSON | Not interpreted by receiver |
| OTAP | `u16` record boundary | 65,535 accepted; overflow impossible |
| Batch | Next record crosses byte/count bound | Existing batch seals; refused record rewound |
| Batch | Multiple records same file | Contiguous delta coalesces |
| Batch | Delta gap or epoch mismatch | Rejected |
| Batch | 4,096 distinct file deltas | One atomic progress transaction remains encodable |
| Batch | Record would add a 4,097th distinct file delta | Current batch seals before the record; record is reconstructed for the next batch |
| Ack | Matching current attempt | Atomic durable delta application |
| Ack topology | Required broadcast with automatic propagation and `all` aggregation | One aggregate Ack only after every required eligible subscriber Acks |
| Ack topology | Required subscriber Nack or disappearance | One aggregate Nack under the engine topic contract |
| Ack topology | Required broadcast without all-required automatic aggregation | Graph rejected by engine topology validation |
| Ack topology | Retry after partial subscriber success | Same publication may redeliver to the previously successful subscriber |
| Ack | Duplicate/late/old attempt | Ignored and counted |
| Ack | Prior file epoch | Cannot advance replacement |
| Ack | Delta set over transaction operation bound | Entire Ack rejected before any advance |
| Nack | Retryable | Same retained batch, bounded backoff, no reread |
| Nack | Permanent default | Terminal without progress |
| Nack | `drop_and_continue` | Explicit loss and durable atomic advance |
| Nack | Exhaustion | Configured terminal policy |
| Checkpoint | Crash before registration sync | File was never eligible to read |
| Checkpoint | Crash after send before Ack | Reconstruct and duplicate if bytes survive |
| Checkpoint | Crash after Ack before delayed sync | Duplicate window only |
| Checkpoint | Torn final transaction | Only exact format-defined tail discarded |
| Checkpoint | Complete final bad checksum | Fail closed |
| Checkpoint | Corruption before tail | Fail closed |
| Checkpoint | Unknown version/operation | Fail closed |
| Checkpoint | Snapshot or WAL namespace digest differs from selected ID or its peer | Distinct namespace-mismatch error before applying records |
| Checkpoint | Valid `CURRENT` names missing generation file | Distinct missing-authoritative-generation error; no fallback |
| Checkpoint | Valid `CURRENT` names unreadable or incomplete authoritative generation | Distinct fail-closed recovery error; no fallback |
| Checkpoint | Compaction fault before publication | Previous generation authoritative |
| Checkpoint | Cleanup interruption | Resumable, bounded artifacts |
| Checkpoint | Cleanup sees current generation among retired candidates | Files named by `CURRENT` are retained |
| Checkpoint | Retention age without runtime absence | Not removed |
| Checkpoint | Quarantined retention candidate | Not removed |
| Checkpoint | Administrative removal wrong namespace | Fail closed |
| Checkpoint | Quarantine administration available | Exclusive, audited inspect/reset/remove path operates without manual byte edits |
| Checkpoint | Administration omitted from initial delivery | Only explicit deterministic failures can quarantine; unavailable release is prominent |
| Rotation | POSIX rename/create | Old and replacement read independently |
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
| Rotation | D17 pending terminal bytes | Quarantine transition, never `RotatedFinalized` |
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
emit-ready first. The later malformed unit then quarantines the file. The failure never
suppresses or overtakes the earlier complete record.

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
identity and progress. B never inherits A's offset.

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

Batch attempt 1 contains old epoch 7. Detectable truncation under `read_new` persists
epoch 8 and offset zero. A late Ack for epoch 7 is stale and cannot advance epoch 8.

### Example 22: Retry without reread

Batch 10 attempt 1 receives retryable Nack. The receiver waits bounded backoff and sends
the retained batch as attempt 2. It does not reopen or reread any file. A late Ack for
attempt 1 is ignored.

### Example 23: Atomic distinct-delta bound

An open batch already carries deltas for 4,096 distinct files. A record from a
4,097th file is not added. The existing batch seals and remains encodable as one
atomic progress transaction; the refused record is reconstructed for the next
batch. Ack preflight would reject the entire transaction rather than partially
apply it if this construction invariant were violated.

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

### Example 29: D17 terminal quarantine

A rotated UTF-8 file ends with an incomplete scalar in an unterminated newline
record. Idle flush is disabled. `preserve_raw` does not make the record
framing-eligible, so D17 quarantines the identity at its last applied offset.
The pending bytes are neither emitted nor committed, and the identity is not
marked `RotatedFinalized`.

### Example 30: Aggregate broadcast completion

Batch 12 attempt 1 is published to required subscribers A and B. A Acks and B
Nacks. The engine returns one aggregate Nack; filelog retains and retries the
batch. Attempt 2 may reach A again even though A previously succeeded. Only an
aggregate Ack from an attempt on which all required eligible subscribers Ack
can authorize progress.
