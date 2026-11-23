# SQL Watermark Compatibility

<!-- markdownlint-disable MD013 -->

This is an inventory of implemented contracts, not a proposal to expand accepted
types, canonicalize existing checkpoints, or promise cross-vendor parity.
The source baseline is
[`cf874973c87bafa3a873d97c3f9f443307107a42`](https://github.com/open-telemetry/otel-arrow/commit/cf874973c87bafa3a873d97c3f9f443307107a42).

**Supported** means admitted by the receiver's configuration, metadata, and
value checks, subject to the restrictions below. **Rejected** means the current
code fails closed. **Different** identifies an explicit adapter-specific
contract; it does not imply a decision to align it. **Unknown** means this
inventory has not established the behavior through the stated evidence.

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

Oracle `NUMBER(19,0)` and `NUMBER(20,0)` describe wider decimal domains than
`i64` and `u64`. Metadata admission is not proof every row fits: extraction must
reject values outside the configured Rust integer range. PostgreSQL and SQL
Server likewise reject binding values outside their narrower native widths.

The exact signed ranges are `-32768..32767` for PostgreSQL `int2` / SQL Server
`smallint`, `-2147483648..2147483647` for `int4` / `int`, and
`-9223372036854775808..9223372036854775807` for `int8` / `bigint` / Oracle's
`int64` scalar. Oracle's `uint64` scalar range is
`0..18446744073709551615`. There is no null scalar variant: an initial scalar
JSON `null` value is invalid, whereas a snapshot cursor itself serializes as
JSON `null` by design.

Source evidence:

- Oracle: [`validate_described_watermark`, `validate_scalar_value`, `extract_normalized_watermark`, and composite metadata checks](../contrib-nodes/src/receivers/oracle_receiver/adapter.rs#L1096-L1270); [configuration normalization](../contrib-nodes/src/receivers/oracle_receiver/config.rs#L130-L160); [string SQL and charset checks](../contrib-nodes/src/receivers/oracle_receiver/scalar_config_tests.rs); [`validate_scalar_charset`](../contrib-nodes/src/receivers/oracle_receiver/adapter.rs#L1310-L1340).
- PostgreSQL: [`Plan::resolve`](../contrib-nodes/src/receivers/postgresql_receiver/query.rs#L117-L188); [`validate_scalar`, native parameter binding, and decoding](../contrib-nodes/src/receivers/postgresql_receiver/value.rs#L100-L171); [`scalar_cell` and `row`](../contrib-nodes/src/receivers/postgresql_receiver/value.rs#L493-L590).
- SQL Server: [`ResultPlan::new`, `row_cursor`, and `bind_scalar`](../contrib-nodes/src/receivers/sql_server_receiver/data.rs#L106-L319); [configuration scalar-kind checks](../contrib-nodes/src/receivers/sql_server_receiver/config.rs#L110-L128); [`convert`](../contrib-nodes/src/receivers/sql_server_receiver/data.rs#L321-L430).

## Timestamp representation and ordering

| Contract | Oracle | PostgreSQL | SQL Server |
| --- | --- | --- | --- |
| Fractional precision | Different: up to nanoseconds; `DATE` requires whole seconds; binding rejects nonzero discarded digits | Different: up to microseconds; unknown/default typmod `-1` means 6; binding rejects nonzero discarded digits | Different: up to 100ns; binding rejects nonzero discarded digits at the discovered scale |
| Offset handling on initialization/bind | Converts zoned input to UTC with checked date arithmetic; binds the discovered Oracle family, not a universal zoned type | Converts RFC3339 offsets to UTC; `timestamp` binds naive UTC, `timestamptz` binds a UTC instant | Converts RFC3339 offsets to UTC; binds timezone-less `datetime2` under the UTC contract |
| Timezone-less input | Different: native parser accepts date-only and supported naive timestamp spellings; naive means UTC | Different: accepts space-separated naive date/time as UTC, not date-only input | Different: accepts space-separated naive date/time as UTC, not date-only input |
| Canonical configured timestamp | Different: native display, e.g. `2026-01-01 00:00:00.123456789 +00:00` | Different: `2026-01-01T00:00:00.123456Z` | Different: `2026-01-01T00:00:00.1234567Z` |
| Fetched cursor timestamp | Different: nine fractional digits; no zone for `DATE`/plain `TIMESTAMP`, numeric offset for zoned types; extracted text is retained | Six fractional digits plus `Z` for both native timestamp kinds | Seven fractional digits plus `Z` for `datetime2`; body-only `datetimeoffset` retains its offset |
| Date range | Unknown as a complete cross-layer contract: native Oracle parser/checked conversion and shared chrono parsing impose constraints; this inventory does not qualify BC dates or all OCI/server versions | Supported years `0001..9999`; infinity and out-of-range native values rejected | Supported years `0001..9999`; UTC normalization crossing that boundary rejected |
| Equal timestamp instants | Shared comparison parses chronological instants, not lexical spelling | Native `CursorPosition` compares decoded instants | Shared cursor comparison parses instants |

No receiver should be assumed to preserve a user's original timezone label in a
checkpoint. PostgreSQL's native `timestamptz` representation has no original
offset to recover. Oracle explicitly sets new sessions to UTC
([`prepare_session`](../contrib-nodes/src/receivers/oracle_receiver/adapter.rs#L1301-L1310)).
Its session/database timezone interactions for `TIMESTAMP WITH LOCAL TIME ZONE`
still need live qualification; formatter/normalization unit tests alone do not
establish deployment-wide temporal semantics.

Source evidence:

- Oracle: [`parse_cursor_timestamp`, `normalize_cursor_timestamp`, `cursor_bind_timestamp`](../contrib-nodes/src/receivers/oracle_receiver/adapter.rs#L860-L963); [`normalize_cell` and `format_oracle_timestamp`](../contrib-nodes/src/receivers/oracle_receiver/adapter.rs#L1538-L1617).
- PostgreSQL: [`cursor_time`, `normalize_cursor_time`, `validate_timestamp`](../contrib-nodes/src/receivers/postgresql_receiver/value.rs#L42-L97); [native epoch decoding and timestamp text](../contrib-nodes/src/receivers/postgresql_receiver/value.rs#L191-L258); [`CursorPosition`](../contrib-nodes/src/receivers/postgresql_receiver/value.rs#L451-L490).
- SQL Server: [`normalize_timestamp`](../contrib-nodes/src/receivers/sql_server_receiver/config.rs#L491-L526); [native temporal conversion](../contrib-nodes/src/receivers/sql_server_receiver/data.rs#L389-L475).
- Shared: [`ScalarValue::compare`](src/database/scalar.rs#L13-L82), [`Cursor::compare`](src/database/page.rs#L119-L150), and [`parse_utc_timestamp`](src/database/otap.rs#L477-L493). Source cursor range is separate from representable unsigned-nanosecond OTLP event time.

## Tie-breakers, modes, and durable identity

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
| Existing compatibility guard | Frozen original composite/scalar/snapshot identity payloads reopen checkpoints | Frozen `postgresql/v5-watermark-time` payloads reopen checkpoints | Frozen version-1 identity payloads reopen checkpoints |
| Cross-adapter checkpoint transfer | Not supported by this inventory | Not supported by this inventory | Not supported by this inventory |

The shared checkpoint envelope is version 1. Scalar cursors retain their tagged
type and exact value, composite cursors retain the legacy untagged object, and
snapshot cursors serialize as JSON `null`. The checksum includes serialized
payload bytes. Timestamp equality as an instant does not imply identical stored
bytes or fingerprint compatibility. Source identity, mode/type, query semantics,
and adapter-specific fingerprint inputs must still match. No checkpoint rewrite,
normalization migration, or fingerprint change is made by this inventory.

Source evidence:

- [Shared checkpoint envelope and compatibility errors](src/checkpoint.rs#L34-L209); [cursor serialization](src/database/page.rs); [scalar roundtrip and legacy-byte fixtures](src/database/scalar_tests.rs); [snapshot fixtures](src/database/snapshot_tests.rs).
- [Oracle frozen identity/checkpoint tests](../contrib-nodes/src/receivers/oracle_receiver/scalar_config_tests.rs), [PostgreSQL frozen identity/checkpoint tests](../contrib-nodes/src/receivers/postgresql_receiver/test.rs#L1185-L1280), and [SQL Server frozen identity/checkpoint tests](../contrib-nodes/src/receivers/sql_server_receiver/tests.rs#L136-L181).
- [Oracle fingerprint inputs](../contrib-nodes/src/receivers/oracle_receiver/config.rs#L175-L214), [PostgreSQL identity version/precision marker](../contrib-nodes/src/receivers/postgresql_receiver/config.rs#L333-L345), and [SQL Server fingerprint fields](../contrib-nodes/src/receivers/sql_server_receiver/config.rs#L432-L489).

## Focused conformance regressions

Existing scalar/composite/snapshot fixtures already cover accepted/rejected
types, offset-equivalent initialization, precision-loss rejection, shared cursor
ordering, and frozen checkpoint identities. The additional regressions exercise
boundary paths rather than duplicate that matrix:

- Oracle `scalar_adapter_tests.rs`: null extraction for every scalar decoder,
  plus signed decimal underflow immediately below `i64::MIN`.
- PostgreSQL `scalar_tests.rs`: both signed endpoints for all three native
  widths, adjacent overflow, and one-microsecond values on both sides of the
  PostgreSQL epoch. Decode -> cursor JSON -> native parameter encoding must
  reproduce exact wire bytes.
- SQL Server `adapter_tests.rs`: stock Tiberius row extraction/rebinding at
  each signed width's endpoints; nullable cursor rejection; first/last-date
  100ns ticks through an actual temporary checkpoint and native rebind.

These are deterministic decoder/binding/filesystem tests. Constructed Tiberius
rows are not live SQL Server coverage; Oracle normalized-cell tests do not fetch
from OCI; PostgreSQL binary codec tests do not execute SQL. Live native server,
collation, client-version, timezone-session, and query-plan qualification remain
separate work.

## UTC standardization tracking

The [follow-up tracker](https://github.com/athomas9195/otel-arrow/issues/16)
explicitly says to find the already-created UTC timestamp work item before
opening another.
The [original timestamp review](https://github.com/open-telemetry/otel-arrow/pull/4093#discussion_r4051927090)
requests ISO 8601 UTC text; its
[reply](https://github.com/open-telemetry/otel-arrow/pull/4093#discussion_r4077845184)
says a work item was created but supplies no issue number.

Read-only searches of both repositories for timestamp/UTC terms and the shared
scraper tracker did not identify a dedicated SQL cursor-text issue. Its identity
therefore remains **unknown**, not absent. No duplicate issue was created.
[open-telemetry/otel-arrow#2369](https://github.com/open-telemetry/otel-arrow/issues/2369)
and [open-telemetry/otel-arrow#4154](https://github.com/open-telemetry/otel-arrow/issues/4154)
concern Arrow timestamp-column timezone metadata, not SQL cursor text or
checkpoint migration, and must not be presented as that missing work item.

Before any later alignment, recover the original work-item link and decide
accepted native types, precision-loss policy, naive timestamp semantics, and
compatibility/migration requirements explicitly. The current differences above
are evidence, not authorization to change these policies.
