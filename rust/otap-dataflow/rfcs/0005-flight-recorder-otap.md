---
Proposal Name: flight-recorder-otap
Start Date: 2026-08-21
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC 0005: Arrow Flight Recorder Data

> **Amendment note (2026-08-24):** Implementation of the prototype showed
> that Flight should not become a fourth OpenTelemetry semantic signal. This
> RFC now specifies the Flight Arrow data model and graph invariants. RFC 0006
> specifies the mixed-signal extension envelope, representation identity, and
> alternative wire encodings needed to carry Flight through OTAP pipelines.

## Summary

Define Flight Recorder as a **Pluggable PData Arrow Representation**: a fixed
and bounded set of Arrow `RecordBatch` schemas. Flight Recorder data represents
ordered runtime and operating-system observations, their source event schemas,
typed values, samples, stacks, mappings, symbols, loss, and correlation.

Flight is not a fourth OpenTelemetry semantic signal. It is a source-preserving
Arrow representation that can overlap logs, metrics, traces, and future
profiles in one mixed pipeline message. Its attachment to pipeline data,
extension identity, capability rules, and alternative wire encodings are
defined by RFC 0006.

The representation is designed around four compatible systems:

- one_collect as a source of runtime, EventPipe, ETW, perf, tracefs, stack, and
  symbol observations;
- NetTrace V6 as a mixed-event serialization format for recording and replay;
- OpenTelemetry Profiles as the normalized destination for profile-producing
  observations; and
- OTAP as the columnar processing model used between receivers, processors,
  and exporters.

Flight Recorder OTAP is not an Arrow encoding of NetTrace blocks and is not an
opaque one_collect capture. It is a normalized relational representation with
14 statically defined payload types: the existing resource and scope attribute
payloads plus 12 Flight Recorder payloads. Of the 14 payloads, two are existing
common OTAP payloads, one specializes the common OTAP attribute schema, seven
are aligned with the graph needed by a future OTAP Profiles signal, and four
are unique to source-observation fidelity. Its sample and stack tables are
intentionally shaped so Arrow operations can transform them into OTAP
Profiles.

This RFC defines the initial data model and its invariants. Exact semantic
mappings from Flight Recorder data to OpenTelemetry logs, metrics, and traces
will be refined in follow-up revisions.

## Motivation

`otap-dataflow` uses Arrow record batches to process OpenTelemetry data in a
columnar form. Logs, metrics, and traces each consist of a small, statically
known collection of related tables. Profiles are not yet implemented in OTAP,
but are expected to follow the same model.

Runtime flight recorders present the same design problem. Their inputs are
heterogeneous and extensible:

- EventPipe and ETW providers define event schemas at runtime;
- Linux perf and tracefs provide samples and typed tracepoint records;
- one_collect combines events, samples, stacks, modules, and symbols;
- NetTrace V6 stores metadata, events, stacks, labels, threads, and sequence
  points in one stream; and
- profile samples must retain stack and mapping identity so they can become
  OTLP Profiles without stringifying or reparsing data.

Converting all of this directly into logs, metrics, traces, and profiles at the
receiver boundary is irreversible. A source event may contribute to several
signals, may require stateful interpretation, or may have useful fields that do
not have an OpenTelemetry semantic mapping. Conversely, retaining only an
opaque NetTrace or one_collect payload prevents columnar filtering,
aggregation, projection, correlation, and profile construction.

The desired architecture is:

```text
one_collect ----------------------+
                                  |
NetTrace V6 ----------------------+-> Flight Recorder OTAP
                                  |          |
other structured event sources --+          +-> NetTrace V6
                                             |
                                             +-> OTAP Profiles
                                             |
                                             +-> OTAP logs
                                             |
                                             +-> OTAP metrics
                                             |
                                             +-> OTAP traces
```

The primary round-trip requirement is semantic source-record equivalence:

```text
source observation
  -> Flight Recorder OTAP
  -> NetTrace V6
  -> Flight Recorder OTAP
  -> equivalent source observation
```

This does not require byte-identical NetTrace files. Block boundaries,
metadata IDs, interning order, dictionary encoding, and sequence-point
placement may differ. Provider identity, event schema, typed values,
timestamps, process and thread context, correlation, stacks, and explicit loss
must remain equivalent when they were present in the source.

## Guide-level explanation

### Flight Recorder is a Pluggable PData Arrow Representation

Flight is a validated set of Arrow record batches attached to a mixed OTAP
message through the extension mechanism in RFC 0006:

```text
OTAP message
  semantic signals:
    zero or more of logs, metrics, traces, and future profiles
  Pluggable PData Arrow Representations:
    zero or more named and versioned Arrow record sets
      +-- otap.flight 1.0 (the 14 tables in this RFC)
```

`SignalType` continues to describe OpenTelemetry semantic signals. It does not
gain `FlightRecords`. A message containing Flight may contain no predetermined
semantic signal or may also contain standard signals. Components that do not
declare support for the Flight representation reject the message or route it
around that component; they never silently drop the representation.

There is initially no OTLP Flight Recorder service and no requirement to carry
NetTrace bytes inside pipeline pdata. Receivers decode source bytes into the
Flight Arrow representation. A NetTrace V6 exporter encodes that representation
at the wire boundary.

### Observations and interpretations are separate

A Flight Recorder record is a source observation. It is not necessarily an
OpenTelemetry log, metric, span, or profile sample.

For example, a .NET allocation tick may be retained as a structured runtime
event and also interpreted as a profile sample:

```text
FlightRecord(
  provider = "Microsoft-Windows-DotNETRuntime",
  event = "GCAllocationTick",
  thread_id = 71,
  stack_id = 19,
  fields = {
    allocation_amount = 4096,
    type_name = "Example.Widget"
  }
)
```

The event is represented in the generic event tables. A recognized profile
mapping adds a row to `flight_samples` and measurement rows such as:

```text
record_id | sample_type | unit  | value
----------+-------------+-------+------
42        | allocations | count | 1
42        | space       | bytes | 4096
```

Both representations refer to the same root record. Unknown events remain
usable structured events without requiring an OpenTelemetry interpretation.

### The table set is static

Providers do not create Arrow schemas or columns. Runtime event schemas are
rows in `flight_event_schemas` and `flight_schema_fields`. Event values are
rows in `flight_event_values`, using nullable typed value columns.

