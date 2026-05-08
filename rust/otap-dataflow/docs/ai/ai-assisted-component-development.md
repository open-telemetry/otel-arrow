# AI-Assisted Component Development in the OTAP Dataflow Engine

This document explains how to choose between the two recommended AI-assisted
approaches for developing Rust components in the OTAP Dataflow Engine from
existing OpenTelemetry implementations.

The intended reader is an engineer already familiar with the OTAP Dataflow
Engine, Rust, and OpenTelemetry. It focuses on choosing the right approach and
recording the decisions that make the work reviewable.

AI can accelerate analysis, implementation, testing, and documentation. It does
not define correctness. Accepted behavior must be backed by traceable evidence,
an OTAP design decision, or a documented intentional divergence.

## The Two Approaches

The project uses two complementary approaches. Some components use both.

### Spec-Constrained Oracle Reimplementation

Use this approach when the main risk is interoperability and a mature reference
implementation can act as an executable oracle for observable behavior.

Typical cases include:

- protocol encoders and decoders
- wire formats
- file formats
- schemas and generated-code contracts
- framing, compression envelopes, and handshakes

The Rust implementation must satisfy an explicit compatibility contract for the
declared scope while remaining OTAP-native. Specifications and schemas define
normative constraints; the reference implementation validates observable
behavior admitted into scope.

Primary validation comes from compatibility evidence, such as reference
fixtures, cross-implementation checks, semantic comparison, byte-level checks
where required, and malformed-input tests.

See [Spec-Constrained Oracle Reimplementation](spec-constrained-oracle-reimplementation.md)
for the companion guidance.

### Reference-Informed OTAP-Native Capability Design

Use this approach when the main goal is to deliver a new or improved OTAP
capability inspired by an existing receiver, processor, exporter, extension, or
contrib component.

Typical cases include:

- implementing a receiver, processor, exporter, or extension from contrib
- splitting one existing component across several OTAP components
- merging responsibilities differently for usability or performance
- building a shared component library or component family
- redesigning configuration, defaults, telemetry, or runtime behavior

The reference implementation is evidence, not an oracle. The Rust design should
learn from existing code, tests, documentation, issues, operational feedback,
and documented future direction, then deliver the best OTAP-native capability
for the declared user scenarios.

Primary validation comes from scenario tests, integration tests, design
rationale, failure coverage, and OTAP runtime behavior.

See [Reference-Informed OTAP-Native Capability Design](reference-informed-otap-native-capability-design.md)
for the companion guidance.

## Choosing the Right Approach

| Question | Spec-Constrained Oracle | Reference-Informed Design |
|---|---:|---:|
| Is interoperability the primary risk? | Yes | Sometimes |
| Is a protocol, schema, file format, or framing model central? | Yes | Sometimes |
| Should the reference implementation define observable behavior? | Yes, for declared scope | No |
| Is byte-level or semantic compatibility required? | Often | Sometimes |
| Is the component boundary open to redesign? | Usually no | Usually yes |
| Is improving user experience or design a primary goal? | Not usually | Yes |
| Should future direction and user feedback shape the result? | Limited | Yes |
| Main validation style | Compatibility tests | Scenario and integration tests |

Hybrid projects should split the decision by responsibility. For example, an
exporter may use the oracle approach for a wire protocol and the
reference-informed approach for configuration, telemetry, and runtime behavior.

## Shared Requirements

Every AI-assisted component effort must remain controlled and reviewable:

- define the declared scope and first useful end-to-end scenario
- record the evidence used for design and validation
- classify preserved, changed, unsupported, rejected, or divergent behavior
- make unsupported behavior and intentional divergences explicit
- validate AI-generated conclusions with tests, fixtures, references, or
  documented OTAP decisions

Every implementation must fit the OTAP Dataflow Engine:

- OTAP-native data representation and component composition
- ownership, allocation, and hot-path materialization behavior
- thread-per-core, share-nothing execution model
- live reconfiguration behavior and transition semantics
- backpressure, retry, acknowledgement, shutdown, and failure behavior
- configuration validation, telemetry, diagnostics, and performance
- security, authentication, and sensitive data handling, when relevant

Matching or learning from an existing implementation is not enough. The result
must also behave correctly inside OTAP.

## Development Note and PR Strategy

Each component using either approach must keep a short development note, for
example `<component-module>/DEVELOPMENT.md`.

The note should cover:

- approach followed and scope selected
- reference implementation, specifications, feedback, or future direction used
- behavior preserved, changed, unsupported, rejected, or intentionally divergent
- OTAP integration decisions
- validation coverage and known remaining work

Before opening implementation PRs, start with an issue or design note that names
the component, user goal, chosen approach, first scope, evidence, OTAP
integration concerns, and main risks.

Split larger efforts so reviewers can evaluate decisions separately from
implementation details. A typical sequence is:

1. Scope, evidence, and design or compatibility contract.
2. First working implementation slice.
3. Compatibility or scenario coverage, including failures and unsupported
   behavior.
4. OTAP runtime behavior, telemetry, robustness, and performance.

Small components may combine steps. Larger efforts should avoid one PR that
mixes design, implementation, tests, benchmarks, and documentation.

## Completion Criteria

A component is ready for its declared scope when:

- evidence and design decisions are recorded
- compatibility expectations or user-facing behavior are explicit
- unsupported behavior and intentional divergences are documented and tested
- the relevant compatibility or scenario tests pass
- the OTAP integration contract is defined, including runtime behavior, live
  reconfiguration, component composition, and telemetry
- robustness and performance are acceptable for the intended use
- the component development note reflects the current state

For oracle work, completion evidence must include compatibility tests. For
reference-informed design, completion evidence must include scenario tests and
design rationale. Hybrid projects need both kinds of evidence.
