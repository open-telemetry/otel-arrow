# AI-Assisted Component Development in the OTAP Dataflow Engine

This document describes the two recommended AI-assisted approaches for developing
new Rust components in the OTAP Dataflow Engine from existing OpenTelemetry
implementations, especially the Go Collector and contrib repositories.

The goal is not to mechanically translate Go code into Rust. The goal is to use
AI to accelerate analysis, implementation, testing, and documentation while
keeping the result controlled, reviewable, interoperable, and aligned with the
OTAP Dataflow Engine.

## The Two Approaches

The project uses two complementary approaches.

### 1. Spec-Constrained Oracle Reimplementation

Use this approach when the main risk is interoperability.

Typical cases include:

- protocol encoders and decoders
- wire formats
- file formats
- schemas

In this approach, the specification and schemas define the normative rules. A
mature reference implementation, often the Go Collector, is used as an executable
oracle for observable behavior.

The Rust implementation must satisfy a declared compatibility contract while
remaining OTAP-native.

See [Spec-Constrained Oracle Reimplementation](docs/ai/spec-constrained-oracle-reimplementation.md)
for a detailed contributor framework for this approach.

### 2. Reference-Informed OTAP-Native Capability Design

Use this approach when the main goal is to deliver a new OTAP capability inspired
by an existing receiver, processor, exporter, extension, or contrib component.

Typical cases include:

- implementing a receiver that already exists in contrib
- implementing a processor with known user feedback
- implementing an exporter with a different OTAP-native runtime model
- splitting a Go component into several OTAP components
- merging responsibilities differently to improve usability or performance

In this approach, the reference implementation is evidence, not an oracle. The
Rust implementation should learn from existing code, tests, documentation,
issues, user feedback, and operational experience, then produce the best
OTAP-native design for users.

See [Reference-Informed OTAP-Native Capability Design](docs/ai/reference-informed-otap-native-capability-design.md)
for a detailed contributor framework for this approach.

## Choosing the Right Approach

| Question | Use Spec-Constrained Oracle Reimplementation | Use Reference-Informed OTAP-Native Capability Design |
|---|---:|---:|
| Is there a protocol, schema, file format, or framing model? | Yes | Sometimes |
| Is interoperability the primary risk? | Yes | Sometimes |
| Is byte-level or semantic compatibility required? | Often | Sometimes |
| Should the Go implementation define observable behavior? | Often | Not necessarily |
| Is the component boundary open to redesign? | Usually no | Yes |
| Can one Go component become several OTAP components? | Rarely | Yes |
| Is the goal to preserve behavior or improve a capability? | Preserve declared behavior | Improve user capability |
| Main validation style | Compatibility tests | Scenario and integration tests |

Some projects use both approaches.

For example, an exporter may use Spec-Constrained Oracle Reimplementation for
its wire protocol, while using Reference-Informed OTAP-Native Capability Design
for configuration, telemetry, and runtime behavior.

## Shared Principles

### AI accelerates the work, but does not define correctness

AI may help with:

- reading reference implementations
- summarizing behavior
- proposing Rust code
- generating tests
- identifying edge cases
- comparing designs
- drafting documentation
- optimizing performance

AI-generated output must be reviewed and validated. A behavior is accepted only
when it is backed by a specification, schema, test, fixture, reference behavior,
OTAP design decision, or documented intentional divergence.

### The Go Collector is not always the target design

The Go Collector is extremely valuable, but it should not always be copied.

For compatibility-sensitive protocol work, the Go implementation may be a strong
behavioral oracle.

For receivers, processors, exporters, and higher-level capabilities, the Go
implementation is usually a reference source. It helps us understand the
ecosystem, but the OTAP implementation may intentionally differ.

### OTAP-native behavior is required

Every implementation must fit the OTAP Dataflow Engine.

Contributors should consider:

- OTAP-native data representation
- ownership and memory behavior
- scheduling
- backpressure
- retry behavior
- ack and nack behavior
- shutdown and cancellation
- telemetry
- configuration validation
- live reconfiguration updates
- failure reporting
- composability
- performance

A component is not complete only because it matches an existing implementation.
It must also behave correctly inside OTAP.

## Required Per-Component Methodology Note

Every component developed using one of these approaches should include a
dedicated methodology note in the module directory.

Suggested file name:

```text
<component-module>/DEVELOPMENT.md
```

or, for larger modules:

```text
<component-module>/docs/development-methodology.md
```

The note should explain:

- which approach was followed
- which reference implementation was analyzed
- which scope was selected
- which behavior was preserved
- which behavior was changed
- which behavior is unsupported
- which OTAP-specific constraints were considered
- which tests and benchmarks were added
- what remains to be done

The note should be updated as the implementation evolves.

## Recommended Issue and PR Strategy

Before opening implementation PRs, contributors should start with an issue that
briefly describes:

- the component or capability they want to contribute
- the user goal or problem being addressed
- the approach they plan to follow
- the expected scope of the first implementation
- known references, such as Go Collector packages, contrib components,
  specifications, schemas, issues, or examples

Large translation or reimplementation efforts should then be split into a small
number of reviewable pull requests whenever practical. A typical effort should
fit into 2 to 5 PRs.

Suggested sequence:

1. Design, scope, and methodology note.
2. First working implementation slice.
3. Compatibility or scenario coverage, including failures and unsupported
   behavior.
4. OTAP runtime hardening, telemetry, robustness, and performance work.
5. Documentation, release checklist, and remaining cleanup.

Small components may combine some steps. Larger components should avoid one
large PR that mixes design, implementation, tests, benchmarks, and documentation.

## Shared Completion Criteria

A component should not be considered complete until the declared scope is clear.

At minimum, contributors should document:

- supported features
- unsupported features
- expected failure modes
- relevant reference implementation commit or version
- OTAP runtime behavior
- configuration behavior
- telemetry
- tests added
- known limitations
- follow-up work

For Spec-Constrained Oracle Reimplementation, the completion evidence should
include compatibility tests.

For Reference-Informed OTAP-Native Capability Design, the completion evidence
should include scenario tests and design rationale.

For hybrid projects, both kinds of evidence are expected.
