# SQL Watermark Compatibility

<!-- markdownlint-disable MD013 -->

Use this reference to choose a watermark type and understand how Oracle,
PostgreSQL, and SQL Server receivers preserve source positions. The receivers
share polling and checkpoint infrastructure, but their supported native types
and timestamp representations differ.

- **Supported**: Accepted when configuration, result metadata, and values meet the stated restrictions.
- **Rejected**: Not accepted as a watermark; validation or cursor extraction returns an error.
- **Different**: Receiver-specific behavior that must not be assumed to work identically across databases.
- **Unknown**: Behavior is not guaranteed across the database, client, or timezone configurations described.

If a query returns a NULL or out-of-range watermark, the receiver stops without
advancing the checkpoint past the last committed position; it does not skip the
invalid row. Restarting from that checkpoint encounters the same error if the
query still returns the offending row.

For connection setup and configuration examples, see the
[Oracle](../contrib-nodes/src/receivers/oracle_receiver/README.md),
[PostgreSQL](../contrib-nodes/src/receivers/postgresql_receiver/README.md), and
[SQL Server](../contrib-nodes/src/receivers/sql_server_receiver/README.md)
receiver guides.

## Scalar type matrix

Output-value support is not watermark support. A native value that can be emitted
in a log body may still be rejected as a cursor.

| Contract | Oracle | PostgreSQL | SQL Server |
| --- | --- | --- | --- |
| Signed integer scalar | Supported: native `Int64` or `NUMBER(p,0)`, `p=1..19`; exact `i64` value required | Supported: `int2`, `int4`, `int8`; exact native width required | Supported: `smallint`, `int`, `bigint`; exact native width required |
| Unsigned integer scalar | Supported: native `UInt64` or `NUMBER(p,0)`, `p=1..20`; exact `u64` value required | Rejected: `uint64` configuration | Rejected: `uint64` configuration; `tinyint` is output-only even with an `int64` configuration |
| String scalar | Supported with restrictions: `VARCHAR2`, explicit `COLLATE BINARY` predicate/order, `AL32UTF8`; nonempty, no NUL, at most 1024 UTF-8 bytes | Rejected: `string` configuration, even though text output is supported | Rejected: `string` configuration, even though text output is supported |
| Decimal/floating/boolean scalar | Rejected except the exact integral `NUMBER` cases above | Rejected, including integral `numeric` | Rejected, including scale-zero `decimal` |
| Temporal scalar | Supported: `DATE`, `TIMESTAMP(p)`, `TIMESTAMP WITH TIME ZONE(p)`, `TIMESTAMP WITH LOCAL TIME ZONE(p)`; `p=0..9` | Supported: `timestamp(p)`, `timestamptz(p)`; `p=0..6` | Supported: `datetime2(p)`; `p=0..7` |
| Other temporal keys | Rejected outside the timestamp family above | Rejected, including `date` | Rejected: `date`, `time`, `datetimeoffset`, legacy `datetime`/`smalldatetime`; `datetimeoffset` is output-only |
| SQL NULL key | Rejected for every scalar kind, including nullable result columns | Rejected at cursor extraction; arbitrary result expressions/views have nullable metadata | Rejected at cursor extraction even when nullable output values are allowed |
| Signedness/coercion | Different: unsigned decimal text is parsed as `u64`; negative/overflow/fractional text fails. Native signed cells do not become unsigned keys | Different: only signed native integer codecs; no decimal/string-to-integer cursor coercion | Different: `tinyint` output widens to `Int64`, but cursor binding deliberately has no `tinyint` branch |

Oracle `NUMBER(19,0)` and `NUMBER(20,0)` can store values outside the `i64` and
`u64` ranges. A column with an accepted type can therefore still contain an
invalid cursor value. Such rows are rejected rather than truncated or coerced.
PostgreSQL and SQL Server also reject cursor parameters outside the native
column's integer range.

The exact signed ranges are `-32768..32767` for PostgreSQL `int2` / SQL Server
`smallint`, `-2147483648..2147483647` for `int4` / `int`, and
`-9223372036854775808..9223372036854775807` for `int8` / `bigint` / Oracle's
`int64` scalar. Oracle's `uint64` scalar range is
`0..18446744073709551615`. There is no null scalar variant: an initial scalar
JSON `null` value is invalid, whereas a snapshot cursor itself serializes as
JSON `null` by design.

## Timestamp representation and ordering