The complete initial payload set is:

<!-- markdownlint-disable MD013 -->
| Payload | Role |
| --- | --- |
| `RESOURCE_ATTRS` | Existing OTAP resource attributes referenced by root records |
| `SCOPE_ATTRS` | Existing OTAP instrumentation scope attributes referenced by root records |
| `FLIGHT_RECORDS` | Ordered source observations and their execution context |
| `FLIGHT_ATTRS` | Common OTAP typed attributes associated with Flight Recorder entities |
| `FLIGHT_EVENT_SCHEMAS` | Provider and event definitions |
| `FLIGHT_SCHEMA_FIELDS` | Field definitions for event schemas |
| `FLIGHT_EVENT_VALUES` | Decoded typed values for individual records |
| `FLIGHT_SAMPLES` | Profile-producing interpretation of records |
| `FLIGHT_SAMPLE_VALUES` | Typed measurements associated with samples |
| `FLIGHT_SAMPLE_TYPES` | Measurement type, unit, and temporality definitions |
| `FLIGHT_STACK_FRAMES` | Ordered location references for stacks |
| `FLIGHT_LOCATIONS` | Instruction locations and mapping references |
| `FLIGHT_MAPPINGS` | Loaded images, modules, and address ranges |
| `FLIGHT_SYMBOLS` | Function, inline frame, and source location information |
<!-- markdownlint-enable MD013 -->

The set can evolve through the same schema-versioning process as existing OTAP
payloads, but it does not grow with the number of providers or event types.

### How the 14 payloads divide

The payloads deliberately separate common OTAP infrastructure,
profile-compatible execution structure, and Flight Recorder source fidelity:

<!-- markdownlint-disable MD013 -->
| Payload | Classification | Reuse or association |
| --- | --- | --- |
| `RESOURCE_ATTRS` | Existing common OTAP | Reused unchanged with the common embedded resource struct |
| `SCOPE_ATTRS` | Existing common OTAP | Reused unchanged with the common embedded scope struct |
| `FLIGHT_ATTRS` | Common OTAP schema specialization | Reuses the common typed attribute columns, extended with optional unit metadata and a Flight parent kind |
| `FLIGHT_SAMPLES` | Profile-aligned | Identifies root records with normalized profile-sample semantics |
| `FLIGHT_SAMPLE_VALUES` | Profile-aligned | Corresponds to the measurement vector in an OTLP Profile sample |
| `FLIGHT_SAMPLE_TYPES` | Profile-aligned | Corresponds to Profile value type, unit, and aggregation temporality |
| `FLIGHT_STACK_FRAMES` | Profile-aligned | Represents the ordered location sequence used by Profile samples |
| `FLIGHT_LOCATIONS` | Profile-aligned | Corresponds to the OTLP Profiles location table |
| `FLIGHT_MAPPINGS` | Profile-aligned | Corresponds to the OTLP Profiles mapping table |
| `FLIGHT_SYMBOLS` | Profile-aligned | Supplies the function and line information used by OTLP Profiles |
| `FLIGHT_RECORDS` | Flight-specific | Preserves the ordered observation timeline and execution context |
| `FLIGHT_EVENT_SCHEMAS` | Flight-specific | Preserves source provider and event definitions |
| `FLIGHT_SCHEMA_FIELDS` | Flight-specific | Preserves source field trees and wire types |
| `FLIGHT_EVENT_VALUES` | Flight-specific | Preserves lossless decoded source values |
<!-- markdownlint-enable MD013 -->

Numerically, the division is:

```text
existing common OTAP payloads              2
common attribute schema specialization     1
profile-aligned graph payloads              7
Flight-specific observation payloads        4
                                           --
                                           14
```

Profile-aligned means that the logical schema and graph operation should be
shared with OTAP Profiles where the semantics agree. It does not require
Flight Recorder and Profiles to use the same Arrow payload enum. A Flight
sample remains attached to an ordered source record, while a Profile sample
belongs to a profile window and may represent an aggregation.

### A typical recording pipeline

```yaml
receivers:
  runtime:
    type: receiver:one_collect
    config:
      sources:
        dotnet:
          discover: true
        perf:
          cpu_sampling: true

exporters:
  recording:
    type: exporter:file
    config:
      format: nettrace_v6
      path: /var/log/telemetry/runtime-{generation}-{segment}.nettrace

service:
  pipelines:
    runtime_recording:
      receivers: [runtime]
      exporters: [recording]
```

The receiver copies callback-owned observations into bounded local builders.
It does not retain a whole-capture one_collect `ExportMachine`. The file
exporter serializes Flight Recorder records, definitions, stacks, and
correlation into one mixed NetTrace V6 stream.

### Profile construction is columnar

Recognized sources populate `flight_samples` and related tables. A generic
profile assembler then:

1. filters sample records;
2. partitions them by resource, scope, collection window, and sample-type set;
3. joins sample values and stack locations;
4. groups compatible samples;
5. sums measurement vectors and retains sample timestamps;
6. compacts referenced mappings, locations, and symbols; and
7. emits the future OTAP Profiles tables.

The generic assembler does not parse ETW, EventPipe, perf, or NetTrace
payloads. Source-specific decoding and classification happen before its input.

### Round trips have explicit boundaries

The design distinguishes serialization round trips from semantic projections.
Within documented source and codec capabilities, each representation can have
a semantically lossless NetTrace V6 serialization:

```text
Flight Recorder OTAP <-> NetTrace V6 source records
OTAP logs            <-> NetTrace V6 normalized log records
OTAP traces          <-> NetTrace V6 normalized trace records
OTAP metrics         <-> NetTrace V6 normalized metric records
OTAP Profiles        <-> NetTrace V6 normalized profile records
```

The transformations from Flight Recorder observations into normalized OTel
signals are semantic projections:

```text
Flight Recorder OTAP --+-> OTAP logs
                       +-> OTAP traces
                       +-> OTAP metrics
                       +-> OTAP Profiles
```

They are not general inverses. Logs can omit source schemas and unmapped
fields. Traces can combine start, stop, and Activity observations into spans.
Metrics can interpret and aggregate counters. Profiles can combine many
ordered sample observations into one sample in a profile window. A normalized
signal cannot reconstruct unrelated runtime events, original wire types,
metadata ordering, observation boundaries, or source arrival order.

