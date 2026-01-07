# Stability and compatibility guide

Status: Draft

This document defines the stability model, compatibility rules, and change
process for internal telemetry in the OTAP dataflow engine.

Telemetry is treated as a stable interface. This guide defines what that means
in practice and how we evolve telemetry without breaking operators and
downstream consumers.

## Scope

This guide applies to all telemetry schema elements defined in our semantic
convention registry and emitted by the system, including:

- metric names, units, instrument semantics, and attribute sets
- metric sets: the shared attribute set + grouped metrics for an entity
- event names (LogRecord `event_name`), event attributes, and event body shape
- trace span names and attributes (when tracing is implemented)
- project-defined entity attributes and their semantics

## Stability levels

Every schema element that is intended for reuse by operators or downstream
tooling MUST declare a stability level:

- **experimental**
  - may change without backward compatibility guarantees
  - intended for iteration and proving utility
- **stable**
  - only backward compatible evolution is allowed
  - breaking changes require versioning and a migration plan
- **deprecated**
  - still emitted for a migration window
  - has a documented replacement
  - has a planned removal milestone

### What must carry stability

At minimum, stability MUST be declared for:

- each metric
- each metric set
- each event name
- each project-defined attribute that is part of stable signals
- each trace span name (when implemented)

## Compatibility rules
 
### General rule

For stable telemetry, changes MUST preserve the ability for existing dashboards,
alerts, and queries to continue working with the same meaning.

### Backward compatible changes (generally allowed)

For **stable** signals, the following are typically backward compatible:

- adding a new metric to an existing metric set
- adding a new optional attribute whose cardinality is bounded and documented
- adding a new enum value to a documented closed set when existing meaning
  remains valid
- clarifying descriptions without changing meaning

### Breaking changes (require migration)

For **stable** signals, the following are breaking changes and require:

- a registry version bump (see Versioning)
- a migration plan
- dual emission where practical (old and new) during a migration window

Breaking changes include:

- renaming metrics, metric sets, event names, span names, or attribute keys
- changing units
- changing instrument semantic meaning (counter vs gauge semantics,
  monotonicity, temporality assumptions)
- removing a metric, event, span, or attribute
- changing the meaning of an attribute value
- widening attribute cardinality such that aggregations change meaning or cost

## Compatibility by signal type

### Metrics

Metric identity is defined by:

- metric name
- unit
- instrument semantic meaning
- attribute keys and their meaning (including enum value sets)

Stable metrics MUST follow these rules:

- Name MUST NOT change. If a rename is required, add a new metric and deprecate
  the old one.
- Unit MUST NOT change. If a unit correction is required, add a new metric and
  deprecate the old one.
- Attribute keys for a stable metric MUST remain compatible:
    - You MAY add a new optional bounded attribute.
    - You MUST NOT remove an attribute or repurpose it.
- Enum-like attributes MUST be documented as closed sets. Adding values is
  allowed if aggregation meaning remains safe.

### Metric sets

A metric set is a collection of metrics sharing the same entity attribute set.

For stable metric sets:

- the metric set name MUST remain stable
- the shared entity attribute set MUST remain stable
- adding a new metric is allowed (additive evolution)
- changing the shared attribute set is breaking unless it is strictly additive
  and optional

### Events

Event names act as schema identifiers.

For stable events:

- the event name MUST remain stable
- required attributes MUST remain required
- removing or renaming attributes is breaking
- adding new optional bounded attributes is allowed
- event body shape SHOULD remain compatible:
    - avoid changing body from string to object (or vice versa) for stable
      events
    - if richer payload is required, prefer introducing a new event name and
      deprecating the old one

### Traces

(When implemented)

For stable spans:

- span names MUST remain stable
- required attributes MUST remain required
- avoid repurposing attribute meaning
- exception span events MUST use the canonical exception event naming and
  attributes (see semantic conventions guide)

## Deprecation process

When deprecating stable telemetry, follow this process:

1) Mark the signal as **deprecated** in the registry.
2) Introduce the replacement signal first.
3) Emit both old and new during a migration window.
4) Provide migration guidance:
    - mapping table (old -> new)
    - example queries and dashboard update notes
5) Remove the deprecated signal only after the migration window ends.

### Recommended migration window

Default guideline:

- at least 2 releases for internal dashboards
- longer if external consumers exist or long-lived dashboards depend on it

## Versioning model

The semantic convention registry MUST be versioned.

Recommended approach:

- Use SemVer for the registry version:
    - MAJOR for breaking changes to stable telemetry
    - MINOR for backward compatible additions
    - PATCH for documentation corrections or strictly non-semantic fixes

A release that includes a breaking change to stable telemetry MUST:

- bump the registry MAJOR version
- include migration guidance in release notes
- include dual emission where practical for at least one migration window

## Migration patterns

Prefer these patterns:

- **dual emission**
    - emit old and new signals together temporarily
- **alias and translate at export**
    - if exporter can map old key to new key without losing meaning
- **side-by-side dashboards**
    - validate new telemetry before switching alerts

Avoid:

- silent renames
- silent unit changes
- implicit meaning changes without versioning

## Review checklist

For any telemetry change:

- Stability level is declared or updated.
- Compatibility impact is assessed (additive vs breaking).
- Breaking changes include a migration plan and version bump.
- Additions have bounded cardinality and documented meaning.
- Docs and generated artifacts are updated.
- CI validation passes.
