# Transform Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:transform` (`urn:otel:processor:transform`)
- Feature gate: Default
- Stability: Experimental

## Overview

The transform processor applies query-language transformations to OTAP batches.
It accepts KQL, OPL, OTTL log statements, or declarative `parse_logs` configuration.
Queries may emit zero, one, or multiple output batches. Parsing preserves record
order and emits one batch without filtering.

This processor and its query engine integration are under active development.

## Getting Started

Write the query to transform your telemetry in your preferred language:

```yaml
type: processor:transform
config:
  kql_query: "logs | where body != ''"

  # Pending request tracking limits.
  inbound_request_limit: 1024
  outbound_request_limit: 512

  # Skips result sanitization when true (default: false).
  skip_sanitize_result: false

  # Controls filter attribute key matching (default: true).
  filter_attribute_keys_case_sensitive: true
```

## Examples

OTTL log statements:

```yaml
type: processor:transform
config:
  ottl:
    log_statements:
      - set(attributes["processed"], true)
```

## Parsing Framed Logs

Configure exactly one of `parse_logs`, `kql_query`, `opl_query`, or `ottl`.
Additional queries can run in a separate transform node. Parsing interprets
already-framed string log bodies; it does not discover files, assemble multiline
records, infer CSV headers, retry delivery, or advance source checkpoints.
Byte/map bodies and non-log signals pass through unchanged.

These complete node bodies need input and output connections in an engine graph.
The processor tests use synthetic framed OTAP records, not a filelog receiver.

### Regex

```yaml
type: processor:transform
config:
  parse_logs:
    format: regex
    pattern: '(?s)^START: (?P<ts>\S+) (?P<sev>\S+) (?P<message>.*)$'
    timestamp: {source: ts, format: rfc3339, on_missing: observed}
    severity: {source: sev, mapping: {ERROR: 17, INFO: 9}}
    on_error: preserve
    limits:
      max_input_bytes: 1048576
      max_scratch_bytes: 8388608
      max_pattern_bytes: 4096
      max_compiled_regex_bytes: 1048576
      max_json_depth: 32
      max_entries: 4096
```

Regex matches the entire record. Use explicit dot-all `(?s)` for multiline text.
Optional unmatched captures are missing values. Omitting `body` retains markers
and internal newlines. Each receiver fragment is parsed independently; parsing
does not reassemble fragments or remove provenance attributes.

### JSON

```yaml
type: processor:transform
config:
  parse_logs:
    format: json
    body: {source: /raw-data}
    timestamp: {source: /ts, format: rfc3339, on_missing: observed}
    severity: {source: /sev, mapping: {ERROR: 17, INFO: 9}}
    on_error: preserve
    limits:
      max_input_bytes: 1048576
      max_scratch_bytes: 8388608
      max_pattern_bytes: 4096
      max_compiled_regex_bytes: 1048576
      max_json_depth: 32
      max_entries: 4096
```

JSON requires one complete object with no duplicate keys or trailing data.
Sources are RFC 6901 JSON Pointers: `/a~1b` selects `a/b`, and `/nested/0/value`
selects an array element's field. Present mapped values must be strings, not
numbers or objects. JSON null is missing for timestamp mapping.

### CSV

```yaml
type: processor:transform
config:
  parse_logs:
    format: csv
    columns: [ts, sev, raw-data]
    delimiter: ','
    header: none
    body: {source: raw-data}
    timestamp: {source: ts, format: rfc3339, on_missing: observed}
    severity: {source: sev, mapping: {ERROR: 17, INFO: 9}}
    on_error: preserve
    limits:
      max_input_bytes: 1048576
      max_scratch_bytes: 8388608
      max_pattern_bytes: 4096
      max_compiled_regex_bytes: 1048576
      max_json_depth: 32
      max_entries: 4096
```

CSV requires exactly the configured number of fields, a one-byte ASCII delimiter,
double-quote quoting, doubled-quote escaping, and no embedded CR/LF. Values are not
trimmed. Empty fields are present empty strings. `header: none` is required;
a header-looking row is ordinary data and no header state survives between rows.

### Mapping And Errors

- A body mapping requires a present string; an empty string is allowed.
- Timestamp strings must be RFC 3339 with an explicit offset and at most nine
  fractional digits. Pre-epoch, overflow, leap-second, empty and malformed values
  are errors, not missing values. Successful parsing replaces event time.
- A missing timestamp preserves existing nonzero event time. Otherwise
  `on_missing: observed` uses nonzero observed time or reports an error;
  `on_missing: preserve` leaves event time untouched. Observed time never changes.
- Severity matching is exact and case-sensitive. Mapped numbers must be in
  `1..=24`; text and number are updated together. Unknown/missing severity is an error.
- `on_error: preserve` is the only policy. Any configured mapping failure leaves
  the entire record unchanged, while valid neighbors transform. No diagnostic
  attributes are added. Error priority is input/type limit, extraction, body,
  timestamp, severity. A fallback is counted only after its batch updates commit.
- Internal update failures abort the operation without a partial output.
  Local parsing success never generates an early upstream Ack. Downstream
  completion and failed sends retain the transform processor's existing behavior.

All six positive limits are required, including limits unused by the selected
format. `max_entries` counts JSON members and array elements or CSV fields;
the JSON root container has depth one. JSON configuration caps `max_json_depth`
at 64 to bound the recursive parser stack; deeper limits fail configuration.
Input and conservative scratch reservations
are checked before extraction allocations. Exceeding a runtime limit preserves the
record. Regex compilation limits reject configuration. Scratch reservations include
library temporary storage but exclude staged batch output and allocator overhead;
they are not an RSS guarantee. Compilation/storage and normal batch output remain
subject to processor resource limits.

