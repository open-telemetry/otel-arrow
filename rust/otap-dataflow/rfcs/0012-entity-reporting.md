---
Proposal Name: entity-reporting
Start Date: 2026-11-12
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC 0012: Entity Reporting

## Summary

The Dataflow Engine (DFE) has a well-defined entity model. The goal of this RFC
is to formalize that model in OpenTelemetry terms and outline how the DFE will
communicate it using OpenTelemetry signals.

For the sake of completeness, this RFC consolidates some information already
present about entities in the following documents:

- DFE: [Entity model][dfe-entity-model], [Attributes
  guide][dfe-attributes-guide], [Events guide][dfe-events-guide], and
  [Implementation gaps][dfe-implementation-gaps]
- OpenTelemetry: [Entity data model][otel-entity-data-model], [Entity
  events][otel-entity-events], and [Entity propagation][otel-entity-propagation]

## Motivation

DFE entities have status information that is valuable for monitoring and
troubleshooting. That status is available through the `/api/v1/status` admin
endpoint, but in a bespoke format, and it is not exported through the Internal
Telemetry System (ITS).

This entity information is also incomplete. While it does describe pipelines and
pipeline groups in some detail, it omits information about other entities such
as nodes, channels, and ports as well as their relationships.

We want full entity information to be available to DFE telemetry consumers like
any other internal telemetry. This includes:

- The current state of each entity
- Lifecycle events, including creation and deletion
- Relationships between entities
- Signals belonging to an entity

The natural approach is to align with the OpenTelemetry Entity model, but that
model and its representation in OTel Signals is still in the process of being
defined.

Under those conditions, this RFC:

- Re-defines the DFEs existing entity model in OpenTelemetry terms
- Proposes changes to the model for closer alignment to entity semantics
- Extends the model to communicate the full list of entity-related information
  above

## Guide-level explanation

### Entity modeling

Every entity reported by the DFE follows the abstract OpenTelemetry [entity data
model][otel-entity-data-model]. Every entity has a **type**, a set of
**identifying attributes**, and a set of **descriptive attributes**.

The DFE additionally distinguishes between two kinds of entities:

- Ambient - Entities that define the environment the DFE runs in: the host,
  container, process, service instance, and so on. The DFE does not create or
  own them. They are discovered by resource detectors, provided through [entity
  propagation][otel-entity-propagation] (`OTEL_ENTITIES`), or configured
  explicitly under `engine.telemetry.resource`.
