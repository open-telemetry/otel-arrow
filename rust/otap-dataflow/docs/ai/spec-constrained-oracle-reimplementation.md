# Spec-Constrained Oracle Reimplementation

This document provides a contributor framework for implementing a Rust component
when compatibility with an existing protocol, format, schema, or framing behavior
is the main risk.

Use this approach when a mature implementation exists and can be used as a
behavioral oracle, but the Rust implementation must still be constrained by the
specification, schemas, and OTAP Dataflow Engine semantics.

## When to Use This Approach

Use this approach for components or libraries involving:

- protocol encoding and decoding
- wire formats
- file formats
- schema-based serialization
- framing
- compression envelopes
- handshakes
- streaming message assembly
- compatibility-sensitive request and response behavior

The reference implementation is useful because specifications and schemas often
do not describe every behavior required for interoperability.

## Goal

The goal is to produce a Rust implementation that is compatible for a declared
scope.

Compatibility does not always mean byte-for-byte equality. In many cases,
semantic equivalence is the right target.

The implementation should be:

- traceable to specifications and schemas
- compatible with the selected reference implementation
- explicit about unsupported behavior
- robust against malformed input
- efficient in OTAP hot paths
- integrated with OTAP runtime semantics

## Core Principle

The specification and schemas define normative constraints.

The reference implementation acts as an executable oracle for observable
behavior.

The Rust implementation must satisfy the declared compatibility contract, not
blindly reproduce every behavior of the reference implementation.

If the specification, schema, oracle, and OTAP runtime model disagree, the
difference must be documented and resolved intentionally.

## Workflow

### 1. Define the Scope

Start by selecting the smallest useful compatibility slice.

Document:

- supported input formats
- supported output formats
- supported data types
- unsupported data types
- expected failure modes
- semantic compatibility requirements
- byte-level compatibility requirements
- the first end-to-end scenario that proves useful interoperability

The first scope should be small enough to review, test, and release
incrementally.

### 2. Pin the Evidence

Record the exact evidence used for implementation and validation.

Include:

- specification version or commit
- schema version
- generated-code version, if relevant
- reference implementation commit
- fixture or corpus commit, if relevant
- toolchain versions used for validation

These pins should appear in the component methodology note or in test metadata.

### 3. Classify Oracle Behavior

Not every behavior in the reference implementation should be copied.

Classify important observed behavior as one of:

| Classification | Meaning |
|---|---|
| Spec-required | Required by the normative specification. |
| Schema-required | Required by a schema, IDL, or generated-code contract. |
| Ecosystem-required | Not fully specified, but required for interoperability. |
| Implementation artifact | Observable in the oracle, but not required. |
| Reference bug | Believed to be incorrect and intentionally not reproduced. |
| Out of scope | Intentionally excluded from the current Rust scope. |
| Unknown | Requires more investigation before release. |

Only spec-required, schema-required, and approved ecosystem-required behavior
should become compatibility requirements.

### 4. Define the OTAP Integration Contract

Before implementing the Rust logic, define how it fits into OTAP.

Document:

- OTAP-native input and output representation
- when data remains passthrough
- when decoding or materialization occurs
- ownership and memory expectations
- backpressure behavior
- retry behavior, if relevant
- ack and nack behavior, if relevant
- cancellation and shutdown behavior
- telemetry emitted by the component
- configuration and failure semantics

This prevents the Rust implementation from importing runtime assumptions from
the reference implementation.

### 5. Build Compatibility Evidence

Use fixtures and tests to compare behavior across implementations.

Recommended evidence:

- reference-generated fixtures decoded by Rust
- Rust-generated fixtures decoded by the reference implementation
- semantic comparison of decoded values
- byte-level checks for required protocol fields
- malformed input tests
- unsupported input tests
- regression fixtures for every discovered mismatch

When practical, wrap the reference implementation with a small oracle adapter
that can generate, encode, decode, validate, and report structured errors.

### 6. Add Robustness and Performance Validation

