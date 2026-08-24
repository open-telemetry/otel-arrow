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
| [Filelog Receiver Design](filelog-receiver.md) | Architecture, scope, guarantees, decisions, and tradeoffs |
| [Filelog Receiver Phase 1 Behavioral Specification](filelog-receiver-phase1-spec.md) | Exact Phase 1 runtime behavior and state transitions |
| This document | Exact durable byte format and replay representation |

The architecture remains authoritative for system boundaries and accepted
compromises. This specification refines those decisions; it does not override
them.

**Compatibility:** version 1 is the first version of this format. There is no
prior format to migrate from. A conforming implementation MUST reject any
other `format_version` value in any header (snapshot, WAL, or `CURRENT`
marker) as an unsupported-version error, distinct from corruption. See
[Cross-version and migration behavior](#cross-version-and-migration-behavior).

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
- **Variable-length byte fields:** a `u16` big-endian length prefix followed
  by exactly that many bytes. Every such field has a documented maximum
  length in this specification. A decoder MUST validate the declared length
  against both the field's documented maximum and the number of bytes
  actually remaining in the input **before** allocating or slicing, using
  checked arithmetic. A declared length that exceeds either bound is a
  decode error; it is never silently truncated or wrapped.
- **Self-delimiting records:** every snapshot record, WAL transaction, and WAL
  operation carries its own length prefix and its own CRC-32C, so it can be
  parsed and validated independently of anything before or after it in the
  file.
- **Checksum:** CRC-32C, i.e. the Castagnoli polynomial variant (poly
  `0x1EDC6F41` normal / `0x82F63B78` reflected, init `0xFFFFFFFF`, reflected
  input and output, xorout `0xFFFFFFFF`; this is the same parametrization as
  iSCSI CRC-32C). This is **not** `crc32fast`'s default IEEE 802.3 polynomial
  (`0x04C11DB7`); an implementation MUST use a Castagnoli-parametrized CRC-32
  implementation (for example the `crc` crate's `CRC_32_ISCSI` catalog
  entry). Reference vector: `CRC-32C("123456789") = 0xE3069283`.
- **Digest:** SHA-256 (FIPS 180-4), 32 raw bytes, used only for the
  framing-profile digest (see
  [Framing-profile canonical serialization and digest](#framing-profile-canonical-serialization-and-digest)).
  It is not used as a checksum for structural integrity; CRC-32C alone
  guards structural integrity, matching the rest of the durability code in
  this repository.

## Namespace and active-generation selection

The on-disk namespace layout implements the
[Phase 1 checkpoint semantic contract](filelog-receiver-phase1-spec.md#checkpoint-semantic-contract):

```text
${engine.state_dir}/filelog/<checkpoint.id>/
  CURRENT
  offsets-<generation>.snapshot
  offsets-<generation>.wal
  ownership.lock
```

- `<checkpoint.id>` is percent-encoded using the existing journald path
  convention (see `journald_receiver::checkpoint::encode_path_segment`); this
  document does not change that convention.
- `<generation>` is the ASCII decimal rendering of a `u64` generation number
  with no leading zeros (`0`, `1`, `2`, ... `18446744073709551615`). A
  generation number is assigned once, at compaction time, and is never
  reused; the pair of files `offsets-<generation>.snapshot` and
  `offsets-<generation>.wal` sharing a generation number are always read and
  written together.
- `CURRENT` is a small fixed-width binary marker (not free-form text) that
  names the active generation. Its exact layout is defined in
  [The `CURRENT` marker](#the-current-marker).
- `ownership.lock` is an empty lock file used only for advisory OS locking
  (POSIX `flock`/`fcntl` or Windows `LockFileEx`, per architecture decision
  [D15](filelog-receiver.md#decisions-requested)); it has no format of its
  own and is out of scope for this document.

Recovery always reads `CURRENT` first to select the generation, then loads
`offsets-<generation>.snapshot` as the recovery base, then replays
`offsets-<generation>.wal` from sequence `1`. A generation directory MAY
contain snapshot/WAL files for more than one generation simultaneously during
compaction (the previous generation stays present and valid until `CURRENT`
is atomically repointed); this document only defines the byte format of each
individual file, not the atomic-replacement procedure for `CURRENT` itself,
which is a durable-checkpoint-store concern.

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
of these are corruption/fail-closed conditions; there is no torn-tail
leniency for `CURRENT` because it is written and synced as a single small
atomic replacement, never appended to.

## Magic values, versions, and fixed widths at a glance

| File | Magic (8 bytes) | Header width | Footer |
| --- | --- | --- | --- |
| `CURRENT` marker | `"FLOGCUR\0"` | 24 bytes (whole file) | none |
| Snapshot | `"FLOGSNP\0"` | 28 bytes | 24 bytes, magic `"FLOGSFT\0"` |
| WAL | `"FLOGWAL\0"` | 24 bytes | none (append-only) |

`format_version` is `u16` and is `1` for every header in this version. The
snapshot/WAL/`CURRENT` format version is a single coherent number for the
whole on-disk encoding; it is distinct from `framing_profile_version`, which
versions only the framing-profile canonical serialization and digest
algorithm (see below) and can in principle advance independently.

## Snapshot file format

### Snapshot header (28 bytes)

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 8 | `magic` | `"FLOGSNP\0"` |
| 8 | 2 | `format_version` | `u16` BE, `1` |
| 10 | 2 | `flags` | `u16` BE, reserved, MUST be `0` |
| 12 | 8 | `generation` | `u64` BE; MUST equal the generation encoded in the file name |
| 20 | 4 | `record_count` | `u32` BE; number of snapshot records that follow |
| 24 | 4 | `header_crc32c` | `u32` BE, CRC-32C over bytes `[0, 24)` |

### Snapshot record (self-delimiting)

```text
record_len        : u32 BE                     -- length of `payload` in bytes
payload            : record_len bytes
record_crc32c      : u32 BE                     -- CRC-32C over (record_len as 4 BE bytes) || payload
```

`payload` field order (this is the exact, implementation-ready field order;
an encoder MUST write fields in this order and a decoder MUST read them in
this order):

| # | Field | Encoding | Max length |
| --- | --- | --- | --- |
| 1 | `file_id` | `[u8; 16]`, opaque | fixed |
| 2 | `file_epoch` | `u32` BE | fixed |
| 3 | `committed_offset` | `u64` BE | fixed |
| 4 | `fingerprint_len` | `u16` BE | fixed |
| 5 | `fingerprint_bytes` | `[u8; fingerprint_len]` | `FINGERPRINT_MAX_BYTES = 65535` |
| 6 | `ignored_header_bytes` | `u32` BE | fixed |
| 7 | `locator` | [`Locator`](#locator-encoding) | see below |
| 8 | `framing_profile_version` | `u16` BE | fixed |
| 9 | `framing_profile_digest` | `[u8; 32]`, opaque (SHA-256) | fixed |
| 10 | `framing_resume` | [`FramingResume`](#framingresume-encoding) | see below |
| 11 | `lifecycle_state` | `u8` discriminant, see [Lifecycle state](#lifecycle-state-discriminant) | fixed |
| 12 | `quarantine_evidence` | present iff `lifecycle_state == Quarantined`, see below | see below |
| 13 | `last_seen_time_unix_nano` | `u64` BE | fixed |
| 14 | `advisory_path_len` | `u16` BE | fixed |
| 15 | `advisory_path_bytes` | `[u8; advisory_path_len]`, opaque bytes (not required to be UTF-8; paths are matching/advisory evidence, not a text contract) | `ADVISORY_PATH_MAX_BYTES = 4096` |

`file_id` is the record's key and MUST be unique across every record in a
single snapshot file. An encoder MUST refuse to write two records sharing a
`file_id`, and a decoder MUST fail closed (rather than keeping only the
last-seen record for that key) if it encounters two records sharing a
`file_id`.

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
field at position 7; `update_metadata` is defined below to never change it
for a quarantined record.

`quarantine_evidence` presence and `lifecycle_state` MUST agree in both
directions: an encoder MUST refuse to write a record whose `lifecycle_state`
is `Quarantined` but which carries no evidence, and MUST equally refuse to
write a record whose `lifecycle_state` is not `Quarantined` but which
carries evidence anyway. Both are structural encode-time failures, not
debug-only assertions, since a decoder has no way to recover the correct
shape from an already-inconsistent value.

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

- fewer than 28 bytes available for the header, or header magic/version/flags/CRC invalid;
- fewer than `record_count` complete, individually CRC-valid records available;
- a record whose declared `record_len` exceeds the remaining buffer;
- two records declaring the same `file_id`;
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
  normalized representation, and higher-level policy decides what a
  receiver does when it encounters this value.
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

## `FramingResume` encoding

```text
kind : u8
```

| `kind` | Name | Additional fields | Total encoded size |
| --- | --- | --- | --- |
| `0x00` | `Clean` | none | 1 byte |
| `0x01` | `Continuation` | `record_start_offset: u64` BE, `next_fragment_index: u32` BE | 13 bytes |
| `0x02`..`0xFF` | reserved | -- | decode fails closed |

`Clean` is the common durable resume state: the next complete source unit
starts a new logical record. `Continuation` is the split-record durable resume
state: the original record's start offset and the next fragment index to emit,
both of which are required to reconstruct
`otel_arrow.filelog.fragment.id` and `.index` deterministically after
restart. `kind` is structural; `0x02`..`0xFF` fail decoding closed.

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
| `0x0004`-`0x00FF` | reserved for future built-in reasons; profile incompatibility fails closed without mutating the record |
| `0x0100`-`0xFFFF` | available for distribution- or extension-defined reasons |

`removal_reason` has no assigned values in v1 beyond the requirement that an
encoder MUST NOT write `0x0000`; `0x0000` is reserved exactly as above.

## WAL file format

### WAL header (24 bytes)

| Offset | Size | Field | Value |
| --- | --- | --- | --- |
| 0 | 8 | `magic` | `"FLOGWAL\0"` |
| 8 | 2 | `format_version` | `u16` BE, `1` |
| 10 | 2 | `flags` | `u16` BE, reserved, MUST be `0` |
| 12 | 8 | `generation` | `u64` BE; MUST equal the generation encoded in the file name |
| 20 | 4 | `header_crc32c` | `u32` BE, CRC-32C over bytes `[0, 20)` |

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
tx_len       : u32 BE              -- length of the transaction body in bytes
tx_body       : tx_len bytes
tx_crc32c     : u32 BE              -- CRC-32C over (tx_len as 4 BE bytes) || tx_body
```

`tx_body` layout:

```text
sequence  : u64 BE
op_count  : u16 BE                  -- 1..=WAL_MAX_OPS_PER_TX (4096); 0 is invalid
ops       : op_count Operation entries (self-delimiting, see below)
```

- **Sequences** start at `1` for the first transaction ever written into a
  fresh WAL generation and increase by exactly `1` for every subsequent
  transaction, with no gaps and no repeats. A sequence that is not exactly
  `previous + 1` is an ordering error and fails replay closed; it is never
  treated as a torn tail (a torn tail is only a **length/CRC** availability
  problem, defined precisely below, not a semantic ordering problem).
- **Atomicity:** a transaction's operations become visible only as a
  complete, validated set. A decoder MUST NOT expose any operation from a
  transaction whose `tx_crc32c` does not validate, and MUST NOT expose a
  partial prefix of a transaction's operations.
- `op_count == 0` is rejected: every transaction carries at least one
  operation. This keeps the "smallest replayable unit" concept simple and
  matches the behavioral requirement that one transaction contains a
  [bounded set of operations](filelog-receiver-phase1-spec.md#logical-operations).

### Operation framing (self-delimiting)

```text
op_len       : u32 BE               -- length of op_payload in bytes
op_payload    : op_len bytes         -- op_code (u8) || operation-specific fields
op_crc32c     : u32 BE               -- CRC-32C over (op_len as 4 BE bytes) || op_payload
```

Every operation is individually length-prefixed and individually
CRC-32C-checked, even though the enclosing transaction's `tx_crc32c` already
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
| 4 | `fingerprint_len` | `u16` BE | fixed |
| 5 | `fingerprint_bytes` | `[u8; fingerprint_len]` | `FINGERPRINT_MAX_BYTES = 65535` |
| 6 | `ignored_header_bytes` | `u32` BE | fixed |
| 7 | `locator` | `Locator` | see above |
| 8 | `framing_profile_version` | `u16` BE | fixed |
| 9 | `framing_profile_digest` | `[u8; 32]` | fixed |
| 10 | `framing_resume` | `FramingResume` | see above |
| 11 | `last_seen_time_unix_nano` | `u64` BE | fixed |
| 12 | `advisory_path_len` | `u16` BE | fixed |
| 13 | `advisory_path_bytes` | `[u8; advisory_path_len]` | `ADVISORY_PATH_MAX_BYTES = 4096` |

#### `update_progress` (`0x02`)

| # | Field | Encoding |
| --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` |
| 2 | `expected_committed_offset` | `u64` BE |
| 3 | `expected_file_epoch` | `u32` BE |
| 4 | `new_committed_offset` | `u64` BE |
| 5 | `new_framing_resume` | `FramingResume` |
| 6 | `new_last_seen_time_unix_nano` | `u64` BE |
| 7 | `finalize` | `u8` bool (`0x01` transitions to `RotatedFinalized`) |

#### `reset_after_truncate` (`0x03`)

| # | Field | Encoding |
| --- | --- | --- |
| 1 | `file_id` | `[u8; 16]` |
| 2 | `expected_active_epoch` | `u32` BE |
| 3 | `observed_truncated_size` | `u64` BE (informational evidence, not independently verified by the codec) |
| 4 | `resulting_epoch` | `u32` BE |
| 5 | `new_committed_offset` | `u64` BE |
| 6 | `new_framing_resume` | `FramingResume` |
| 7 | `reset_time_unix_nano` | `u64` BE |
| 8 | `reason_code` | `u16` BE (opaque; see [reason codes](#reason-codes-are-not-structural)) |

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
| 2 | `presence_flags` | `u8` bitfield: bit 0 (`0x01`) = `LOCATOR_PRESENT`, bit 1 (`0x02`) = `PATH_PRESENT`; other bits reserved, MUST be `0` | always |
| 3 | `locator` | `Locator` | only if `LOCATOR_PRESENT` |
| 4 | `last_seen_time_unix_nano` | `u64` BE | always |
| 5 | `advisory_path_len` | `u16` BE | only if `PATH_PRESENT` |
| 6 | `advisory_path_bytes` | `[u8; advisory_path_len]`, max `ADVISORY_PATH_MAX_BYTES = 4096` | only if `PATH_PRESENT` |

A field marked "only if" is entirely absent from the byte stream when its
presence bit is clear -- there is no placeholder or zero-length stand-in
other than for the length-prefixed `advisory_path` case, where the natural
representation of "absent" and "present with a zero-length value" would
otherwise collide; `update_metadata` therefore uses the explicit
`PATH_PRESENT` bit rather than an empty-string sentinel to distinguish "do
not touch the advisory path" from "set it to an empty value" (v1 encoders
never need the latter, but the wire format is unambiguous either way).

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
| 6 | `new_framing_resume` | `FramingResume` | see above |
| 7 | `reset_time_unix_nano` | `u64` BE | fixed |
| 8 | `audit_reason_len` | `u16` BE | fixed |
| 9 | `audit_reason_bytes` | `[u8; audit_reason_len]`, UTF-8 | `AUDIT_REASON_MAX_BYTES = 1024`; MUST be non-empty (`audit_reason_len >= 1`) |

`action` is structural (its meaning determines which apply-time invariant
governs `resulting_epoch`/`resulting_offset`, see
[`reset_quarantined_file` semantics](#reset_quarantined_file) below), so an
unrecognized value fails decoding closed. `audit_reason` is always present
and always non-empty in this operation, because every quarantine reset is by
definition an operator-authorized administrative action (the
[Phase 1 administrative reset path](filelog-receiver-phase1-spec.md#quarantine-and-administrative-recovery)).

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
| 8 | `namespace_id_bytes` | `[u8; namespace_id_len]`, UTF-8 | `NAMESPACE_ID_MAX_BYTES = 256`; MUST be non-empty when `administrative == 0x01`, MUST be absent (length `0`) otherwise |
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
| `ADVISORY_PATH_MAX_BYTES` | `4096` | `advisory_path_bytes` |
| `AUDIT_REASON_MAX_BYTES` | `1024` | `audit_reason_bytes` |
| `NAMESPACE_ID_MAX_BYTES` | `256` | `namespace_id_bytes` |
| `WAL_MAX_OPS_PER_TX` | `4096` | `op_count` per transaction |
| framing-profile pattern | `4096` | canonical serialization pattern bytes (see below) |

These are the only field-specific maximums. The remaining length-typed
fields (`record_count: u32`, `tx_len: u32`, `op_len: u32`) are bounded only by
their integer width and by the number of bytes actually present in the input
buffer; a decoder never trusts a declared length larger than what is
actually available, regardless of the field's nominal integer range. Higher
layers additionally bound the practical number of tracked files and the
practical transaction size through configuration (`limits.max_tracked_files`,
checkpoint sync/compaction policy); this document does not duplicate those
runtime policies as format constants.

## CRC-32C coverage

| Checksum field | Covers |
| --- | --- |
| `CURRENT.marker_crc32c` | bytes `[0, 20)` of the marker (magic, format_version, flags, generation) |
| snapshot `header_crc32c` | bytes `[0, 24)` of the snapshot header |
| snapshot `record_crc32c` | the record's own 4-byte `record_len` field followed by its `payload` |
| snapshot `footer_crc32c` | bytes `[0, 20)` of the footer (footer_magic, total_record_bytes, record_count_echo) |
| WAL `header_crc32c` | bytes `[0, 20)` of the WAL header |
| `tx_crc32c` | the transaction's own 4-byte `tx_len` field followed by its `tx_body` |
| `op_crc32c` | the operation's own 4-byte `op_len` field followed by its `op_payload` |

Every checksum covers its own length prefix in addition to its payload; this
prevents a corrupted length field with an otherwise-valid-looking payload
suffix from passing validation.

## Replay preconditions, idempotency, and exact transition restrictions

Replay processes one WAL generation's transactions in strictly increasing
sequence order against an in-memory table keyed by `file_id`, seeded from the
snapshot. Each operation below states its precondition against that table (a
missing precondition match is either an "impossible transition" fail-closed
error or an explicitly documented idempotent no-op) and its effect.

A general restriction that holds for every operation: identity fields
(`file_id`) are the key and are never mutated by any operation; an operation
that is not `register_file` never creates a record, and an operation that is
not `remove_file` never deletes one.

### `register_file`

- Precondition: `file_id` is absent from the table, **or** `file_id` is
  present and every persisted field of the existing record is bit-for-bit
  identical to this operation's fields (a benign replay of an
  already-durable registration). Any other collision -- an existing record
  with any differing field -- fails replay closed.
- `file_epoch` MUST be exactly `1`; any other value is an impossible
  transition and fails replay closed (registration always begins a file's
  first epoch).
- `framing_resume` MUST be `Clean`; `Continuation` at registration is an
  impossible transition.
- Effect: creates the record as `Active` with the operation's fields.

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
- `new_committed_offset` and `new_framing_resume` are applied atomically:
  both are updated together or the whole operation is rejected; there is no
  partially-applied state.
- Effect: updates `committed_offset`, `framing_resume`, and
  `last_seen_time_unix_nano`. If `finalize == 0x01`, additionally transitions
  `lifecycle_state` to `RotatedFinalized` (a terminal state for this
  operation: a later `update_progress` against a `RotatedFinalized` record
  fails replay closed rather than being treated as a further advance).

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
  recovery policy yet).
- `reason_code` MUST equal `TRUNCATE_RESET_REASON_READ_NEW = 0x0001`; any
  other value fails replay closed (business-rule check, not a decode-time
  structural check -- see [reason codes](#reason-codes-are-not-structural)).
- Effect: sets `file_epoch = resulting_epoch`, `committed_offset = 0`,
  `framing_resume = Clean`, `last_seen_time_unix_nano = reset_time_unix_nano`;
  `lifecycle_state` remains `Active`. Because the epoch strictly increases,
  any `update_progress` still carrying the prior (now stale) epoch
  necessarily fails its own `expected_file_epoch` precondition on replay,
  which is the mechanism behind "an earlier-epoch Ack cannot advance the
  resulting stream."

### `update_fingerprint`

- Precondition: `file_id` is present, `lifecycle_state == Active`, the
  stored `file_epoch == expected_file_epoch`, and the stored
  `(fingerprint_len, fingerprint_bytes)` exactly equal
  `(expected_fingerprint_len, expected_fingerprint_bytes)`.
- Effect: replaces `(fingerprint_len, fingerprint_bytes)` with
  `(new_fingerprint_len, new_fingerprint_bytes)`. Never changes `file_id`,
  `committed_offset`, `file_epoch`, `framing_resume`, or `lifecycle_state`.

### `update_metadata`

- Precondition: `file_id` is present and `lifecycle_state` is `Active` or
  `Quarantined`. `RotatedFinalized` and absent both fail replay closed (a
  finalized record's metadata is no longer mutable in v1).
- For an `Active` record: `LOCATOR_PRESENT` replaces the stored `locator`;
  `PATH_PRESENT` replaces the stored `advisory_path`; `last_seen_time_unix_nano`
  is always replaced.
- For a `Quarantined` record: `locator` (even if `LOCATOR_PRESENT` is set)
  and `lifecycle_state`/quarantine evidence are **never** modified -- only
  `last_seen_time_unix_nano` and, if `PATH_PRESENT`, `advisory_path` are
  updated. A `Quarantined` record with `LOCATOR_PRESENT` set MUST still
  decode and replay successfully; the locator payload bytes are read (to
  keep the operation self-delimiting and independently parseable) and then
  intentionally discarded rather than applied. This is the wire-level
  enforcement of "a quarantined record's recorded quarantine locator,
  lifecycle state, and failure evidence remain immutable."
- Effect: never changes `file_id`, `committed_offset`, `file_epoch`,
  `framing_resume`, or `lifecycle_state`.

### `quarantine_file`

- Precondition, ordinary case: `file_id` is present, `lifecycle_state ==
  Active`, and the stored `file_epoch == expected_file_epoch ==
  quarantine_epoch`.
- Idempotency: if the stored record is already `Quarantined` and its
  `(quarantine_epoch, reason_code, locator, observed_size,
  quarantine_time_unix_nano)` are all bit-for-bit identical to this
  operation's fields, replay succeeds as a no-op (a benign replay of an
  already-durable quarantine, matching "replaying an identical quarantine is
  idempotent"). If the stored record is `Quarantined` with **any** differing
  field, replay fails closed ("conflicting data fails closed"). Any other
  state (absent, `RotatedFinalized`, or `Active` at a different epoch) fails
  replay closed.
- Effect (ordinary case): transitions `lifecycle_state` to `Quarantined`,
  freezes `locator` as the immutable quarantine locator, and stores
  `(reason_code, observed_size, quarantine_epoch, quarantine_time_unix_nano)`
  as immutable quarantine evidence. `committed_offset`, `fingerprint`, and
  `framing_resume` are left exactly as they were at the moment of
  quarantine.

### `reset_quarantined_file`

- Precondition: `file_id` is present, `lifecycle_state == Quarantined`, and
  the stored `quarantine_epoch == expected_quarantine_epoch`. Any other state
  fails replay closed.
- `action == keep_failed` (`0x03`): `lifecycle_state` remains `Quarantined`
  (no transition). `resulting_epoch` MUST equal the stored `quarantine_epoch`
  and `resulting_offset` MUST equal the stored `committed_offset` unchanged;
  this operation exists purely as a durable, audited record of an operator's
  explicit decision not to release quarantine, and MUST NOT be usable to
  smuggle a silent state change through a nominally no-op action.
- `action == reset_to_beginning` (`0x01`): `resulting_epoch` MUST equal
  `expected_quarantine_epoch + 1` (checked addition) and `resulting_offset`
  MUST equal exactly `0`.
- `action == reset_to_end` (`0x02`): `resulting_epoch` MUST equal
  `expected_quarantine_epoch + 1` (checked addition); `resulting_offset` is
  accepted as given (the codec has no independent way to verify the
  replacement stream's actual EOF; supplying a correct value is a Phase 1
  runtime responsibility).
- For either reset action: `new_framing_resume` MUST be `Clean`. Effect:
  `lifecycle_state` transitions to `Active`, `file_epoch = resulting_epoch`,
  `committed_offset = resulting_offset`, `framing_resume = Clean`,
  `last_seen_time_unix_nano = reset_time_unix_nano`; the quarantine evidence
  fields are cleared (no longer part of an `Active` record's persisted
  shape).
- `audit_reason` is required and is not independently validated for content
  by replay beyond the non-empty length check already enforced at decode
  time; it exists for the durable audit trail, not as a machine-checked
  precondition.

### `remove_file`

- Idempotency: if `file_id` is absent from the table, replay succeeds as a
  no-op regardless of the operation's other fields ("replay against an
  already absent `file_id` is idempotent").
- Precondition when `file_id` is present: the stored `lifecycle_state` MUST
  equal `expected_prior_state`.
  - If the stored state is `Active` or `RotatedFinalized`: the stored
    `file_epoch` MUST equal `expected_file_epoch`. `administrative` MAY be
    `0x00` or `0x01` (ordinary retention may remove either state).
  - If the stored state is `Quarantined`: the stored `quarantine_epoch` MUST
    equal `expected_file_epoch` (the field is reused as "the epoch value the
    caller expects to match" regardless of lifecycle state, to keep the
    operation's shape uniform), `administrative` MUST be `0x01`, and
    `administrative == 0x00` against a `Quarantined` record fails replay
    closed ("ordinary retention cannot remove quarantined state").
  - **Namespace validation applies whenever `administrative == 0x01`,
    regardless of the stored lifecycle state, not only when removing a
    `Quarantined` record.** Whenever `administrative == 0x01`,
    `namespace_id_bytes` MUST exactly equal the checkpoint namespace's
    `checkpoint.id` that the containing WAL generation belongs to (supplied
    to the replay function by its caller, since a single WAL file has one
    fixed, known namespace); a mismatched `namespace_id_bytes` fails replay
    closed with the same `NamespaceMismatch` error whether the target record
    is `Active`, `RotatedFinalized`, or `Quarantined`. This prevents an
    administrative removal recorded against the wrong namespace from
    silently succeeding against an `Active` or `RotatedFinalized` record
    just because namespace checking was previously reachable only via the
    `Quarantined` removal path.
  - Any other mismatch (wrong `expected_prior_state`, wrong epoch, or a
    conflicting live record with different evidence) fails replay closed
    ("the operation removes only a matching record").
- Effect: removes the record entirely from the table.

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
2. If `1 <= R < 4`: there are not enough bytes to read a complete `tx_len`
   field. This is a **torn tail**: discard the remaining `R` bytes and stop
   replay. This is the only transaction that may be discarded.
3. If `R >= 4`: read `tx_len`. Let `needed = tx_len + 4` (the transaction
   body plus the trailing `tx_crc32c`), computed with a checked (overflow
   -detecting) addition.
   - If `R - 4 < needed`, i.e. fewer bytes remain than the declared frame
     requires: this is a **torn tail**: discard the remaining `R` bytes
     (including the `tx_len` field just read) and stop replay.
   - If `R - 4 >= needed`: the complete frame (`tx_len` bytes of body plus 4
     bytes of CRC) is physically present. Validate `tx_crc32c` against the
     actual bytes.
     - If the CRC does not match: this is **corruption**, not a torn tail,
       even if this is the last transaction in the file. A torn write, by
       construction, never has a complete, self-consistent frame with a
       merely-wrong checksum; a complete frame with a bad checksum indicates
       genuine bit-level corruption (or a non-append modification of an
       already-written region) and fails recovery closed.
     - If the CRC matches: parse and validate `sequence`, `op_count`, and
       each operation as normal. A structural failure at this point (for
       example an operation's own `op_crc32c` not matching, or a length
       field pointing past the already-validated transaction boundary) is
       also corruption, because the enclosing `tx_crc32c` already proved
       these exact bytes were written intentionally; it fails recovery
       closed rather than being treated as a torn tail.
4. A **torn tail can only be the last transaction attempted**: once any
   transaction has been discarded as a torn tail, replay stops immediately
   (there is nothing after it to scan, by definition -- the discarded bytes
   were already everything remaining in the file, `R`).

Discarding a torn tail never uncommits an already-Ack'd, previously
validated transaction that happened to precede it; those were already
applied. Discarding a torn tail also never partially applies any of its own
operations -- the whole trailing partial region is dropped as a unit.

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
alongside the digest in the snapshot and in `register_file`) is `3` in this
version of this document. It is independent of the snapshot/WAL/`CURRENT`
`format_version` field: this version tracks only the digest recipe below and
can advance when the identity or framing compatibility inputs change without
requiring a change to the snapshot/WAL byte layout.

### Canonical serialization input

```text
UTF-8("otel-arrow-filelog-framing-profile-v3\0") ||
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
| `fingerprint_profile_version` | `1` for the raw-prefix evidence algorithm defined by Phase 1 |
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
that affect neither identity nor framing, such as `checkpoint.id`, `limits.*`,
`batch.*`, and `retry.*`.

The digest is:

```text
framing_profile_digest = SHA-256(canonical serialization input above)
```

The construction is unkeyed and domain-separated with the same style of
fixed ASCII prefix (including the trailing NUL) used by the
[fragment-ID construction](filelog-receiver-phase1-spec.md#split-behavior),
so the two digests can never collide with each other by construction even if
their remaining field shapes were ever to coincide.

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
canonical_bytes = 6f74656c2d6172726f772d66696c656c6f672d6672616d696e672d70726f66696c652d763300000103e800000000010100000000000000000000100000000000000010000001000001f400000000000001f4
framing_profile_digest = 84cd122c62b3a4aea428db9f2c41166ed9af1f8087ab3e562f8033a9eedcf513
```

A second vector, changing only `multiline_mode` to end-pattern
(`multiline_mode = 0x02`, `regex_profile_version = 1`, pattern
`"^END request$"`, all other fields unchanged), MUST produce a different
95-byte canonical input and digest:

```text
canonical_bytes = 6f74656c2d6172726f772d66696c656c6f672d6672616d696e672d70726f66696c652d763300000103e8000000000101020001000d5e454e442072657175657374240000000000100000000000000010000001000001f400000000000001f4
framing_profile_digest = 49834e9d951d6c68f351d1a51f70cca9ed4da0b8eef18670607b71aa15c03637
```

Both vectors are executable conformance tests in
`crates/core-nodes/src/receivers/filelog_receiver/checkpoint/framing_profile.rs`
and MUST continue to match exactly; a change to either value without a
`framing_profile_version` bump is a specification regression.

## Unknown version, discriminator, operation, and extension behavior (summary)

| Situation | Behavior |
| --- | --- |
| `format_version` other than `1` in `CURRENT`, snapshot, or WAL header | fail closed with a distinct "unsupported version" error, checked before any record/transaction is parsed |
| nonzero header `flags`, or a nonzero reserved bit in `update_metadata`'s `presence_flags` | fail closed (v1 defines no flag bits) |
| unknown `locator.kind`, `framing_resume.kind`, `lifecycle_state`, `op_code`, `reset_quarantined_file.action`, or `remove_file.expected_prior_state` | fail closed; these are all structural discriminants |
| unknown `reason_code` / `removal_reason` value | accepted at decode time (opaque, non-structural); may still be rejected by apply-time business rules for specific operations |
| a declared length exceeding either its documented maximum or the bytes actually remaining | fail closed before allocating or slicing |
| any field arithmetic that would overflow its integer width (for example `expected_active_epoch + 1`, or `tx_len + 4`) | fail closed via checked arithmetic; never wraps |
| trailing bytes after a structurally complete snapshot | fail closed (no torn-tail leniency for snapshots) |
| a structurally incomplete final WAL transaction | discarded (the sole torn-tail exception); replay stops there |
| a structurally complete WAL transaction with an invalid CRC, even if it is last | fail closed (corruption, not a torn tail) |
| any "extension" bytes beyond a record/operation's defined fields | there are none in v1; every record/operation is exactly as long as its defined fields require, and any extra bytes claimed by a self-delimiting length prefix but not consumed by defined fields is a decode error (the length must exactly match the sum of the fields it was declared to contain) |

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

Two operations are exclusively administrative (operator-authorized) actions,
and both are represented distinctly on the wire so that a checkpoint
auditor or migration tool can scan a WAL and find every administrative
action without ambiguity:

- **`reset_quarantined_file`** (`0x07`) is unconditionally administrative:
  every instance carries a mandatory, non-empty `audit_reason`. There is no
  "ordinary" variant of this operation; releasing quarantine is always an
  explicit, audited action, whether the caller chooses
  `reset_to_beginning`, `reset_to_end`, or `keep_failed`.
- **`remove_file`** (`0x08`) is conditionally administrative, distinguished
  by its `administrative` byte. When `administrative == 0x01`, it MUST also
  carry a non-empty `namespace_id` (the exact `checkpoint.id` being
  targeted) and a non-empty `audit_reason`; replay independently checks the
  supplied `namespace_id` against the namespace the WAL actually belongs to,
  so an administrative removal recorded in (or replayed against) the wrong
  checkpoint fails closed rather than silently applying. This is the only
  path capable of removing a `Quarantined` record, matching the
  [Phase 1 retention contract](filelog-receiver-phase1-spec.md#retention).

## Cross-version and migration behavior

- Every stored format (`CURRENT` marker, snapshot, WAL) carries its own
  explicit `format_version`. This is the first version; there is no prior
  version to be compatible with and no automatic migration is implemented.
- A future incompatible change to byte layout, discriminant meaning, or
  field semantics requires a new `format_version` value, a superseding
  version of this document defining the new encoding completely (not as a
  diff), and explicit compatibility/migration vectors analogous to this
  document's golden vectors.
- A reader encountering an unrecognized `format_version` MUST fail closed
  with a distinct "unsupported version, migration required" error. It MUST
  NOT attempt to guess a compatible subset of the new layout, and MUST NOT
  silently reset or discard durable progress merely because the version is
  unrecognized.
- The `framing_profile_version` follows the same policy independently: an
  unrecognized profile version, or a recognized version whose digest does
  not match the currently configured framing profile's freshly computed
  digest, fails closed and requires an explicit configuration change or
  administrative migration before the affected file can resume from its
  persisted `framing_resume` state. It is never silently reconciled.
- This document defines the on-disk checkpoint format's own migration
  policy only. It does not define a generic importer for unrelated path- or
  native-identity-keyed state from another product; the
  [architecture](filelog-receiver.md#goals-and-non-goals) treats such an
  importer, if any, as a separate, explicitly versioned tool outside the
  receiver.

## Golden and conformance vectors

Executable checkpoint-byte vectors live in
`crates/core-nodes/src/receivers/filelog_receiver/checkpoint/test_vectors.rs`
and are consumed by `checkpoint/tests.rs`. The two current profile-v3
canonical-byte and SHA-256 vectors live in `checkpoint/framing_profile.rs`.
All expected bytes and digests are independently computed (using a
separately verified CRC-32C implementation and standard SHA-256, not by
round-tripping through this crate's own encoder). They cover:

- encode/decode round-trip for a minimal `Active` snapshot record with a
  POSIX locator and `Clean` framing resume;
- encode/decode round-trip for a `Quarantined` snapshot record with a
  Windows volume/file-ID locator;
- a WAL generation containing a `register_file` followed by an
  `update_progress` transaction, replayed end to end;
- a WAL whose final transaction is torn (fewer trailing bytes than its
  declared length): the preceding transactions replay and the torn tail is
  discarded without error;
- a WAL whose final transaction is structurally complete but has a
  corrupted CRC: replay fails closed rather than discarding it;
- a snapshot and a WAL header each declaring an unsupported
  `format_version`: both fail closed with the unsupported-version error
  before any record or transaction is parsed;
- decoding a Windows-locator snapshot record and a POSIX-locator snapshot
  record on the same (arbitrary host) platform, demonstrating that decoding
  never depends on the host's own native locator type; and
- the two framing-profile digest compatibility vectors above.

The snapshot/WAL byte fixtures intentionally retain opaque legacy
`framing_profile_version = 1` fields so codec conformance is independent of
the current recipe. Runtime identity tests separately prove that a
non-finalized profile-v1 record fails closed against profile v3 even when no
candidate is present.

Phase 1 conformance requires these tests, plus the corresponding
round-trip/corruption/torn-write/cross-version/cross-platform/migration tests,
to pass and remain in lockstep with this specification.
