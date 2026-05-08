# Spec-Constrained Oracle Reimplementation

This document describes how to reimplement compatibility-sensitive behavior in
Rust when a mature implementation in another language can be used as an
executable oracle.

The intended reader is an engineer already familiar with the OTAP Dataflow
Engine, Rust, and OpenTelemetry. It focuses on the decisions that must be made
and recorded; it does not prescribe every implementation step.

Use this approach for protocols, codecs, wire formats, file formats, schemas,
framing layers, compression envelopes, handshakes, and other components where
interoperability is a primary requirement.

The goal is not to translate the reference implementation mechanically. The
goal is to build an OTAP-native Rust implementation whose supported behavior is
traceable to specifications, schemas, reference behavior, tests, and OTAP
runtime semantics.

## Core Principle

The specification and schemas define the normative constraints. The reference
implementation provides executable evidence for behavior that specifications
often leave implicit.

The Rust implementation should satisfy an explicit compatibility contract, not
copy every observable behavior from the oracle. When the specification, schema,
oracle, and OTAP runtime model disagree, document the decision and the reason.

## Define the Compatibility Scope

Start with the smallest useful slice that can prove interoperability.

Record:

- supported and unsupported input/output formats
- supported and unsupported data types or protocol features
- expected failure modes
- whether compatibility is semantic, byte-level, or both
- the first end-to-end scenario that demonstrates useful interoperability

The first scope should be narrow enough to review, test, and release
incrementally.

## Pin the Evidence

Record the exact sources used for implementation and validation:

- specification and schema versions
- generated-code versions, when relevant
- reference implementation repository and commit
- fixture or corpus source and revision
- validation toolchain versions, when they affect results

These pins can live in the component's development note, test metadata, or
fixtures directory. They must be easy to find during review.

## Classify Oracle Behavior

Do not treat every observed oracle behavior as in scope. Classify important
observations before relying on them:

| Classification | Meaning |
|---|---|
| Spec-required | Required by the normative specification. |
| Schema-required | Required by a schema, IDL, or generated-code contract. |
| Ecosystem-required | Not fully specified, but required for interoperability. |
| Implementation artifact | Observable in the oracle, but not required. |
| Reference bug | Believed to be incorrect and intentionally not reproduced. |
| Out of scope | Intentionally excluded from the current Rust scope. |
| Unknown | Requires more investigation before release. |

For behavior admitted into the declared scope, compatibility with the executable
oracle is the required check. The classification explains why that oracle
behavior matters and records any intentional divergence, such as implementation
artifacts, reference bugs, OTAP-specific constraints, or behavior left out of
the current scope.

## Define the OTAP Integration Contract

Before implementing the core logic, define how the component fits into OTAP:

- OTAP-native input and output representation
- passthrough versus decoded/materialized data
- ownership, allocation, and hot-path expectations
- fit with the thread-per-core, share-nothing execution model
- composition with existing OTAP receivers, processors, exporters, extensions,
  shared libraries, and configuration conventions
- live reconfiguration behavior and transition semantics
- backpressure, retry, acknowledgement, and shutdown behavior, when relevant
- configuration, failure semantics, and telemetry

This prevents the Rust implementation from importing runtime assumptions from
the reference implementation.

## Compatibility Model

Prefer semantic compatibility for structured data. Semantic compatibility means
both implementations produce equivalent domain objects for the declared scope,
even if their byte output differs (e.g. a JSON object with different key
ordering or whitespace).

Use byte-level compatibility only when exact bytes are required for
interoperability, such as magic bytes, version fields, fixed headers, schema
identifiers, frame ordering, checksums, signatures, or canonical encodings.

Do not require byte-for-byte equality when multiple valid encodings are allowed.
The comparison rules should account for known sources of equivalent variation,
such as map ordering, defaults, unknown fields, metadata, timestamps, and
floating-point values.

## Validation Expectations

Build enough evidence to show that the Rust implementation is compatible for
the declared scope and safe outside it.

Use the reference implementation to generate or validate fixtures when
practical. Add regression coverage for each discovered mismatch, especially for
intentional divergences.

Define the oracle comparison surface explicitly: inputs, outputs, normalization
rules, expected error categories, and whether the oracle is used through
fixtures, an adapter, or both.

At minimum, malformed or unsupported input must not cause panics, hangs, silent
corruption, uncontrolled memory growth, or misleading success telemetry.

Measure performance on the paths that matter for the component. Compare against
the relevant Rust baseline and acceptance targets; compare against the reference
implementation only when that comparison is meaningful.

## PR Strategy

Start with an issue or design note that names the component, oracle, first
scope, compatibility contract, OTAP integration contract, and main risks.

Split larger efforts so reviewers can evaluate decisions and evidence
separately from implementation details. A typical sequence is:

1. Design, evidence pins, and compatibility contract.
2. Core Rust implementation for the first supported path.
3. Cross-implementation fixtures and mismatch regression tests.
4. OTAP runtime behavior, telemetry, robustness, and performance validation.

Small efforts may combine these steps. The important point is that
compatibility decisions remain explicit and reviewable.

## Component Development Note

Each component using this approach must keep a short development note, for
example `<component-module>/DEVELOPMENT.md`.

The note should cover:

- current supported and unsupported scope
- pinned specifications, schemas, reference implementation, and fixtures
- semantic and byte-level compatibility requirements
- intentional divergences from the oracle
- OTAP integration decisions
- validation coverage and known remaining work

Keep the note brief. It should help reviewers understand the compatibility
contract without duplicating the implementation or test suite.

## Completion Criteria

For the declared scope, the implementation is ready when:

- evidence sources are pinned
- compatibility expectations and unsupported behavior are explicit
- semantic or byte-level compatibility is validated
- malformed input is handled safely
- the OTAP integration contract is defined, including runtime behavior, live
  reconfiguration, component composition, and telemetry
- performance is acceptable for the intended use
- the component development note reflects the current state
