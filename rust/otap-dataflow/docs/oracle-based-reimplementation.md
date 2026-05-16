# Oracle-Based Reimplementation Methodology

This document describes a reusable method for implementing a Rust component
from an existing reference implementation while preserving interoperability,
performance, and alignment with the OTAP Dataflow Engine.

Use this method when all of the following are true:

- a protocol, file format, schema, or runtime behavior already exists
- at least one mature implementation can be used as a behavioral reference
- a specification or schema exists, but does not fully describe every
  interoperability detail
- the Rust implementation must integrate with OTAP-native data structures and
  runtime semantics

## Core Approach

The recommended approach is a **spec-constrained oracle method**.

The specification and schemas define the normative constraints. The reference
implementation acts as an executable oracle for observable behavior. The Rust
implementation must satisfy both: it should remain traceable to the spec, while
also matching the behavior that real systems depend on.

This avoids two common failure modes:

- implementing only what the spec says and missing behavior required for
  interoperability
- copying reference behavior without understanding whether it is required,
  accidental, or specific to that implementation

## Sources of Truth

Every project using this method should explicitly classify its sources:

- **Normative source:** specification sections, protocol definitions, wire
  format descriptions, or RFC-like documents.
- **Schema source:** schema language, IDL, generated code, compatibility
  contracts, and versioned schema files.
- **Behavioral oracle:** mature implementations, command-line tools, services,
  examples, and integration tests that define observable behavior.
- **Corpus source:** golden files, fixtures, regression tests, fuzz corpora,
  malformed inputs, and real production captures when available.
- **Local architecture source:** OTAP Dataflow Engine data model, scheduling,
  ownership, memory, backpressure, telemetry, and failure semantics.

The Rust implementation is considered compatible only when its behavior is
explainable through these sources.

## Compatibility Model

Compatibility should be defined before implementation begins.

Prefer semantic compatibility for structured data:

- both implementations decode to equivalent domain objects
- both implementations preserve required metadata
- both implementations reject unsupported or malformed inputs intentionally
- roundtrips preserve the data supported by the Rust scope

Require byte-level compatibility only for stable protocol elements:

- magic bytes, versions, and fixed headers
- schema identifiers and required root metadata
- frame or message ordering where the protocol requires it
- handshake, first-message, or end-of-message rules
- checksums, signatures, or canonical encodings when specified

Do not require byte-for-byte equality for fields that permit multiple valid
encodings, unless the protocol or interoperability target explicitly requires
canonical output.

## Method Steps

### 1. Pin the Evidence

- [ ] Pin the specification version or commit.
- [ ] Pin schema versions and generated-code versions.
- [ ] Pin the reference implementation commit.
- [ ] Pin fixture and fuzz-corpus commits.
- [ ] Record toolchain versions used for validation.
- [ ] Store all pins in documentation or test metadata.

### 2. Define the First Scope

- [ ] List supported input and output formats.
- [ ] List supported data types.
- [ ] List unsupported data types and expected failure modes.
- [ ] Identify which behavior must be semantically equivalent.
- [ ] Identify which behavior must be byte-for-byte equivalent.
- [ ] Define the first end-to-end scenario that proves useful
  interoperability.

### 3. Build a Traceability Matrix

- [ ] Map each implemented protocol feature to a spec section.
- [ ] Map each implemented schema feature to a schema definition.
- [ ] Map each compatibility behavior to an oracle test or fixture.
- [ ] Map each unsupported feature to an explicit error path.
- [ ] Map each performance-sensitive path to an OTAP runtime decision.

The matrix does not need to be elaborate, but it must make gaps visible.

### 4. Create Golden Compatibility Corpora

- [ ] Generate reference-implementation fixtures for every supported shape.
- [ ] Generate Rust fixtures for the same shapes.
- [ ] Include empty inputs, small inputs, and large inputs.
- [ ] Include low-cardinality and high-cardinality metadata.
- [ ] Include boundary values and repeated values.
- [ ] Include malformed and unsupported inputs.
- [ ] Store fixtures with metadata describing generator, version, commit, and
  expected result.

Golden corpora are the fastest way to detect accidental compatibility drift.

### 5. Add Cross-Decoder Tests

- [ ] Decode reference-generated fixtures with Rust.
- [ ] Decode Rust-generated fixtures with the reference implementation.
- [ ] Assert semantic equivalence after decoding.
- [ ] Assert compatible errors for invalid or unsupported input.
- [ ] Keep all previously discovered failures as regression fixtures.

This verifies that both sides can consume each other's supported output.

### 6. Add Differential Roundtrip Tests

