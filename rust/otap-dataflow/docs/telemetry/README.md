# Internal Telemetry Documentation

## Design Principles

- Schema-first definitions for entities, signals, and attributes.
- Performance-focused, NUMA-aware collection paths.
- Multivariate metric sets to share attribute tuples efficiently.

In the medium term, we aim for observability by design: entities and signals
should be described via OpenTelemetry semantic conventions, and Weaver should
be integrated into CI to enforce schema compliance, generate telemetry
documentation, and produce type-safe, optimized client SDKs.

## Instrumentation Guides

**Instrumentation** is the act of adding telemetry signals (metrics, events,
traces) to the codebase to observe the system behavior and performance.

The [entity model](entity-model.md) defines the **observed things (entities)**
and how signals describe them. Entities are described by attributes that provide
context to metrics, events, and traces, and a single signal can involve multiple
entities at once. **Attribute cardinality must be bounded** to keep telemetry
efficient and aggregations meaningful. Identifier stability matters for
correlation across signals and restarts; refer to the stability guarantees in
the entity model when adding new attributes.

The naming conventions, units and general guidelines are in the
[semantic conventions guide](semantic-conventions-guide.md). **Please read it
before introducing new telemetry.**

The guides below provide a framework for defining **good, consistent, and
actionable signals**. They are not an exhaustive list of every signal in the
project, but a shared reference for how to introduce and evolve telemetry:

- [System Metrics Guide](metrics-guide.md)
- [System Events Guide](events-guide.md)
- [System Traces Proposal](tracing-proposal.md) (not yet implemented)

## Implementation Details

For implementation details of the telemetry SDK, including macros, schema
handling, and the dataflow for metric collection, see the
[telemetry implementation description](/crates/telemetry/README.md).

## Instrumentation Validation, Coverage, and CI Integration

TBD
