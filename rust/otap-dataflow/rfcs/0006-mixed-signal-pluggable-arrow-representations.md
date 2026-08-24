---
Proposal Name: mixed-signal-pluggable-arrow-representations
Start Date: 2026-08-24
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC 0006: Mixed-Signal PData and Pluggable PData Arrow Representations

## Summary

Add two related capabilities to OTAP pipeline data:

1. **Mixed-Signal PData**, which allows one message to carry zero or more
   OpenTelemetry semantic signals plus source-observation content; and
2. **Pluggable PData Arrow Representations**, which allow a registered,
   named, and versioned set of Arrow `RecordBatch` values to be a first-class
   in-pipeline PData representation.

This RFC aligns with
[issue #3452](https://github.com/open-telemetry/otel-arrow/issues/3452),
**Pluggable PData Byte Representations**. Issue #3452 generalizes OTLP
protobuf-byte passthrough to other encoded byte formats that remain
first-class inside the engine. This RFC defines its Arrow counterpart and the
mixed-signal envelope both representation families need.

The motivating example is the NetTrace V6 Flight Recorder prototype from RFC
0005. Its in-pipeline representation is the 14-table `otap.flight` 1.0 Arrow
record set, a Pluggable PData Arrow Representation. NetTrace bytes are not
carried in the pipeline: a receiver decodes them before producing PData, and a
dedicated file or transport exporter encodes them at the boundary. Therefore
`nettrace_v6` is a wire-only codec profile, not a Pluggable PData Byte
Representation.

## Motivation

The current `OtapPdata` model assumes one homogeneous OpenTelemetry signal and
one active representation:

```text
one message
  -> one SignalType
  -> one of OTLP bytes or OTAP Arrow records
```

That is appropriate for ordinary logs, metrics, and traces, but it cannot
express several important cases:

- a source observation that may project into logs, traces, metrics, or
  profiles without already being any one of them;
- a message containing several standard signals that must be recorded or
  committed together;
- a source-preserving Arrow graph that overlaps normalized OpenTelemetry
  signals;
- a Pluggable PData Byte Representation from issue #3452;
- a Pluggable PData Arrow Representation consisting of several related record
  batches;
  and
- an exporter such as NetTrace V6 that serializes a mixed stream through one
  stateful writer rather than one file per semantic signal.

RFC 0005 originally proposed Flight Recorder as a fourth `SignalType`.
Prototype implementation showed why that is the wrong abstraction:

- Flight is a representation of source observations, not a new OpenTelemetry
  semantic signal.
- Adding a signal changes exhaustive routing, metrics, batching, exporters,
  protocol enums, and durable-buffer logic even when those components have no
  Flight semantics.
- The same message may contain Flight plus logs, traces, metrics, or future
  profiles.
- NetTrace V6 is a wire format, while Flight is the useful in-pipeline Arrow
  form. Treating either as the signal conflates semantics and encoding.
- A global Arrow payload enum and 64-bit payload masks do not provide a
  scalable namespace for pluggable Arrow table families.

The desired model is:

```text
OTAP message
  semantic content:
    zero or more of logs, metrics, traces, and future profiles

  PData representations:
    native OTAP signal record sets
    Pluggable PData Arrow Representations
    Pluggable PData Byte Representations

  boundary-only codecs:
    NetTrace V6
    other formats that are never carried as PData bytes
```

The model must keep unknown PData representations visible, bounded, and
rejectable. No processor, transport, durable buffer, or exporter may silently
drop a representation it does not understand.

### Relationship to Pluggable PData Byte Representations

Issue #3452 already defines the terminology and intent for Pluggable PData Byte
Representations:

- a stable `PDataEncoding` identity;
- bytes carried as a first-class representation inside `OtapPdata`;
- a registered `PDataCodec` to encode or decode to native OTAP;
- passthrough without decoding for routing, retry, buffering, and compatible
  exporters;
- lazy conversion when a processor needs telemetry contents;
- supported signal metadata available outside the encoded bytes; and
- optional encoded-buffer batching without inspecting individual records.

This RFC does not replace that accepted issue. It adds the corresponding Arrow
representation model and updates the PData envelope so either pluggable
representation family can participate in mixed-signal messages.

The distinction from a dedicated exporter is normative:

> Use a Pluggable PData Byte Representation when encoded bytes must be carried,
> routed, retried, buffered, or lazily decoded inside the engine. Use a
> dedicated exporter when bytes are produced only at the final boundary.

NetTrace V6 follows the second case.

## Guide-level explanation

### Semantic signals and PData representations are different axes

A semantic signal states what OpenTelemetry data means: logs, metrics, traces,
or profiles. A representation states how some message content is physically
carried.

`SignalType` remains the vocabulary for individual OpenTelemetry semantic
signals. Flight Recorder, NetTrace V6, OTLP protobuf, and OTAP Arrow are not
new semantic signals.

A message has a semantic signal set:

```text
{}
{logs}
{traces}
{logs, traces}
{logs, metrics, traces}
```

It also has one or more representation entries. A message with an empty
semantic signal set is valid only when it carries a registered Pluggable PData
Arrow Representation whose contract defines non-signal source observations,
such as Flight Recorder.

### Message classification is derived

Mixedness is not stored as another enum beside the manifest. Pipeline code
derives it from the semantic signal set and representation entries:

```rust
fn semantic_signals(&self) -> SignalSet;
fn single_signal(&self) -> Option<SignalType>;
fn has_pluggable_representations(&self) -> bool;
```

An implementation may offer a convenience classification, but it is a pure
derived value and no semantic decision may depend on it instead of the
manifest. This avoids creating a second source of truth or recreating
`SignalType::Mixed` under another name.

### PData content entries

Each content entry has a message-local identity, declared semantic coverage,
provenance, and exactly one active physical representation:

```text
ContentEntry
  content_id
  semantic_signals: set<SignalType>
  provenance: independent | source_observation | derived_from(content_id...)

  representation:
  format_id
  format_version: major.minor
  physical_kind: arrow | bytes
  payload
```

An alternative encoding replaces the active representation of a content entry
while retaining its `content_id`; it is not stored as a second live copy that
must remain synchronized. A derived log or trace is a new content entry with a
new `content_id` and provenance pointing to its source entry. It is not an
alternative representation of the source.

This rule avoids authority and staleness ambiguity while still permitting a
content entry to move between OTLP bytes, standard OTAP Arrow, a Pluggable
PData Arrow layout, or another registered encoding.

### Pluggable PData Arrow Representations

A Pluggable PData Arrow Representation is a named set of record batches:

```text
ArrowRecordSet
  format_id: "otap.flight"
  format_version: 1.0
  tables:
    local_table_id -> RecordBatch
```

Table identifiers are local to the representation format and version. They do
not consume global `ArrowPayloadType` enum values and are not limited by the
current `u64` payload masks used by standard OTAP stores and Quiver slots.

The format contract defines:

- the table vocabulary and canonical schemas;
- required and optional tables;
- root tables and item accounting;
- local identifier domains and graph invariants;
- self-containment rules;
- retained-memory and logical-size accounting;
- split, concatenate, filter, sanitize, and reindex behavior; and
- compatibility rules between format versions.

Format IDs never embed a version suffix. A format version is a `(major,
minor)` pair. A major version change may alter required schemas or semantics
and is incompatible by default. A minor version may add optional,
self-describing content only when the format contract defines how older
implementations skip it safely.

Unknown Pluggable PData Arrow formats are rejected in the first
implementation. A future opaque pass-through capability may be added only with
explicit bounds and trust-boundary rules.

### Pluggable PData Byte Representations

A Pluggable PData Byte Representation follows issue #3452 and is also named
and versioned:

```text
ByteRepresentation
  encoding: PDataEncoding
  format_version: major.minor
  media_type
  bytes
```

Bytes may be carried inside a pipeline when doing so has a concrete benefit,
such as zero-copy forwarding, deferred decoding, durable buffering, or
preserving a source encoding. The PData representation mechanism does not
require every wire codec to appear as an in-pipeline byte representation.

In particular, NetTrace V6 bytes are absent from the Flight pipeline:

```text
NetTrace V6 receiver
  bytes --decode--> otap.flight 1.0 Arrow record set

NetTrace V6 exporter
  otap.flight 1.0 Arrow record set --encode--> bytes
```

This avoids carrying a redundant opaque blob beside the queryable Arrow graph.
Because no pipeline component routes, retries, buffers, or passes through
NetTrace bytes, a `nettrace_v6` `PDataCodec` would provide no benefit.

### The NetTrace V6 Flight Recorder mode

The provisional profile uses two independent format identities:

```text
Arrow representation:   otap.flight 1.0
wire codec profile:      nettrace_v6
```

RFC 0005 defines the `otap.flight` 1.0 tables, graph, and provisional
source-observation mapping to NetTrace V6. This RFC defines how that record set
is attached to mixed pdata. A follow-up profile must define normalized
logs/metrics/traces/profiles in mixed V6 streams and their derivation
provenance.

One NetTrace V6 recording may contain runtime source observations, normalized
logs, metrics, traces, and future profiles. Decoding does not force every
record into a predetermined signal. A configured receiver or projection policy
chooses among three behaviors:

1. preserve source observations only as the `otap.flight` Pluggable PData
   Arrow Representation;
2. materialize any supported combination of logs, metrics, traces, and future
   profiles as derived content entries; or
3. retain both source observations and selected derived signals with explicit
   provenance.

Unknown or intentionally uninterpreted records remain plugin data in
`otap.flight`. They are not coerced into logs merely to make them routable.

An Arrow-representation-only capture:

```text
semantic signals: {}
representations:
  - content_id=1
    provenance=source_observation
    format=otap.flight 1.0
```

A capture with user-selected derived logs and traces:

```text
semantic signals: {logs, traces}
representations:
  - content_id=1, provenance=source_observation, format=otap.flight 1.0
  - content_id=2, provenance=derived_from(1), format=otap.logs 1.0
  - content_id=3, provenance=derived_from(1), format=otap.traces 1.0
```

The Flight and normalized entries may share provenance but do not share
implicit table identifiers. Every record set has its own local ID domains.

When OTAP Profiles is standardized, its canonical representation is native
OTAP for the Profiles semantic signal. Flight's profile-aligned sample, stack,
location, mapping, and symbol tables remain source-observation plugin data. A
projection may create a derived Profiles content entry without changing the
classification of the original Flight entry.

### Component behavior is capability-based

A component declares behavior per representation format:

```text
decode
encode
inspect
transform
pass_through
persist
reject
```

Capabilities are explicit:

- An OTLP exporter may accept standard signals and reject Flight.
- A Flight projection processor inspects `otap.flight` 1.0 and produces
  derived standard representations.
- A NetTrace V6 exporter inspects Flight and may also accept registered
  standard signal representations through the same writer profile.
- A durable buffer persists a representation only when it has a registered
  encoding for that representation.

Implicitly removing an unsupported representation is always an error. Explicit
configured removal is a transform with manifest, provenance, and telemetry
updates.

## Reference-level explanation

### Logical envelope model

The conceptual envelope is:

```rust
struct PdataEnvelope {
    context: Context,
    contents: Vec<ContentEntry>,
}

struct ContentEntry {
    content_id: ContentId,
    semantic_signals: SignalSet,
    provenance: Provenance,
    representation: Representation,
}

enum Representation {
    StandardArrow(StandardSignalRecords),
    PluggableArrow(PDataArrowRecordSet),
    EncodedBytes(PDataEncodedBytes),
}
```

The content-entry descriptors are the manifest. `semantic_signals()` derives
the message signal set as their union; it is not separately stored.

The exact Rust storage may keep common single-signal cases inline to avoid a
`Vec` allocation. The observable model and invariants are normative; the
layout is not.

### Envelope invariants

A valid envelope satisfies all of the following:

1. Every content entry has a unique message-local `content_id`.
2. Every descriptor names a registered format ID and version.
3. The message semantic signal set is the union of entry coverage.
4. Every content entry has exactly one active physical representation.
5. Re-encoding a content entry preserves its `content_id` and semantic
   coverage.
6. A derived entry names one or more source `content_id` values and never
   claims general reversibility.
7. Every Arrow record set validates independently before entering a pipeline.
8. Identifier references never cross record-set boundaries unless a format
   contract defines an explicit cross-entry relation.
9. Unknown or unsupported entries cause the whole message to be rejected.
10. All representations participate in retained-memory and size limits.
11. Removing an entry is legal only as an explicit configured transform that
    updates the manifest and provenance and records the removal in component
    telemetry.

### Signal set and existing APIs

The existing infallible `signal_type() -> SignalType` API cannot coexist
safely with mixed or Arrow-representation-only messages. The mixed envelope
introduces:

```rust
fn semantic_signals(&self) -> SignalSet;
fn single_signal(&self) -> Option<SignalType>;
```

The implementation must migrate existing callers to `single_signal()` before
mixed or Arrow-representation-only construction becomes public. Callers that
require one homogeneous signal reject `None` explicitly. This compile-enforced
migration prevents mixed content from being routed to a guessed signal topic,
exporter, or metric bucket.

Per-signal metrics iterate the semantic signal set. Pluggable-Arrow-only item,
memory, rejection, and processing metrics use a bounded startup registry that
maps registered format IDs to a closed telemetry attribute. Unregistered
format strings are never emitted as metric attributes; they use an `other`
bucket or are rejected before instrumentation.

### Pluggable PData Arrow record-set container

The in-memory Pluggable PData Arrow container is conceptually:

```rust
struct PDataArrowRecordSet {
    format: PDataArrowFormat,
    version: FormatVersion,
    tables: Vec<PDataArrowTable>,
}

struct FormatVersion {
    major: u16,
    minor: u16,
}

struct PDataArrowTable {
    local_table_id: u32,
    schema_id: SchemaId,
    batch: RecordBatch,
}
```

A Pluggable PData Arrow record set may reference a standard OTAP payload schema
by its global schema identity, for example the shared resource and scope
attribute schemas. Its table ID and graph identifier domains remain local to
the registered Arrow format.

The current Flight prototype uses table discriminants 1, 2, and 46 through 57
so its isolated store can reuse current standard schemas and 64-slot in-memory
tooling during experimentation. These values are promotion-compatible
prototype assignments, not global protocol reservations or a persistence
format. Durable buffering rejects Pluggable PData Arrow record sets until the
new envelope framing exists.

The protocol form must carry the envelope manifest and content identity before
interpreting local table IDs. A sketch is:

```protobuf
message MixedPdataEnvelope {
  repeated ContentEntry contents = 1;
}

message ContentEntry {
  uint64 content_id = 1;
  repeated SemanticSignal semantic_signals = 2;
  Provenance provenance = 3;
  oneof representation {
    StandardSignalRecords standard = 4;
    PluggablePDataArrowRecordSet pluggable_arrow = 5;
    EncodedPData bytes = 6;
  }
}

message PluggablePDataArrowRecordSet {
  string format_id = 1;
  uint32 format_major = 2;
  uint32 format_minor = 3;
  repeated PluggablePDataArrowPayload payloads = 4;
}

message PluggablePDataArrowPayload {
  uint32 local_table_id = 1;
  string schema_id = 2;
  bytes record_batch = 3;
}

message Provenance {
  enum Kind {
    INDEPENDENT = 0;
    SOURCE_OBSERVATION = 1;
    DERIVED = 2;
  }
  Kind kind = 1;
  repeated uint64 source_content_ids = 2;
}
```

This sketch is not a field-number assignment. The final protocol change
requires separate compatibility review. Schema IDs, compression, dictionary
handling, and IPC framing should reuse existing OTAP protocol machinery where
possible.

### Pluggable PData Byte Representation integration

Issue #3452 owns `PDataEncoding`, `PDataCodec`, lazy conversion, passthrough,
and encoded-buffer batching. Mixed-Signal PData uses those byte
representations as content-entry payloads without changing their codec
contract:

```rust
enum Representation {
    EncodedBytes {
        encoding: PDataEncoding,
        bytes: Bytes,
    },
    // Native and Pluggable PData Arrow variants omitted.
}
```

The content entry supplies a `SignalSet` rather than issue #3452's original
single `SignalType`. A byte codec that supports only homogeneous content
declares that restriction and rejects mixed content. A PData-agnostic
processor may preserve the entry unchanged when it needs only headers,
delivery metadata, or semantic signal coverage.

### Dedicated boundary codec lifecycle

A format produced only at export remains a dedicated exporter, following the
criterion established in the #3452 discussion. Stateful NetTrace V6 encoding
therefore uses a segment lifecycle rather than `PDataCodec`:

```rust
trait StatefulBoundaryEncoder {
    fn begin_segment(&mut self, metadata: SegmentMetadata) -> Result<(), Error>;
    fn write_message(&mut self, message: &PdataEnvelope) -> Result<(), Error>;
    fn finish_segment(&mut self) -> Result<(), Error>;
}
```

The trait shape is illustrative. The lifecycle includes bounded interning,
transactional message staging, explicit finalization, and a terminal poisoned
state after sink I/O failure. Registering `nettrace_v6` does not add a
`PDataEncoding` because NetTrace bytes are absent from ordinary pipeline
messages.

### Ordering

An envelope is a set of representations, not a global row sequence. A format
that requires ordering stores order explicitly:

- Flight preserves root observation order and source timestamps.
- NetTrace V6 uses a monotonic container clock plus explicit source time.
- Standard signals use their existing timestamps and ordering contracts.

Envelope arrival order on a pipeline link defines segment order and ACK order.
Within one envelope, cross-representation ordering is not inferred from `Vec`
order or Arrow batch order. A codec that interleaves representations must
define a total deterministic ordering, including tie breakers such as
`(content_id, row_index)`, and reject inputs that lack required ordering
information.

### Resource, scope, and identifier domains

Record sets do not implicitly share resource IDs, scope IDs, dictionaries, or
graph identifiers. Even when two formats use identical logical resource and
scope schemas, their identifiers remain format-local.

Processors may deduplicate or establish explicit cross-entry relations, but
the envelope never treats equal integer IDs as shared identity. This permits
standard signals and Flight to evolve independently and avoids accidental
joins between overlapping representations.

### ACK, NACK, and partial success

ACK and NACK apply to the whole envelope. A component must not ACK a message
after processing only the representations it understands.

For a component with several required outputs:

- success means every required representation was accepted or transformed;
- retryable failure returns the complete envelope when return-data policy
  requires it;
- permanent failure identifies the unsupported or invalid representation; and
- partial external side effects must follow the component's existing
  transaction, checkpoint, or idempotency contract.

The current NetTrace V6 prototype establishes that staging bytes in an
unterminated partial file is not an ACK boundary. Pipeline integration requires
either a durable recoverable checkpoint or final publication. Publication-only
ACK also requires a bounded segment/checkpoint cadence and a bounded upstream
in-flight message count.

### Item, byte, and memory accounting

The envelope exposes separate measurements:

```text
semantic items by signal
root items by Pluggable PData Arrow Representation
logical bytes by representation
retained bytes by representation and total
```

Summing semantic items and Pluggable PData Arrow roots into one count would
double-count overlapping source and derived observations. Each content entry
is measured once in its own metric domain: standard entries report semantic
items by signal, while source-observation Arrow representations report
format-defined root items. No combined "message item count" is used for billing
or flow control. Arrow-representation-only flows use registry-bounded,
representation-qualified root-item metrics.

All admission control, queues, batching, retry, and durable-buffer limits use
the total retained-memory estimate across every representation.

### Split, batch, and concatenate

Generic batching may combine envelopes only when:

- their context compatibility rules permit combination;
- every representation format has a registered concatenate operation;
- content and provenance identities can be remapped without ambiguity;
  and
- the result remains within all representation and envelope limits.

Splitting a mixed envelope is not equivalent to splitting each table
independently. Each Arrow graph format supplies a self-contained split and
reindex operation. Unknown entries are rejected in the first implementation.
A future opaque entry would make structural split impossible unless its format
declared byte-range framing.

Removing a representation after projection is an explicit transform. The
component must declare transform capability for the format, update semantic
coverage and provenance, and increment a bounded
`representations_removed`-style metric. Unsupported implicit removal remains
an error.

### Durable buffering

A durable buffer must do one of three things for every representation:

1. persist it using a registered durable encoding;
2. route the entire envelope around the buffer; or
3. reject the entire envelope explicitly.

The current Quiver 64-slot bitmap and global Arrow payload enum are not the
Pluggable PData Arrow namespace. A future durable format should store:

- the content manifest;
- standard signal payloads;
- each Pluggable PData Arrow format ID and version;
- each format-local table ID and encoded record batch; and
- checksums and item/memory metadata needed for recovery.

The persisted manifest is authoritative during recovery. If a restarted
process no longer supports a stored format/version, it quarantines or halts
the affected segment according to configured policy; it never reconstructs
only the standard portion or guesses a semantic signal from table slots.

Until that format exists, the durable-buffer processor must reject messages
containing unsupported Pluggable PData Arrow record sets. It must not persist
only the standard signal portion.

### Initial transport behavior

The first implementation is fail-closed. A transport endpoint has a configured
closed set of supported envelope, representation, and codec versions. Unknown
formats or unsupported major versions are rejected. A known major version may
accept a newer minor version only when the format contract explicitly permits
skipping unknown optional tables or fields.

Dynamic negotiation and bounded opaque pass-through are future work. No
transport silently downgrades or strips a representation.

### Minimum viable envelope

The first implementation deliberately supports less than the complete future
model:

- a content manifest and semantic signal set;
- exactly one active physical representation per content entry;
- existing OTLP bytes and standard OTAP Arrow record sets;
- one registered Pluggable PData Arrow record set, `otap.flight` 1.0;
- Arrow-representation-only and mixed standard-signal messages;
- content-entry-level provenance only;
- fail-closed behavior for unknown formats and unsupported versions; and
- no opaque pass-through, dynamic negotiation, or simultaneously resident
  equivalent encodings.

This scope is sufficient to carry Flight to the NetTrace V6 exporter and to
project Flight into standard signals without hard-coding a Flight sidecar.

### Capability declarations

Component inventory and wiring validation should express representation
capabilities in addition to semantic signals. A capability key is at least:

```text
(representation_id, version_range, operation)
```

Build-time validation may reject an obviously incompatible path. Runtime
validation remains necessary for content-dependent limits and version checks.

`SignalFormat` remains the optimized vocabulary for existing homogeneous
single-signal `OtapRecords` and `OtlpBytes` paths during migration. Registered
`format_id` plus version is the general representation identity and eventually
subsumes `SignalFormat`; the two are not independent authorities for one
content entry.

OTLP conversion of a mixed message is an explicit split-by-signal transform.
OTLP exporters do not split implicitly: they accept one standard signal entry
or reject the envelope.

### Security and privacy

Pluggable PData representations may preserve more sensitive data than
normalized OpenTelemetry signals. The representation framework therefore
requires:

- explicit enablement of source-fidelity and opaque byte formats;
- total and per-representation retained-byte limits;
- bounded dictionary and table counts;
- schema and graph validation before inspection;
- no automatic logging of unknown bytes or source field values;
- representation-aware redaction and sanitization capabilities;
- authorization decisions that cover every representation in the envelope;
  and
- no silent fallback from a protected structured representation to an
  uninspected opaque byte representation.

### Follow-up issue split

After the RFC terminology and invariants are accepted, implementation is
tracked through two new issues.

#### Issue A: Introduce Mixed-Signal PData

Scope:

- add `SignalSet` and content-entry identity/provenance;
- replace infallible `signal_type()` use with `single_signal()`;
- support messages with several standard signals or no semantic signal;
- define whole-envelope ACK/NACK, item, byte, and retained-memory accounting;
- define explicit split-by-signal and representation-removal transforms; and
- preserve the existing homogeneous fast path.

Acceptance requires mixed and Arrow-representation-only messages to fail
closed in every component that still requires one signal.

#### Issue B: Introduce Pluggable PData Arrow Representations

Scope:

- register `PDataArrowFormat` identity and major/minor version;
- carry a named set of format-local Arrow record batches;
- register validation, root-item accounting, memory accounting, split,
  concatenate, filter, sanitize, and reindex operations;
- add representation-aware component capabilities;
- attach `otap.flight` 1.0 as the first implementation; and
- add protocol and durable-buffer framing without consuming global payload
  enum values.

This issue builds on the Mixed-Signal PData carrier and aligns its registry,
capability, passthrough, and batching terminology with issue #3452. It does not
reimplement `PDataCodec`.

### Implementation sequence

The implementation should proceed in reviewable stages:

1. Introduce manifest and signal-set types without changing standard
   single-signal storage.
2. Migrate infallible `signal_type()` callers to `single_signal()` and keep
   mixed/Arrow-representation-only construction private until the migration
   is complete.
3. Add the Pluggable PData Arrow record-set container and whole-envelope
   accounting.
4. Attach `otap.flight` 1.0 and route Arrow-representation-only messages through
   explicitly capable components.
5. Add mixed standard-signal messages.
6. Add representation-aware component capabilities and wiring validation.
7. Add protocol encoding for mixed Arrow record sets.
8. Add durable-buffer persistence for manifests and Pluggable PData Arrow
   tables.
9. Integrate the mixed envelope with issue #3452 Pluggable PData Byte
   Representations.

Each stage keeps unsupported paths fail-closed.

## Drawbacks

- The envelope, routing, metrics, batching, transport, and durable-buffer
  surfaces become more complex.
- A message can contain overlapping representations, creating duplicate
  counting and provenance risks.
- Future unknown-representation pass-through would limit structural processing
  and increase security review requirements.
- Stateful mixed codecs complicate ACK boundaries, shutdown, checkpointing,
  and file rotation.
- Arrow-format-local table namespaces require new protocol and persistence
  framing rather than reusing the current global enum everywhere.
- Format versioning and future capability negotiation add configuration and
  testing burden.

## Rationale and alternatives

### Add every pluggable representation as a SignalType

Rejected. Representations and codecs are not OpenTelemetry semantic signals.
This approach also cannot naturally carry several signals or overlapping
source and derived representations.

### Add only SignalType::Mixed

Insufficient by itself. `Mixed` does not identify which semantic signals or
representations are present, whether entries overlap, or which component can
process them. This RFC derives mixedness from the envelope manifest.

### Add more global ArrowPayloadType values

Rejected as the general PData representation mechanism. It creates one global
namespace,
widens exhaustive matches, interacts with current 64-bit masks and Quiver
slots, and does not identify format ownership or version. Existing standard
payload values remain valid inside standard record sets.

### Carry only opaque bytes

Rejected for Flight and similar processing use cases. Opaque bytes prevent
columnar filtering, projection, graph validation, and profile construction.
Bytes remain available as an optional representation when deferred decoding or
pass-through is the actual requirement.

### Carry both Flight Arrow and NetTrace bytes

Not permitted in the minimum viable envelope as two equivalent live copies. It
duplicates retained memory and creates authority and staleness questions after
processors mutate one representation. A representation transform may replace
Flight Arrow with a byte entry while preserving `content_id`; simultaneously
resident equivalents require a future concrete use case and synchronization
contract.

### Use a Flight sidecar without a general PData representation model

Useful as a prototype, but rejected as the architecture. It would hard-code one
representation into `OtapPdata`, leave mixed signals and pluggable encodings
unresolved, and force the next representation to repeat the design.

### Do nothing

RFC 0005 Flight data could remain isolated inside pdata tests and a direct file
backend, but it could not traverse normal pipelines without pretending to be a
signal or risking silent loss. Other mixed or alternative representations
would encounter the same limitation.

## Prior art

### Multipart and content-negotiated protocols

HTTP multipart messages and media-type negotiation separate a container from
the formats of its parts. They demonstrate the value of explicit part identity
and the danger of assuming all recipients understand every part.

### Apache Arrow

Arrow schemas and record batches support zero-copy columnar processing, but
Arrow does not define OTAP semantic signals, representation capabilities,
graph reachability, or cross-record-set provenance. Those remain OTAP
contracts.

### OpenTelemetry pdata and OTLP

OpenTelemetry pdata separates logs, metrics, traces, and profiles into semantic
signal models. OTLP protobuf is one wire encoding for those models. This RFC
preserves that distinction and extends it to mixed and source-observation
representations.

### Existing OTAP BatchArrowRecords

The existing protocol demonstrates efficient transport of a statically known
set of Arrow payloads. The pluggable Arrow container retains its IPC and
dictionary
lessons while adding a representation namespace and local table vocabulary.

### NetTrace V6

NetTrace V6 is a self-describing mixed-event stream with metadata, events,
stacks, labels, threads, and sequence points. The RFC 0005 prototype proves
that a source-neutral Arrow graph can be encoded into valid V6 bytes and read
by an independent TraceEvent implementation without carrying V6 blocks as
Arrow tables or V6 bytes inside the pipeline.

## Unresolved questions

1. What stable naming authority and syntax should format IDs use?
2. Should `otap.flight` be renamed before becoming wire-visible to avoid
   confusion with the Apache Arrow Flight RPC protocol?
3. Which manifest fields are required on the first implementation, and which
   provenance relations may be deferred?
4. Should standard logs, metrics, traces, and profiles become ordinary named
   representation entries immediately, or remain optimized fields in the
   first mixed envelope?
5. What exact `SignalSet` API minimizes churn in existing single-signal
   components?
6. How are schema IDs assigned for format-local Arrow
   tables?
7. What is the first durable on-disk envelope for mixed pluggable Arrow record
   sets?
8. What bounded checkpoint cadence makes a stateful file exporter write
   ACK-safe?
9. When opaque pass-through is added, which unknown representations may cross
   trust or transport boundaries?
10. Does provenance need stable capture IDs in addition to message-local
    content IDs?
11. What ordering metadata is mandatory when a wire codec interleaves several
    representations?

## Future possibilities

- OTAP Profiles carried beside Flight source observations in one message.
- NetTrace V6 streams containing source observations and normalized logs,
  metrics, traces, and profiles with explicit derivation provenance.
- Zero-copy forwarding of registered alternative byte encodings.
- Query-engine adapters exposing pluggable Arrow tables and virtual columns.
- Remote or deferred decoding based on representation capabilities.
- Durable mixed-signal queues that preserve content manifests and local
  table namespaces.
- Representation-aware routing, sampling, redaction, and policy enforcement.
- Negotiated third-party Arrow record sets without changes to the global OTAP
  payload enum.
