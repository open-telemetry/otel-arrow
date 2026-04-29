# Microsoft Common Schema Processor

URN: `urn:microsoft:processor:common_schema_otel_logs`

This processor promotes decoded Microsoft Common Schema log attributes into typed
OTLP log fields. It is intended for pipelines where the producer is known to
emit Microsoft Common Schema records, such as the Linux `user_events` receiver
using `event_header` format.

## Configuration

The processor currently has no options:

```yaml
processors:
  common_schema:
    kind: "urn:microsoft:processor:common_schema_otel_logs"
    config: {}
```

Unknown configuration fields are rejected.

## Promotion Rules

Only records with `__csver__ == 0x400` and `PartB._typeName == "Log"` are
promoted. Other records are forwarded unchanged.

| Common Schema attribute | OTLP destination |
| --- | --- |
| `PartA.time` | `time_unix_nano` |
| `PartA.name` | `event_name` when `PartB.name` is absent |
| `PartB.name` | `event_name` |
| `PartA.ext_dt_traceId` | `trace_id` |
| `PartA.ext_dt_spanId` | `span_id` |
| `PartA.ext_dt_traceFlags` | `flags` |
| `PartA.ext_cloud_role` | `service.name` attribute |
| `PartA.ext_cloud_roleInstance` | `service.instance.id` attribute |
| `PartB.body` | `body` |
| `PartB.severityNumber` | `severity_number` |
| `PartB.severityText` | `severity_text` |
| `PartB.eventId` | `eventId` attribute |
| `PartC.*` | Attribute with the `PartC.` prefix removed |

`PartB.name` takes precedence over `PartA.name`. `PartB.eventId` takes
precedence over `PartC.eventId`. Severity values above the OTLP range are
clamped to `24`; severity `0` is preserved as OTLP `UNSPECIFIED`.
Recognized `PartA.*` and `PartB.*` fields are removed after promotion. Unknown
`PartA.*` and `PartB.*` fields are preserved with their original names.

Malformed trace and span IDs are kept as attributes named `trace.id` and
`span.id` instead of being promoted to typed ID fields.

## Arrow Conversion Cost

The current implementation promotes records by converting payloads to OTLP proto
bytes and mutating OTLP `LogsData` / `LogRecord` values. Incoming Arrow batches
that do not contain a `__csver__` log attribute are forwarded without this
conversion. Arrow batches that contain a `__csver__` log attribute still pay the
Arrow-to-OTLP conversion cost. If no records are promoted after inspection, the
original Arrow payload is forwarded unchanged.

This processor should be wired only where Common Schema records are expected.
Using it in a heterogeneous logs pipeline may add conversion work to batches
that do not benefit from promotion.

The full Arrow-native implementation is deferred. It needs a reusable
cross-batch transform that can read log attributes by `parent_id`, set typed
root log columns, and rebuild `LogAttrs` with promoted fields removed while
preserving the remaining attributes.

## Metrics

The processor reports these metrics:

- `records_seen`: log records inspected after conversion to OTLP.
- `records_promoted`: log records promoted as Microsoft Common Schema logs.
- `records_skipped_not_common_schema`: inspected records that did not match
  Common Schema gating.
- `batches_promoted`: log batches with at least one promoted record.
- `arrow_batches_skipped_no_csver`: Arrow batches forwarded without OTLP
  conversion because no `__csver__` attribute was present.
