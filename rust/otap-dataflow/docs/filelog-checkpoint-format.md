# Filelog receiver checkpoint format (version 1)

<!-- markdownlint-disable MD013 -->

**Status:** Proposed normative Phase 1 checkpoint format.

This document specifies the exact, implementation-ready version-1 byte
encoding referenced by the
[filelog receiver architecture](filelog-receiver.md) and the
[Phase 1 checkpoint semantic contract](filelog-receiver-phase1-spec.md#checkpoint-semantic-contract).
The architecture defines the durable-checkpoint goals and accepted tradeoffs.
The Phase 1 behavioral specification defines when logical checkpoint operations
occur and their behavioral preconditions. This document defines everything
needed to encode, decode, validate, and replay those logical entities as bytes:
magic values, versions, byte order, fixed widths, field order, maximum encoded
lengths, checksum coverage, and every persisted discriminant.

The companion documents divide normative ownership as follows:

| Document | Normative ownership |
| --- | --- |
| [Filelog Receiver Architecture](filelog-receiver.md) | Architecture, scope, guarantees, decisions, and tradeoffs |
| [Filelog Receiver Phase 1 Behavioral Specification](filelog-receiver-phase1-spec.md) | Exact Phase 1 runtime behavior and state transitions |
| [Filelog Receiver Phase 1 Conformance Specification](filelog-receiver-phase1-conformance.md) | Resource models, telemetry semantics, validation cases, and normative examples |
| This document | Exact durable byte format and replay representation |

The architecture remains authoritative for system boundaries and accepted
compromises. This specification refines those decisions; it does not override
them.

**Compatibility:** version 1 is the first version of this format. There is no
prior format to migrate from. A conforming implementation MUST reject any
other `format_version` value in any header (snapshot, WAL, or `CURRENT`
marker) as an unsupported-version error, distinct from corruption. See
[Cross-version and migration behavior](#cross-version-and-migration-behavior).
Version 1 remains unfrozen while this proposal has no released conforming
implementation. After the first release freezes v1, every incompatible layout
or semantic change follows the version-bump policy below.

This document never describes serializing a native Rust, C, or operating
system structure directly (for example `stat`, `FILE_ID_INFO`, or a Rust
struct's in-memory layout). Every field below is an explicitly defined,
normalized, fixed- or bounded-length wire value.

Capitalized `MUST`, `MUST NOT`, `SHOULD`, and `MAY` carry their RFC 2119
meanings in this byte-format specification.

## Conventions used in this document

- **Byte order:** every multi-byte integer field is big-endian (network byte
  order), including the CRC-32C checksum fields, which are computed as
  32-bit values and then serialized big-endian like any other integer.
- **Booleans:** one byte; `0x00` is false, `0x01` is true. Any other value is
  an unknown discriminant and fails decoding closed.
- **Opaque byte blobs** (`file_id`, SHA-256 digests, the Windows 128-bit file
  ID): stored verbatim as raw bytes. Byte-order conventions for integers do
  not apply to them; they are not interpreted as numbers.
- **Variable-length byte fields:** each field uses the exact `u16`, `u32`, or
  `u64` length stated in its layout. A decoder MUST validate the declared
  length against the field maximum and bytes remaining **before** allocating
  or slicing, using checked arithmetic. It never truncates or wraps.
- **Self-delimiting records:** snapshot records and WAL operations carry
  explicit length prefixes and CRC-32C. WAL transactions carry a fixed
  36-byte header with protected `body_len`, followed by a frame CRC. Each
  outer unit can be bounded and validated independently.
- **Checksum:** CRC-32C, i.e. the Castagnoli polynomial variant (poly
  `0x1EDC6F41` normal / `0x82F63B78` reflected, init `0xFFFFFFFF`, reflected
  input and output, xorout `0xFFFFFFFF`; this is the same parametrization as
  iSCSI CRC-32C). This is **not** `crc32fast`'s default IEEE 802.3 polynomial
  (`0x04C11DB7`); an implementation MUST use a Castagnoli-parametrized CRC-32
  implementation (for example the `crc` crate's `CRC_32_ISCSI` catalog
  entry). Reference vector: `CRC-32C("123456789") = 0xE3069283`. CRC detects
  accidental corruption; it is not a MAC, signature, or authentication
  mechanism and does not protect against hostile replacement.
- **Digest:** within this byte format, SHA-256 (FIPS 180-4), 32 raw bytes, is
  used for namespace association (see
  [Namespace digest](#namespace-digest)), bounded full-path evidence (see
  [`AdvisoryPath`](#advisorypath-encoding)), and the framing-profile digest (see
  [Framing-profile canonical serialization and digest](#framing-profile-canonical-serialization-and-digest)),
  and committed-frontier continuity evidence (see
  [Committed-frontier guard](#committed-frontier-guard)).
  The constructions use distinct domain separators. None is a checksum for
  structural integrity or an authentication mechanism; CRC-32C
  guards structural integrity.

## Namespace and active-generation selection

The on-disk namespace layout implements the
[Phase 1 checkpoint semantic contract](filelog-receiver-phase1-spec.md#checkpoint-semantic-contract):

```text
${engine.state_dir}/filelog/@v1/<checkpoint-id-hex>/
  CURRENT
  offsets-<generation>.snapshot
  offsets-<generation>.wal
  ownership.lock
```

Later-generation compaction uses only the bounded temporary names
`offsets-<generation>.snapshot.compact.tmp`,
`offsets-<generation>.wal.compact.tmp`, and `CURRENT.compact.tmp` defined by
the behavioral compaction state machine. They are never authoritative.

- `@v1` is the literal namespace-layout version directory. The `@` byte is not
  in the configured checkpoint-ID alphabet, so this component cannot be
  confused with an encoded ID.
- `<checkpoint-id-hex>` is the lowercase hexadecimal encoding of the exact
  UTF-8 bytes of the final explicit or derived `checkpoint.id` produced by the
  [Phase 1 contract](filelog-receiver-phase1-spec.md#fields-defaults-and-variants).
  Each raw byte becomes exactly two ASCII digits `0-9` or `a-f`, with no
  prefix, separator, Unicode normalization, or case folding before encoding.
  For example, `AppLogs` becomes `4170704c6f6773` and `applogs` becomes
  `6170706c6f6773`.
- The raw `checkpoint.id` contains 1 to 127 ASCII bytes and uses only ASCII
  alphanumerics, `_`, `-`, and `.`. Values such as `.` and `..` are logical
  IDs, not path components; they encode as `2e` and `2e2e`. The encoded
  component is therefore 2 to 254 bytes and fits the common 255-byte
  filesystem component bound.
- The mapping is byte-wise injective even on case-insensitive filesystems and
  avoids interpreting the raw ID as a Windows reserved name or
  path-normalized component. Different raw IDs are different namespaces, and
  recovery never searches sibling encodings.
- `<generation>` is the ASCII decimal rendering of a `u64` generation number
  with no leading zeros (`0`, `1`, `2`, ... `18446744073709551615`). A
  generation number becomes assigned when a durable `CURRENT` first publishes
  it and is never reused after publication; an unpublished proposal may be
  reused only after its exact abandoned artifacts are removed and the
  directory is synced under exclusive ownership. The pair of files `offsets-<generation>.snapshot` and
  `offsets-<generation>.wal` sharing a generation number are always read and
  written together.
- `CURRENT` is a small fixed-width binary marker (not free-form text) that
  names the active generation. Its exact layout is defined in
  [The `CURRENT` marker](#the-current-marker).
- `ownership.lock` is an empty lock file used only for exclusive checkpoint-
  namespace ownership under architecture decision
  [D15](filelog-receiver.md#decisions-requested). The lock mechanism MUST
  reject concurrent acquisition by another process and by another independent
  store instance in the same process. A process-associated lock that permits a
  second in-process acquisition does not satisfy this contract. The lock file
  has no byte format of its own and is otherwise out of scope for this
  document.

Recovery always reads `CURRENT` first to select the generation, then opens
both files named by that generation. A missing, unreadable, or incomplete
authoritative file fails closed with the distinct recovery errors defined
below; recovery never selects an older generation by modification time. After
header validation, it loads `offsets-<generation>.snapshot` as the recovery
base, then replays `offsets-<generation>.wal` from sequence `1`. A generation directory MAY
contain snapshot/WAL files for more than one generation simultaneously during
compaction (the previous generation stays present and valid until `CURRENT`
is atomically repointed); this document only defines the byte format of each
individual file, not the atomic-replacement procedure for `CURRENT` itself,
which is a durable-checkpoint-store concern.

The behavioral
[compaction state machine](filelog-receiver-phase1-spec.md#publication-compaction-cleanup-and-recovery)
owns publication ordering and crash outcomes. In particular, a valid
post-crash `CURRENT` names either the complete old or complete new generation;
unnamed generation artifacts are never authoritative, a published generation
is never reused, and generation overflow fails before wrap.

Version 1 has no released predecessor namespace layout. The path above is the
first supported v1 layout and matches the existing pre-release implementation.
Recovery does not search for or migrate a direct-`checkpoint.id` directory.
The raw ID remains unchanged in the namespace digest and administrative
operation fields; only its filesystem path component is lowercase-hex encoded.

## Namespace digest

Snapshot and WAL headers bind an artifact to the selected opaque
`checkpoint.id`. Given its exact validated bytes:

```text
namespace_digest = SHA-256(
  UTF-8("otel-arrow-filelog-checkpoint-namespace-v1\0") ||
  checkpoint_id_len : u16 BE ||
  checkpoint_id_bytes : checkpoint_id_len bytes
)
```

The format can represent `checkpoint_id_len` in `1..=255`, matching the
administrative namespace-ID field bound. A conforming Phase 1 runtime supplies
only a configured ID in `1..=127` because of the lowercase-hex filesystem
component bound. No Unicode normalization, case folding, path escaping, or
other transformation occurs before hashing. The digest is stored in both the
snapshot and WAL headers. It is deliberately not added to `CURRENT`, whose
sole authority function remains selecting a generation.

Recovery derives the expected digest from the selected namespace before
loading records. After validating each header's own CRC, it requires the
snapshot digest and WAL digest to equal the expected digest and one another.
Any mismatch fails closed with `NamespaceMismatch` before a snapshot record or
WAL transaction is applied. The digest detects accidental artifact
misplacement; it does not authenticate checkpoint data.

## Trust boundary

`engine.state_dir` and this namespace are assumed to be trusted host-local
state protected from writes by untrusted principals. Implementations MUST use
least-privilege local file and directory permissions and MUST reject symlink,
reparse-point, non-regular-file, or replacement substitution for namespace
artifacts rather than following an untrusted path.

The checksums and digests in this format detect accidental corruption and
configuration incompatibility; they do not authenticate the writer or bytes.
Protecting checkpoint confidentiality and integrity against a hostile local
principal is an operational access-control responsibility, not a property of
this encoding.

## The `CURRENT` marker

Fixed width, 24 bytes total:

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 8 | `magic` | `"FLOGCUR\0"` (`0x46 0x4C 0x4F 0x47 0x43 0x55 0x52 0x00`) |
| 8 | 2 | `format_version` | `u16` BE, `1` in this version |
| 10 | 2 | `flags` | `u16` BE, reserved, MUST be `0` |
| 12 | 8 | `generation` | `u64` BE, the selected generation number |
| 20 | 4 | `marker_crc32c` | `u32` BE, CRC-32C over bytes `[0, 20)` |

A decoder MUST reject: a length other than exactly 24 bytes, an unrecognized
`magic`, a `format_version` other than `1` (unsupported-version error, see
[Cross-version and migration behavior](#cross-version-and-migration-behavior)),
a nonzero `flags` value (v1 defines no flag bits), or a CRC-32C mismatch. All
of these conditions fail closed. An unsupported `format_version` remains
distinct from corruption. There is no torn-tail leniency for `CURRENT`
because it is written and synced as a single small atomic replacement, never
appended to.

## Authoritative-generation recovery errors

After a valid `CURRENT` selects generation `G`, recovery opens exactly
`offsets-G.snapshot` and `offsets-G.wal`. It never searches another generation
or chooses by modification time.

| Condition after valid `CURRENT` | Distinct fail-closed result |
| --- | --- |
| Either named file does not exist | `AuthoritativeGenerationMissing` |
| Either named file cannot be opened or completely read for an environmental or permission reason | `AuthoritativeGenerationUnreadable` |
| The snapshot is physically incomplete, or the WAL lacks its complete 56-byte header | `AuthoritativeGenerationIncomplete` |
| A validated snapshot or WAL header namespace digest differs from the selected namespace or from its peer | `NamespaceMismatch` |
| A complete required structure has an invalid CRC, impossible length, or other invalid bytes | The specific corruption error defined by the affected structure |

The allowed incomplete final WAL transaction remains the sole torn-tail
exception and does not make the authoritative generation incomplete. No error
above authorizes automatic WAL-prefix recovery after a complete bad-CRC frame.
Cleanup, including recovery after interrupted cleanup, MUST reread `CURRENT`
under exclusive namespace ownership and MUST NOT delete either file belonging
to the generation it currently names.

## First-generation namespace publication

A namespace is genuinely absent only when the namespace directory itself and
all artifacts beneath its intended path are absent. An existing directory,
`CURRENT` temporary, snapshot, WAL, or other artifact without a valid
`CURRENT` is never interpreted as an empty namespace.

Version 1 uses these fixed first-publication temporary names:

```text
offsets-0.snapshot.create.tmp
offsets-0.wal.create.tmp
CURRENT.create.tmp
```

Under least-privilege permissions, first publication is:

1. Require `engine.state_dir` to be an already durable engine-owned root.
   Open or create `filelog`, validate that it is a directory, and sync
   `engine.state_dir` unconditionally. Then open or create `@v1`, validate it,
   and sync `filelog` unconditionally. These parent syncs are required even
   when another process created the visible ancestor.
2. Open or atomically create the namespace directory, validate it, and sync
   the `@v1` parent unconditionally before creation or recovery continues. An
   already-existing path enters recovery rather than creation.
3. Create/acquire `ownership.lock` and hold exclusive ownership.
4. Inventory the directory. A fresh creation contains only `ownership.lock`.
5. Write a complete empty snapshot to `offsets-0.snapshot.create.tmp`, sync the
   file, and close it.
6. Write the complete 56-byte WAL header to
   `offsets-0.wal.create.tmp`, sync the file, and close it.
7. Rename both temporary generation files to `offsets-0.snapshot` and
   `offsets-0.wal`, then sync the namespace directory where the platform
   supports directory sync.
8. Write and sync a complete generation-zero marker to `CURRENT.create.tmp`.
9. Atomically rename it to `CURRENT`, then sync the namespace directory where
   supported.

No source is registered or read before step 9 and every required ancestor sync
complete. A platform cannot claim the Phase 1 durable-publication guarantee if
it cannot provide the required parent-directory durability.

With no valid `CURRENT`, an empty namespace directory is recognized as an
interruption after durable directory creation but before lock-file creation.
Recovery creates/acquires `ownership.lock`, then inventories again under that
exclusive ownership. Repair proceeds only when the directory contains
`ownership.lock` plus a subset of the three exact temporary names and the two
generation-zero final names, and contains no other artifact. Recovery reports
the interruption, removes the recognized temporary and generation artifacts
while retaining the held lock, syncs the namespace directory where supported,
ensures the namespace entry is durable in its parent, and restarts from the
step 4 inventory. This is explicit interrupted-publication repair, not
treatment as an empty namespace. Any other artifact set without a valid
`CURRENT` is `AuthorityMissingOrAmbiguous` and fails closed.

Once valid `CURRENT` exists, it alone selects authority. A leftover
`CURRENT.create.tmp` or generation temporary is never authoritative and may be
removed only after protecting the generation named by `CURRENT`. The Windows
directory-sync limitation remains as documented by the behavioral platform
contract.

## Magic values, versions, and fixed widths at a glance

| File | Magic (8 bytes) | Header width | Footer |
| --- | --- | --- | --- |
| `CURRENT` marker | `"FLOGCUR\0"` | 24 bytes (whole file) | none |
| Snapshot | `"FLOGSNP\0"` | 60 bytes | 24 bytes, magic `"FLOGSFT\0"` |
| WAL | `"FLOGWAL\0"` | 56 bytes | none (append-only) |
| WAL transaction | `"FLOGTXN\0"` | 36 bytes | 4-byte `frame_crc32c` |

`format_version` is `u16` and is `1` for every header in this version. The
snapshot/WAL/`CURRENT` format version is a single coherent number for the
whole on-disk encoding; it is distinct from `framing_profile_version`, which
versions only the framing-profile canonical serialization and digest
algorithm (see below) and can in principle advance independently.
`tx_envelope_version` is also `1`; it versions the fixed WAL transaction
framing within format version 1, and any other value fails before body-length
classification.

## Snapshot file format

### Snapshot header (60 bytes)

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 8 | `magic` | `"FLOGSNP\0"` |
| 8 | 2 | `format_version` | `u16` BE, `1` |
| 10 | 2 | `flags` | `u16` BE, reserved, MUST be `0` |
| 12 | 8 | `generation` | `u64` BE; MUST equal the generation encoded in the file name |
| 20 | 32 | `namespace_digest` | SHA-256 digest defined in [Namespace digest](#namespace-digest) |
| 52 | 4 | `record_count` | `u32` BE; number of snapshot records that follow |
| 56 | 4 | `header_crc32c` | `u32` BE, CRC-32C over bytes `[0, 56)` |

Before allocating record state, recovery validates
`record_count <= configured limits.max_tracked_files <= u32::MAX`.

### Snapshot record (self-delimiting)

```text
record_len        : u32 BE                     -- length of `payload` in bytes
payload            : record_len bytes
record_crc32c      : u32 BE                     -- CRC-32C over (record_len as 4 BE bytes) || payload
```

`record_len` MUST be no greater than
`SNAPSHOT_MAX_RECORD_PAYLOAD_BYTES` and the bytes remaining before allocation.

`payload` field order (this is the exact, implementation-ready field order;
an encoder MUST write fields in this order and a decoder MUST read them in
this order):

| # | Field | Encoding | Max length |
| --- | --- | --- | --- |
| 1 | `file_id` | `[u8; 16]`, opaque | fixed |
| 2 | `file_epoch` | `u32` BE | fixed |
| 3 | `committed_offset` | `u64` BE | fixed |
| 4 | `committed_frontier_guard` | [`CommittedFrontierGuard`](#committed-frontier-guard) | 34 bytes |
| 5 | `fingerprint_len` | `u16` BE | fixed |
| 6 | `fingerprint_bytes` | `[u8; fingerprint_len]` | `FINGERPRINT_MAX_BYTES = 65535` |
| 7 | `ignored_header_bytes` | `u32` BE | fixed |
| 8 | `locator` | [`Locator`](#locator-encoding) | see below |
| 9 | `framing_profile_version` | `u16` BE | fixed |
| 10 | `framing_profile_digest` | `[u8; 32]`, opaque (SHA-256) | fixed |
| 11 | `framing_resume` | [`FramingResume`](#framingresume-encoding) | see below |
| 12 | `lifecycle_state` | `u8` discriminant, see [Lifecycle state](#lifecycle-state-discriminant) | fixed |
| 13 | `quarantine_evidence` | present iff `lifecycle_state == Quarantined`, see below | see below |
| 14 | `last_seen_time_unix_nano` | `u64` BE | fixed |
| 15 | `advisory_path` | [`AdvisoryPath`](#advisorypath-encoding) | `44 + stored_path_len` bytes |

`file_id` is the record's key and MUST be unique across every record in a
single snapshot file. An encoder MUST refuse to write two records sharing a
`file_id`, and a decoder MUST fail closed (rather than keeping only the
last-seen record for that key) if it encounters two records sharing a
`file_id`.

The exact locator is also a global live-state key. At most one record whose
state is `Active` or `Quarantined` may carry a given non-`Unspecified`
`Locator`. `RotatedFinalized` records do not participate in this uniqueness
rule because a later identity may legitimately reuse their native locator.
Snapshot encoders and decoders enforce the rule across the complete record
set; a conflict is `InvalidSnapshotState`, never an iteration-order choice.

`quarantine_evidence` (present only when `lifecycle_state == Quarantined`;
entirely absent from the byte stream for `Active` and `RotatedFinalized`
records -- there is no placeholder or presence flag because
`lifecycle_state` itself already determines presence):

| Field | Encoding |
| --- | --- |
| `reason_code` | `u16` BE, opaque diagnostic value (see [Reason codes are not structural](#reason-codes-are-not-structural)) |
| `observed_size` | `u64` BE |
| `quarantine_epoch` | `u32` BE |
| `quarantine_time_unix_nano` | `u64` BE |

The quarantine record's immutable quarantine locator is the same `locator`
field at position 8; `update_metadata` is defined below to never change it
for a quarantined record.

`quarantine_evidence` presence and `lifecycle_state` MUST agree in both
directions: an encoder MUST refuse to write a record whose `lifecycle_state`
is `Quarantined` but which carries no evidence, and MUST equally refuse to
write a record whose `lifecycle_state` is not `Quarantined` but which
carries evidence anyway. Both are structural encode-time failures, not
debug-only assertions, since a decoder has no way to recover the correct
shape from an already-inconsistent value.

### Snapshot reachable-state invariants

A CRC-valid record is still rejected unless it is reachable through the
version-1 logical operations. Snapshot encoders and decoders enforce all of the
following before WAL replay:

- `file_epoch >= 1`;
- `file_id` is unique in the snapshot;
- every non-`Unspecified` locator has at most one `Active` or `Quarantined`
  claimant across the snapshot;
- `locator` is a recognized non-`Unspecified` kind;
- `committed_frontier_guard.window_len ==
  min(committed_offset, COMMITTED_FRONTIER_GUARD_WINDOW_BYTES)`;
- fingerprint and advisory-path lengths satisfy their bounds, and
  `ignored_header_bytes + fingerprint_len` is representable as `u64`;
- `framing_profile_version != 0` and the digest is exactly 32 bytes; current
  registration emits version 1, while an unrecognized nonzero stored version
  is preserved and blocked as per-file framing incompatibility rather than
  snapshot corruption;
- `FramingResume::Continuation` has `next_fragment_index >= 1`,
  `record_start_offset < committed_offset`, and either
  `record_end_offset == 0` or `committed_offset < record_end_offset`;
- `Active` has no quarantine evidence;
- `RotatedFinalized` has `FramingResume::Clean` and no quarantine evidence;
- `Quarantined` has evidence, `reason_code != 0`,
  `quarantine_epoch == file_epoch`, and preserves the record's locator,
  fingerprint, committed offset, committed-frontier guard, and framing resume; and
- every `AdvisoryPath` flag, length, kind, alignment, suffix-selection, and
  recomputable untruncated digest invariant holds.

Unknown lifecycle or nested discriminants remain structural decode errors.
Failure of a reachable-state rule is `InvalidSnapshotState`, not a candidate
for repair or record omission. The same invariants constrain compaction output,
so a snapshot encoder cannot serialize an in-memory state that replay could
never produce.

After the stored framing profile is established as compatible with the
selected runtime configuration, every record fingerprint length MUST also be
no greater than configured `identity.fingerprint_bytes`. A compatible record
exceeding that evidence window fails closed rather than weakening the
configuration-derived recovery admission model. An incompatible profile
remains the distinct per-file `FramingProfileIncompatible` condition.

### Snapshot footer (24 bytes)

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 8 | `footer_magic` | `"FLOGSFT\0"` |
| 8 | 8 | `total_record_bytes` | `u64` BE; sum of each record's total on-wire size (`4 + record_len + 4`) across all `record_count` records |
| 16 | 4 | `record_count_echo` | `u32` BE; MUST equal the header's `record_count` |
| 20 | 4 | `footer_crc32c` | `u32` BE, CRC-32C over bytes `[0, 20)` |

### Snapshot torn-tail policy: none

Unlike the WAL, a snapshot file has **no torn-tail tolerance**. A snapshot is
written completely, synced, and only then made reachable through `CURRENT`
(the [Phase 1 publication and compaction algorithm](filelog-receiver-phase1-spec.md#publication-compaction-cleanup-and-recovery));
a reader never observes a snapshot that is genuinely still being written.
Therefore any of the following is corruption and fails recovery closed, with
no leniency, even if it is the physical end of the file:

- fewer than 60 bytes available for the header, or header magic/version/flags/CRC invalid;
- fewer than `record_count` complete, individually CRC-valid records available;
- a record whose declared `record_len` exceeds the remaining buffer;
- two records declaring the same `file_id`;
- two `Active`/`Quarantined` records claiming the same exact locator;
- a CRC-valid record violating any reachable-state invariant;
- the 24-byte footer missing, truncated, or CRC-invalid;
- `record_count_echo` or `total_record_bytes` inconsistent with what was actually parsed;
- any trailing bytes remaining after a structurally valid footer.

## Locator encoding

The locator is a normalized, platform-neutral representation. It never
serializes a native `stat` structure, `dev_t`/`ino_t`, or Windows
`FILE_ID_INFO` struct layout directly; it copies out only the specific
integer or byte-array values needed for equality comparison, in a
fixed, explicitly defined shape. A conforming codec has no OS-specific types
and performs no OS FFI; it operates purely on these normalized values, which
the Phase 1 runtime is responsible for populating from the current platform.

```text
kind : u8
```

| `kind` | Name | Additional fields | Total encoded size |
| --- | --- | --- | --- |
| `0x00` | `Unspecified` | none | 1 byte |
| `0x01` | `PosixDevIno` | `dev: u64` BE, `ino: u64` BE | 17 bytes |
| `0x02` | `WindowsVolumeFileId` | `volume_serial: u64` BE, `file_id: [u8; 16]` opaque | 25 bytes |
| `0x03`..`0xFF` | reserved | -- | decode fails closed |

- **`Unspecified`** represents "no runtime locator recorded" -- for example,
  the documented case where the required Windows identity is unavailable.
  The [Phase 1 platform contract](filelog-receiver-phase1-spec.md#platform-requirements)
  requires this case to be defined and tested rather than silently
  substituting a fallback identity; `Unspecified` is that explicit,
  normalized representation. Version-1 reachable snapshot and registration
  state rejects it; a version-1 encoder MUST NOT emit it in a snapshot or
  `register_file`, and a candidate without a required locator is not read.
- **`PosixDevIno`** normalizes POSIX `(st_dev, st_ino)`. Both `dev` and `ino`
  are widened to `u64` regardless of the native platform's underlying
  integer width (`dev_t`/`ino_t` sizes vary by OS and architecture); a
  narrower native value is zero-extended before being written, never
  reinterpreted or truncated.
- **`WindowsVolumeFileId`** normalizes `(volume_serial, FILE_ID_INFO)`.
  `volume_serial` is the full 64-bit
  `FILE_ID_INFO.VolumeSerialNumber`; it is never truncated to the older
  `BY_HANDLE_FILE_INFORMATION.dwVolumeSerialNumber` field. `file_id` is the
  128-bit `FILE_ID_INFO.FileId` byte array, copied verbatim in the byte order
  the platform API already returns (it is an opaque 16-byte identifier, not
  an integer, so no additional byte-order conversion is applied by this
  format).
- `0x03`..`0xFF` are reserved for a future locator kind. `kind` is a
  structural discriminant (it determines how many following bytes exist), so
  an unrecognized value fails decoding closed rather than being skipped or
  guessed at.

## Committed-frontier guard

`CommittedFrontierGuard` is fixed-width continuity evidence for the raw source
bytes immediately preceding `committed_offset`:

```text
window_len : u16 BE
digest     : [u8; 32]
```

`COMMITTED_FRONTIER_GUARD_WINDOW_BYTES = 64`. The required window length is:

```text
window_len = min(committed_offset, 64)
```

For a nonzero offset, `window_bytes` are exactly source range
`[committed_offset - window_len, committed_offset)`. The digest is:

```text
digest = SHA-256(
  UTF-8("otel-arrow-filelog-frontier-guard-v1\0") ||
  window_len : u16 BE ||
  window_bytes
)
```

Offset zero requires `window_len == 0` and the digest of the domain prefix plus
zero encoded as `u16` BE. A decoder validates length consistency against the
stored offset. Recovery obtains the same raw range from the validated handle
and compares the digest before inheriting progress. The digest is matching
evidence, not authentication or proof against a replacement that reproduces
the checked locator, prefix, size bound, and frontier window.

Normative vectors:

| Offset/window | Expected digest |
| --- | --- |
| Offset zero, empty window | `be47d023a06e82fd6da2daa0631547d6eca297b7ac532cba6471ab90829ec5b9` |
| Offset 4, raw window `abc\n` | `23321df310e76dad74d895ad8e8e99d64f331fa350d4117f1f818a755d0a306a` |

## `AdvisoryPath` encoding

Advisory paths are bounded diagnostics, never identity or progress evidence.
The encoding is:

```text
path_kind          : u8
path_flags         : u8
full_path_len      : u64 BE
stored_path_len    : u16 BE
stored_path_bytes  : stored_path_len bytes
full_path_digest   : [u8; 32]
```

| `path_kind` | Name | Complete native-byte representation |
| --- | --- | --- |
| `0x00` | `Unavailable` | Empty |
| `0x01` | `UnixBytes` | Native Unix path bytes, with no UTF-8 requirement |
| `0x02` | `WindowsUtf16Le` | Native UTF-16 code units serialized individually as `u16` little-endian |
| `0x03`..`0xFF` | reserved | Decode fails closed |

`path_flags` bit `0x01` is `TRUNCATED`; every other bit is reserved and MUST
be zero. `ADVISORY_PATH_STORED_MAX_BYTES = 4096`.

For `UnixBytes` and `WindowsUtf16Le`, `full_path_len` is the complete native
byte length, MUST be representable as `u64`, and MUST be nonzero. An
unrepresentable length fails evidence/registration before encoding. Windows
length and stored length MUST be even
so truncation never splits a UTF-16 code unit. The storage rule is exact:

- when `full_path_len <= 4096`, `TRUNCATED` is clear,
  `stored_path_len == full_path_len`, and the stored bytes are the complete
  native representation;
- when `full_path_len > 4096`, `TRUNCATED` is set,
  `stored_path_len == 4096`, and the stored bytes are the final 4,096 bytes of
  the complete native representation.

`Unavailable` requires flags zero, lengths zero, and no stored bytes.

The digest is:

```text
full_path_digest = SHA-256(
  UTF-8("otel-arrow-filelog-advisory-path-v1\0") ||
  path_kind : u8 ||
  full_path_len : u64 BE ||
  complete_native_path_bytes
)
```

For an untruncated path, a decoder recomputes and validates the digest. For a
truncated path, the complete bytes are unavailable to the decoder, so the
digest is retained as opaque comparison/diagnostic evidence and cannot
authenticate the omitted prefix. The runtime emits bounded provenance and a
truncation marker under the
[Phase 1 project attribute registry](filelog-receiver-phase1-spec.md#provenance).
It never uses the stored suffix or digest to match identity or inherit
progress.

The runtime computes full length, SHA-256, and the rolling final 4,096-byte
suffix in one bounded pass; checkpoint preparation does not allocate a second
buffer proportional to the complete native path.

## `FramingResume` encoding

```text
kind : u8
```

| `kind` | Name | Additional fields | Total encoded size |
| --- | --- | --- | --- |
| `0x00` | `Clean` | none | 1 byte |
| `0x01` | `Continuation` | `record_start_offset: u64` BE, `record_end_offset: u64` BE, `next_fragment_index: u32` BE | 21 bytes |
| `0x02`..`0xFF` | reserved | -- | decode fails closed |

`Clean` is the common durable resume state: the next complete source unit
starts a new logical record. `Continuation` is the split-record durable resume
state: the original record's start, its termination evidence, and the next
fragment index to emit, which are required to reconstruct
the fragment identifier and fragment index defined by the
[Phase 1 split contract](filelog-receiver-phase1-spec.md#split-behavior)
deterministically after restart. `kind` is structural; `0x02`..`0xFF` fail
decoding closed.

For `Continuation`, `record_end_offset == 0` is the scan-to-next-physical-LF
termination mode used by an oversized physical line whose LF was not known when
an earlier bounded fragment committed. A nonzero value is the exact already-known
deterministic end. No other sentinel meaning is defined.

## Lifecycle state discriminant

```text
lifecycle_state : u8
```

| Value | Name |
| --- | --- |
| `0x00` | reserved, invalid; decode fails closed |
| `0x01` | `Active` |
| `0x02` | `RotatedFinalized` |
| `0x03` | `Quarantined` |
| `0x04`..`0xFF` | reserved; decode fails closed |

There is no `Absent` value: an absent record is represented by the record's
key (`file_id`) simply not appearing in the snapshot and not having a live
record after WAL replay, never by a persisted "absent" tag.

## Reason codes are not structural

`reason_code` (in `quarantine_evidence` and `quarantine_file`) and
`removal_reason` (in `remove_file`) are **opaque `u16` diagnostic values**,
not structural discriminants: their value never changes how many following
bytes exist or how the rest of the record/operation is parsed. A decoder
therefore accepts any `u16` value for these fields without failing, including
values this document does not yet assign a name to; this lets a future minor
addition of a new diagnostic reason code ship without a `format_version`
bump. Business-rule validation of specific reason values (for example, that
`reset_after_truncate`'s reason must equal `TRUNCATE_RESET_REASON_READ_NEW`)
is an **apply-time** semantic check against the logical state machine,
documented per-operation below, and is reported as a distinct replay/apply
error, never as a decode error.

This document defines these `reason_code` values for `quarantine_file` /
`quarantine_evidence` (informational; decoders do not enforce this list):

| Value | Name |
| --- | --- |
| `0x0000` | reserved, MUST NOT be produced by an encoder |
| `0x0001` | decode-error `fail` policy quarantine |
| `0x0002` | truncate `fail` policy quarantine |
| `0x0003` | recovery-mismatch `fail` policy quarantine |
| `0x0004` | reserved; a version-1 encoder MUST NOT emit it |
| `0x0005`-`0x00FF` | reserved for future built-in reasons; profile incompatibility fails closed without mutating the record |
| `0x0100`-`0xFFFF` | available for distribution- or extension-defined reasons |

`removal_reason` has no assigned values in v1 beyond the requirement that an
encoder MUST NOT write `0x0000`; `0x0000` is reserved exactly as above.

## WAL file format

### WAL header (56 bytes)

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 8 | `magic` | `"FLOGWAL\0"` |
| 8 | 2 | `format_version` | `u16` BE, `1` |
| 10 | 2 | `flags` | `u16` BE, reserved, MUST be `0` |
| 12 | 8 | `generation` | `u64` BE; MUST equal the generation encoded in the file name |
| 20 | 32 | `namespace_digest` | SHA-256 digest defined in [Namespace digest](#namespace-digest) |
| 52 | 4 | `header_crc32c` | `u32` BE, CRC-32C over bytes `[0, 52)` |

The header itself follows the same no-torn-tail policy as the snapshot
header: it is written once when the WAL generation is created and never
rewritten in place, so an incomplete or invalid header is corruption, not a
torn write.

After the header, the WAL body is a sequence of **transactions** that
continues until end of file. There is no footer, because the WAL is
append-only and its last transaction is the only place a torn write can ever
occur (see [Torn-tail versus corruption](#torn-tail-versus-corruption)).

### Transaction framing

```text
transaction_header : 36 bytes
operation_body     : body_len bytes
frame_crc32c       : u32 BE
```

The fixed transaction header is:

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 8 | `tx_magic` | `"FLOGTXN\0"` |
| 8 | 2 | `tx_envelope_version` | `u16` BE, `1` |
| 10 | 2 | `tx_flags` | `u16` BE, reserved, MUST be `0` |
| 12 | 8 | `sequence` | `u64` BE |
| 20 | 4 | `body_len` | `u32` BE; total encoded operation-frame bytes |
| 24 | 4 | `body_len_complement` | `body_len XOR 0xFFFFFFFF` |
| 28 | 2 | `op_count` | `u16` BE, `1..=WAL_MAX_OPS_PER_TX` |
| 30 | 2 | `reserved` | `u16` BE, MUST be `0` |
| 32 | 4 | `header_crc32c` | CRC-32C over header bytes `[0, 32)` |

The operation body is exactly `op_count` self-delimiting operation frames and
must consume exactly `body_len` bytes. `frame_crc32c` is CRC-32C over the
complete 36-byte transaction header, including `header_crc32c`, followed by
the operation body.

- **Sequences** start at `1` for the first transaction ever written into a
  fresh WAL generation and increase by exactly `1` for every subsequent
  transaction, with no gaps and no repeats. A sequence that is not exactly
  `previous + 1` is an ordering error and fails replay closed; it is never
  treated as a torn tail.
- **Envelope validation order:** after 36 header bytes are physically present,
  a decoder validates magic, envelope version, flags, reserved, length
  complement, header CRC, body-length bounds, operation-count bounds, and
  sequence before using `body_len` to classify the remaining suffix. A failure
  is corruption or unsupported version, never a torn tail.
- **Atomicity:** a transaction's operations become visible only as a
  complete, validated set. A decoder MUST NOT expose any operation from a
  transaction whose `frame_crc32c` does not validate, and MUST NOT expose a
  partial prefix of a transaction's operations.
- `op_count == 0` is rejected: every transaction carries at least one
  operation. This keeps the "smallest replayable unit" concept simple and
  matches the behavioral requirement that one transaction contains a
  [bounded set of operations](filelog-receiver-phase1-spec.md#logical-operations).
- `TX_MIN_BODY_BYTES = 34` and `TX_MIN_FRAME_BYTES = 74`; the minimum body is
  one `update_fingerprint` operation that strictly extends an empty
  fingerprint by one byte.
- `body_len` MUST be within
  `TX_MIN_BODY_BYTES..=WAL_MAX_TX_BODY_BYTES`. Recovery never scans inner
  operations to reinterpret an invalid outer envelope.
- After operation parsing, a transaction is either progress-only (every
  operation is `update_progress`) or non-progress (no operation is
  `update_progress`). Mixed transactions fail replay closed. Progress-only
  transactions may contain up to `WAL_MAX_OPS_PER_TX` and MUST contain at
  most one operation for each `file_id`; a duplicate progress key fails the
  complete transaction closed before application. Non-progress transactions
  may contain at most `WAL_MAX_NON_PROGRESS_OPS_PER_TX`. Both classes remain
  subject to `WAL_MAX_TX_BODY_BYTES` before allocation.

### Operation framing (self-delimiting)

```text
op_len       : u32 BE               -- length of op_payload in bytes
op_payload    : op_len bytes         -- op_code (u8) || operation-specific fields
op_crc32c     : u32 BE               -- CRC-32C over (op_len as 4 BE bytes) || op_payload
```

`op_len` is validated against `MAX_OPERATION_PAYLOAD_BYTES`, the enclosing
`body_len`, and the bytes remaining before allocation or slicing.

Every operation is individually length-prefixed and individually
CRC-32C-checked, even though the enclosing transaction's `frame_crc32c` already
covers the same bytes. This redundancy is intentional: it lets an operation
be extracted, inspected, or replayed independently (for example by a future
offline audit or migration tool) without needing the enclosing transaction
context, and it gives defense-in-depth against an offset-computation bug
silently shifting operation boundaries within an otherwise CRC-valid
transaction body.

`op_code` (the first byte of `op_payload`) is a structural discriminant:

| `op_code` | Operation |
| --- | --- |
| `0x00` | reserved, invalid; decode fails closed |
| `0x01` | `register_file` |
| `0x02` | `update_progress` |
| `0x03` | `reset_after_truncate` |
| `0x04` | `update_fingerprint` |
| `0x05` | `update_metadata` |
| `0x06` | `quarantine_file` |
| `0x07` | `reset_quarantined_file` |
| `0x08` | `remove_file` |
| `0x09`..`0xFF` | reserved; decode fails closed |

Every operation payload begins with `file_id: [u8; 16]` immediately after
`op_code`, identifying the checkpoint record the operation applies to.

### Operation payloads (exact field order)

All fields not otherwise noted use the shared [`Locator`](#locator-encoding)
and [`FramingResume`](#framingresume-encoding) encodings above.

#### `register_file` (`0x01`)

| # | Field | Encoding | Max length |
| --- | --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` | fixed |
| 2 | `file_epoch` | `u32` BE | fixed |
| 3 | `committed_offset` | `u64` BE | fixed |
| 4 | `committed_frontier_guard` | `CommittedFrontierGuard` | 34 bytes |
| 5 | `fingerprint_len` | `u16` BE | fixed |
| 6 | `fingerprint_bytes` | `[u8; fingerprint_len]` | `FINGERPRINT_MAX_BYTES = 65535` |
| 7 | `ignored_header_bytes` | `u32` BE | fixed |
| 8 | `locator` | `Locator` | see above |
| 9 | `framing_profile_version` | `u16` BE | fixed |
| 10 | `framing_profile_digest` | `[u8; 32]` | fixed |
| 11 | `framing_resume` | `FramingResume` | see above |
| 12 | `last_seen_time_unix_nano` | `u64` BE | fixed |
| 13 | `advisory_path` | `AdvisoryPath` | `44 + stored_path_len` bytes |

#### `update_progress` (`0x02`)

| # | Field | Encoding |
| --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` |
| 2 | `expected_committed_offset` | `u64` BE |
| 3 | `expected_file_epoch` | `u32` BE |
| 4 | `new_committed_offset` | `u64` BE |
| 5 | `new_committed_frontier_guard` | `CommittedFrontierGuard` |
| 6 | `new_framing_resume` | `FramingResume` |
| 7 | `new_last_seen_time_unix_nano` | `u64` BE |
| 8 | `finalize` | `u8` bool (`0x01` transitions to `RotatedFinalized`) |

#### `reset_after_truncate` (`0x03`)

| # | Field | Encoding |
| --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` |
| 2 | `expected_active_epoch` | `u32` BE |
| 3 | `observed_truncated_size` | `u64` BE (informational evidence, not independently verified by the codec) |
| 4 | `resulting_epoch` | `u32` BE |
| 5 | `new_committed_offset` | `u64` BE |
| 6 | `new_framing_resume` | `FramingResume` |
| 7 | `new_fingerprint_len` | `u16` BE |
| 8 | `new_fingerprint_bytes` | `[u8; new_fingerprint_len]`; maximum `FINGERPRINT_MAX_BYTES` |
| 9 | `reset_time_unix_nano` | `u64` BE |
| 10 | `reason_code` | `u16` BE (opaque; see [reason codes](#reason-codes-are-not-structural)) |

#### `update_fingerprint` (`0x04`)

| # | Field | Encoding | Max length |
| --- | --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` | fixed |
| 2 | `expected_file_epoch` | `u32` BE | fixed |
| 3 | `expected_fingerprint_len` | `u16` BE | fixed |
| 4 | `expected_fingerprint_bytes` | `[u8; expected_fingerprint_len]` | `FINGERPRINT_MAX_BYTES` |
| 5 | `new_fingerprint_len` | `u16` BE | fixed |
| 6 | `new_fingerprint_bytes` | `[u8; new_fingerprint_len]` | `FINGERPRINT_MAX_BYTES` |

#### `update_metadata` (`0x05`)

| # | Field | Encoding | Presence |
| --- | --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` | always |
| 2 | `expected_prior_state` | `u8`: `0x01` `Active` or `0x03` `Quarantined`; other values fail decoding closed | always |
| 3 | `expected_file_epoch` | `u32` BE | always |
| 4 | `presence_flags` | `u8` bitfield: bit 0 (`0x01`) = `PATH_PRESENT`; other bits reserved, MUST be `0` | always |
| 5 | `last_seen_time_unix_nano` | `u64` BE | always |
| 6 | `advisory_path` | `AdvisoryPath` | only if `PATH_PRESENT` |

A field marked "only if" is entirely absent from the byte stream when its
presence bit is clear. `PATH_PRESENT` distinguishes "do not update path" from
an explicit `AdvisoryPath::Unavailable`.

#### `quarantine_file` (`0x06`)

| # | Field | Encoding |
| --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` |
| 2 | `expected_file_epoch` | `u32` BE |
| 3 | `reason_code` | `u16` BE (opaque) |
| 4 | `locator` | `Locator` |
| 5 | `observed_size` | `u64` BE |
| 6 | `quarantine_epoch` | `u32` BE |
| 7 | `quarantine_time_unix_nano` | `u64` BE |

#### `reset_quarantined_file` (`0x07`)

| # | Field | Encoding | Max length |
| --- | --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` | fixed |
| 2 | `expected_quarantine_epoch` | `u32` BE | fixed |
| 3 | `action` | `u8`: `0x01` `reset_to_beginning`, `0x02` `reset_to_end`, `0x03` `keep_failed`; other values fail decoding closed | fixed |
| 4 | `resulting_epoch` | `u32` BE | fixed |
| 5 | `resulting_offset` | `u64` BE | fixed |
| 6 | `new_committed_frontier_guard` | `CommittedFrontierGuard` | 34 bytes |
| 7 | `new_framing_resume` | `FramingResume` | see above |
| 8 | `new_fingerprint_len` | `u16` BE | fixed |
| 9 | `new_fingerprint_bytes` | `[u8; new_fingerprint_len]` | `FINGERPRINT_MAX_BYTES` |
| 10 | `action_time_unix_nano` | `u64` BE; audit event time, applied to operational state only for a reset action | fixed |
| 11 | `namespace_id_len` | `u16` BE | fixed |
| 12 | `namespace_id_bytes` | `[u8; namespace_id_len]`, UTF-8 | `NAMESPACE_ID_MAX_BYTES`; MUST be non-empty |
| 13 | `audit_reason_len` | `u16` BE | fixed |
| 14 | `audit_reason_bytes` | `[u8; audit_reason_len]`, UTF-8 | `AUDIT_REASON_MAX_BYTES = 1024`; MUST be non-empty (`audit_reason_len >= 1`) |

`action` is structural (its meaning determines which apply-time invariant
governs `resulting_epoch`/`resulting_offset`, see
[`reset_quarantined_file` semantics](#reset_quarantined_file) below), so an
unrecognized value fails decoding closed. `audit_reason` is always present
and always non-empty in this operation, because every quarantine reset is by
definition an operator-authorized administrative action (the
[Phase 1 administrative reset path](filelog-receiver-phase1-spec.md#quarantine-and-administrative-recovery)).
`namespace_id_bytes` carries the exact `checkpoint.id` selected by the
administrative caller and is validated before record lookup, matching the
administrative `remove_file` guard below.

#### `remove_file` (`0x08`)

| # | Field | Encoding | Presence / max length |
| --- | --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` | always |
| 2 | `expected_file_epoch` | `u32` BE | always |
| 3 | `expected_prior_state` | `u8`: `0x01` `Active`, `0x02` `RotatedFinalized`, `0x03` `Quarantined`; other values fail decoding closed | always |
| 4 | `removal_reason` | `u16` BE (opaque) | always |
| 5 | `removal_time_unix_nano` | `u64` BE | always |
| 6 | `administrative` | `u8` bool | always |
| 7 | `namespace_id_len` | `u16` BE | always; MUST be `0` when `administrative == 0x00` |
| 8 | `namespace_id_bytes` | `[u8; namespace_id_len]`, UTF-8 | `NAMESPACE_ID_MAX_BYTES = 255`; MUST be non-empty when `administrative == 0x01`, MUST be absent (length `0`) otherwise |
| 9 | `audit_reason_len` | `u16` BE | always; MUST be `0` when `administrative == 0x00` |
| 10 | `audit_reason_bytes` | `[u8; audit_reason_len]`, UTF-8 | `AUDIT_REASON_MAX_BYTES = 1024`; MUST be non-empty when `administrative == 0x01`, MUST be absent otherwise |

`namespace_id_bytes` carries the exact `checkpoint.id` the administrative
caller believes they are operating on. Apply-time validation (below) rejects
the operation if it does not match the actual namespace the WAL belongs to,
giving the "exact checkpoint namespace, exact `file_id`, and an audit reason"
requirement a concrete, checkable wire representation rather than relying on
file-path convention alone.

## Maximum encoded lengths (summary)

| Constant | Value | Applies to |
| --- | --- | --- |
| `FINGERPRINT_MAX_BYTES` | `65535` (`u16::MAX`) | `fingerprint_bytes`, `expected_fingerprint_bytes`, `new_fingerprint_bytes` |
| `COMMITTED_FRONTIER_GUARD_WINDOW_BYTES` | `64` | raw source bytes preceding committed progress |
| `ADVISORY_PATH_STORED_MAX_BYTES` | `4096` | `AdvisoryPath.stored_path_bytes` |
| `AUDIT_REASON_MAX_BYTES` | `1024` | `audit_reason_bytes` |
| `NAMESPACE_ID_MAX_BYTES` | `255` | `namespace_id_bytes` |
| `SNAPSHOT_MAX_RECORD_PAYLOAD_BYTES` | `69854` | Quarantined record with maximum guard, locator, continuation, fingerprint, and advisory path |
| `SNAPSHOT_MAX_RECORD_FRAME_BYTES` | `69862` | Length + maximum snapshot payload + CRC |
| `WAL_MAX_OPS_PER_TX` | `4096` | `op_count` per transaction |
| `WAL_MAX_NON_PROGRESS_OPS_PER_TX` | `256` | operation count for registration, metadata, fingerprint, reset, quarantine, and removal transactions |
| `WAL_HEADER_BYTES` | `56` | Fixed WAL generation header included in artifact-size accounting |
| `TX_HEADER_BYTES` | `36` | Fixed transaction envelope header |
| `TX_MIN_BODY_BYTES` | `34` | One minimum strict fingerprint-extension operation frame |
| `TX_MIN_FRAME_BYTES` | `74` | Header + minimum body + frame CRC |
| `MAX_OPERATION_PAYLOAD_BYTES` | `131095` | Structural decode/allocation ceiling for an `update_fingerprint` payload |
| `MAX_OPERATION_FRAME_BYTES` | `131103` | Length + structural payload ceiling + operation CRC |
| `MAX_VALID_UPDATE_FINGERPRINT_PAYLOAD_BYTES` | `131094` | Maximum semantically valid strict fingerprint-extension payload |
| `MAX_VALID_UPDATE_FINGERPRINT_FRAME_BYTES` | `131102` | Length + maximum valid strict-extension payload + operation CRC |
| `REGISTER_FILE_MAX_OP_PAYLOAD_BYTES` | `69812` | Maximum `register_file` payload with guard and required `Clean` resume |
| `WAL_MAX_TX_BODY_BYTES` | `16777216` | Hard 16 MiB body cap for every transaction class |
| `WAL_MAX_TX_FRAME_BYTES` | `16777256` | 36-byte header + maximum body + frame CRC |
| `UPDATE_PROGRESS_MAX_OP_PAYLOAD_BYTES` | `101` | Maximum `update_progress` payload with guard and `Continuation` resume |
| `UPDATE_PROGRESS_MAX_OP_FRAME_BYTES` | `109` | Length + maximum payload + CRC |
| `MAX_PROGRESS_TX_BODY_BYTES` | `446464` | 4,096 maximum progress operation frames |
| `MAX_PROGRESS_TX_FRAME_BYTES` | `446504` | 36-byte header + maximum progress body + frame CRC |
| framing-profile pattern | `4096` | canonical serialization pattern bytes (see below) |

The Phase 1 behavioral contract reserves zero non-progress operations in an
Ack or explicit-drop transaction, so its distinct-file delta maximum is
exactly `WAL_MAX_OPS_PER_TX = 4096`. The maximum values above derive as:

```text
update_progress payload =
  1 op_code + 16 file_id + 8 expected_offset + 4 epoch
  + 8 new_offset + 34 CommittedFrontierGuard
  + 21 maximum FramingResume
  + 8 last_seen + 1 finalize
  = 101 bytes

maximum progress transaction body =
  4096 * (4 + 101 + 4)
  = 446464 bytes

maximum progress transaction frame =
  36 header + 446464 body + 4 frame CRC
  = 446504 bytes

structural maximum operation payload =
  update_fingerprint fixed fields + two 65535-byte fingerprints
  = 131095 bytes

maximum valid strict fingerprint-extension payload =
  fixed fields + 65534 expected bytes + 65535 new bytes
  = 131094 bytes

maximum valid strict fingerprint-extension frame =
  4 op length + 131094 payload + 4 op CRC
  = 131102 bytes

minimum transaction body =
  one update_fingerprint frame extending an empty fingerprint by one byte
  = 4 op length + 26 payload + 4 op CRC
  = 34 bytes

maximum snapshot record payload =
  maximum fixed/quarantine fields + 65535 fingerprint
  + 34 guard + 25 locator + 21 resume + (44 + 4096) advisory path
  = 69854 bytes

maximum register_file payload =
  op code + maximum Active registration fields + 34-byte guard
  + one-byte Clean resume
  = 69812 bytes

WAL_MAX_TX_BODY_BYTES = 16 * 1024 * 1024 = 16777216 bytes

WAL_MAX_TX_FRAME_BYTES = 36 + 16777216 + 4 = 16777256 bytes
```

`MAX_OPERATION_PAYLOAD_BYTES` and `MAX_OPERATION_FRAME_BYTES` are structural
decode/allocation ceilings: parsing must bound both length fields before the
strict-extension relationship can be evaluated, so they include the
semantically invalid equal-maximum pair. The separate maximum-valid constants
describe encoder output. The remaining length-typed fields (`record_count:
u32`, `body_len: u32`, `op_len: u32`) are
validated against these constants and the bytes actually present before
allocation or slicing. Higher layers additionally constrain tracked files and
artifact sizes through configuration.

## CRC-32C coverage

| Checksum field | Covers |
| --- | --- |
| `CURRENT.marker_crc32c` | bytes `[0, 20)` of the marker (magic, format_version, flags, generation) |
| snapshot `header_crc32c` | bytes `[0, 56)` of the snapshot header |
| snapshot `record_crc32c` | the record's own 4-byte `record_len` field followed by its `payload` |
| snapshot `footer_crc32c` | bytes `[0, 20)` of the footer (footer_magic, total_record_bytes, record_count_echo) |
| WAL `header_crc32c` | bytes `[0, 52)` of the WAL header |
| transaction `header_crc32c` | fixed transaction-header bytes `[0, 32)` |
| transaction `frame_crc32c` | complete 36-byte transaction header followed by exactly `body_len` operation bytes |
| `op_crc32c` | the operation's own 4-byte `op_len` field followed by its `op_payload` |

The independently checked length complement and header CRC are validated before
`body_len` can classify a suffix as torn. The frame CRC then protects the
validated header and operation body together. Operation CRCs independently
protect their own length and payload.

## Replay preconditions, idempotency, and exact transition restrictions

Replay processes one WAL generation's transactions in strictly increasing
sequence order against an in-memory table keyed by `file_id`, seeded from the
snapshot. Each operation below states its precondition against that table (a
missing precondition match is either an "impossible transition" fail-closed
error or an explicitly documented idempotent no-op) and its effect.

The format does not store a last-filesystem-synced sequence number. Recovery
therefore replays every complete valid transaction present in order, including
a later Ack-authorized transaction that survived without a guaranteed sync.
The behavioral durable frontier is a guaranteed recovery floor, not a replay
cap. The defined torn-tail exception may discard an incomplete final
transaction; a structurally complete transaction with an invalid CRC fails
closed.

A general restriction that holds for every operation: identity fields
(`file_id`) are the key and are never mutated by any operation; an operation
that is not `register_file` never creates a record, and an operation that is
not `remove_file` never deletes one.

Every transaction is first applied to a staged copy or equivalent reversible
view. Before the transaction becomes visible, the resulting complete table
MUST satisfy the snapshot reachable-state invariants, including global live
locator uniqueness. No operation order may expose a successful prefix, and a
transaction that would leave two `Active`/`Quarantined` records claiming one
locator fails closed as a whole.

Before encoding any lifecycle, epoch, interpretation, reset, or removal
transition, the runtime must satisfy the behavioral
[unresolved-delta invariant](filelog-receiver-phase1-spec.md#universal-unresolved-delta-ordering).
The byte guards make stale replay fail closed but do not replace that runtime
ordering proof.

### `register_file`

- Precondition: `file_id` is absent from the table, **or** `file_id` is
  present with `lifecycle_state == Active`, no quarantine evidence,
  `file_epoch == 1`, and every field carried by this operation bit-for-bit
  equal to the corresponding stored field (a benign replay of an
  already-durable registration). Any other lifecycle, evidence, epoch, or
  field collision fails replay closed.
- `file_epoch` MUST be exactly `1`; any other value is an impossible
  transition and fails replay closed (registration always begins a file's
  first epoch).
- `framing_resume` MUST be `Clean`; `Continuation` at registration is an
  impossible transition.
- A version-1 producer MUST write `framing_profile_version = 1`. A decoder can
  preserve another nonzero value so recovery can report per-file
  `FramingProfileIncompatible`; zero is invalid.
- Every field MUST satisfy the `Active`
  [snapshot reachable-state invariants](#snapshot-reachable-state-invariants),
  including guard, locator, framing profile, fingerprint arithmetic, and
  `AdvisoryPath`.
- After profile compatibility is established, `fingerprint_len` MUST be no
  greater than configured `identity.fingerprint_bytes`.
- No other `Active` or `Quarantined` record may claim the registration's
  locator in the staged transaction state. A replacement transition removes
  the prior live claim before registering the new `file_id`; a
  `RotatedFinalized` record with that locator does not conflict.
- Effect: creates the record as `Active` with the operation's fields.

The encoder preflights and encodes a complete registration operation and
containing transaction before append. Any field, bound, invariant, collision,
or encoding failure rejects the whole registration transaction without
creating live state or making the file eligible to read. A multi-file
registration transaction is all-or-nothing.

### `update_progress`

- Precondition: `file_id` is present, `lifecycle_state == Active`, the
  stored `committed_offset == expected_committed_offset`, and the stored
  `file_epoch == expected_file_epoch`. Any mismatch (including a record that
  is `RotatedFinalized` or `Quarantined`, or absent) fails replay closed.
- `new_committed_offset` MUST be `>= expected_committed_offset` (monotonic,
  non-decreasing within the unchanged epoch); a smaller value is a
  regression and fails replay closed. This operation never changes
  `file_epoch` -- there is no field to change it, by construction, which is
  the wire-level enforcement of "an ordinary Ack-driven update cannot change
  `file_epoch`."
- Equality is valid. In particular, `finalize == 0x01` MAY carry
  `new_committed_offset == expected_committed_offset` for the behavioral
  contract's zero-delta finalization after every source delta is already
  applied.
- `new_committed_frontier_guard.window_len` MUST equal
  `min(new_committed_offset, COMMITTED_FRONTIER_GUARD_WINDOW_BYTES)`. For a
  zero-delta update it MUST equal the stored guard bit-for-bit.
- For a zero-delta update, `new_framing_resume` MUST also equal the stored
  framing resume bit-for-bit. Therefore zero-delta finalization is valid only
  when the stored resume is already `Clean`; it cannot discard a
  `Continuation`.
- When `new_framing_resume` is `Continuation`, it MUST satisfy
  `record_start_offset < new_committed_offset`, `next_fragment_index >= 1`,
  and either `record_end_offset == 0` or
  `new_committed_offset < record_end_offset`.
- For a nonzero-delta update, if the stored resume is `Continuation` with a
  nonzero `record_end_offset` and the new offset remains below that end, the
  new resume MUST remain `Continuation` for the same `record_start_offset`
  and `record_end_offset`, and `next_fragment_index` MUST be greater than the
  stored index. Installing `Clean` or unrelated continuation coordinates
  before reaching the known end fails replay closed.
- Reaching or passing a stored nonzero record end may install `Clean`, or may
  install a valid continuation for a later record when one coalesced progress
  delta spans both transitions. A later continuation's
  `record_start_offset` MUST be at least the completed stored
  `record_end_offset`.
- From stored `Clean`, a new continuation's `record_start_offset` MUST be at
  least `expected_committed_offset`.
- For a stored scan-to-LF continuation whose `record_end_offset == 0`, the
  codec cannot prove the source-derived LF boundary. A nonzero-delta update
  before that proof MUST preserve the same start and zero-ended mode with an
  advanced fragment index. After framing establishes the boundary, `Clean`
  may be encoded; a later continuation MUST start at or after
  `expected_committed_offset`.
- `new_committed_offset`, `new_committed_frontier_guard`, and
  `new_framing_resume` are applied atomically: all are updated together or the
  whole operation is rejected; there is no partially-applied state.
- Effect: updates `committed_offset`, `committed_frontier_guard`, `framing_resume`, and
  `last_seen_time_unix_nano`. If `finalize == 0x01`, additionally transitions
  `lifecycle_state` to `RotatedFinalized` (a terminal state for this
  operation: a later `update_progress` against a `RotatedFinalized` record
  fails replay closed rather than being treated as a further advance).
- When `finalize == 0x01`, `new_framing_resume` MUST be `Clean`; a continuation
  cannot be finalized. The runtime-only preconditions concerning EOF,
  retained batches, D17, Nack policy, and descriptor ownership are enforced
  before encoding as specified by the behavioral finalization contract.

### `reset_after_truncate`

- Precondition: `file_id` is present, `lifecycle_state == Active`, and the
  stored `file_epoch == expected_active_epoch`.
- `resulting_epoch` MUST equal `expected_active_epoch + 1` (checked
  addition; an overflowing epoch fails replay closed rather than wrapping).
  This is the wire-level enforcement of "the only non-administrative
  operation that may increment `file_epoch`."
- `new_committed_offset` MUST be exactly `0` and `new_framing_resume` MUST be
  `Clean` in this version (Phase 1 truncate recovery always restarts at the
  beginning of the replacement stream; there is no partial-offset truncate
  recovery policy yet). The resulting committed-frontier guard is the
  format-defined empty guard and therefore is not carried redundantly.
- The carried new fingerprint MUST be the bounded evidence derived
  from the validated replacement stream under the stored fingerprint
  configuration. It replaces the old stream's fingerprint atomically with
  the epoch, offset, guard, and framing reset; no durable new epoch may retain
  the prior stream's fingerprint. Its length MUST be no greater than the
  compatible configured `identity.fingerprint_bytes`.
- `reason_code` MUST equal `TRUNCATE_RESET_REASON_READ_NEW = 0x0001`; any
  other value fails replay closed (business-rule check, not a decode-time
  structural check -- see [reason codes](#reason-codes-are-not-structural)).
- Effect: sets `file_epoch = resulting_epoch`, `committed_offset = 0`,
  `committed_frontier_guard` to the required empty guard,
  `framing_resume = Clean`, fingerprint to the carried replacement evidence,
  and `last_seen_time_unix_nano = reset_time_unix_nano`; `lifecycle_state`
  remains `Active`. The behavioral unresolved-delta
  invariant guarantees that every current old-epoch attempt was terminal
  before this operation was encoded; only later duplicate/superseded
  completions are stale.

### `update_fingerprint`

- Precondition: `file_id` is present, `lifecycle_state == Active`, the
  stored `file_epoch == expected_file_epoch`, and the stored
  `(fingerprint_len, fingerprint_bytes)` exactly equal
  `(expected_fingerprint_len, expected_fingerprint_bytes)`.
- `new_fingerprint_len` MUST be greater than `expected_fingerprint_len`, and
  `new_fingerprint_bytes` MUST begin with the complete expected fingerprint.
  Shrink, same-length no-op, or conflicting replacement fails replay closed.
  `new_fingerprint_len` MUST also be no greater than the compatible configured
  `identity.fingerprint_bytes`.
  Epoch-changing reset operations, not this operation, own replacement-stream
  fingerprint rebasing.
- Effect: strictly extends `(fingerprint_len, fingerprint_bytes)` to
  `(new_fingerprint_len, new_fingerprint_bytes)`. Never changes `file_id`,
  `committed_offset`, `file_epoch`, `framing_resume`, or `lifecycle_state`.

### `update_metadata`

- Precondition: `file_id` is present, its `lifecycle_state` equals
  `expected_prior_state`, its `file_epoch == expected_file_epoch`, and the
  expected state is `Active` or `Quarantined`. `RotatedFinalized`, an epoch or
  state mismatch, and absent state fail replay closed (a finalized record's
  metadata is no longer mutable in v1).
- For either lifecycle, `PATH_PRESENT` replaces the stored `advisory_path` and
  `last_seen_time_unix_nano` is always replaced. Locator is not carried and
  cannot be changed by this operation. A different locator requires a new
  `file_id` under the behavioral identity contract.
- Effect: never changes `file_id`, `committed_offset`, `file_epoch`,
  `framing_resume`, or `lifecycle_state`.

### `quarantine_file`

- Precondition, ordinary case: `file_id` is present, `lifecycle_state ==
  Active`, and the stored `file_epoch == expected_file_epoch ==
  quarantine_epoch`. The operation's `locator` MUST equal the stored locator;
  a different locator fails replay closed.
- `reason_code` MUST be nonzero; zero is structurally parseable but is an
  unreachable apply-time value.
- Idempotency: if the stored record is already `Quarantined`, the stored
  `file_epoch == expected_file_epoch == quarantine_epoch`, and its
  `(reason_code, locator, observed_size, quarantine_time_unix_nano)` are all
  bit-for-bit identical to this operation's fields, replay succeeds as a
  no-op (a benign replay of an already-durable quarantine, matching
  "replaying an identical quarantine is idempotent"). If the stored record is
  `Quarantined` with **any** differing field, replay fails closed
  ("conflicting data fails closed"). Any other
  state (absent, `RotatedFinalized`, or `Active` at a different epoch) fails
  replay closed.
- Effect (ordinary case): transitions `lifecycle_state` to `Quarantined`,
  freezes `locator` as the immutable quarantine locator, and stores
  `(reason_code, observed_size, quarantine_epoch, quarantine_time_unix_nano)`
  as immutable quarantine evidence. `committed_offset`,
  `committed_frontier_guard`, `fingerprint`, and `framing_resume` are left
  exactly as they were at the moment of quarantine.
  `last_seen_time_unix_nano` is also preserved; quarantine time is recorded
  only in `quarantine_time_unix_nano` and is not a successful source
  observation.

### `reset_quarantined_file`

- Before table lookup, `namespace_id_bytes` MUST exactly equal the selected
  containing namespace's raw `checkpoint.id`. A mismatch fails
  `NamespaceMismatch`, even when `file_id` is absent.
- Precondition: `file_id` is present, `lifecycle_state == Quarantined`, and
  the stored `quarantine_epoch == expected_quarantine_epoch`. Any other state
  fails replay closed.
- `action == keep_failed` (`0x03`): `lifecycle_state` remains `Quarantined`
  (no transition). `resulting_epoch` MUST equal both stored `file_epoch` and
  `quarantine_epoch`; `resulting_offset` MUST equal stored
  `committed_offset`; and `new_framing_resume` MUST be bit-for-bit equal to
  stored `framing_resume`. `new_committed_frontier_guard` MUST be bit-for-bit
  equal to the stored guard. The carried new fingerprint MUST be
  bit-for-bit equal to the stored fingerprint. `action_time_unix_nano` is
  audit-event data in the WAL and MUST NOT update
  `last_seen_time_unix_nano`. Locator, fingerprint, advisory metadata,
  framing profile, quarantine evidence, and every other operational field
  remain byte-identical. Any attempted difference fails closed with
  `KeepFailedStateChange`.
- `action == reset_to_beginning` (`0x01`): `resulting_epoch` MUST equal
  `expected_quarantine_epoch + 1` (checked addition) and `resulting_offset`
  MUST equal exactly `0`; `new_committed_frontier_guard` MUST be the required
  empty guard.
- `action == reset_to_end` (`0x02`): `resulting_epoch` MUST equal
  `expected_quarantine_epoch + 1` (checked addition); `resulting_offset` is
  accepted as given (the codec has no independent way to verify the
  replacement stream's actual EOF);
  `new_committed_frontier_guard.window_len` MUST equal
  `min(resulting_offset, COMMITTED_FRONTIER_GUARD_WINDOW_BYTES)`, and its
  digest MUST cover exactly that final raw-source window. Supplying the
  correct offset and digest is a Phase 1 runtime responsibility.
- For either reset action, the carried new fingerprint MUST be the bounded
  evidence derived by the administrative caller from the validated current
  same-locator source stream and is installed atomically with the new epoch.
  Its length MUST be no greater than the compatible configured
  `identity.fingerprint_bytes`. The codec validates its structural bound; the
  supported administrative tool owns the handle-based source-correspondence
  proof before encoding.
- For either reset action: `new_framing_resume` MUST be `Clean`. Effect:
  `lifecycle_state` transitions to `Active`, `file_epoch = resulting_epoch`,
  `committed_offset = resulting_offset`, `committed_frontier_guard =
  new_committed_frontier_guard`, `framing_resume = Clean`, fingerprint becomes
  the carried replacement-stream evidence,
  `last_seen_time_unix_nano = action_time_unix_nano`; the quarantine evidence
  fields are cleared (no longer part of an `Active` record's persisted
  shape).
- `audit_reason` is required and is not independently validated for content
  by replay beyond the non-empty length check already enforced at decode
  time; it records the operator decision while the WAL transaction remains.
  Compaction preserves resulting operational state but need not retain the
  historical action; permanent audit history requires a separate sink.

### `remove_file`

- Before table lookup or absent-file idempotency, every administrative
  operation validates that `namespace_id_bytes` exactly equals the selected
  containing namespace. A mismatch always fails `NamespaceMismatch`, even if
  `file_id` is absent.
- After namespace validation, an absent administrative target succeeds as an
  idempotent no-op. An absent non-administrative target fails closed because
  the identity-supersede transition requires the existing locator evidence.
- Precondition when `file_id` is present: the stored `lifecycle_state` MUST
  equal `expected_prior_state`.
  - If the stored state is `Active`: the stored `file_epoch` MUST equal
    `expected_file_epoch`. `administrative == 0x00` is valid only when this
    transaction is the behavioral exact-locator mismatch transition: it
    removes the superseded record and registers a new `file_id` for the same
    staged locator atomically. The transaction preflight retains the removed
    locator for that comparison. `administrative == 0x01` is also permitted.
  - If the stored state is `RotatedFinalized`: the stored `file_epoch` MUST
    equal `expected_file_epoch` and `administrative` MUST be `0x01`.
  - If the stored state is `Quarantined`: the stored `quarantine_epoch` MUST
    equal `expected_file_epoch` (the field is reused as "the epoch value the
    caller expects to match" regardless of lifecycle state, to keep the
    operation's shape uniform), `administrative` MUST be `0x01`, and
    `administrative == 0x00` against a `Quarantined` record fails replay
    closed ("non-administrative removal cannot remove quarantined state").
  - The already-completed administrative namespace check applies identically
    to `Active`, `RotatedFinalized`, and `Quarantined` targets.
  - Any other mismatch (wrong `expected_prior_state`, wrong epoch, or a
    conflicting live record with different evidence) fails replay closed
    ("the operation removes only a matching record").
- Effect: removes the record entirely from the table.

Retention never encodes `remove_file`. It stages the complete vetted removal
set in a filtered snapshot and makes that set authoritative only through
durable compaction publication. This operation therefore has no retention
chunking or partial-removal semantics.

## Torn-tail versus corruption

The
[Phase 1 recovery algorithm](filelog-receiver-phase1-spec.md#publication-compaction-cleanup-and-recovery)
discards **only** a structurally incomplete final transaction and fails
closed on every other error. This document defines "structurally incomplete"
precisely, so the distinction is mechanical rather than a matter of judgment:

Given a WAL body being scanned sequentially for transactions, starting from
a byte offset immediately after the last successfully validated transaction
(or immediately after the header, for the first transaction), with `R` bytes
remaining in the file from that offset:

1. If `R == 0`: clean end of file. Nothing to discard.
2. If `1 <= R < 36`: the final append contains an incomplete fixed header.
   This is a **torn tail**; discard those bytes and stop.
3. If `R >= 36`: parse the complete fixed header and, before trusting
   `body_len`, validate transaction magic, envelope version, flags, reserved,
   length complement, header CRC, body-length bounds, operation count, and
   expected sequence.
   - Any failure is **corruption** (or a distinct unsupported-envelope-version
     error), even at physical EOF. Upward- or downward-corrupted lengths cannot
     become torn tails because redundancy and header CRC fail first.
   - For a valid header, compute `needed = 36 + body_len + 4` with checked
     arithmetic.
4. If `R < needed` after a valid complete header, the body or trailing frame
   CRC was only partially appended. This is a **torn tail**; discard the whole
   suffix beginning at this transaction and stop.
5. If `R >= needed`, validate `frame_crc32c` over the header and body.
   - A mismatch is corruption, even for the final frame.
   - A valid frame is then parsed as exactly `op_count` operation frames
     consuming exactly `body_len`. Any operation CRC, length, discriminant, or
     exact-consumption failure is corruption. Recovery never scans inner
     operation bytes to reinterpret an invalid outer envelope.
   - Continue at the byte immediately after this frame; any later incomplete
     suffix is classified again from that known boundary.
6. A **torn tail can only be the last transaction attempted**: once any
   transaction has been discarded as a torn tail, replay stops immediately
   (there is nothing after it to scan, by definition -- the discarded bytes
   were already everything remaining in the file, `R`).

Discarding a torn tail never uncommits an already-Acked, previously
validated transaction that happened to precede it; those were already
applied. Discarding a torn tail also never partially applies any of its own
operations -- the whole trailing partial region is dropped as a unit.

## WAL append failure and repair

The writer encodes and validates a complete transaction in bounded memory
before issuing an append. It tracks the byte offset immediately after the last
complete validated transaction. Under exclusive namespace ownership:

| Append result | Required handling |
| --- | --- |
| Definitively no bytes written | Keep the known boundary and retry within the store-failure policy |
| Known partial write | Mark the live store unavailable, reopen the WAL, validate through the known boundary, classify the suffix with the fixed envelope, truncate to the known boundary only when the suffix is a mechanically valid torn append, sync the truncation, then retry |
| Ambiguous write result | Reopen and validate: accept an exactly sequenced complete valid transaction as appended; truncate and retry only a mechanically valid torn append beginning at the known boundary; fail closed on any complete invalid header/frame or unclassifiable bytes |
| Append completed but required sync failed | Reopen and validate; if the transaction is complete and valid, do not append it again--retry the sync under the store-failure policy; if incomplete, use the torn-append repair above before applying progress |

Repair never scans for an inner operation pattern, never truncates before the
known last-valid boundary, and never converts a complete bad header CRC, length
redundancy failure, frame CRC failure, sequence error, or operation corruption
into a torn append. After startup recovery discards a permitted torn tail, the
store must truncate and sync that exact suffix before any new append.

Logical application occurs only after the append is known complete and valid.
A failed required sync may leave already applied progress newer than the
durable frontier, but it does not authorize a duplicate append or partial
operation application.

## Framing-profile canonical serialization and digest

The
[Phase 1 compatibility contract](filelog-receiver-phase1-spec.md#configuration-changes-and-resumable-state)
requires each checkpoint to store a framing-profile version and a digest
covering identity matching plus all configuration affecting record boundaries
or deterministic replay. A version or digest mismatch against resumable state
fails closed. Binding the identity profile here is also what makes a change to
`fingerprint_bytes`, `ignored_header_bytes`, or the fingerprint algorithm
detectable even when every tracked file is shorter than both the old and new
fingerprint windows. This section defines the exact canonical serialization fed
to the digest and the digest algorithm, so two independent implementations
produce byte-identical digests for the same configuration.

`framing_profile_version` (the profile/digest recipe version, stored
alongside the digest in the snapshot and in `register_file`) is `1` in this
first version of the design. It is independent of the snapshot/WAL/`CURRENT`
`format_version` field: this version tracks only the digest recipe below and
can advance when the identity or framing compatibility inputs change without
requiring a change to the snapshot/WAL byte layout.

### Canonical serialization input

```text
UTF-8("otel-arrow-filelog-framing-profile-v1\0") ||
fingerprint_profile_version : u16 BE ||
fingerprint_bytes            : u16 BE ||
ignored_header_bytes         : u32 BE ||
encoding                     : u8  ||
on_decode_error              : u8  ||
multiline_mode               : u8  ||
regex_profile_version        : u16 BE  ||
pattern_len                  : u16 BE  ||
pattern_bytes                : pattern_len bytes (UTF-8 regex source) ||
max_line_bytes               : u64 BE  ||
max_record_bytes             : u64 BE  ||
max_log_size_behavior        : u8  ||
max_multiline_lines          : u32 BE  ||
force_flush_period_millis    : u64 BE
```

Field values:

| Field | Values |
| --- | --- |
| `fingerprint_profile_version` | `1` for the raw-prefix plus fixed committed-frontier-guard evidence profile defined by Phase 1 |
| `fingerprint_bytes` | configured matching-evidence window, `16..=65535` |
| `ignored_header_bytes` | configured byte count skipped before fingerprint evidence |
| `encoding` | `0x01` utf-8, `0x02` ascii, `0x03` utf-16le, `0x04` utf-16be, `0x05` raw |
| `on_decode_error` | `0x01` preserve_raw, `0x02` replace, `0x03` fail |
| `multiline_mode` | `0x00` newline (default framing), `0x01` start-pattern, `0x02` end-pattern |
| `regex_profile_version` | `0` when `multiline_mode == 0x00`; otherwise the versioned RE2-compatible executable-subset number (`1` for `re2-v1`, whose exact syntax and ASCII Perl-class semantics are defined by the architecture contract) |
| `pattern_len` / `pattern_bytes` | `0` / empty when `multiline_mode == 0x00`; otherwise the configured pattern source, at most `4096` bytes |
| `max_log_size_behavior` | `0x01` split, `0x02` truncate |
| `force_flush_period_millis` | configured idle-flush period in milliseconds; `0` means disabled |

This list includes the complete Phase 1 identity profile
(`fingerprint_profile_version`, `fingerprint_bytes`,
`ignored_header_bytes`) and exactly the
[Phase 1 framing configuration](filelog-receiver-phase1-spec.md#fields-defaults-and-variants)
that changes record boundaries, emitted bodies, failure behavior, or replay
determinism (`encoding`, `on_decode_error`, the multiline mode and pattern,
`max_line_bytes`, `max_record_bytes`, `max_log_size_behavior`,
`max_multiline_lines`, `force_flush_period`). It deliberately excludes knobs
that affect neither identity nor framing, such as `limits.*`, `batch.*`, and
`retry.*`. `checkpoint.id` is bound separately by the header
`namespace_digest`; it is not part of the framing profile.

The digest is:

```text
framing_profile_digest = SHA-256(canonical serialization input above)
```

The construction is unkeyed and uses a fixed ASCII domain prefix (including
the trailing NUL) distinct from namespace, advisory-path, and
[fragment-ID](filelog-receiver-phase1-spec.md#split-behavior) inputs. Equal
remaining field bytes therefore do not reuse an identical SHA-256 preimage
across those purposes.

### Compatibility vectors

Given the default identity profile (`fingerprint_profile_version = 1`,
`fingerprint_bytes = 1000`, `ignored_header_bytes = 0`) and the
newline-framing default profile (`encoding = utf-8 (0x01)`,
`on_decode_error = preserve_raw (0x01)`,
`multiline_mode = newline (0x00)`, `regex_profile_version = 0`,
`pattern_len = 0`, `max_line_bytes = 1048576`, `max_record_bytes = 1048576`,
`max_log_size_behavior = split (0x01)`, `max_multiline_lines = 500`,
`force_flush_period_millis = 500`), the canonical serialization is 82 bytes
(38-byte domain-separated prefix + 2 + 2 + 4 + 1 + 1 + 1 + 2 + 2 + 0 + 8 +
8 + 1 + 4 + 8):

```text
canonical_bytes = 6f74656c2d6172726f772d66696c656c6f672d6672616d696e672d70726f66696c652d763100000103e800000000010100000000000000000000100000000000000010000001000001f400000000000001f4
framing_profile_digest = b89a44439258d045238a81d1d608cb41abede895ab1e047eef2b83898d3e0b25
```

A second vector, changing only `multiline_mode` to end-pattern
(`multiline_mode = 0x02`, `regex_profile_version = 1`, pattern
`"^END request$"`, all other fields unchanged), MUST produce a different
95-byte canonical input and digest:

```text
canonical_bytes = 6f74656c2d6172726f772d66696c656c6f672d6672616d696e672d70726f66696c652d763100000103e8000000000101020001000d5e454e442072657175657374240000000000100000000000000010000001000001f400000000000001f4
framing_profile_digest = 1c3159dd242ae99f29b6aace2f40c9d16192db416810456c5975ba8b9a020b54
```

Both are normative conformance vectors. A change to either value without a
`framing_profile_version` bump is a specification regression.
The format remains unfrozen only under the explicit pre-release rule near this
document's [Compatibility](#filelog-receiver-checkpoint-format-version-1)
declaration; after first release, the cross-version policy is mandatory.

## Unknown version, discriminator, operation, and extension behavior (summary)

| Situation | Behavior |
| --- | --- |
| `format_version` other than `1` in `CURRENT`, snapshot, or WAL header | fail closed with a distinct "unsupported version" error, checked before any record/transaction is parsed |
| `tx_envelope_version` other than `1` | fail closed before trusting `body_len` |
| nonzero file/transaction flags or reserved fields, nonzero reserved `AdvisoryPath` flags, or a nonzero reserved bit in `update_metadata.presence_flags` | fail closed |
| snapshot or WAL `namespace_digest` differs from the expected selected-namespace digest or from its peer | fail closed with `NamespaceMismatch` before applying any record or transaction |
| valid `CURRENT` names a missing, unreadable, or incomplete authoritative generation | fail closed with the corresponding distinct authoritative-generation error; never select another generation |
| unknown `locator.kind`, `AdvisoryPath.path_kind`, `framing_resume.kind`, `lifecycle_state`, `op_code`, `update_metadata.expected_prior_state`, `reset_quarantined_file.action`, or `remove_file.expected_prior_state` | fail closed; these are all structural discriminants |
| unknown `reason_code` / `removal_reason` value | accepted at decode time (opaque, non-structural); may still be rejected by apply-time business rules for specific operations |
| a declared length exceeding either its documented maximum or the bytes actually remaining | fail closed before allocating or slicing |
| complete transaction header with invalid length complement or header CRC | corruption, never torn-tail classification |
| any field arithmetic that would overflow its integer width (for example `expected_active_epoch + 1`, or `36 + body_len + 4`) | fail closed via checked arithmetic; never wraps |
| CRC-valid snapshot record violating a reachable-state invariant | fail closed with `InvalidSnapshotState` before WAL replay |
| trailing bytes after a structurally complete snapshot | fail closed (no torn-tail leniency for snapshots) |
| incomplete final transaction header, or complete valid header with incomplete body/frame CRC | discarded as the sole torn-tail exception; replay stops there |
| complete transaction header or frame with invalid CRC, even if last | fail closed (corruption, not a torn tail) |
| any "extension" bytes beyond defined record/operation fields, or transaction body bytes not consumed by exactly `op_count` operations | fail closed; v1 defines no trailing extension area |

v1 does not define a TLV-style forward-compatible extension mechanism for
individual records or operations. A future version that needs additional
fields bumps `format_version` and defines the new layout explicitly and
completely in a superseding version of this document, per
[Cross-version and migration behavior](#cross-version-and-migration-behavior).
This is a deliberate simplicity/safety choice consistent with this format's
fail-closed posture: guessing at how to skip unknown trailing bytes in an
unversioned extension area is exactly the kind of silent, unverifiable
behavior this format is designed to avoid.

## Administrative reset and removal representation

Administrative operator actions use the two operations below and are
represented distinctly on the wire so that a checkpoint auditor or migration
tool can scan a WAL and find every administrative action without ambiguity:

- **`reset_quarantined_file`** (`0x07`) is unconditionally administrative:
  every instance carries a mandatory, non-empty `namespace_id` and
  `audit_reason`. Replay validates the exact namespace before record lookup.
  There is no "ordinary" variant of this operation; releasing quarantine is
  always an explicit, audited action, whether the caller chooses
  `reset_to_beginning`, `reset_to_end`, or `keep_failed`.
- **`remove_file`** (`0x08`) is conditionally administrative, distinguished
  by its `administrative` byte. When `administrative == 0x01`, it MUST also
  carry a non-empty `namespace_id` (the exact `checkpoint.id` being
  targeted) and a non-empty `audit_reason`; replay independently checks the
  supplied `namespace_id` against the namespace the WAL actually belongs to,
  so an administrative removal recorded in (or replayed against) the wrong
  checkpoint fails closed rather than silently applying. This is the only
  path capable of removing a `Quarantined` record. Its non-administrative form
  is reserved for the exact-locator identity-supersede transition; retention
  uses filtered compaction and never encodes this operation.

These encodings do not define or authorize a CLI or API. A separately reviewed
engine administrative interface or offline tool MUST acquire exclusive
namespace ownership and append validated operations through the checkpoint
store. Operators MUST NOT edit snapshots, WAL transactions, checksums, or
`CURRENT` manually. An operable Phase 1 release with durable quarantine MUST
provide such a surface with exact namespace, `file_id`, expected lifecycle and
epoch, bounded evidence inspection, and all defined reset/removal actions.
WAL administrative entries remain operational history only until compaction;
permanent audit retention requires a separate audit sink.

These operations also do not define whole-namespace corruption reset. That
procedure requires a separate crash-safe operations design with exclusion
independent of potentially corrupt namespace authority, durable evidence backup,
namespace-incarnation and generation rules, no-replace publication, required
parent syncs, and exhaustive interrupted-reset recovery. Until that design is
approved, missing or corrupt authority remains fail closed and no tool is
authorized to replace or recreate the namespace.

## Cross-version and migration behavior

- Every stored format (`CURRENT` marker, snapshot, WAL) carries its own
  explicit `format_version`. This is the first version; there is no prior
  version to be compatible with and no automatic migration is implemented.
- A future incompatible change to byte layout, discriminant meaning, or
  field semantics requires a new `format_version` value, a superseding
  version of this document defining the new encoding completely (not as a
  diff), and explicit compatibility/migration vectors analogous to this
  document's golden vectors.
- A future incompatible transaction-envelope layout increments both the
  containing WAL `format_version` and `tx_envelope_version`; version 1 never
  mixes envelope versions in one WAL.
- A reader encountering an unrecognized `format_version` MUST fail closed
  with a distinct "unsupported version, migration required" error. It MUST
  NOT attempt to guess a compatible subset of the new layout, and MUST NOT
  silently reset or discard durable progress merely because the version is
  unrecognized.
- The `framing_profile_version` follows the same policy independently: an
  unrecognized profile version, or a recognized version whose digest does
  not match the currently configured framing profile's freshly computed
  digest, fails the affected record closed without creating a new identity.
  Resumption requires configuration restored to the exact compatible stored
  profile, audited removal, or a separately designed migration before
  the affected file can resume from persisted `framing_resume`. A
  normal configuration change never resets or removes that state; it is never
  silently reconciled.
- This document defines the on-disk checkpoint format's own migration
  policy only. It does not define a generic importer for unrelated path- or
  native-identity-keyed state from another product; the
  [architecture](filelog-receiver.md#goals-and-non-goals) treats such an
  importer, if any, as a separate, explicitly versioned tool outside the
  receiver.
- The `@v1/<checkpoint-id-hex>` namespace path is the first supported v1
  namespace layout. No released direct-ID layout exists, and version 1 defines
  no migration, fallback search, or automatic import from such a directory.

## Required conformance vectors

The implementation must provide machine-readable version 1 fixtures consumed
by its conformance tests. Those fixtures belong with the implementation and
tests rather than this design-document set. They must be generated
independently from this specification, not by round-tripping the encoder under
test, and must cover:

- `CURRENT`, an empty snapshot, and a WAL header;
- reachable `Active`, `Quarantined`, and `RotatedFinalized` snapshots;
- one complete transaction for every version 1 operation;
- valid `keep_failed` state preservation; and
- structurally valid `keep_failed` state mutation rejected as
  `KeepFailedStateChange`.

Independent generation and validation use standard SHA-256 and reflected
Castagnoli CRC-32C, first checking
`CRC-32C("123456789") == 0xE3069283`. The fixed compatibility values are:

| Item | Expected value |
| --- | --- |
| Namespace digest for exact `checkpoint.id` bytes `app-logs` | `400aa7032f9128c39cc7e1403b8745dcccf6c9a5acfc665e908f15e798ac9531` |
| Namespace path segment for exact `checkpoint.id` bytes `AppLogs` | `4170704c6f6773` |
| Namespace path segment for exact `checkpoint.id` bytes `applogs` | `6170706c6f6773` |
| Unix path digest for `/var/log/app.log` | `337a8fdfc197d2f02179162dccb0e86c430452449e51368104bbd5cc98fca49b` |
| Windows UTF-16LE path digest for `C:\logs\app.log` | `eaf5f1242c984fbf5c1ec523bd5dd9d1fa8c21b7f09a9394ee7b74b3ecdb8357` |
| Unix `UnixBytes` digest for 5,000 bytes of `0x78` (digest covers all bytes; stored path is the final 4,096-byte suffix) | `4edffb8c0486f5658b188d349af1b47270dc02bc0459b60dbfd3c314d9ecffa2` |
| Empty committed-frontier guard at offset zero | `be47d023a06e82fd6da2daa0631547d6eca297b7ac532cba6471ab90829ec5b9` |
| Committed-frontier guard for raw bytes `abc\n` at offset 4 | `23321df310e76dad74d895ad8e8e99d64f331fa350d4117f1f818a755d0a306a` |
| Default framing-profile digest | `b89a44439258d045238a81d1d608cb41abede895ab1e047eef2b83898d3e0b25` |
| End-pattern framing-profile digest | `1c3159dd242ae99f29b6aace2f40c9d16192db416810456c5975ba8b9a020b54` |

Negative conformance cases mutate a valid fixture and recompute enclosing CRCs
only when the case is intended to reach logical-state validation. Required
cases include every snapshot reachable-state invariant; duplicate `file_id`;
incomplete fixed transaction header or body; upward and downward body-length
corruption; bad header, operation, or frame CRC; sequence gaps; unsupported
versions and operations; append repair; and rejection before encoding a
4,097th progress operation.

All round-trip, reachable-state, corruption, torn-write, append-repair,
namespace, cross-version, cross-platform, publication, and recovery tests and
their implementation-owned fixtures must remain in lockstep with this
specification and the framing-profile compatibility vectors above.