The reverse transformation from a normalized signal can create synthetic
Flight records for processing convenience, but those records are normalized
signal records, not reconstructed source observations. In particular:

```text
OTAP Profiles
  -> NetTrace V6 normalized profile records
  -> OTAP Profiles
```

can preserve profile semantics without implying:

```text
OTAP Profiles -> original Flight Recorder observations
```

A mixed NetTrace V6 stream therefore multiplexes two record classes:

```text
source observation records <-> OTAP Flight
normalized OTel records     <-> OTAP logs, traces, metrics, and Profiles
```

A receiver demultiplexes these classes into the Flight Pluggable PData Arrow
Representation and their corresponding standard OTAP signals. An exporter may
interleave Flight plus four standard signal classes in one V6 stream while
preserving record class and stream ordering. If both a source observation and
its derived normalized signal are recorded, the normalized record must be
marked as derived and linked to its source capture or record range so a reader
does not interpret the pair as independent duplicate observations.

## Reference-level explanation

### Design principles

The representation follows these rules:

1. **Static schemas.** The number of Arrow payload types is independent of
   provider and event cardinality.
2. **Columnar access.** Common filters and profile construction operate on
   typed columns, not serialized payload inspection.
3. **Source fidelity.** Provider schemas, wire types, and optionally raw
   payloads preserve information needed for reconstruction.
4. **Profile alignment.** Samples, stacks, mappings, locations, and symbols
   map directly into the OTLP Profiles graph.
5. **Self-contained batches.** References do not cross pdata boundaries.
6. **Bounded operation.** Receivers, processors, and exporters bound pending
   rows, bytes, dictionaries, definitions, and state.
7. **Explicit loss.** Source, kernel, receiver, and processor losses are
   represented or reported; they are not hidden as successful input.
8. **Logical IDs.** OTAP IDs are batch-local graph keys, not NetTrace,
   EventPipe, ETW, or one_collect internal IDs.
9. **No fabricated semantics.** Unknown events stay structured source events
   until a mapper can represent them correctly.

### Payload graph

The root and child relationships are:

```text
RESOURCE_ATTRS              SCOPE_ATTRS
       ^                         ^
       |                         |
       +------ FLIGHT_RECORDS ---+
                    |
        +-----------+------------+------------------+
        |           |            |                  |
        v           v            v                  v
  FLIGHT_ATTRS  EVENT_VALUES  FLIGHT_SAMPLES  STACK_FRAMES
                       |            |                  |
                       |            v                  v
                       |       SAMPLE_VALUES      LOCATIONS
                       |            |                  |
                       |            v          +-------+-------+
                       |       SAMPLE_TYPES     v               v
                       |                     MAPPINGS         SYMBOLS
                       v
              EVENT_SCHEMAS
                       |
                       v
                SCHEMA_FIELDS
```

`FLIGHT_ATTRS` uses the common OTAP attribute representation plus a parent-kind
discriminator. This permits attributes on Flight records and executable graph
entities without introducing one attribute payload per entity.

### Identifier conventions

IDs are unsigned integers local to one Flight Recorder pdata:

- root record IDs use `u32`;
- schema, stack, location, mapping, and symbol IDs use `u32`;
- field and sample-type IDs may use `u16` where builders enforce the limit;
- nullable references mean the relationship is unknown or inapplicable; and
- zero is a valid ID unless a specific schema reserves it.

A builder must start a new pdata before an ID domain overflows. No foreign key
may refer to a previous or future pdata.

Sources maintain session-local caches that map source IDs to definitions. When
a pdata is finished, the builder includes every definition reachable from its
root records and remaps source IDs into compact batch-local IDs.

### Root records

`FLIGHT_RECORDS` contains one row per ordered observation:

```text
id: u32, non-null
kind: u8, non-null

resource: struct, nullable
  id: u16, nullable
  schema_url: dictionary<utf8>, nullable
  dropped_attributes_count: u32, nullable

scope: struct, nullable
  id: u16, nullable
  name: dictionary<utf8>, nullable
  version: dictionary<utf8>, nullable
  dropped_attributes_count: u32, nullable

schema_url: dictionary<utf8>, nullable

time_unix_nano: i64, non-null
observed_time_unix_nano: i64, nullable

schema_id: u32, nullable

process_id: u32, nullable
thread_id: u32, nullable
capture_thread_id: u32, nullable
cpu: u16, nullable

sequence_number: u64, nullable
stack_id: u32, nullable

trace_id: fixed_size_binary[16], nullable
span_id: fixed_size_binary[8], nullable
activity_id: fixed_size_binary[16], nullable
related_activity_id: fixed_size_binary[16], nullable

raw_payload: binary, nullable
raw_payload_format: dictionary<utf8>, nullable
flags: u32, non-null
```

The `resource` and `scope` structs follow the same physical and semantic
pattern as the existing logs, metrics, and traces root payloads. Their IDs are
the parent IDs used by `RESOURCE_ATTRS` and `SCOPE_ATTRS`. The root
`schema_url` applies to telemetry produced by the scope; the resource schema
URL remains inside the resource struct.

Source provider identity is not automatically an instrumentation scope.
EventSource, ETW, perf, tracefs, and other provider identities remain in
`FLIGHT_EVENT_SCHEMAS`. A source adapter or semantic mapper assigns a scope
only when the source has instrumentation-scope semantics or a documented
mapping establishes them.

The initial `kind` vocabulary is:

```text
EVENT
SAMPLE
COUNTER
PROCESS
THREAD
MAPPING
SEQUENCE_POINT
LOSS
CLOCK_SYNC
```

The enum describes the role of an observation. Provider-specific identity
belongs in the referenced event schema, not in `kind`.

`time_unix_nano` is source event time converted to Unix nanoseconds.
`observed_time_unix_nano` records receiver observation time when available.
Clock conversion uncertainty or discontinuity is represented through flags,
attributes, and `CLOCK_SYNC` records rather than silently rewriting order.

Physical input ordering is preserved by row order and, where available,
`sequence_number`. Components that reorder rows must update the ordering
contract they advertise. Event time alone is not sufficient because multiple
clocks, equal timestamps, and late events are possible.

