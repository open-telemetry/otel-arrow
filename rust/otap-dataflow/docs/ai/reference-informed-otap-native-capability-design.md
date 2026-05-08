# Reference-Informed OTAP-Native Capability Design

This document describes how to design an OTAP-native Rust capability when an
existing OpenTelemetry Collector component can inform the work but should not
define the result.

The intended reader is an engineer already familiar with the OTAP Dataflow
Engine, Rust, and OpenTelemetry. It focuses on the decisions that must be made
and recorded; it does not prescribe every implementation step.

Use this approach for receivers, processors, exporters, extensions, shared
component libraries, or component families inspired by existing Collector or
contrib implementations.

The goal is not to translate the reference implementation mechanically. The
goal is to deliver the best OTAP-native capability for the declared user
scenarios, using the reference implementation, tests, documentation, issues,
and operational feedback as evidence. Reimplementation is an opportunity to
improve the design and user experience based on what existing users and
maintainers have learned from the current solution.

## Core Principle

The reference implementation is evidence, not an oracle.

The main question is not how to reproduce the existing component in Rust. The
main question is what OTAP-native capability should exist for users.

Preserve reference behavior only when it is required for compatibility,
interoperability, user expectation, or migration. Improve, simplify, decompose,
or reject behavior when OTAP constraints or user outcomes justify it.

## Define the Capability Scope

Start from the user outcome, not from the structure of the reference
implementation.

Record:

- the primary user scenarios
- input and output data
- external systems involved
- supported and unsupported behavior
- important operational and failure modes
- scale, cardinality, and resource assumptions
- the first useful end-to-end scenario

The first scope should be narrow enough to validate the architecture and
release incrementally.

## Gather the Evidence

Use the reference implementation to understand existing behavior, but do not
assume that every behavior should be preserved.

Record the evidence used for design decisions:

- reference implementation repository, version, commit, and relevant modules
- configuration options, defaults, validation rules, and lifecycle behavior
- batching, retry, queueing, shutdown, and error handling behavior
- telemetry, tests, examples, and known edge cases
- community issues, pull requests, discussions, operational feedback, and
  documented future direction from specifications, accepted design notes,
  roadmaps, or maintainer-approved issues

Community feedback and future direction are especially important for this
approach because the best OTAP design may fix known usability, reliability, or
performance problems, or implement an accepted future plan directly, rather
than reproduce the current implementation.

## Classify Findings

Classify important findings before they become part of the Rust design:

| Classification | Meaning |
|---|---|
| Preserve | Required for compatibility, user expectation, or interoperability. |
| Improve | Useful behavior that should be safer, clearer, faster, or more robust. |
| Simplify | Can be represented with a simpler OTAP model. |
| Decompose | Should be split across OTAP components or libraries. |
| Compose | Should be achieved by combining existing OTAP components. |
| Avoid | Legacy, surprising, fragile, or tied to implementation details. |
| Reject | Conflicts with OTAP design, reliability, safety, or scope. |
| Investigate | Requires more evidence before release. |

The classification records the intended outcome for each relevant finding and
why that decision is intentional.

## Design the OTAP-Native Architecture

Decide where each responsibility belongs in OTAP before implementing the first
slice.

The design should cover:

- component boundaries: receiver, processor, exporter, extension, or shared
  library
- composition with existing OTAP components and configuration conventions
- OTAP-native input and output representation
- passthrough versus decoded/materialized data
- ownership, allocation, and hot-path expectations
- fit with the thread-per-core, share-nothing execution model
- live reconfiguration behavior and transition semantics
- backpressure, retry, acknowledgement, shutdown, and failure behavior
- security, authentication, and sensitive data handling, when relevant
- telemetry and diagnostics

The final architecture should be justified by user needs and OTAP constraints,
not by the package layout of the reference implementation.

## Define the User-Facing Contract

Before implementing broad option coverage, define what users will experience:

- configuration schema, defaults, and validation
- supported and unsupported behavior
- failure behavior and error reporting
- retry, backpressure, shutdown, and resource-limit behavior
- telemetry emitted by the capability
- migration guidance, when existing users are expected to move from another
  implementation

This is where the Rust implementation can intentionally improve usability
without losing the behaviors that users depend on.

## Validation Expectations

Validate the capability through scenarios rather than exact equivalence with
the reference implementation.

The first validation target should be the first useful end-to-end scenario.
Additional coverage should focus on configuration validation, representative
input/output data, pipeline integration, backpressure, retry, cancellation and
shutdown, live reconfiguration when relevant, unsupported features, known
community-reported edge cases, and regression tests for discovered problems.

Robustness validation should show that the component avoids panics, silent data
corruption, uncontrolled memory growth, unbounded queues, retry storms, hangs
during shutdown, misleading success telemetry, and silent data loss for
unsupported behavior.

Measure performance on the paths that matter for the OTAP design. Compare
against relevant Rust baselines and acceptance targets; compare against the
reference implementation only when that comparison is useful.

## PR Strategy

Start with an issue or design note that names the capability, user scenarios,
reference implementation, first scope, OTAP architecture decision, user-facing
contract, and main risks.

Split larger efforts so reviewers can evaluate design decisions separately from
implementation details. A typical sequence is:

1. Capability scope, evidence summary, and OTAP architecture decision.
2. User-facing contract, configuration validation, and component skeleton.
3. First useful end-to-end scenario with basic telemetry and failures.
4. Additional scenarios, runtime behavior, robustness, and performance.

Small efforts may combine these steps. The important point is that preserved,
changed, unsupported, and rejected behavior remain explicit and reviewable.

## Component Development Note

Each component using this approach must keep a short development note, for
example `<component-module>/DEVELOPMENT.md`.

The note should cover:

- capability scope and primary user scenarios
- reference implementation, feedback, and future direction reviewed
- important finding classifications
- OTAP architecture and component-boundary decisions
- user-facing contract and intentional behavior changes
- unsupported behavior
- validation coverage and known remaining work

Keep the note brief. It should help reviewers understand the design rationale
without duplicating the implementation or test suite.

## Completion Criteria

For the declared scope, the implementation is ready when:

- the user capability and first useful scenario are clear
- reference behavior, relevant feedback, and documented future direction have
  been analyzed
- important findings have been classified
- OTAP component boundaries and composition choices are justified
- preserved, improved, simplified, decomposed, composed, avoided, unsupported,
  and rejected behavior are intentional
- the user-facing contract is documented
- scenario tests cover the main user paths and failure modes
- the OTAP integration contract is defined, including runtime behavior, live
  reconfiguration, component composition, and telemetry
- robustness and performance are acceptable for the intended use
- the component development note reflects the current state