- Internal - Entities that are created, owned, and deleted by the DFE: the
  engine, pipeline groups, pipelines, pipeline instances, nodes, channels, and
  extensions. All of the entity types in [Entity types](#entity-types) are
  internal.

#### Entity types

Internal Entity types are major components of the dataflow engine that are
present at runtime. An entity model is an abstraction like any other and care
must be exercised in choosing what to model and what not to.

- TODO: Add more concrete rules for entity types. Some ideas for changes are
  proposed in the reference guide but they're not explicit rules.

Entity types consisting of more than one word should join those words with an
underscore rather than a period e.g. `pipeline_instance`.

Example internal entity types:

| Type                | Description                                                                                                                                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `engine`            | The DFE instance running inside a process. Root of all entities the DFE owns.                                                                                                                                |
| `pipeline_group`    | A named group of pipelines.                                                                                                                                                                                  |
| `pipeline`          | A logical pipeline as defined in configuration. Survives rollouts, resizes, and replacements.                                                                                                                |
| `pipeline_instance` | A runtime instance of a logical pipeline. Replaced on every rollout that changes the pipeline.                                                                                                               |
| `node`              | A receiver, processor, or exporter running inside a pipeline instance.                                                                                                                                       |
| `port`              | An output port of a node. It exists so that each port's connection to a channel can be reported. It has no metrics of its own: the node records per-port metrics with `node.port` as a data point attribute. |
| `channel`           | A resolved runtime channel (one queue) contained by a pipeline instance. Its endpoints are not entities of their own (see [Channels and their endpoints](#channels-and-their-endpoints)).                    |

#### Entity hierarchy

DFE Entities form a hierarchy of containment where one entity is `part_of`
another. This containment tree is rooted at the engine and each internal entity
has exactly one containing entity.

```text
engine
  +-- pipeline_group
        +-- pipeline
              +-- pipeline_instance
                    +-- node
                    |     +-- port
                    +-- channel
                    +-- extension
```

The **containment chain** of an entity is the path from the engine to that
entity. For example the containment chain for a node is
`engine -> pipeline_group -> pipeline -> pipeline_instance -> node`.

Note: In addition to being contained by a pipeline instance, channels have
additional relationships to the other entities (see
[Entity relationships](#entity-relationships)).

#### Identifying attributes

Each internal entity is assigned an ID that is unique within its containing
entity.

For some entities, this is the `<entity>.id`, a human-friendly,
[Navigational][otel-navigation] attribute that points to the entity by name.

Some entities, however, are not uniquely described by their navigational id.
This happens when the lifetime of an entity is shorter than the lifetime of the
next entity in its containment chain.

For example, if a pipeline is deleted and recreated under the same name in the
same pipeline group, these are different entities and need to be distinguished
from each other.

For these entities, we assign an `<entity>.uid` (a [UUID v7][uuid-v7]) when the
entity is created. This allows us to track different incarnations of entities
with the same navigational attributes across time.

| Type                | Identifying attributes  | Owning Entity            |
| ------------------- | ----------------------- | ------------------------ |
| `engine`            | `engine.uid`            | Ambient entities, if any |
| `pipeline_group`    | `pipeline_group.uid`    | engine                   |
| `pipeline`          | `pipeline.uid`          | pipeline_group           |
| `pipeline_instance` | `pipeline_instance.uid` | pipeline                 |
| `node`              | `node.id`               | pipeline_instance        |
| `port`              | `port.id`               | node                     |
| `channel`           | `channel.id`            | pipeline_instance        |

#### Identity Scope

Since each entity ID is only **local** to the containing entity, to identify and
entity at a larger scope we join the ID of the entity with the ID of the
containing entity. This forms a **compound ID**.

This process can be repeated as necessary to form an ID which is unique at a
larger scope. A compound identifier unique across the entire engine is referred
to as an **engine-level ID**.

#### Descriptive attributes

All attributes other than the identifying attributes are descriptive and belong
to one of two classes:

- Navigational attributes - Attributes that never change during the lifetime of
  the entity and are helpful in finding the entity in the hierarchy. These are
  used for entities which are assigned a UID as their sole identifier. Examples:
  `pipeline_group.id` or `pipeline.id`.
- State attributes - Attributes indicating status or state that may change
  during the lifetime of the entity. Examples: the lifecycle phase of a pipeline
  instance (`Running`, `Draining`, ...), its conditions, or its last heartbeat
  time.

The identifying and navigational attributes of an entity, combined with those of
the entities that contain it (see
[Entity Containment Chain](#entity-containment-chain)), make up its [telescoping
identity][otel-telescoping].

Navigational and State attributes are modeled differently on signals.

#### Attribute properties

These principles yield useful properties:

- Delete and recreate is unambiguous - Every entity that can be recreated under
  the same name gets a fresh `uid`, and every other entity inherits one from its
  container.
- Creation order is recoverable - UUID v7 embeds a millisecond timestamp, so
  consumers can order instances of the same logical entity without an extra
  attribute.
- They are [minimally sufficient][otel-minimal-identity] at any scope
- The full [telescoping identity][otel-telescoping] is preserved, either as
  identifying or descriptive attributes.

### Signal Modeling and Entity Refs

Ambient entities and the `engine` are carried on a signal's resource. All other
internal entities are carried on the instrumentation scope.

#### Resource

The `Resource` contains the union of the attributes of all resource entities:
every ambient entity plus the `engine`. Every signal the DFE emits shares the
same resource.

Each resource entity is described with an [`EntityRef`][otlp-entity-ref] in
`Resource.entity_refs`. An entity ref names the entity type and lists which keys
in `Resource.attributes` are identifying and which are descriptive.

```text
Resource:
  attributes:
    host.id: 9f2c...
    host.name: edge-01
    service.name: otap-dataflow
    service.instance.id: 0b9e...
    engine.uid: 0193a1c2-...
  entity_refs:
    - type: host
      id_keys: [host.id]
      description_keys: [host.name]
    - type: service.instance
      id_keys: [service.name, service.instance.id]
    - type: engine
      id_keys: [engine.uid]
```

#### Scope

The DFE already places internal entity attributes on the scope rather than
repeating them on every log record or data point. This RFC keeps that approach
and further specifies what goes into the scope.

Every signal belongs to an entity. The scope attributes for a signal are the
engine-level identity of the owning entity plus the entity's navigational
attributes.

Additionally we set the scope name to the type of entity being described and all
metric sets registered to that entity type are merged under one scope when
reported.

For example, a metric from any of a node's metric sets carries:

```text
InstrumentationScope:
  name: node
  attributes:
    # pipeline_group
    pipeline_group.uid: 0193a1c2-...
    pipeline_group.id: ingest

    # pipeline
    pipeline.uid: 0193a1c3-...
    pipeline.id: otlp_to_kafka

    # pipeline.instance
    pipeline.instance.uid: 0193a1d7-...

    # node
    node.id: batch
```

State attributes are reported only in entity events and never on the scope,
because scopes are meant to identify the entity attached to a signal. State
attributes don't contribute to the minimal or telescoping identity and in the
case of metrics, the scope attributes are a part of the identity of the metric
stream.

### Entity Events

Logs and metrics give each internal entity's identity and stable description,
but they leave three things out:

- State attributes
- Explicit relationships, including other than containment
- Lifecycle events such as an explicit create or delete

The DFE reports these using OpenTelemetry [entity events][otel-entity-events].
Entity events are log records with a well-known `EventName`, and they flow
through the ITS logs pipeline like any other DFE log.

Entity events are emitted for **internal entities** only. Ambient entities are
owned by other observers (the OpenTelemetry SDK, a host agent, an orchestrator),
and the internal telemetry system does not have insight into their state.

#### Entity state events

The DFE may emit an `entity.state` event for an internal entity when:

- the entity is created,
- any of its state attributes changes, or
- the periodic report interval elapses (see
  [Periodic emission](#periodic-emission)).

Each event carries the complete current state of the entity:

- `entity.type` and `entity.id` (the identifying attributes),
- `entity.description`, containing both the navigational and the state
  attributes,
- `entity.relationships`, and
- `entity.report.interval`.

For example, a pipeline instance:

```text
LogRecord:
  EventName: entity.state
  Attributes:
    entity.type: pipeline.instance
    entity.id:
      pipeline.instance.uid: 0193a1d7-...
    entity.description:
      pipeline_group.id: ingest
      pipeline.id: otlp_to_kafka
      core.id: 3
      numa.node.id: 0
      deployment.generation: 2
      phase: Running                 # state
      conditions: [...]              # state
    entity.relationships:
      - relationship.type: part_of
        entity.type: pipeline
        entity.id:
          pipeline.uid: 0193a1c3-...
    entity.report.interval: 60
```

Note that the attributes under `entity.id` differ from scope attributes on
signals which are a union of identifying attributes for the containment chain as
well as descriptive navigational attributes.

Resource attributes are also not represented here and entity events inherit the
resource attributes from the signal resource they are contained within.

- TODO: Decide whether the description should include only the entity's own
  stable attributes, or the entity's attributes plus all attributes inherited
  along its containment chain.

#### Entity delete events

When an internal entity is removed, the DFE emits an `entity.delete` event with
its `entity.type`, its `entity.id`, and an `entity.delete.reason`. Possible
reasons include:

| Reason     | Example                                                                        |
| ---------- | ------------------------------------------------------------------------------ |
| `deleted`  | A pipeline or pipeline group was removed from configuration.                   |
| `replaced` | A pipeline instance was superseded by a newer deployment generation.           |
| `scaled`   | A pipeline instance was removed because the pipeline's core allocation shrank. |
| `failed`   | A pipeline instance stopped because of an unrecoverable error.                 |
| `shutdown` | The engine is shutting down.                                                   |

Deleting an entity implicitly deletes all relationships that involve it.

- TODO: Decide whether contained entities (nodes, channels) get their own delete
  events when their pipeline instance is deleted, or whether consumers infer
  their deletion through a containment relationship.

Note that the OpenTelemetry specification [does not
guarantee][otel-entity-delete] that delete events are delivered. For example,
they are lost if the process crashes. Consumers must also expire entities whose
state events stop arriving, using the last reported `entity.report.interval`.

#### Entity relationships

Relationships are reported in the `entity.relationships` attribute of the entity
state event. OpenTelemetry recommends placing a relationship on the entity with
the [shorter lifespan or higher churn][otel-relationship-placement], so that
creating or deleting short-lived entities does not force state events for
long-lived ones. The DFE follows this rule.

Containment is reported as a `part_of` relationship from each internal entity to
its containing entity:

| Source              | Relationship | Target              |
| ------------------- | ------------ | ------------------- |
| `pipeline_group`    | `part_of`    | `engine`            |
| `pipeline`          | `part_of`    | `pipeline_group`    |
| `pipeline.instance` | `part_of`    | `pipeline`          |
| `node`              | `part_of`    | `pipeline.instance` |
| `port`              | `part_of`    | `node`              |
| `channel`           | `part_of`    | `pipeline.instance` |

- TODO: Decide which ambient entity the `engine` relates to (`service.instance`,
  `process`, or both), given that `service.instance.id` can be overridden by
  users.
- TODO: Decide whether to report relationships to cores or NUMA nodes
- TODO: Add different extension types to this table

Connections describe how data moves between nodes, in the direction it flows.
They are reported as `connected_to` relationships:

| Source    | Relationship   | Target      | Meaning                                                                                                                             |
| --------- | -------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `port`    | `connected_to` | `channel`   | The port sends into the channel. Every port has exactly one.                                                                        |
| `channel` | `connected_to` | `node`      | The channel delivers to the node. A pdata channel has one per destination node; a control channel has one, to the node it controls. |
| `channel` | `connected_to` | `extension` | The control channel delivers to the extension.                                                                                      |

Channels report the actual runtime channels after connections are merged, so
these relationships describe the topology the engine runs, not the configuration
as written. See [Example pipeline](#example-pipeline) for a complete set.

A control channel has no sending port. It is fed by the pipeline instance's
runtime controller, and the pipeline instance already contains it, so no
separate relationship is needed for its sender.

All of these entities share the lifetime of their pipeline instance, so the
placement rule does not choose a side. The DFE reports `port -> channel` on the
port and `channel -> node` on the channel, so that each relationship is reported
by the entity it starts from.

#### Periodic emission

Periodic state events are the liveness signal for internal entities. They also
recover from lost events. The ITS configuration gets a new setting:

```yaml
engine:
  telemetry:
    entities:
      # Interval between periodic entity.state events for each entity.
      # 0 disables periodic emission; state events are then emitted only on
      # creation and state change, and delete events on removal.
      report_interval: 60s
```

The configured value is reported in each state event as
`entity.report.interval`. A value of `0` matches the OpenTelemetry meaning: no
periodic state events will be sent.

The way periodic events are spread over the interval, the behavior under
backpressure, and the ordering guarantees are described in the
[Reference-level explanation](#reference-level-explanation).

### `/api/v1/status`

- TODO: Explain how we're going to represent status. Should we change the shape
  of engine status to be aligned to the entity model more explicitly? This could
  maybe be in the "unresolved questions" or "future possibilities" section

## Reference-level explanation

The technical portion of the RFC. Explain the design in enough detail that:

- Its interaction with existing features (crates, macros, factories, the engine,
  xtask, CI) is clear.
- It is reasonably clear how the feature would be implemented, and in which
  crates.
- Corner cases are covered by example.

Return to the examples from the guide-level section and show how the detailed
design makes them work.

- TODO: UUID generation and storage. Describe where `uid`s are generated and
  stored (attribute sets, entity registry, controller state for logical
  pipelines and groups). Cover how a `pipeline.uid` survives rollouts while a
  `pipeline.instance.uid` does not.
- TODO: Event sources. Map entity registry register and unregister calls, and
  observed state store transitions, to `entity.state` and `entity.delete`
  events.
- TODO: Sweeping strategy. Spread periodic state events across the report
  interval (for example with a cursor over the entity registry) rather than
  emitting every entity at once. Give rough volume estimates for large
  deployments (many cores times pipelines times nodes and channels).
- TODO: Configuration plumbing and validation for `engine.telemetry.entities`,
  and its interaction with the internal telemetry

### Merging Entity-typed scopes

Today the OTLP encoder emits one scope per registered metric set:

- The scope `name` is the metric set name, for example `node.output`.
- The scope attributes are the attributes of the entity the metric set is
  registered against.
- Each metric name is the bare field name, for example `messages`.
- Scopes are keyed by metric set descriptor and entity, so an entity with N
  metric sets is reported in N scopes that repeat the same attributes. Metric
  sets that share a name do not merge either: a node's `node.output` messages,
  items, and size metric sets produce three scopes with identical names and
  attributes.

Instead this RFC proposes:

- An entity attribute set declares its entity type:
  `#[attribute_set(scope, name = "node.attrs", entity = "node")]`. The type is
  exposed through `AttributeSetHandler::entity_type()` and is not part of the
  entity's identity in the entity registry.
- The encoder names each scope after the entity type and keys scopes by entity
  instance, so every metric set registered against one entity is reported in one
  scope.
- All metric sets for the same entity type are merged under a single scope
- Each metric name is qualified with its metric set name, for example
  `node.output.messages`.

Measured impact for this change in a pipeline with five nodes and four edges:

| Raw bytes                 | Gzip bytes             | Scopes   | Data points |
| ------------------------- | ---------------------- | -------- | ----------- |
| 58,212 -> 42,921 (-26.3%) | 8,409 -> 8,056 (-4.2%) | 61 -> 19 | 164 -> 164  |

### Changes to existing entities and metric sets

Proposed changes to specific entities and metric sets have been removed from
this RFC as an orthogonal concern, each of which can be evaluated independently.

## Drawbacks

- The reporting methods of ambient and internal entities are different and
  consumers have to interpret events to get internal DFE entities.
- Entity events have to be interpreted in the context of their containing
  resource. This is not documented in OpenTelemetry entities today
- Scopes still contain some amount of redundant information that could be
  communicated by relationships instead, though it's not clear how to do this
  using existing signals.
- Using scope name as the entity is our own invention and undocumented
- Additional bytes for UUIDs which every entity has at least one and up to three
  of in their containment chain.
- Every metric name and scope name changes with
  [Entity-typed scopes](#entity-typed-scopes). This is a breaking change for
  existing dashboards and alerts on every internal metric.
- With `report_interval: 0`, where we don't report events, there's not a good
  interval for the consumer to use to expire an entity. So if the delete event
  is lost it's not clear how the consumer will expire it.

## Rationale and alternatives

### A `uid` on every entity

An earlier draft of this RFC assigned a `uid` to every internal entity,
including nodes, channels, and extensions. This was rejected for three reasons.
This was rejected to maintain a minimal unique id.

- TODO: Explain whether `engine.uid` is necessary. It is kept because the
  engine's containing entities are ambient and `service.instance.id` can be
  overridden, and because it lets consumers track engine restarts over time.

### Other considerations

- TODO: We should consider prefixing our entities with `dfe.` or some other
  prefix based on the final name given to the engine. There is a compelling
  reason in a lot of systems where a dfe and an otc, both with "pipelines"
  exists.

## Prior art

Discuss prior art, both good and bad. What can we learn from how other projects
(Rust, OpenDAL, the OpenTelemetry Collector, etc.) solved this? If there is no
prior art, say so -- new ideas are welcome.

## Unresolved questions

- What parts of the design do you expect to resolve through the RFC process
  before merge?
- What related issues are out of scope for this RFC and could be addressed
  independently later?

### Telescoping identity on the wire

This RFC places every navigational attribute on the scope. A deployment could
instead choose to send only the identifying attributes and rely on entity events
to join the navigational attributes back in. That would mean moving toward the
UUID-only end of the [telescoping identity][otel-telescoping] spectrum. Should
this be configurable, and at what granularity?

## Future possibilities

What natural extensions or follow-ups does this enable? This is a place to
record ideas that are out of scope now but worth remembering.

[dfe-entity-model]: ../docs/telemetry/entity-model.md
[dfe-attributes-guide]: ../docs/telemetry/attributes-guide.md
[dfe-events-guide]: ../docs/telemetry/events-guide.md
[dfe-implementation-gaps]: ../docs/telemetry/implementation-gaps.md
[dfe-stability-guide]: ../docs/telemetry/stability-compatibility-guide.md
[otel-entity-data-model]:
  https://opentelemetry.io/docs/specs/otel/entities/data-model/
[otel-minimal-identity]:
  https://opentelemetry.io/docs/specs/otel/entities/data-model/#minimally-sufficient-identity
[otel-repeatable-identity]:
  https://opentelemetry.io/docs/specs/otel/entities/data-model/#repeatable-identity
[otel-entity-propagation]:
  https://opentelemetry.io/docs/specs/otel/entities/entity-propagation/
[otel-entity-events]:
  https://opentelemetry.io/docs/specs/otel/entities/entity-events/
[otel-relationship-placement]:
  https://opentelemetry.io/docs/specs/otel/entities/entity-events/#relationship-placement
[otel-entity-delete]:
  https://opentelemetry.io/docs/specs/otel/entities/entity-events/#entity-delete-event
[otel-telescoping]:
  https://github.com/open-telemetry/opentelemetry-specification/blob/a4e05d9704fc5218d85a4154fb5b143ea3f5340e/specification/resource/README.md#telescoping
[otel-navigation]:
  https://github.com/open-telemetry/opentelemetry-specification/blob/a4e05d9704fc5218d85a4154fb5b143ea3f5340e/specification/resource/README.md#navigation
[otlp-entity-ref]:
  https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/common/v1/common.proto
[uuid-v7]: https://www.rfc-editor.org/rfc/rfc9562#name-uuid-version-7
