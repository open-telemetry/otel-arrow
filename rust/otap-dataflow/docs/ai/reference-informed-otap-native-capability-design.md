# Reference-Informed OTAP-Native Capability Design

This document provides a contributor framework for implementing a receiver,
processor, exporter, extension, or related capability inspired by an existing
OpenTelemetry Collector component.

Use this approach when the goal is to deliver the best OTAP-native capability,
not necessarily to reproduce the existing Go implementation as-is.

## When to Use This Approach

Use this approach when implementing or redesigning:

- receivers
- processors
- exporters
- extensions
- shared component libraries
- component families inspired by Go Collector or contrib components

This approach is appropriate when the existing implementation is useful as a
source of evidence, but the OTAP implementation may intentionally differ.

For example:

- one Go receiver may become one Rust receiver
- one Go receiver may become a receiver plus an extension
- one Go receiver may become a receiver plus one or more processors
- several Go components may become one shared OTAP capability
- configuration may be simplified
- defaults may be changed
- error handling may be made stricter
- runtime behavior may be redesigned for OTAP

## Goal

The goal is to provide a capability that is better for OTAP users.

Better may mean:

- safer defaults
- clearer configuration
- better error messages
- better composability
- better performance
- lower memory use
- fewer surprising behaviors
- stronger validation
- clearer telemetry
- better reliability under load
- cleaner separation of responsibilities

The reference implementation should inform the design, but it should not
automatically define the Rust component boundary or internal structure.

## Core Principle

The reference implementation is evidence, not an oracle.

The main question is not:

```text
How do we translate this Go component into Rust?
```

The main question is:

```text
What OTAP-native capability should we provide to users?
```

## Workflow

### 1. Define the Capability

Start from the user outcome.

Document:

- what users need to accomplish
- what data enters the component
- what data leaves the component
- which external systems are involved
- which operational problems matter
- which failure modes matter
- which scale and cardinality are expected
- what the first useful release should support
- what is out of scope

This helps avoid copying the shape of the existing Go component too early.

### 2. Analyze the Reference Implementation

Use AI to accelerate analysis, but review the conclusions manually.

Extract:

- responsibilities
- configuration options
- defaults
- validation rules
- lifecycle behavior
- batching behavior
- retry behavior
- authentication behavior
- queueing behavior
- shutdown behavior
- error handling
- telemetry
- supported data types
- unsupported data types
- tests and examples
- edge cases
- performance-sensitive paths

Do not assume that every extracted behavior should be preserved.

### 3. Learn from Community and Operational Feedback

When available, review issues, pull requests, documentation, discussions, and
known operational experience.

Look for:

- common misconfigurations
- confusing options
- surprising defaults
- recurring bugs
- performance complaints
- memory issues
- high-cardinality problems
- unclear errors
- security-sensitive behavior
- deprecated behavior
- common workarounds
- feature requests

This is often where the best design improvements come from.

### 4. Classify Findings

Classify each important finding before it becomes part of the Rust design.

| Classification | Meaning |
|---|---|
| Preserve | Required for compatibility, user expectation, or interoperability. |
| Improve | Useful behavior, but should be safer, clearer, faster, or more robust. |
| Simplify | Can be represented with a simpler OTAP model. |
| Decompose | Should be split across receiver, processor, exporter, extension, or library. |
| Compose | Should be achieved by combining existing OTAP components. |
| Avoid | Legacy, surprising, fragile, or tied to Go implementation details. |
| Reject | Conflicts with OTAP design, reliability, safety, or scope. |
| Investigate | More evidence is required. |

This classification should be captured in the module methodology note.

### 5. Design the OTAP-Native Architecture

Decide where each responsibility belongs.

Possible outcomes:

- receiver only
- processor only
- exporter only
- extension only
- receiver plus processor
- receiver plus extension
- exporter plus processor
- shared protocol library
- shared configuration module
- reusable authentication or connection module

Questions to answer:

- Is the Go component combining responsibilities that should be separated?
- Can existing OTAP components provide part of this capability?
- Should enrichment happen in a processor instead of a receiver?
- Should authentication or connection management live in an extension?
- Does the design preserve composability?
- Does the design avoid unnecessary materialization?
- Does the design fit OTAP backpressure and scheduling?
- Does the design avoid unbounded memory growth?
- Does the design produce useful telemetry?

The final architecture should be justified by user needs and OTAP constraints,
not by the layout of the Go implementation.

### 6. Define the User-Facing Contract

Before implementing all options, define what users will experience.

Document:

- configuration schema
- required fields
- defaults
- validation rules
- supported data types
- unsupported data types
- error behavior
- telemetry
- retry behavior
- backpressure behavior
- shutdown behavior
- resource limits
- migration guidance, when relevant

This is where the Rust implementation can intentionally improve usability.