### Event schemas

`FLIGHT_EVENT_SCHEMAS` normalizes provider and event identity:

```text
id: u32, non-null
source: u8, non-null
provider_name: dictionary<utf8>, nullable
provider_id: fixed_size_binary[16], nullable
event_id: u32, nullable
event_name: dictionary<utf8>, nullable
version: u16, nullable
level: u8, nullable
opcode: u8, nullable
task: u16, nullable
keywords: u64, nullable
byte_order: u8, nullable
pointer_width: u8, nullable
source_metadata: binary, nullable
flags: u32, non-null
```

The `source` value distinguishes at least EventPipe, ETW, perf, tracefs,
user_events, synthetic OpenTelemetry records, and unknown sources.

`source_metadata` is reserved for metadata that is necessary to decode or
re-emit a schema but does not yet have a canonical column. It must not become a
default dumping ground for fields that should be queryable.

`FLIGHT_SCHEMA_FIELDS` contains a pre-order schema tree:

```text
schema_id: u32, non-null
field_id: u16, non-null
parent_field_id: u16, nullable
ordinal: u16, non-null
name: dictionary<utf8>, nullable
logical_type: u8, non-null
wire_type: u16, nullable
element_type: u8, nullable
offset: u32, nullable
fixed_size: u32, nullable
length_field_id: u16, nullable
flags: u16, non-null
```

`logical_type` includes signed integer, unsigned integer, float, boolean,
string, bytes, GUID, timestamp, array, and struct. `wire_type` preserves
source-specific distinctions such as `FILETIME`, UTF-16, counted string,
pointer, or tracefs dynamic-relative data.

The distinction is necessary because two different wire encodings can produce
the same logical Arrow value while requiring different source reconstruction.

### Event values

`FLIGHT_EVENT_VALUES` represents arbitrary fields without dynamic columns:

```text
record_id: u32, non-null
field_id: u16, non-null
value_id: u32, non-null
parent_value_id: u32, nullable
element_index: u32, nullable
value_type: u8, non-null

signed_value: i64, nullable
unsigned_value: u64, nullable
double_value: f64, nullable
boolean_value: bool, nullable
string_value: utf8, nullable
bytes_value: binary, nullable
```

Exactly one scalar value column is present for scalar rows. Container rows have
no scalar column and own child rows through `parent_value_id`.
`element_index` preserves array order. A presence flag distinguishes absent,
null, empty, truncated, and decode-error cases where the source makes those
states distinct.

Nullable typed columns are favored over string conversion or a provider-
specific Arrow schema. They preserve integer signedness, byte strings, and
numeric filtering while keeping the payload vocabulary fixed.

The receiver supports three fidelity policies:

<!-- markdownlint-disable MD013 -->
| Policy | Decoded values | Raw payload | Intended use |
| --- | --- | --- | --- |
| `decoded` | Yes | No, except unsupported values | Columnar processing with bounded size |
| `source_fidelity` | Yes | Yes | Reversible archival and codec validation |
| `minimal` | Selected fields | Yes | Low-cost capture before later decoding |
<!-- markdownlint-enable MD013 -->

The default policy remains unresolved. Raw payload retention can contain
sensitive data and substantially increase memory and file usage.

### Attributes and correlation

`FLIGHT_ATTRS` specializes the existing OTAP typed attribute value model:

```text
parent_id: u32, non-null
parent_kind: u8, non-null
key: dictionary<utf8>, non-null
type: u8, non-null
value columns: existing OTAP AnyValue representation
unit: dictionary<utf8>, nullable
```

The `key`, `type`, `str`, `int`, `double`, `bool`, `bytes`, and serialized-value
columns retain the names, types, and dictionary conventions of existing OTAP
attribute payloads. The initial parent kinds are record, mapping, and location.
Sample attributes attach to the sample's root record. Resource and
instrumentation scope attributes continue to use `RESOURCE_ATTRS` and
`SCOPE_ATTRS`.

The optional `unit` column is a common OTAP attribute-schema extension needed
by OTLP Profiles, whose `AttributeUnit` relates an attribute key to a unit.
OTAP denormalizes that relation onto attribute rows so a split or filtered
pdata remains self-contained and dictionary encoding compresses repeated
units. Within one pdata, all non-null units for the same attribute key must
agree. A Profiles encoder deduplicates `(key, unit)` pairs into the OTLP
Profiles attribute-unit table.

Attribute units are distinct from metric instrument units and profile sample
type units. Existing OTLP logs, metrics, and traces cannot carry attribute-unit
metadata. Their OTAP attribute payloads may adopt the optional column as a
common physical-schema extension, but their signal contracts do not assign it
meaning unless the corresponding OTLP signal gains that capability.

Common correlation fields remain dedicated columns on `FLIGHT_RECORDS` so
trace/span filtering does not scan an attribute table. Source-specific labels
and V6 label kinds without dedicated semantics use attributes. A normative
mapping must define when Activity IDs and W3C trace/span IDs are equivalent and
when both must be retained.

### Samples and sample types

`FLIGHT_SAMPLES` identifies records with normalized profile-sample semantics:

```text
record_id: u32, non-null
period_type_id: u16, nullable
period: i64, nullable
flags: u16, non-null
```

The record supplies timestamp, resource, scope, process, thread, stack, and
trace correlation. Sample-specific labels use `FLIGHT_ATTRS` with the root
record as parent.

`FLIGHT_SAMPLE_VALUES` contains the measurement vector:

```text
record_id: u32, non-null
sample_type_id: u16, non-null
value: i64, non-null
```

`FLIGHT_SAMPLE_TYPES` defines each measurement:

```text
id: u16, non-null
type: dictionary<utf8>, non-null
unit: dictionary<utf8>, non-null
aggregation_temporality: u8, non-null
```

One sample may have several values. This matches the OTLP Profiles requirement
that every `Sample.value` position corresponds to a `Profile.sample_type`
entry.

The source adapter or a source-specific mapping processor creates sample rows.
The generic profile assembler never infers sample semantics from provider or
event display names.

### Stacks, mappings, locations, and symbols

`FLIGHT_STACK_FRAMES` is the stack-to-location relation:

```text
stack_id: u32, non-null
ordinal: u16, non-null
location_id: u32, non-null
```

