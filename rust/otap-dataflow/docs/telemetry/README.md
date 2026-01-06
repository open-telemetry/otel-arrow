# Internal Telemetry Documentation and Policy

Status: **Draft** – under active development.

## Scope

This documentation applies to all telemetry produced by the OTAP dataflow engine
(runtime and its core libraries):

- metrics
- events (structured logs with an event name)
- traces (when implemented)
- resource metadata (service, host, container, process)

## Normative Language

The key words MUST, SHOULD, and MAY are to be interpreted as normative
requirements in all the documentation within this directory.

## Overview

Internal telemetry is a first-class concern of this project. As with any complex
system, reliable operation, performance analysis, and effective debugging
require intentional and well-designed instrumentation. This document defines the
principles, guidelines, and implementation details governing the project's
internal telemetry.

We follow an **observability by design** approach: observability requirements
are defined early and evolve alongside the system itself. All entities or
components are expected to be instrumented consistently, using well-defined
schemas and conventions, so that emitted telemetry is coherent, actionable, and
suitable for long-term analysis and optimization.

This approach is structured around the following lifecycle:

1) **Set clear goals**: Define observability objectives up front. Identify which
   signals are required and why.
2) **Automate**: Use tooling to derive code, documentation, tests, and schemas
   from shared conventions.
3) **Validate**: Detect observability and schema issues early through CI and
   automated checks, not in production.
4) **Iterate**: Refine telemetry based on real-world usage, feedback, and
   evolving system requirements.

Telemetry is treated as a **stable interface** of the system. As with any public
API, backward compatibility, semantic clarity, and versioning discipline are
essential. Changes to telemetry should be intentional, reviewed, and aligned
with the overall observability model.

See the [Stability and Compatibility Guide](stability-compatibility-guide.md)
for the stability model, compatibility rules, and deprecation process.

## Goals

Internal telemetry MUST enable:

- reliable operation and incident response
- performance analysis and regression detection
- capacity planning and saturation detection
- change impact analysis (deploys, config reloads, topology changes)
- long-term trend analysis with stable schema and naming

Telemetry MUST NOT compromise:

- system safety and correctness
- performance budgets on hot paths
- confidentiality (PII, secrets, sensitive payloads)

## Core Principles

The principles below define how internal telemetry is designed, implemented, 
validated, and evolved in this project. They are intentionally opinionated and
serve as a shared contract between contributors, tooling, and runtime behavior.

### 1. Schema-first

All telemetry is defined **schema-first**. Entities, signals, attributes, and
their relationships must be described explicitly in a schema before or alongside
their implementation.

Schemas are treated as versioned artifacts and as the primary source of truth
for:

* instrumentation requirements,
* validation rules,
* documentation generation,
* and client SDK generation.

Ad hoc or implicit telemetry definitions are discouraged, as they undermine
consistency, tooling, and long-term maintainability.

### 2. Entity-centric

Telemetry is modeled around **entities**, which represent stable, identifiable
subjects of observation. Signals describe the state, behavior, or performance of
one or more entities at a given point in time.

This project favors:

* clear separation between **entity attributes** (stable context) and
  **signal-specific attributes** (dynamic context),
* bounded and well-justified attribute cardinality,
* stable identifiers to support correlation across signals, restarts, and
  system boundaries.

Entity modeling is a prerequisite for producing telemetry that is interpretable,
composable, and operationally useful at scale.

### 3. Type-safe and performance-focused instrumentation

The telemetry SDK is **type-safe by construction** and **performance-aware**.

Instrumentation APIs should:

* prevent invalid or non-compliant telemetry at compile time whenever possible,
* minimize overhead on hot paths,
* avoid unnecessary allocations and dynamic behavior,
* make the cost of instrumentation explicit and predictable.

Correctness, efficiency, and safety take precedence over convenience.

### 4. Alignment with OpenTelemetry semantic conventions

We adopt **OpenTelemetry semantic conventions** as its baseline vocabulary and
modeling framework.

Where existing conventions are sufficient, they are reused directly. Where
project-specific concepts are required, they are defined in a **custom semantic
convention registry**, aligned with OpenTelemetry principles and formats.

This registry formally describes:

* the entities relevant to the project,
* the signals emitted by the system,
* the allowed attributes, types, units, and stability guarantees.

### 5. First-class support for multivariate metrics

The internal telemetry model and SDK natively support **multivariate metric
sets**.

This enables:

* efficient sharing of attribute tuples,
* coherent modeling of related measurements,
* reduced duplication and cardinality explosion compared to naive univariate
  metrics.

Multivariate metrics are treated as a fundamental modeling capability rather
than a post-processing optimization.

Note: OTLP and OTAP protocols do not yet have first-class support for
multivariate metrics. The SDK and exporters handle the necessary translation and
encoding. We plan to contribute multivariate support to OpenTelemetry protocols
in the future. In the meantime, this project serves as a proving ground for the
concept.

### 6. Tooling-driven validation and documentation with Weaver

Telemetry correctness and completeness are enforced through **tooling, not
convention alone**.

This project integrates with **Weaver** to:

* validate emitted telemetry against the versioned semantic convention registry,
* perform registry compliance checks in CI,
* execute live checks during tests to ensure that expected signals are actually
  produced,
* generate authoritative documentation in Markdown or HTML from the registry.

An administrative endpoint exposes the live, resolved schema at runtime to
support inspection, debugging, and tooling integration.

Security and deployment guidance for this endpoint is in the
[Security and Privacy Guide](security-privacy-guide.md).

### 7. Automated client SDK generation (longer term)

In the longer term, the custom semantic convention registry will be used to
generate **type-safe Rust client SDKs** via Weaver.

The objective is to:

* eliminate manual duplication between schema and code,
* ensure strict alignment between instrumentation and specification,
* provide contributors with safe, ergonomic APIs that encode observability
  rules directly in types.

This is considered a strategic investment and will be introduced incrementally.

### 8. Telemetry as a stable interface

Telemetry is treated as a **stable interface of the system**.
Refer to [Stability and Compatibility Guide](stability-compatibility-guide.md).

For items that are documented but not yet implemented or enforced, see
[Implementation Gaps](implementation-gaps.md).

## Runtime Safety and Failure Behavior

Telemetry MUST be non-fatal and bounded:

- Export failures MUST NOT break the dataflow engine.
- Telemetry pipelines MUST use bounded buffers.
- Under pressure, the default behavior SHOULD be to drop telemetry rather than
  block critical work.
- Drops SHOULD be observable via counters (by drop reason) and optionally debug
  events.

The telemetry system MUST NOT introduce deadlocks, unbounded memory growth, or
process termination.

## Instrumentation Guides

**Instrumentation** is the act of adding telemetry signals (metrics, events,
traces) to the codebase to observe the system behavior and performance.

The [entity model](entity-model.md) defines the observed things, the "nouns" of
our system, and how signals describe them. Entities are described by attributes
that provide context to metrics, events, and traces, and a single signal can
involve multiple entities at once. **Attribute cardinality must be bounded** to
keep telemetry efficient and aggregations meaningful. Identifier stability
matters for correlation across signals and restarts; refer to the stability
guarantees in the entity model when adding new attributes.

The naming conventions, units and general guidelines are in the
[semantic conventions guide](semantic-conventions-guide.md). **Please read it
before introducing new telemetry.**

The guides below provide a framework for defining **good, consistent, secure,
and actionable signals**. They are not an exhaustive list of every signal and
attribute in the project, but a shared reference for how to introduce and evolve
telemetry:

- [Attributes Guide](attributes-guide.md)
- [System Metrics Guide](metrics-guide.md)
- [System Events Guide](events-guide.md)
- [System Traces Draft - Not For Review](tracing-draft-not-for-review.md)
- [Stability and Compatibility Guide](stability-compatibility-guide.md)
- [Security and Privacy Guide](security-privacy-guide.md)

## Implementation Details

For implementation details of the telemetry SDK, including macros, schema
handling, and the dataflow for metric collection, see the
[telemetry implementation description](/crates/telemetry/README.md).

Note: This SDK is internal to the project and optimized for our use cases. It is
not intended for public use (at least not yet). It may change without notice.

The documentation in this directory focuses on the intented design and policy
aspects of internal telemetry. The current implementation does not yet fully
realize all goals and principles described here, but it is evolving rapidly.
The [implementation gaps](implementation-gaps.md) document tracks the progress.

## Contributor workflow (minimum)

When adding or changing telemetry:

1) Update the semantic convention registry first (schema-first).
2) Regenerate documentation and code (when applicable).
3) Run CI validation (registry checks, live checks in tests).
4) If the change is breaking, bump the registry version and add a migration
   note.

**Important Note**: This workflow is not yet fully supported by tooling. It is
described here to set expectations for the future. For now, please coordinate
with the maintainers when making telemetry changes.
