# OTAP Dataflow semantic conventions

This directory contains the exhaustive semantic-convention contract for
production attributes, entities, metrics, and events emitted by OTAP Dataflow.
Definitions use Weaver's experimental
[`definition/2` syntax][definition-v2].

Tracking issue:
<https://github.com/open-telemetry/otel-arrow/issues/1613>

## Layout

```text
semconv/
  manifest.yaml
  registry/
    attributes.yaml
    entities.yaml
    metrics/*.yaml
    events/*.yaml
```

The manifest imports upstream OpenTelemetry semantic conventions when the
project can reuse an existing attribute. All project-owned definitions live
under `registry/`.

## Signal and entity model

Each Rust scope-level `#[attribute_set]` is represented by an entity. Item-level
attribute sets used while recording metrics are referenced by the metric's
standard `attributes` field instead. Composed scope attribute sets are
flattened into the entity identity, along with
`service.instance.id`. The hierarchy between scope entities is recorded in the
`otap_dataflow.parent_entities` annotation. Semantic-convention v2 supports
associating a metric or event with entities, but it does not define a native
entity-to-entity relationship expression.

Metrics use `<metric_set>.<instrument_name>` as their canonical convention name.
Each metric records only its numeric `code_generation.metric_value_type`, its
`otap_dataflow.metric_set` membership, and exceptional generation overrides.
The standard metric fields remain the source of truth for the instrument, unit,
and description.

Events retain an existing wire name as their convention name when it is valid
v2 syntax. An invalid wire name receives a normalized `otap.*` convention name,
while `otap_dataflow.wire.event_name` preserves the emitted value. Event
definitions include every statically declared attribute, scope, severity, and
source location.

Metrics and events use `entity_associations` to identify the entity carrying
their scope attributes. When a signal can originate from several alternative
scope types, the association uses `one_of`.

The `otap_dataflow` annotations are project metadata. Weaver validates the
standard v2 fields and references.

## Validation

Run the check from `rust/otap-dataflow`:

```bash
weaver registry check --v2 --registry semconv
```

The command validates the v2 registry and imported references. CI pins Weaver
to v0.25.1 and always passes `--v2`.

## Updating the contract

When a production telemetry declaration changes, update the corresponding v2
definition in the same change. Run the Weaver registry check before submitting
the change.

[definition-v2]: https://github.com/open-telemetry/weaver/blob/v0.25.1/schemas/semconv-syntax.v2.md