The ordinal direction is fixed by the schema version and must not depend on the
source's native stack ordering.

`FLIGHT_LOCATIONS` represents executable locations:

```text
id: u32, non-null
mapping_id: u32, nullable
address: u64, non-null
is_folded: bool, non-null
flags: u16, non-null
```

`FLIGHT_MAPPINGS` represents loaded images and address ranges:

```text
id: u32, non-null
process_id: u32, nullable
memory_start: u64, non-null
memory_limit: u64, non-null
file_offset: u64, non-null
filename: utf8, nullable
build_id: binary, nullable
load_time_unix_nano: i64, nullable
unload_time_unix_nano: i64, nullable
flags: u16, non-null
```

`FLIGHT_SYMBOLS` contains zero or more symbol rows per location:

```text
location_id: u32, non-null
inline_ordinal: u16, non-null
function_name: utf8, nullable
system_name: utf8, nullable
filename: utf8, nullable
start_line: i64, nullable
line: i64, nullable
column: i64, nullable
flags: u16, non-null
```

Keeping symbols separate from locations allows asynchronous symbolization to
enrich a graph without replacing stack references. It also represents inline
frames while avoiding a separate function-interning table in the Flight
Recorder representation. Conversion to Profiles may dictionary-encode and intern
repeated functions into the Profiles function table.

Native and JIT mappings use the same table. Source attributes distinguish
mapping kinds and carry runtime-specific identity that has no normalized field.

### Referential integrity

A valid Flight Recorder pdata satisfies all of the following:

- root record IDs are unique;
- every non-null schema ID resolves;
- every event value references a root record and a field in its schema;
- every sample references a root record;
- every sample value references both a sample and a sample type;
- every non-null stack ID has one or more ordered stack-frame rows;
- every stack frame references a location;
- every non-null location mapping resolves;
- every symbol references a location;
- every attribute parent resolves for its declared parent kind; and
- every referenced resource and scope definition is present.

Rows not reachable from a root record are invalid unless a payload type
explicitly permits prospective definitions. The initial version does not
permit prospective definitions.

Validation checks graph shape, type consistency, duplicate IDs, numeric
overflow, and configurable nesting and collection limits.

### Split, concatenate, filter, and reindex

Flight Recorder is a graph representation. Existing root-only batch operations
are not sufficient.

Splitting or filtering performs reachability:

1. select root records;
2. find referenced schemas, stacks, sample types, resources, and scopes;
3. transitively select fields, values, locations, mappings, symbols,
   attributes, samples, and sample values;
4. compact each selected table; and
5. remap every foreign key.

Concatenation assigns disjoint ID ranges or builds remapping tables before
combining children. Transport optimization may sort and delta-encode IDs only
when it remaps all affected descendants transactionally.

These operations should be implemented as reusable graph-table primitives.
Future OTAP Profiles needs the same reachability, compaction, and remapping
capabilities for its string, attribute, mapping, location, function, and link
tables.

### Association with OTAP semantic signals

Flight Recorder uses the same resource and instrumentation-scope hierarchy as
the existing OTAP signals:

```text
Resource
  -> InstrumentationScope
       -> signal root
```

For Flight Recorder, the representation root is `FLIGHT_RECORDS`. For standard
signals the roots are logs, spans, or metrics. A derived signal inherits the
Flight record's resource and scope unless a documented semantic mapping
establishes a different scope. A source provider name alone is not sufficient
to invent an instrumentation scope.

The following columns and structures reuse existing OTAP decisions where their
semantics match:

<!-- markdownlint-disable MD013 -->
| Flight Recorder concept | Existing or future OTAP association |
| --- | --- |
| embedded resource and scope | Same root structs and `RESOURCE_ATTRS` and `SCOPE_ATTRS` hierarchy as logs, metrics, and traces |
| `time_unix_nano` | Event or measurement time used by logs and metric data points |
| `observed_time_unix_nano` | Receiver observation time used by logs |
| `trace_id`, `span_id`, and `flags` | Log correlation columns, trace identity, and future Profile links |
| `schema_url` | Scope telemetry schema URL used by existing OTAP roots |
| `dropped_attributes_count` | Existing OTel and OTAP loss convention |
| `FLIGHT_ATTRS` value columns | Existing OTAP typed AnyValue attribute representation |
| attribute `unit` | Future common extension corresponding to OTLP Profiles `AttributeUnit` |
| sample type `unit` | Profile `ValueType.unit`; related to but distinct from a metric instrument unit |
| `aggregation_temporality` | Existing metrics vocabulary reused by OTLP Profiles |
| parent IDs and graph remapping | Existing OTAP child-table and transport-optimization pattern |
<!-- markdownlint-enable MD013 -->

Source event values are not automatically OTel attributes. Event values may
contain unsigned integers, GUIDs, nested structures, repeated fields, source
wire types, or distinctions between absent, null, truncated, and invalid
values. `FLIGHT_EVENT_VALUES` preserves those source semantics. A mapper
promotes only fields with defined OTel semantics into attributes or dedicated
signal columns.

#### Logs

A structured Flight event can produce an OTAP log when a mapping defines its
event name, body, severity, attributes, and timestamp semantics. The derived
log reuses the Flight resource, scope, event and observed times, trace and span
correlation, flags, and common attribute representation. Provider event fields
that have no log semantic mapping remain in Flight event values rather than
being stringified or silently dropped.

An arbitrary source event is not automatically a log. Pipelines may retain the
Flight record, emit a derived log, or do both.

#### Traces

Activity, EventPipe, ETW, and similar records may contribute to spans, span
events, or links. Span construction is stateful because start, stop, event,
parent, and link information can arrive in separate records. Dedicated Flight
trace and span columns permit filtering and correlation without scanning
attributes, while source Activity identifiers remain separately available
when they are not equivalent to W3C trace context.

The trace mapper must define incomplete-span, late-event, rundown,
out-of-order, and bounded-state behavior. Flight preserves the contributing
observations even when a complete span cannot be constructed.

#### Metrics

Runtime counters and event pairs may produce OTAP gauges, sums, histograms,
exemplars, or other metric points only when a mapping defines instrument name,
description, unit, temporality, monotonicity, point timestamps, and reset
behavior. The derived metric inherits resource and scope and uses the existing
OTAP metric and data-point payloads.

