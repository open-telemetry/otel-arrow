# Log Parser Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:log_parser` (`urn:otel:processor:log_parser`)
- Feature gate: Default
- Stability: Experimental

## Overview

The log parser interprets already-framed string log bodies using regex,
JSON Pointers, or headerless CSV. Mappings update body, event time, and severity
atomically per record. Malformed records retain their original logical contents
and order; valid neighbors are still parsed. Byte/map bodies and non-log signals
pass through unchanged.

This processor does not discover files, assemble multiline records, infer CSV
headers, retry delivery, or advance checkpoints. Framing belongs upstream.
Connect the default output to a transform processor for OPL filtering,
enrichment, or routing. Parsing inside OPL/query-engine is future work; no
scoped or scalar parsing syntax is committed by this component.

## Configuration

Put parser fields directly under `config`, without a `parse_logs` wrapper.
The three examples below are complete node bodies; add graph connections to
the default output. Unknown fields, query settings, and routing request-capacity
settings are rejected at construction. Patterns and selectors are validated and
compiled before records arrive.

`skip_sanitize_result` is an optional boolean, defaulting to `false`. Keep the
default when parsing removes sensitive data: unused Arrow buffer values are
sanitized after parsing. Setting it to `true` permits unused values to remain.
Channel capacity and pipeline resource policies provide runtime backpressure;
this default-output-only processor does not allocate routing completion slots.

## Examples

### Regex

```yaml
type: processor:log_parser
config:
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
type: processor:log_parser
config:
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
type: processor:log_parser
config:
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
  completion uses the original inbound context. Failed sends return an error
  without a successful Ack. An empty logs result completes locally with an Ack
  instead of emitting empty data.

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

### Compose With Transform

For a pipeline receiving `{"message":"keep"}` in the log body, the parsing node
below extracts `keep`; the downstream query then filters that extracted value.
Malformed JSON remains available to the downstream query unchanged. These node
definitions are a pipeline fragment: connect an upstream receiver to `parse`,
connect `parse` to `filter`, and connect `filter` to an exporter.

```yaml
nodes:
  parse:
    type: processor:log_parser
    config:
      format: json
      body: {source: /message}
      on_error: preserve
      limits:
        max_input_bytes: 1048576
        max_scratch_bytes: 8388608
        max_pattern_bytes: 4096
        max_compiled_regex_bytes: 1048576
        max_json_depth: 32
        max_entries: 4096
  filter:
    type: processor:transform
    config:
      opl_query: 'logs | where body != "drop"'
```

## Telemetry

The primary metric set is `processor.log_parser`. Common engine telemetry also
provides traffic, channel send failures, and downstream completion outcomes.

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `processor.log_parser.operations` | `{operation}` | `signal`, `outcome` | Local parsing operations, counted after dispatch or local empty-result completion. |
| `processor.log_parser.failures` | `{operation}` | `signal`, `error.type` | Local failures grouped by bounded reason. |
| `processor.log_parser.records` | `{record}` | `format`, `reason` | Preserved malformed records or committed observed-time fallbacks. |

`signal` is `logs`; non-log pass-through does not record a parsing operation.
`outcome` is `success` or `failure`. Success means the immediate output send or
local completion notification succeeded, not that a downstream exporter Acked.
`error.type` is `payload_conversion`, `id_decode`, `output_send`, or `internal`.

`format` is `regex`, `json`, or `csv`. Record `reason` is `limit`, `extraction`,
`body`, `timestamp`, `severity`, `unsupported_body`, or `observed_fallback`.
Fallbacks count only after batch updates commit; an internal staging failure
emits neither fallback nor malformed-record counts. Input text, file paths,
customer keys, and error strings are never metric labels. No node-specific
events are emitted.

## Related Docs

- [Transform processor](../transform_processor/README.md)
- [Runtime configuration](../../../../../docs/configuration.md)
