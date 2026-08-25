# Callsite-Planned Internal Log Translation to OTAP

Status: experimental design note

Related prototype: [#3882](https://github.com/open-telemetry/otel-arrow/pull/3882)

## Purpose

This note records a possible continuation of the internal-log stacktrace
prototype. It is not an accepted design. Its purpose is to preserve the
reasoning, constraints, staged implementation path, and open questions needed
to resume the work later.

The central idea is to apply a lesson from one-collect and similar event
systems: separate static event metadata from dynamic event values, identify the
metadata with a stable event or callsite ID, and compile translation work once
per callsite instead of repeating it for every event.

## Motivation

Internal logging has two competing requirements:

1. The caller thread must finish quickly and predictably.
2. The Internal Telemetry Receiver (ITR) should produce canonical OTAP Arrow
   tables efficiently.

The existing self-tracing path favors the first requirement. On the caller
thread it:

- encodes log body and attribute fields as a bounded OTLP protobuf fragment;
- captures a timestamp, callsite ID, and context IDs;
- optionally captures a bounded sequence of instruction pointers;
- sends the resulting `LogEvent` through a bounded channel.

This has useful hot-path properties:

- one bounded allocation for dynamic protobuf bytes;
- sequential writes into a byte buffer;
- no Arrow construction;
- no symbolization;
- no cross-thread synchronization beyond the channel operation.

The stacktrace prototype in #3882 keeps that caller-side representation, but
its ITR path is deliberately simple rather than optimized. ITR completes a
protobuf `ExportLogsServiceRequest`, creates a zero-copy `RawLogsData` view over
that request, invokes the generic OTLP-to-OTAP encoder, and then adds compact
stacktrace tables.

That path proves the representation and semantic conversion, but repeats work
that is largely static for every event from the same tracing callsite.

## Current Prototype Path

```text
caller thread
  LogRecord {
    callsite_id,
    timestamp,
    context,
    body_attrs_bytes,
    instruction_pointers
  }
        |
        v
bounded ITS channel
        |
        v
ITR
  complete ExportLogsServiceRequest
        |
        v
RawLogsData protobuf view
        |
        v
generic encode_logs_otap_batch
        |
        v
four canonical OTAP log tables
        |
        v
append compact stacktrace tables
```

The implementation is principally in:

- `crates/telemetry/src/self_tracing.rs`
- `crates/telemetry/src/self_tracing/encoder.rs`
- `crates/core-nodes/src/receivers/internal_telemetry_receiver/mod.rs`
- `crates/core-nodes/src/receivers/internal_telemetry_receiver/extended_logs.rs`
- `crates/pdata/src/encode/mod.rs`

The generic OTLP-to-OTAP path does not construct Prost message objects. It
still has to traverse protobuf tags, lengths, keys, and nested messages before
building Arrow records.

The prototype also emits one pdata message per log. It therefore constructs
and finishes Arrow builders and record batches for each individual event.

## Key Observation

A Tokio `tracing` callsite has stable metadata:

- event name;
- target or instrumentation scope;
- source file and line;
- severity;
- declared field names;
- declared field order;
- formatting choices at the macro invocation.

The values and encoded lengths vary from event to event, but the sequence of
operations needed to interpret and append those values is normally stable.

The current path repeatedly discovers that sequence from protobuf bytes.
Instead, ITR could cache a validated translation plan keyed by callsite ID.

This follows the same broad model used by one-collect-style event systems:

```text
static metadata + event identity + compact dynamic payload
```

The important distinction is that this proposal initially retains the current
OTLP protobuf fragment as the dynamic payload. A new custom wire format is not
required to test the optimization.

## Goals

- Preserve the bounded, allocation-minimal, synchronization-free caller path.
- Keep Arrow construction and symbolization off the caller thread.
- Produce the same canonical OTAP log tables as the generic converter.
- Avoid constructing a complete OTLP export request inside ITR.
- Avoid rediscovering static field layout for every event.
- Amortize Arrow allocation and record-batch construction across multiple logs.
- Bound all caches, builders, queues, and flush latency.
- Detect layout mismatches rather than silently interpreting bytes using an
  incorrect cached plan.
- Retain an explicit general path for cache misses and new event shapes.

## Non-Goals

- Defining a process-independent internal-log wire protocol in the first
  optimization phase.
- Moving Arrow builders or symbolization onto the caller thread.
- Removing backpressure or bounded-drop behavior from ITS.
- Making process-local callsite IDs stable across restarts.
- Replacing the canonical OTAP log schemas.
- Standardizing the compact stacktrace extension as part of this note.

## Proposed Steady-State Path

```text
caller thread                         ITR thread
-------------                         ----------
callsite ID                  -------> callsite plan cache
timestamp                    -------> reusable OTAP builders
OTLP field fragment          -------> planned value decoder
context IDs                  -------> resource and scope caches
instruction pointers         -------> symbol cache
```

ITR would consume `LogEvent` directly instead of first serializing its fields
into a complete OTLP request:

```text
LogEvent
  |
  +-- static callsite metadata
  +-- timestamp
  +-- resource and scope context IDs
  +-- body and attribute protobuf fragment
  +-- stack addresses
  |
  v
CallsitePlan::append(event, OtapLogsBatchBuilder)
```

Only the body and attribute fragment requires protobuf interpretation.
Timestamp, severity, event name, scope name, source location, and context are
already available without encoding and decoding them again.

## Receiver-Local Translation Plans

ITR should own a bounded cache conceptually shaped like:

```rust
CallsiteId -> LogToOtapPlan
```

A plan could contain:

```rust
struct LogToOtapPlan {
    event_name: InternedString,
    scope_name: InternedString,
    severity: SeverityNumber,
    source: SourceLocation,
    fields: Box<[FieldPlan]>,
    shape_fingerprint: ShapeFingerprint,
}
```

Each `FieldPlan` would describe a validated sequence such as:

```text
expect LogRecord body tag
decode string AnyValue
append to the OTAP body column

expect attribute key "pipeline_id"
decode string AnyValue
append log ID, cached key identity, and value
```

The plan is not a cache of absolute byte offsets. Protobuf varint lengths cause
following offsets to move. It is a compiled sequence of parser and Arrow
append operations.

### Cache Miss

The first event for a callsite would:

1. Use the general fragment decoder.
2. Validate the complete fragment.
3. Discover its ordered field layout and wire types.
4. Compile a translation plan.
5. Store the plan in the receiver-local bounded cache.
6. Append the event to the active OTAP batch.

### Cache Hit

Subsequent events would:

1. Look up the plan using the callsite ID.
2. Validate expected tags, wire types, field order, and complete consumption.
3. Decode only dynamic lengths and values.
4. Append directly to known Arrow builders.

The cache remains receiver-local. The caller performs no cache lookup and
acquires no new lock.

### Shape Variants

A callsite usually has one stable shape, but this must be verified rather than
assumed. Dynamic `tracing::field::Value` implementations, optional fields, or
future encoder changes could alter the wire representation.

A plan must validate:

- protobuf field tags;
- protobuf wire types;
- declared field order;
- required and optional fields;
- nested message boundaries;
- complete input consumption.

If validation fails, ITR must surface the mismatch and use a defined recovery
policy. Possible policies include compiling a second plan keyed by
`(callsite_id, shape_fingerprint)` or rejecting the event. It must not silently
decode the event using the wrong plan.

## Reusable Batched OTAP Builders

Callsite planning reduces parsing work, but batching may provide the larger
immediate improvement.

The prototype finishes four canonical and four stack-related Arrow tables for
each log. A future ITR should maintain reusable builders:

```rust
struct InternalLogsBatchBuilder {
    resource_attrs: ResourceAttrsBuilder,
    scope_attrs: ScopeAttrsBuilder,
    logs: LogsBuilder,
    log_attrs: LogAttrsBuilder,
    log_stacks: LogStacksBuilder,
    stack_frames: StackFramesBuilder,
    locations: LocationsBuilder,
    symbols: SymbolsBuilder,
}
```

ITR would drain multiple queued events into one batch and flush on explicit,
bounded conditions:

- maximum number of logs;
- maximum estimated retained bytes;
- maximum elapsed latency;
- downstream or memory pressure;
- shutdown.

The exact bounds require measurement. Example values such as 256 logs,
256 KiB, or 10 ms are hypotheses, not recommendations.

Batching enables:

- amortized Arrow allocation and schema handling;
- real batch-local log and stack IDs;
- resource and scope deduplication;
- symbol and location deduplication;
- fewer pdata channel messages;
- more useful Arrow buffer sizes.

Builder reuse must not retain unbounded high-water-mark allocations after an
unusually large batch.

## Shared Canonical OTAP Builder API

The direct path should not create a second, subtly different implementation of
OTLP log semantics.

The preferred refactoring is to expose a reusable canonical OTAP log builder or
appender below `encode_logs_otap_batch`. Both the generic `LogsDataView`
converter and the optimized internal-log path should append through that
shared implementation.

Conceptually:

```text
RawLogsData -----------+
                       |
                       v
              CanonicalLogsAppender -> canonical OTAP tables
                       ^
                       |
Callsite-planned view -+
```

This preserves one implementation for:

- OTLP attribute and `AnyValue` semantics;
- IDs and parent relationships;
- dropped-attribute counts;
- schema URLs;
- canonical table schemas;
- null and absent field handling.

## Static-Key Optimization Stages

The caller currently writes attribute key strings into every protobuf fragment.
The optimization should proceed in stages.

### Stage 1: Receiver-Only Plan

Make no caller changes. A plan can compare expected key bytes directly and
avoid allocating, hashing, or rediscovering their Arrow destination.

The fragment remains valid protobuf and existing fallback tooling continues to
work.

### Stage 2: Pre-Encoded Protobuf Template

If caller-side key encoding is measurable, cache static protobuf fragments for:

- field tags;
- attribute keys;
- nested message tags;
- static formatting choices.

The caller would copy static fragments and insert dynamic values. This must not
introduce shared mutable state or synchronization on the caller path.

### Stage 3: Callsite-Native Dynamic Payload

Only if measurement justifies it, consider a process-local representation such
as:

```text
otap.internal.logs.callsite.v1
```

It could carry:

```text
callsite ID
timestamp
presence bitmap
ordered dynamic values
context IDs
stack addresses
```

ITR would obtain field names and types from separately cached callsite
metadata. This most closely resembles a one-collect metadata-plus-payload
model, but it introduces ordering, cache-miss, versioning, and diagnostics
requirements. It should not be the first optimization.

## Resource and Scope Caching

ITR already receives process resource bytes and entity context IDs separately
from the dynamic log fields. A direct builder should map these identities to
batch-local resource and scope IDs without serializing them into OTLP.

The design must define:

- cache keys and ownership;
- behavior when entity metadata changes;
- batch-local versus cross-batch dictionaries;
- retained-memory bounds;
- invalidation on configuration or registry changes.

## Stacktrace Integration

The stacktrace path is naturally compatible with batching and callsite plans:

- instruction pointers remain caller-captured dynamic values;
- frame zero remains the log callsite;
- ITR owns bounded symbol and location caches;
- repeated locations can be deduplicated within a batch;
- repeated stacks may share a stack ID;
- symbolization remains outside the caller thread.

Module mappings and build IDs remain separate design work. They are not
required to evaluate callsite-planned log translation.

## Backpressure and Failure Behavior

Optimization must not weaken the existing ITS safety model.

- The caller still uses a bounded channel and configured send policy.
- ITR must not grow a queue or batch indefinitely while downstream is blocked.
- Cache and builder limits must be explicit and observable.
- Shape mismatches and conversion failures must be reported.
- Unsupported pluggable representations must continue to fail closed.
- Shutdown must flush or explicitly account for remaining events within the
  configured deadline.

Plan compilation and first-touch symbolization can create latency spikes on the
ITR thread. Measurements should distinguish cache-miss and steady-state costs.

## Measurement Plan

Optimization should be driven by an end-to-end benchmark rather than parser
microbenchmarks alone.

Measure at least:

- caller-thread nanoseconds per enabled event;
- caller-thread allocations and allocated bytes;
- maximum caller latency under channel pressure;
- ITR events per second;
- ITR CPU time per event;
- ITR allocations and retained bytes per event;
- callsite-plan cache hit rate;
- first-event plan compilation latency;
- Arrow rows and bytes per emitted batch;
- flush latency distribution;
- symbol cache hit rate and first-resolution latency;
- dropped events and conversion failures.

Compare these paths:

1. Current complete-OTLP-request prototype.
2. Direct general fragment-to-OTAP append.
3. Direct append with batching.
4. Batched append with callsite translation plans.
5. Optional static caller templates, only if earlier results justify them.

Use workloads with:

- one hot callsite;
- many callsites;
- mixed attribute types;
- optional fields;
- small and large dynamic strings;
- stack capture disabled and enabled;
- cold and warm caches;
- downstream backpressure.

## Suggested Implementation Sequence

1. Extract a reusable canonical OTAP log appender from
   `encode_logs_otap_batch`.
2. Add a direct `LogEvent` adapter that uses the general decoder only for the
   existing body and attribute fragment.
3. Remove complete `ExportLogsServiceRequest` construction from the extended
   ITR path.
4. Add bounded multi-log batching and flush policies.
5. Establish correctness and performance baselines.
6. Add a bounded receiver-local callsite translation-plan cache.
7. Add shape validation, mismatch tests, and cache observability.
8. Measure caller-side static-key encoding.
9. Consider static protobuf templates or a callsite-native payload only if
   measured costs justify the added protocol complexity.

Each stage should preserve a correctness oracle against the generic
OTLP-to-OTAP converter.

## Correctness Testing

For the same generated `LogEvent`, compare the canonical four OTAP tables from:

- the current complete-OTLP-request conversion;
- the direct general fragment path;
- the planned fast path.

Tests should cover:

- every supported OTLP `AnyValue` type;
- absent and optional fields;
- dropped-attribute counts;
- resource and scope attributes;
- repeated callsites and multiple shape variants;
- malformed or truncated fragments;
- cache eviction;
- batch flush limits;
- shared stack IDs;
- shutdown with a partially filled batch.

Property-based or generated differential tests would provide more confidence
than hand-selected examples alone.

## Open Questions

1. Does `tracing` guarantee enough callsite field stability for one plan, or
   must shape fingerprints always participate in the cache key?
2. What canonical builder API can be shared without exposing Arrow
   implementation details throughout the telemetry crate?
3. Should plans append values directly, or produce a lightweight internal view
   consumed by a shared appender?
4. Which batching limits provide useful Arrow batches without adding
   unacceptable internal-log latency?
5. Should resource and scope dictionaries survive across batches?
6. How should a plan mismatch affect the event: recompile, maintain variants,
   reject, or use the general path with an explicit diagnostic?
7. Can callsite registration safely install immutable caller-side protobuf
   templates without adding synchronization?
8. Is repeated caller-side key encoding measurable after receiver-side
   batching and planning?
9. What metrics are required to make cache behavior and fallback use visible?
10. Should the optimized internal fragment eventually become a named
    pluggable byte representation?

## Pickup Checklist

Before resuming implementation:

1. Rebase or reproduce the #3882 prototype against current `main`.
2. Confirm the current `LogRecord`, ITR, and canonical OTAP builder APIs.
3. Capture baseline caller and ITR profiles.
4. Verify the one-log-per-pdata and complete-OTLP-request costs independently.
5. Start with the shared canonical builder and batching work.
6. Retain the generic converter as a differential correctness oracle.
7. Treat callsite planning as a measured optimization, not an assumed win.