### 7. Implement the First Useful Slice

Start with a small, reviewable implementation that proves the architecture.

The first slice should include:

- representative configuration
- one useful end-to-end scenario
- basic OTAP runtime behavior
- basic telemetry
- explicit unsupported-feature handling
- tests for success paths
- tests for failure paths
- documentation of known limitations

Avoid implementing every option before the design has been validated.

### 8. Validate Through Scenarios

For this approach, scenario tests are usually more important than exact
equivalence with the Go implementation.

Recommended tests:

- valid configuration scenarios
- invalid configuration scenarios
- representative input and output data
- pipeline integration
- external system unavailable
- authentication failure
- timeout
- retry behavior
- backpressure behavior
- cancellation and shutdown
- unsupported features
- known community-reported edge cases
- regression tests for previously discovered failures

The Go implementation may be used as a comparison point, but exact equivalence
is not automatically required.

### 9. Validate Robustness and Performance

Robustness checks should verify that the component avoids:

- panics
- silent data corruption
- uncontrolled memory growth
- unbounded queues
- retry storms
- hangs during shutdown
- misleading success telemetry
- silent data loss for unsupported behavior

Performance validation should measure the OTAP design, not only the translated
logic.

Measure relevant paths:

- ingestion
- transformation
- export
- batching
- retry
- telemetry overhead
- allocations
- throughput
- latency
- peak memory

Compare against acceptance targets and previous Rust baselines. Compare against
the Go implementation when the comparison is useful.

## Suggested PR Slicing

A capability-focused implementation can often be split as follows:

1. Add capability brief and methodology note.
2. Add reference analysis and design decisions.
3. Add configuration schema and validation.
4. Add component skeleton and lifecycle wiring.
5. Add first successful end-to-end scenario.
6. Add unsupported-feature handling.
7. Add telemetry.
8. Add failure handling, retry, backpressure, and shutdown behavior.
9. Add additional supported scenarios.
10. Add regression tests from community feedback.
11. Add benchmarks and performance tuning.
12. Finalize documentation and release checklist.

The exact sequence depends on the component. The goal is to make each PR
reviewable and to keep design choices visible.

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

This component follows the Reference-Informed OTAP-Native Capability Design
approach.

## Capability Summary

This component provides:

- ...

Primary user scenarios:

- ...

Out of scope:

- ...

## Reference Implementation Reviewed

Repository:

- ...

Commit or version:

- ...

Relevant packages or modules:

- ...

Reference behavior analyzed:

- ...

## Community and Operational Feedback

Issues, PRs, discussions, or known feedback reviewed:

- ...

Important findings:

- ...

## Finding Classification

| Finding | Classification | OTAP decision | Rationale |
|---|---|---|---|
| ... | Preserve / Improve / Simplify / Decompose / Compose / Avoid / Reject / Investigate | ... | ... |

## OTAP Architecture Decision

Chosen component structure:

- receiver:
- processor:
- exporter:
- extension:
- shared library:

Why this structure was chosen:

- ...

Alternatives considered:

- ...

## User-Facing Contract

Configuration:

- ...

Defaults:

- ...

Validation:

- ...

Supported behavior:

- ...

Unsupported behavior:

- ...

Telemetry:

- ...

Failure behavior:

- ...

Migration notes, if relevant:

- ...

## OTAP Runtime Notes

- input representation:
- output representation:
- batching:
- backpressure:
- retry:
- ack and nack:
- cancellation:
- shutdown:
- memory behavior:
- hot-path materialization:
- telemetry:

## Tests Added

- configuration tests:
- scenario tests:
- integration tests:
- failure tests:
- unsupported-feature tests:
- regression tests:
- robustness tests:
- benchmarks:

## Checklist

- [ ] Capability defined.
- [ ] Reference implementation reviewed.
- [ ] Community or operational feedback reviewed, when available.
- [ ] Findings classified.
- [ ] OTAP architecture decision documented.
- [ ] User-facing contract documented.
- [ ] Runtime behavior documented.
- [ ] First useful scenario implemented.
- [ ] Unsupported behavior documented and tested.
- [ ] Telemetry added.
- [ ] Failure behavior tested.
- [ ] Backpressure, retry, cancellation, or shutdown tested, when relevant.
- [ ] Benchmarks added, when relevant.
- [ ] Documentation updated.
- [ ] Release criteria satisfied.

## Remaining Work

- ...
```

## Completion Criteria

For the declared scope, the implementation is ready when:

- the user capability is clear
- reference behavior has been analyzed
- important findings have been classified
- OTAP component boundaries are justified
- preserved behavior is intentional
- changed behavior is intentional
- unsupported behavior is documented and tested
- scenario tests cover the main user paths
- failure behavior is controlled and observable
- telemetry is useful
- performance is acceptable for the intended use
- the module methodology note is up to date
