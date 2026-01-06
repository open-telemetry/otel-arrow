# Attributes Guide

Status: Draft

This document consolidates project decisions and guidance on attributes:
naming, placement, cardinality, normalization, and lifecycle.

It complements:

- [semantic-conventions-guide.md](semantic-conventions-guide.md) for upstream
  naming rules
- [entity-model.md](entity-model.md) for the entity attribute sets and
  relationships
- [stability-compatibility-guide.md](stability-compatibility-guide.md) for
  evolution rules
- [security-privacy-guide.md](security-privacy-guide.md) for sensitive-data
  constraints

## Attribute Categories

Attributes fall into three categories.

### 1) Resource Attributes

Describe the producing service and runtime environment.

- MUST be attached at the resource level
- MUST NOT be duplicated on every signal
- SHOULD reuse upstream semantic conventions (`service.*`, `host.*`,
  `process.*`, `container.*`)

### 2) Entity Attributes

Identify stable in-process entities (pipelines, nodes, channels, runtime
threads).

- MUST be attached at the scope level
- MUST NOT be duplicated on every signal
- MUST be stable for the lifetime of the entity
- MUST be bounded and known at entity creation time
- MUST be the foundation of metric set identity for core system telemetry

### 3) Signal-Specific Attributes

Provide additional bounded context needed to interpret a measurement or event
occurrence.

- MAY be used when required for interpretation
- MUST be bounded and documented
- MUST remain meaningful under aggregation (metrics) and filtering (events)

## Naming and Namespaces

### Reuse Upstream First

- Reuse existing OpenTelemetry semantic attributes whenever possible.
- Do not redefine upstream attributes with different meaning.

### Project-Defined Namespace

Project-defined entity attributes MUST be namespaced to avoid collisions with
upstream conventions.

Policy:

- Use `otelcol.*` for project-defined attributes.
- Do not introduce new un-prefixed top-level namespaces for custom entities.

### Closed Sets (enums)

When an attribute represents a categorical dimension:

- The value set MUST be a documented closed set.
- Values MUST be lowercase and stable.
- Avoid synonyms that fragment cardinality (`fail` vs `error` vs `failed`).

Adding enum values for stable telemetry follows the compatibility rules in
[stability-compatibility-guide.md](stability-compatibility-guide.md).

## Placement Rules

- Resource attributes belong on the resource attributes.
- Entity attributes belong on the scope attributes.
- Signal-specific attributes belong only where they apply, and must be bounded.

Do not duplicate information:

- If a value is already present as an entity attribute, do not repeat it as a
  signal-specific attribute.
- Prefer a single canonical key.

## Cardinality Policy

### Core Rule

Attributes attached to core system metrics MUST have bounded cardinality.

Before adding an attribute, ask:

- If I aggregate across this attribute, does the result still make sense?
- Is the value space bounded and known at entity creation time?

If not, the attribute is mis-modeled for core system metrics.

### Prohibited by Default in Core System Metrics

The following are prohibited as metric attributes unless explicitly approved and
normalized:

- user_id, session_id, request_id
- raw URL path, raw query string
- raw SQL, raw error messages, unbounded file paths
- unbounded plugin configuration values

### Normalization Patterns

When context is useful but high cardinality, normalize:

- URL path -> route template
- SQL query -> normalized fingerprint
- IP address -> prefix or bucket
- error message -> error class or error type

## Errors and exceptions

### Error classification

Prefer low-cardinality classification:

- Use `error.type` (or an equivalent stable classifier) when applicable.
- Avoid raw error messages as attributes in stable telemetry.

### Exceptions

When recording an actual exception:

- Use `exception.type` and `exception.message`.
- `exception.stacktrace` must follow
  [security-privacy-guide.md](security-privacy-guide.md) gating rules.

## Attributes vs Event Body

This project distinguishes between queryable attributes and potentially large
bodies:

- Put small, queryable fields in attributes.
- Put large payloads in the body only when strictly required.
- Do not duplicate the same data in both places.

## Checklist

When introducing a new attribute:

- It is categorized (resource, entity, signal-specific).
- It reuses upstream semantic attributes when available.
- If project-defined, it uses the `otelcol.*` namespace.
- Cardinality is bounded and documented.
- For enums, the closed set is documented and stable.
- It follows security and privacy rules (no sensitive data).
- If stable, the change follows the compatibility rules.