| Contract | Oracle | PostgreSQL | SQL Server |
| --- | --- | --- | --- |
| Fractional precision | Different: up to nanoseconds; `DATE` requires whole seconds; binding rejects nonzero discarded digits | Different: up to microseconds; unknown/default typmod `-1` means 6; binding rejects nonzero discarded digits | Different: up to 100ns; binding rejects nonzero discarded digits at the discovered scale |
| Offset handling on initialization/bind | Converts zoned input to UTC with checked date arithmetic; binds the discovered Oracle family, not a universal zoned type | Converts RFC3339 offsets to UTC; `timestamp` binds naive UTC, `timestamptz` binds a UTC instant | Converts RFC3339 offsets to UTC; binds timezone-less `datetime2` under the UTC contract |
| Timezone-less input | Different: native parser accepts date-only and supported naive timestamp spellings; naive means UTC | Different: accepts space-separated naive date/time as UTC, not date-only input | Different: accepts space-separated naive date/time as UTC, not date-only input |
| Canonical configured timestamp | Different: native display, e.g. `2026-01-01 00:00:00.123456789 +00:00` | Different: `2026-01-01T00:00:00.123456Z` | Different: `2026-01-01T00:00:00.1234567Z` |
| Fetched cursor timestamp | Different: nine fractional digits; no zone for `DATE`/plain `TIMESTAMP`, numeric offset for zoned types; extracted text is retained | Six fractional digits plus `Z` for both native timestamp kinds | Seven fractional digits plus `Z` for `datetime2`; body-only `datetimeoffset` retains its offset |
| Date range | Unknown: BC dates and the full range across Oracle client/server versions are not guaranteed; native conversion and shared timestamp parsing constrain accepted values | Supported years `0001..9999`; infinity and out-of-range native values rejected | Supported years `0001..9999`; UTC normalization crossing that boundary rejected |
| Equal timestamp instants | Compared chronologically, not by text spelling | Compared chronologically, not by text spelling | Compared chronologically, not by text spelling |

Do not rely on a checkpoint preserving the timezone label or offset from an
initial timestamp. PostgreSQL's native `timestamptz` representation has no
original offset to recover. Oracle sets new sessions to UTC, but identical
`TIMESTAMP WITH LOCAL TIME ZONE` behavior is not guaranteed across Oracle
client/server and database timezone configurations.

A valid source cursor can fall outside the unsigned-nanosecond range used for
OTLP event timestamps. Source position and representable OTLP event time are
separate constraints.

## Tie-breakers, modes, and checkpoints

All three receivers implement scalar, composite, and snapshot modes.
Scalar pagination has no implicit tie-breaker: the authored strict cursor
predicate/order must identify a stable, strictly increasing source position.
Composite mode orders by timestamp and signed integer together; repeated
timestamps may advance only through the tie-breaker. This does not solve
late commits, mutable ordering keys, or non-unique cursor tuples.

| Contract | Oracle | PostgreSQL | SQL Server |
| --- | --- | --- | --- |
| Composite signed tie-breaker | Different: native `Int64` or `NUMBER(p,0)` with `p=1..18`, not the scalar limit of 19 | `int2`/`int4`/`int8`, with native-width binding checks | `smallint`/`int`/`bigint`, with native-width binding checks |
| Snapshot | No source-column position; complete bounded snapshot, not an ordered row offset | Same shared no-position contract | Same shared no-position contract |

Checkpoints are specific to the receiver and source configuration. Sharing a
logical watermark type across databases does not make their checkpoints
portable. An existing checkpoint must match the configured source identity and
fingerprint, including watermark mode/type and query semantics (the rows selected
and their cursor ordering). Compatibility also depends on receiver-specific
fingerprint inputs, not just equivalent query results: PostgreSQL fingerprints
the authored SQL text, so even a whitespace-only change can prevent startup with
the saved checkpoint. Changing the configured endpoint or initial watermark can
also invalidate it. See [checkpoint compatibility](README.md#checkpoint-compatibility)
before changing a source configuration.

The version-1 checkpoint format preserves scalar cursors with their explicit
type and exact value, composite cursors as the legacy untagged object, and
snapshot cursors as JSON `null`. Checksums cover serialized payload bytes.
Equivalent timestamp instants do not imply identical stored bytes or compatible
fingerprints. Keep checkpoint files intact rather than rewriting timestamp text
to match another receiver's representation.

## Implementation reference

- [Oracle cursor validation and conversion](../contrib-nodes/src/receivers/oracle_receiver/adapter.rs#L1092-L1271)
- [PostgreSQL cursor planning](../contrib-nodes/src/receivers/postgresql_receiver/query.rs#L115-L183) and [timestamp/integer codecs](../contrib-nodes/src/receivers/postgresql_receiver/value.rs#L42-L449)
- [SQL Server cursor bindings and row conversion](../contrib-nodes/src/receivers/sql_server_receiver/data.rs#L110-L462)
- [Shared cursor formats](src/database/page.rs) and [checkpoint compatibility checks](src/checkpoint.rs#L34-L202)