Metric measurement units remain on the metric root. They are not attribute
units. Counter display names or source field names alone are insufficient to
infer metric type, unit, temporality, or monotonicity.

#### Profiles

The seven profile-aligned Flight payloads are intended to share logical
schemas and graph operations with a future OTAP Profiles signal:

<!-- markdownlint-disable MD013 -->
| Flight Recorder payload | Future OTAP Profiles association |
| --- | --- |
| `FLIGHT_SAMPLES` | Profile samples before windowing and optional aggregation |
| `FLIGHT_SAMPLE_VALUES` | Sample measurement vectors |
| `FLIGHT_SAMPLE_TYPES` | Profile sample and period value types |
| `FLIGHT_STACK_FRAMES` | Ordered location-index slices |
| `FLIGHT_LOCATIONS` | Shared location graph |
| `FLIGHT_MAPPINGS` | Shared executable mapping graph |
| `FLIGHT_SYMBOLS` | Input to Profile function and line tables |
<!-- markdownlint-enable MD013 -->

`FLIGHT_ATTRS` also supplies sample, mapping, and location attributes, and
Flight trace/span correlation supplies Profile links. These are common
infrastructure rather than part of the seven profile-aligned payload count.

Profile assembly can group several Flight sample records into one Profile
sample. Consequently, Flight-to-Profiles is a semantic projection even when
the underlying Arrow schemas and graph operations are shared.

### Flight Recorder to Profiles

The conversion has a source-specific stage and a generic stage.

#### Source-specific extraction

Source adapters may create `FLIGHT_SAMPLES` directly. Mapping processors may
also recognize ordinary records using schema identity and typed values:

```text
FLIGHT_RECORDS
  join FLIGHT_EVENT_SCHEMAS
  join FLIGHT_EVENT_VALUES
  filter recognized provider/event/schema
  project normalized sample rows
```

Examples include:

- perf sampling records;
- ETW sampled-profile events;
- EventPipe CPU samples;
- allocation ticks;
- contention samples; and
- exception samples.

Source-specific mappings must define value type, unit, sampling period, stack
direction, process identity, and loss behavior. They must not infer
monotonicity, temporality, or units from display names alone.

#### Generic assembly

The generic assembler uses Arrow filtering, joining, grouping, list
construction, dictionary compaction, and index remapping:

```text
sample records
  -> partition by resource, scope, window, and sample-type set
  -> join measurement vectors
  -> group compatible stack/attribute/link keys
  -> sum values and collect timestamps
  -> collect ordered locations per stack
  -> compact mapping/location/function/attribute/link tables
  -> emit OTAP Profiles
```

The structural mapping is:

<!-- markdownlint-disable MD013 -->
| Flight Recorder | OTLP Profiles |
| --- | --- |
| resource and scope references | `ResourceProfiles` and `ScopeProfiles` |
| `FLIGHT_SAMPLE_TYPES` | `Profile.sample_type` |
| `FLIGHT_SAMPLE_VALUES` | `Sample.value` |
| root event timestamps | `Sample.timestamps_unix_nano` |
| stack ID | sample location slice |
| `FLIGHT_STACK_FRAMES` | `Profile.location_indices` |
| `FLIGHT_LOCATIONS` | location table |
| `FLIGHT_MAPPINGS` | mapping table |
| `FLIGHT_SYMBOLS` | function and line tables |
| sample attributes | attribute table |
| trace/span columns | link table |
<!-- markdownlint-enable MD013 -->

Profile windows, IDs, default sample type, comments, and aggregation policy are
assembler concerns. They are not properties of an individual source
observation.

### one_collect mapping

The one_collect integration maps data at callback and decoder boundaries:

<!-- markdownlint-disable MD013 -->
| one_collect information | Flight Recorder payload |
| --- | --- |
| event format and provider metadata | event schemas and schema fields |
| event callback | root record |
| decoded event fields | event values |
| perf or runtime sample | sample and sample values |
| stack records | stack frames and locations |
| module/mapping records | mappings |
| native and JIT symbols | symbols |
| callback labels and Activity IDs | correlation columns and attributes |
| lost-event callbacks | loss records |
| sample/QPC timestamps | root source and observed timestamps |
<!-- markdownlint-enable MD013 -->

Callback-owned data is copied into a bounded queue or directly into a local
builder under a documented ownership contract. Queue limits include both event
count and retained bytes. Drain work is cooperatively limited by record count,
bytes, and time.

The live receiver does not use a continuously growing one_collect
`ExportMachine` as its handoff. Whole-capture export remains useful for
offline conversion and interoperability tests.

### NetTrace V6 mapping

NetTrace V6 is a serialization backend and source, not the in-memory table
layout:

<!-- markdownlint-disable MD013 -->
| NetTrace V6 concept | Flight Recorder payload |
| --- | --- |
| TraceBlock metadata | capture resource, scope, and stream metadata |
| MetadataBlock | fixed internal provider schemas for Flight table-row events |
| EventBlock | Flight table rows, including source schemas, values, and raw chunks |
| StackBlock | optional informational stack entries; not authoritative Flight IDs |
| Thread and thread-removal blocks | thread context and lifecycle records |
| LabelListBlock | correlation columns and attributes |
| SequencePointBlock | serialization epochs and block-local identity reset |
| EndOfStream | receiver/exporter lifecycle, not a pdata row |
<!-- markdownlint-enable MD013 -->

Loss remains an ordinary `FLIGHT_RECORDS` root row encoded through the declared
internal provider. The native V6 `StackBlock` assigns its own sequential stack
IDs and therefore cannot preserve externally meaningful Flight `stack_id`
values. Authoritative stack, location, mapping, and symbol rows are encoded as
metadata-declared events; native stack entries may be emitted for independent
reader tooling but are not used to reconstruct the Flight graph.

The rows in this table describe the V6 source-observation record class. A
mixed V6 stream may also contain normalized log, trace, metric, and profile
records. Those records decode directly into their corresponding OTAP signals;
they do not pass through `FLIGHT_EVENT_VALUES` and are not presented as
reconstructed source observations.

Conceptually, a mixed V6 receiver has five outputs:

```text
NetTrace V6 mixed stream
  +-> OTAP Flight
  +-> OTAP logs
  +-> OTAP traces
  +-> OTAP metrics
  +-> OTAP Profiles
```

The prototype currently implements the writer and a focused test reader. A
production NetTrace V6 receiver remains a follow-up.

The V6 profile must define record-class identity and optional derivation
provenance independently of provider names. This prevents normalized signal
records from colliding with source providers that use similar event names.

The decoder resolves V6 stream-local IDs and produces compact pdata-local IDs.
The encoder performs the inverse operation, emitting definitions before first
use and maintaining bounded per-segment caches.

V6 event timestamps represent serialization observation order when normalized
OTel signals are written. For source Flight Recorder records, the writer
retains source time explicitly and preserves sequence information. A normative
V6 profile will distinguish container order time from source event time.

### Pdata and protocol integration

RFC 0006 owns the pipeline and protocol integration. This RFC requires the
extension framework to provide:

- a named `otap.flight` Arrow record-set representation at version 1.0;
- extension-local table identifiers for the 14 payloads rather than global
  `ArrowPayloadType` values;
- whole-representation validation before a Flight record set enters a
  pipeline;
- retained-memory, logical-size, and root-item accounting;
- split, concatenate, filter, sanitize, reachability, and reindex support;
- Arrow IPC encoding for the extension record set when an OTAP transport or
  durable store carries it;
- component capabilities for inspect, transform, pass-through, encode, decode,
  and reject behavior; and
- explicit rejection by OTLP exporters unless Flight has been projected into
  a standard OpenTelemetry signal.

The first prototype may use local numeric table values internally, but those
values do not reserve or assign entries in the shared experimental Arrow
payload enum. The RFC 0006 Pluggable PData Arrow container provides the
representation namespace and version.

### Batching and bounded state

A Flight Recorder pdata is self-contained even though sources are streaming.
Receivers may retain bounded session dictionaries for source schemas, stacks,
mappings, and symbols, but each emitted pdata copies and remaps the definitions
reachable from its records.

Builders flush when any configured limit is reached:

- root record count;
- decoded value count;
- total Arrow logical bytes;
- retained raw-payload bytes;
- schema and field definitions;
- stack frames;
- mappings, locations, or symbols;
- dictionary cardinality; or
- cooperative processing deadline.

An oversized individual record is either emitted in a bounded oversize batch
under explicit policy or rejected with loss accounting. It is never allowed to
bypass all limits silently.

### Error and loss semantics

Loss origins remain distinguishable:

- source/runtime loss;
- kernel or recorder buffer loss;
- one_collect decoder loss;
- receiver pending-queue overflow;
- invalid or unsupported schema;
- payload decode failure;
- processor filtering or aggregation loss; and
- exporter failure.

Source and kernel loss observed in the stream produce `LOSS` records with
source, interval, and count where known. Internal component loss also updates
component telemetry. A decode failure does not create a successful-looking
empty event: source-fidelity mode retains the raw record with decode-error
flags, while stricter policies reject it and account for the loss.

### Security and privacy

Runtime events and raw payloads may contain credentials, file paths, query
text, user data, source code identifiers, and arbitrary provider-defined
content. Source-fidelity capture therefore requires:

- explicit configuration;
- documented file and transport sensitivity;
- normal pipeline access controls;
- bounded raw payload length;
- bounded string, array, and nesting sizes;
- rejection of cyclic or invalid schema graphs;
- checked arithmetic for offsets and lengths; and
- fuzzing of NetTrace, EventPipe metadata, ETW metadata, and generic value
  decoding.

Schema and provider cardinality are attacker-controlled for untrusted inputs.
All definition caches require size and eviction policies. Eviction must not
leave unresolved references in an emitted pdata.

### Validation and interoperability

The implementation should include:

- canonical schema fixtures for every payload;
- graph validation tests;
- split/concatenate/filter/reindex property tests;
- source-event round trips through the V6 codec;
- one_collect fixtures for ETW, perf, user_events, and EventPipe where
  supported;
- profile construction fixtures covering native, JIT, and inline frames;
- malformed schema and payload fuzz targets;
- loss and sequence-point fixtures;
- bounded-memory and backpressure tests; and
- interoperability fixtures readable by an independent NetTrace V6 reader.

The central equivalence tests compare normalized logical records, not raw Arrow
buffers or byte-identical V6 files.

## Drawbacks

- Flight depends on a new cross-cutting extension envelope and capability
  system before it can traverse ordinary pipelines.
- Fourteen payloads are a substantial schema and graph-maintenance surface.
- Generic event values use more rows than decoding provider fields into a
  bespoke wide table.
- Retaining both decoded values and raw payloads can duplicate data.
- Self-contained batches repeat hot schemas, stacks, mappings, and symbols.
- Graph-preserving split, filter, and concatenate operations are more
  expensive than slicing a single root table.
- Source-fidelity data can expose more sensitive content than normalized
  OpenTelemetry signals.
- one_collect and NetTrace expose platform-specific details that cannot all be
  assigned portable OpenTelemetry semantics.
- The future Profiles representation may evolve in ways that require changes
  to the proposed stack and sample tables.

## Rationale and alternatives

### Why this design

The design separates three concerns:

1. source fidelity in the event schema and value tables;
2. normalized sampling semantics in the sample tables; and
3. shared execution structure in the stack, location, mapping, and symbol
   tables.

This preserves arbitrary observations while making the common profile path
columnar. The table count is comparable to existing metrics OTAP, and provider
cardinality affects rows and dictionaries rather than the protocol vocabulary.

### Use only opaque source payloads

An opaque payload provides strong byte preservation and a small schema, but
Arrow cannot filter fields, identify samples, aggregate values, or construct
Profiles without reparsing every record. It also ties processors to every
source codec.

Raw payload remains an optional fidelity mechanism, not the primary data
model.

### Use one Arrow schema per provider or event

Provider-specific wide tables provide excellent access for known events but
create unbounded schema cardinality. EventSource, TraceLogging, and user_events
can define schemas dynamically. Pipeline components and the OTAP protocol
cannot enumerate or optimize an unbounded payload set.

### Represent every event as an OpenTelemetry log

Logs can preserve many event fields as attributes and are useful as a semantic
projection. They do not retain all source wire types, stack graphs, metadata
definitions, ordering, sequence points, or sampling structure. Mapping every
event to a log also makes profile construction depend on conventions encoded
in arbitrary log attributes.