At minimum, validate that malformed or unsupported input does not cause:

- panics
- hangs
- silent corruption
- uncontrolled memory growth
- misleading success telemetry

For performance, measure the paths that matter for the component:

- codec-only path
- OTAP conversion path
- end-to-end protocol path, if relevant
- allocations
- throughput
- latency
- peak memory

Compare against the previous Rust baseline and against acceptance targets.
Compare against the reference implementation when that comparison is meaningful.

## Compatibility Model

Prefer semantic compatibility for structured data.

Semantic compatibility means that both implementations decode or process data
into equivalent domain objects for the declared scope.

Semantic comparison should account for:

- map ordering
- default values
- unknown fields
- unsupported fields
- empty values
- timestamps
- floating-point values, when relevant
- metadata
- schema identity

Require byte-level compatibility only when exact bytes are required by the
protocol or by interoperability.

Examples:

- magic bytes
- version fields
- fixed headers
- schema identifiers
- frame ordering
- handshake messages
- checksums
- signatures
- canonical encodings

Do not require byte-for-byte equality when multiple valid encodings are allowed.

## Suggested PR Slicing

A compatibility-focused implementation can often be split as follows:

1. Add methodology note, scope, pins, and traceability summary.
2. Add oracle adapter or fixture generation tooling.
3. Add Rust data model or OTAP representation mapping.
4. Add decoder for the first supported scope.
5. Add encoder for the first supported scope.
6. Add cross-decoder tests.
7. Add differential roundtrip tests.
8. Add malformed and unsupported input tests.
9. Add protocol-level tests, if relevant.
10. Add fuzz targets or corpus replay.
11. Add benchmarks and performance report.
12. Finalize documentation and release checklist.

The PR sequence can be adapted. The important point is to separate design,
compatibility evidence, implementation, robustness, and performance whenever
possible.

## Per-Component Methodology Note

Each module using this approach should include a methodology note.

Suggested file:

```text
<component-module>/DEVELOPMENT.md
```

Suggested content:

```md
# Development Methodology

## Approach

This component follows the Spec-Constrained Oracle Reimplementation approach.

## Scope

Supported:

- ...

Unsupported:

- ...

Out of scope for this phase:

- ...

## Sources

Specification:

- name, version, commit, or link

Schemas:

- name, version, commit, or link

Reference implementation:

- repository
- commit
- relevant packages or modules

Fixtures or corpora:

- location
- generator
- commit or version

## Compatibility Contract

Semantic compatibility required for:

- ...

Byte-level compatibility required for:

- ...

Intentional divergences:

- ...

## OTAP Integration Notes

- input representation:
- output representation:
- passthrough behavior:
- memory and ownership considerations:
- backpressure behavior:
- retry, ack, nack behavior:
- shutdown behavior:
- telemetry:

## Tests Added

- cross-decoder tests:
- roundtrip tests:
- malformed input tests:
- unsupported input tests:
- protocol tests:
- fuzz or corpus replay:
- benchmarks:

## Checklist

- [ ] Scope defined.
- [ ] Evidence pinned.
- [ ] Oracle behavior classified.
- [ ] OTAP integration contract documented.
- [ ] Compatibility fixtures added.
- [ ] Cross-decoder tests added.
- [ ] Roundtrip tests added.
- [ ] Unsupported behavior tested.
- [ ] Malformed input tested.
- [ ] Fuzzing or corpus replay added, when relevant.
- [ ] Benchmarks added, when relevant.
- [ ] Telemetry added.
- [ ] Documentation updated.
- [ ] Release criteria satisfied.

## Remaining Work

- ...
```

## Completion Criteria

For the declared scope, the implementation is ready when:

- sources are pinned
- compatibility expectations are explicit
- unsupported behavior is documented and tested
- semantic or byte-level compatibility is validated
- malformed input is handled safely
- OTAP runtime behavior is documented
- performance is acceptable for the intended use
- the module methodology note is up to date