- [ ] Test `reference encode -> Rust decode -> Rust encode -> reference decode`.
- [ ] Test `Rust encode -> reference decode -> reference encode -> Rust decode`.
- [ ] Assert semantic equivalence at every boundary.
- [ ] Assert required byte-level invariants.
- [ ] Run the same test through every Rust representation that matters, such
  as raw bytes, parsed views, and OTAP-native records.

Differential tests are especially useful when byte-for-byte output is not a
valid requirement.

### 7. Validate Protocol Behavior End to End

- [ ] Test reference client to Rust server.
- [ ] Test Rust client to reference server.
- [ ] Test full reference pipeline to Rust pipeline.
- [ ] Test full Rust pipeline to reference pipeline.
- [ ] Validate fragmentation, chunking, streaming, flow control, cancellation,
  retries, and shutdown.
- [ ] Validate size limits, malformed messages, unknown versions, and
  unsupported options.

Codec compatibility is not enough when protocol behavior spans many messages.

### 8. Add Fuzzing

- [ ] Import reference fuzz corpora as Rust fuzz seeds.
- [ ] Add Rust fuzz targets for parsers and decoders.
- [ ] Add fuzz targets for streaming or message assembly.
- [ ] Add mutation strategies that preserve enough valid structure to reach
  deep decoder logic.
- [ ] Add negative fuzzing for malformed, truncated, overlong, and invalid
  inputs.
- [ ] Replay minimized Rust failures against the reference implementation when
  practical.
- [ ] Store every crash, panic, hang, or semantic mismatch as a regression
  fixture.
- [ ] Run bounded corpus replay in CI and longer fuzzing in scheduled jobs.

Fuzzing should test robustness and compatibility, not only memory safety.

### 9. Establish Performance Confidence

- [ ] Commit reproducible benchmark harnesses.
- [ ] Benchmark codec-only paths.
- [ ] Benchmark end-to-end protocol paths.
- [ ] Measure throughput, latency, allocations, bytes allocated per item, and
  peak memory.
- [ ] Profile hot paths with CPU and allocation profilers.
- [ ] Compare against the reference implementation and against the previous
  Rust baseline.
- [ ] Include realistic batch sizes, cardinalities, and payload shapes.
- [ ] Record machine metadata, commit SHAs, toolchain versions, and command
  lines with every report.
- [ ] Define acceptance targets before declaring the implementation complete.

Performance claims should be reproducible without relying on one-off local
experiments.

### 10. Align with OTAP Dataflow Engine Semantics

- [ ] Preserve OTAP-native representations on hot paths.
- [ ] Avoid unnecessary protobuf or object materialization.
- [ ] Preserve passthrough payloads when passthrough is the correct engine
  representation.
- [ ] Validate ack/nack, retry, backpressure, and memory-limiter behavior.
- [ ] Validate scheduler and batching behavior under sustained load.
- [ ] Add telemetry for accepted items, rejected items, bytes, batches,
  protocol messages, encode errors, decode errors, and unsupported inputs.
- [ ] Document configuration and failure semantics.

Interoperability with an external implementation is not sufficient if the
component violates local engine semantics.

### 11. Automate and Gate

- [ ] Run compatibility tests in CI.
- [ ] Run corpus regression tests in CI.
- [ ] Run bounded fuzz corpus replay in CI.
- [ ] Run longer fuzzing in scheduled jobs.
- [ ] Publish compatibility, performance, and supported-feature matrices.
- [ ] Define release criteria for each supported scope.
- [ ] Require explicit documentation for every unsupported feature.

The method is complete only when compatibility evidence is durable and
repeatable.

## Required Artifacts

A project using this method should produce:

- methodology document referencing this file
- pinned source and oracle versions
- traceability matrix
- supported-feature matrix
- compatibility fixture corpus
- cross-decoder tests
- differential roundtrip tests
- protocol-level integration tests
- fuzz targets and replay corpus
- reproducible benchmark harness
- performance report
- release criteria

## Confidence Argument

This method builds confidence from independent directions:

- **Specification conformance:** implemented behavior is traceable to
  normative documents.
- **Oracle compatibility:** Rust and the reference implementation accept each
  other's supported output.
- **Robustness:** malformed and fuzz-generated inputs are handled without
  panics, hangs, or silent corruption.
- **Performance:** throughput, latency, and allocation behavior are measured
  with reproducible benchmarks.
- **Engine fit:** the implementation preserves OTAP Dataflow Engine semantics
  instead of forcing the engine into the reference implementation's internal
  model.

When all five directions agree, the implementation can be considered highly
compatible for the declared scope.