### Reproduce NetTrace V6 blocks as Arrow tables

This would simplify a V6 codec but expose serialization mechanics to every
processor. Blocks, metadata IDs, sequence points, and definition epochs are
streaming codec concerns, not the best relational model for filtering and
profile construction.

### Use only OTAP Profiles

Profiles represent normalized sample graphs well, but they do not represent
arbitrary runtime events, process and thread lifecycle, sequence points, or
source metadata. Converting at collection time would lose information and make
new semantic mappings require new captures.

### Use Arrow `Map` or nested `Struct` values in one event table

A single nested value column reduces table count, but generic nested traversal
and source schema reuse become harder to optimize. A separate schema table and
typed value relation allow projection of selected fields, dictionary reuse,
and preservation of repeated or nested values using ordinary joins and
filters.

The exact physical representation of generic values remains open to
benchmarking. A dense union or structured AnyValue column may outperform
nullable typed columns while preserving the same logical model.

### Do nothing

Without a Flight Recorder Pluggable PData Arrow Representation, the one_collect
receiver must either emit irreversible logs/metrics/traces, retain opaque
source blobs, or create a component-specific representation outside pdata.
NetTrace V6 replay and future Profiles integration would then duplicate
decoding, stack, and symbol logic.

## Prior art

### Existing OTAP signals

Logs, metrics, and traces already use a root table plus statically known child
tables, parent IDs, shared resource/scope attribute payloads, and transport
reindexing. Flight Recorder follows this pattern while making graph
reachability more explicit.

Metrics demonstrates that a signal can reasonably contain many table types
when they represent distinct data-point and exemplar structures. Traces
demonstrates ordered child entities and correlation identifiers.

### OpenTelemetry Profiles

The development Profiles data model uses shared string, attribute, mapping,
location, function, and link tables. Samples reference location sequences and
measurement vectors. This RFC deliberately aligns samples and executable
structure with that graph.

Profiles also defines `original_payload_format` and `original_payload` for
extensible source formats. That is useful for profile provenance but is not a
replacement for routable and queryable mixed-signal Flight Recorder records.

### one_collect

one_collect provides a cross-platform collection and export model spanning
ETW, perf, tracefs, EventPipe user_events, samples, stacks, modules, symbols,
and NetTrace V6. Its callbacks and whole-capture export machinery demonstrate
both the available source information and the need to avoid unbounded
whole-session accumulation in a live receiver.

### NetTrace V6

NetTrace V6 supplies a self-describing stream with metadata, heterogeneous
events, stacks, threads, labels, and sequence points. Its skippable blocks and
definition mechanisms are suitable for a mixed-signal flight-recorder file,
while its block layout remains an encoding concern.

### pprof

pprof and the profile model derived from it demonstrate compact graph tables
for samples, mappings, locations, functions, and labels. They also demonstrate
that an aggregated profile is not a reversible event timeline.

## Unresolved questions

The following must be resolved before accepting the schema:

1. Should `FLIGHT_EVENT_VALUES` use nullable typed columns, an Arrow dense
   union, or the existing OTAP AnyValue physical representation?
2. Are record, mapping, and location sufficient as the initial
   `FLIGHT_ATTRS` parent kinds, and does the added discriminator permit enough
   reuse of common attribute builders and processors?
3. Which raw-payload fidelity policy is the default, and which sources can
   claim semantic or byte-level reconstruction?
4. Which event-schema wire types are normative across ETW, EventPipe, perf,
   and tracefs?
5. How are capture/session identity, clock domains, and clock conversion
   uncertainty represented?
6. Are process and thread lifecycle observations sufficient as root records,
   or do they require dedicated entity tables?
7. Should function interning be part of Flight Recorder OTAP, or remain a
   Profiles-conversion optimization as proposed?
8. What is the canonical stack direction?
9. How are partially symbolized and subsequently enriched records represented
   without violating pdata immutability?
10. What extension-local table identifiers and schema-version policy should
    `otap.flight` 1.0 use?
11. Which graph operations belong in generic pdata infrastructure rather than
    Flight Recorder-specific code?
12. How closely should the stack and sample schemas track the eventual OTAP
    Profiles schemas before those schemas are standardized?

The common structural associations are defined above. The following detailed
semantic mapping questions remain a subsequent design phase:

1. Which Flight Recorder events map losslessly to OpenTelemetry LogRecords,
   including event name, body, severity, attributes, resource, scope, and
   trace correlation?
2. Which Activity and EventPipe records reconstruct spans, span events, and
   links, and what state and completion rules are required?
3. Which runtime counters and event pairs produce gauges, sums, histograms,
   and exemplars without inventing temporality or monotonicity?
4. How should derived logs, metrics, traces, and profiles retain provenance
   back to a source record without embedding the full source envelope?
5. What V6 fields identify normalized records as derived and link them to a
   source capture, record, or record range without creating unbounded
   provenance state?

Detailed component configuration, the complete OTel-over-NetTrace V6 provider
profile, and the OTAP Profiles table schemas are outside the initial schema
decision and may be specified in follow-up RFCs or amendments.

## Future possibilities

- A one_collect auto-instrumentation receiver that emits Flight Recorder data
  on Windows and Linux.
- A V4/V5 NetTrace receiver that normalizes EventPipe streams into the same
  tables.
- Bidirectional NetTrace V6 file and stream components.
- An OTAP Profiles signal sharing graph validation, reachability, compaction,
  and remapping infrastructure.
- Source-specific processors for .NET, ETW, perf, eBPF, tracefs, JFR, and
  other structured recorders.
- Deferred or remote symbolization that enriches mappings and locations.
- Query-engine views that expose provider fields as virtual columns while
  retaining the static physical schema.
- Predicate pushdown into raw payload decoding so processors decode only
  fields required by a query or semantic mapping.
- Recording policies that retain raw source events only around anomalies while
  continuously producing normalized telemetry.
- RFC 0006 mixed messages containing source records, logs, metrics, traces, and
  profiles, encoded as one time-ordered V6 stream.
- Provenance relations that connect derived OpenTelemetry entities to source
  Flight Recorder IDs within a bounded capture or file segment.