### Qualification

Run the isolated allocation and throughput harness from the Rust workspace:

```sh
cargo bench -p otel-arrow-dfe-core-nodes --features bench --bench parse_logs --profile dev
```

It compiles configurations outside measurement, checks requested-heap peaks with
`dhat`, then times 10,000 record parses with profiling disabled. Each format uses
either all-valid records or 25% malformed records. It also probes escaped JSON
strings and increasing regex capture counts. This is record-local extraction and
mapping. A separate native-batch measurement includes transport-ID decoding,
record parsing, candidate staging, native OTAP updates and sanitization. Neither
measurement includes engine scheduling, output-channel delivery or receiver work.

Initial Windows debug-profile measurements (not production capacity estimates):

| Format | Input Bytes | Malformed | Peak Requested Bytes | Reserved Bytes | Records/s |
| --- | ---: | ---: | ---: | ---: | ---: |
| JSON | 60 | 0% | 3000 | 3740 | 168565 |
| JSON | 60 | 25% | 3000 | 3740 | 211703 |
| Regex | 34 | 0% | 8044 | 14596 | 30470 |
| Regex | 34 | 25% | 8044 | 14596 | 33596 |
| CSV | 34 | 0% | 78 | 134 | 7239 |
| CSV | 34 | 25% | 78 | 134 | 7376 |

The native-batch measurement uses prebuilt batches of 128 and 1024 records.
The 25% error distribution puts malformed text at every fourth record, with two
interleaved file-path attributes. Full logical output and error counts are checked
before timing. Fixture creation and OTLP conversion are excluded; input batch
cloning and output destruction are included. Each workload runs 100 timed batches
with profiling disabled, after a separate `dhat` peak-allocation measurement.

Windows debug-profile native-batch results from the qualification run:

| Format | Records/Batch | Malformed | Batch Peak Requested Bytes | Records/s |
| --- | ---: | ---: | ---: | ---: |
| JSON | 128 | 0% | 21197 | 127210 |
| JSON | 128 | 25% | 21053 | 137967 |
| JSON | 1024 | 0% | 143277 | 112137 |
| JSON | 1024 | 25% | 136477 | 156423 |
| Regex | 128 | 0% | 21197 | 27432 |
| Regex | 128 | 25% | 21053 | 31044 |
| Regex | 1024 | 0% | 143277 | 27784 |
| Regex | 1024 | 25% | 136477 | 32942 |
| CSV | 128 | 0% | 21197 | 7276 |
| CSV | 128 | 25% | 21053 | 7102 |
| CSV | 1024 | 0% | 143277 | 7349 |
| CSV | 1024 | 25% | 136477 | 7259 |

Batch peaks include candidate staging, rebuilt columns and sanitization, but not
the prebuilt input. They are not per-record scratch measurements and are not
compared to `max_scratch_bytes`. Record-local requested peaks remain 3000 bytes
for JSON, 8044 for regex and 78 for CSV in these workloads, below their respective
reservations of 3740, 14596 and 134 bytes. These results are not production capacity
estimates or a bound on process RSS.

Receiver START boundaries, idle flush, limits, restart/replay and recovery require
separate filelog integration qualification. These processor tests do not establish
receiver checkpoint safety.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.transform`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `processor.transform.operations` | `{operation}` | `language`, `signal`, `outcome` | Number of matching input messages whose local transform operation terminated. |
| `processor.transform.failures` | `{operation}` | `language`, `signal`, `error.type` | Failed transform operations grouped by actionable error category. |

The bounded `language` attribute is fixed for a processor instance and is one
of `kql`, `opl`, `ottl`, or `parse_logs`. The `signal` attribute is one of `traces`,
`metrics`, or `logs`.

An operation covers all configured transforms that match one input message.
Messages with no matching transform are passed through without recording an
operation. The `outcome` is `success` only after the transformed default and
routed outputs have been accepted by their immediate sends; otherwise it is
`failure`. Downstream acknowledgements do not change this local outcome.

The bounded `error.type` values are:

- `payload_conversion`: the input could not be converted to OTAP Arrow records.
- `id_decode`: transport-optimized identifiers could not be decoded.
- `query_execution`: the configured query pipeline failed while executing.
- `route_not_configured`: the query referenced an unconfigured output route.
- `inbound_capacity`: `inbound_request_limit` was exhausted.
- `outbound_capacity`: `outbound_request_limit` was exhausted.
- `output_send`: an immediate default or routed output send failed.
- `internal`: an internal transform processor invariant failed.

Common engine telemetry provides total consumed and produced traffic, dropped
items, channel send failures, and downstream acknowledgement outcomes.

Parsing also emits `processor.transform.parse_logs.records` with bounded `format`
(`regex`, `json`, `csv`) and `reason` (`limit`, `extraction`, `body`, `timestamp`,
`severity`, `unsupported_body`, `observed_fallback`) attributes. The fallback bucket
counts committed observed-time fallbacks; other buckets count preserved records.
Input text, file paths, customer keys and exception strings are never metric labels.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- The transformation query surface is still evolving.
- `skip_sanitize_result: true` can leave removed data in unused Arrow buffers;
  keep the default when transformations redact sensitive data.
- OTTL currently supports only updating setting log fields to literal values. Additional
  operations such as filtering, function evaluation, and other expression types, as well as
  applying OTTL transforms to spans and metrics are not yet supported.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Query engine](../../../../query-engine/README.md)
- [Core node catalog](../../../README.md)
