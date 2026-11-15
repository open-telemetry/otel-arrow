---
Proposal Name: entity-reporting
Start Date: 2026-11-12
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC 0012: Entity Reporting

## Summary

The Dataflow Engine (DFE) has a well-defined entity model. The goal of this RFC is
to formalize that model in OpenTelemetry terms and outline how the DFE will
communicate it using OpenTelemetry signals.

For the sake of completeness, this RFC consolidates some information already
present about entities in the following documents:

- DFE: [Entity model][dfe-entity-model],
  [Attributes guide][dfe-attributes-guide],
  [Events guide][dfe-events-guide], and
  [Implementation gaps][dfe-implementation-gaps]
- OpenTelemetry: [Entity data model][otel-entity-data-model],
  [Entity events][otel-entity-events], and
  [Entity propagation][otel-entity-propagation]

Reviewers familiar with the existing DFE entity model and OpenTelemetry Entity
work may want to focus on the following sections which highlight important changes
to our existing model:

- [Entity types](#entity-types): the logical `pipeline` is separated from its
  runtime `pipeline.instance`.
- [Identity](#identity): internal entities that can be recreated under the
  same name get a UUID v7 `uid`. Nodes, channels, and extensions are
  identified by their pipeline instance's `uid` plus their own id.
- [Descriptive attributes](#descriptive-attributes): existing identifiers become
  navigational attributes, distinct from state attributes.
- [Scope](#scope) and
  [Attaching signals to entities](#attaching-signals-to-entities): which
  attributes go on the scope and how a signal is tied to one entity.
- [Entity Events](#entity-events): state, delete, and relationship reporting,
  and the new `engine.telemetry.entities` setting.

## Motivation

DFE entities have status information that is valuable for monitoring and
troubleshooting. That status is available through the `/api/v1/status` admin
endpoint, but in a bespoke format, and it is not exported through the Internal
Telemetry System (ITS).

We want entity information to be available to DFE telemetry consumers like any
other internal telemetry. This includes:

- the current state of each entity,
- lifecycle events, including creation and deletion, and
- relationships between entities.

The natural approach is to align with OpenTelemetry Entities, which are still
being defined. To that end this RFC places the following constraints on
itself:

- DFE entities follow the OpenTelemetry [entity data model][otel-entity-data-model]:
  each has a type, identifying attributes, and descriptive attributes.
- Entity state, lifecycle, and relationships are reported with OpenTelemetry
  [entity events][otel-entity-events] rather than a DFE-specific format.
- No changes to the OTLP protocol are required. Internal entities are carried
  in existing fields (resource, scope, and log record attributes).

The existing entity model also has problems that this RFC addresses:

- **Ambiguous identity.** A pipeline or node that is deleted and recreated with
  the same configuration produces the same attribute values as the original, so
  consumers cannot tell the two apart.
- **No ordering in time.** There is no way to order entities by creation other
  than inferring it from when their signals first appeared.
- **Conflated lifetimes.** The `pipeline` attribute set describes a runtime
  instance (it carries `core.id` and `deployment.generation`), and nothing
  describes the logical pipeline that outlives those instances.
- **Implicit relationships.** Relationships are only implied by attribute sets
  that appear together on the same signal. A relationship cannot be observed
  while the entities involved emit no telemetry.

## Guide-level explanation

### Entity modeling

Every entity reported by the DFE follows the abstract OpenTelemetry
[entity data model][otel-entity-data-model]: it has a **type**, a set of
**identifying attributes**, and a set of **descriptive attributes**.

#### Entity types

The DFE defines the following entity types. This list is intentionally kept at
the level of principles; the full hierarchy (including per-type attribute
tables and how CPU cores and NUMA nodes are modeled) is refined separately.

- TODO: Refine the complete entity hierarchy and per-type attribute tables
  against the principles in this RFC. See [Unresolved questions](#unresolved-questions).
- TODO: `engine` is the type implied by the existing `engine` attribute set
  and metric set. It is a very generic name for a type that may eventually be
  proposed to OpenTelemetry Semantic Conventions; revisit before stabilizing.

| Type | Lifetime | Identifying attributes | Description |
| --- | --- | --- | --- |
| `engine` | Process | `engine.uid` | The DFE instance running inside a process. Root of all entities the DFE owns. |
| `pipeline.group` | Group creation to deletion | `pipeline.group.uid` | A named group of pipelines. |
| `pipeline` | Pipeline creation to deletion | `pipeline.uid` | A logical pipeline as defined in configuration. Survives rollouts, resizes, and replacements. |
| `pipeline.instance` | One deployment generation on one core | `pipeline.instance.uid` | A runtime instance of a logical pipeline. Replaced on every rollout that changes the pipeline. |
| `node` | Same as its pipeline instance | `pipeline.instance.uid`, `node.id` | A receiver, processor, or exporter running inside a pipeline instance. |
| `channel` | Same as its pipeline instance | `pipeline.instance.uid`, `channel.id` | A control or pdata channel owned by a pipeline instance. Its endpoints are not entities (see [Channels and their endpoints](#channels-and-their-endpoints)). |
| `extension` | Same as its hosting scope | `pipeline.instance.uid`, `extension.id` | An extension, currently hosted by a pipeline instance. |

The distinction between `pipeline` and `pipeline.instance` is new. The existing
entity model conflates the logical pipeline with its runtime instances: the
`pipeline` attribute set carries `core.id` and `deployment.generation`, which
makes it describe an instance, while nothing describes the logical pipeline
that outlives those instances. Separating them gives each entity a single,
well-defined lifetime, which is a prerequisite for assigning it a stable
identity.

#### Identity

Internal entities are identified in one of two ways, depending on whether
their natural key can be reused.

**An entity is assigned a `uid` if it can be deleted and recreated with the
same natural key during the lifetime of its containing entity.** The `uid` is
a [UUID v7][uuid-v7] generated when the entity is created, and it is the
entity's sole identifying attribute. The attribute is named `<type>.uid`. This
applies to:

- `pipeline.group` and `pipeline`. A group or pipeline can be deleted and
  recreated with the same id. A recreated pipeline also restarts its
  deployment generation counter at `0`, so it produces exactly the same
  attribute values as the original.
- `pipeline.instance`. A rollback relaunches the previous deployment
  generation on a core, so the same `core.id` and `deployment.generation` can
  belong to two different instances of one pipeline.
- `engine`. The engine is never recreated within its process, but its
  containing entities are ambient, and `service.instance.id` can be overridden
  by users. A `uid` keeps the engine's identity self-contained.

This follows the Kubernetes convention, where `k8s.pod.uid` identifies a pod
and `k8s.pod.name` is a human-friendly, descriptive name. The DFE keeps the
existing human-friendly identifiers of these entities (`pipeline.group.id`,
`pipeline.id`) as navigational attributes (see
[Descriptive attributes](#descriptive-attributes)).

**All other entities have a composite identity**: the identifying attributes
of their containing entity plus their own local id. Nodes, channels, and
extensions are built once per pipeline instance and are never recreated inside
it, so a node is identified by `pipeline.instance.uid` and `node.id`, a
channel by `pipeline.instance.uid` and `channel.id`, and an extension by
`pipeline.instance.uid` and `extension.id`. The local id is unique within the
instance, and the instance `uid` is unique everywhere, so neither attribute is
sufficient on its own and together they are.

This gives the identities useful properties:

- **Delete and recreate is unambiguous.** Every entity that can be recreated
  under the same name gets a fresh `uid`, and every other entity inherits one
  from its container.
- **Creation order is recoverable.** UUID v7 embeds a millisecond timestamp, so
  consumers can order instances of the same logical entity without an extra
  attribute.
- **It is minimally sufficient.** The OpenTelemetry data model asks that the
  identifying attributes be the
  [minimal set sufficient][otel-minimal-identity] to uniquely identify the
  entity. A `uid` is minimal for entities whose natural key can be reused. For
  nodes, channels, and extensions, a `uid` of their own would add nothing that
  the instance `uid` and their local id do not already provide. The data model
  accepts identities that are only unique in context: its process example,
  `process.pid` and `process.start_time`, is unique only within a host. The
  composite identity goes further by including the container's `uid`, so it is
  unique on its own.
- **It matches how users query.** Composite identities use the ids that users
  write in configuration, such as `node.id`, and those stay stable across
  deployment generations.
- **The DFE is the only observer.** OpenTelemetry asks for
  [repeatable identity][otel-repeatable-identity] across observers. The
  internals of the DFE (pipeline instances, nodes, channels) cannot be
  observed from outside the process, so the DFE is the source of truth for
  these entities and is free to choose its own identifiers.

#### Descriptive attributes

All attributes other than the identifying attributes are descriptive. The
attributes of an entity therefore fall into three classes, distinguished by
whether they can change during the lifetime of the entity:

- **Identifying attributes** never change and form the entity's minimal
  identity (see [Identity](#identity)).
- **Navigational attributes** are descriptive attributes that never change
  during the lifetime of the entity. They let users find the entity and act on
  it. Examples: `pipeline.group.id`, `pipeline.id`, `core.id`,
  `deployment.generation`, `node.urn`, `node.type`.
- **State attributes** are descriptive attributes that may change during the
  lifetime of the entity. Examples: the lifecycle phase of a pipeline instance
  (`Running`, `Draining`, ...), its conditions, or its last heartbeat time.

Whether an id is identifying or navigational follows from the identity rule,
not from its name. `pipeline.id` is navigational because a pipeline can be
deleted and recreated with the same `pipeline.id` inside its group. `node.id`
is identifying because a node cannot be recreated with the same `node.id`
inside its pipeline instance. Both are fixed for the lifetime of their entity,
so the distinction does not affect where they appear: identifying and
navigational attributes are placed on signals the same way.

The term "navigational" comes from the [Navigation][otel-navigation] section of
the OpenTelemetry Resource specification, which distinguishes attributes that
identify an entity from attributes that help users find it in their
infrastructure.

The identifying and navigational attributes of an entity, combined with those
of the entities that contain it (see [Entity Containment Chain](#entity-containment-chain)),
make up its [telescoping identity][otel-telescoping].

> [!NOTE]
> Many of the navigational attributes look like identifiers, and some of
> them (for example `pipeline.group.id` + `pipeline.id` + `core.id` +
> `deployment.generation`) were used as a compound identity before
> this RFC. That compound key is not unique across delete and recreate or
> rollback, so it is replaced by `pipeline.instance.uid`. Because the `uid` is
> already minimally sufficient, the
> [minimally sufficient identity][otel-minimal-identity] rule requires every
> one of these attributes to be descriptive rather than identifying.

Identifying and navigational attributes appear on every signal related to the
entity, while state attributes appear only in entity events. The reasons are explained in
[Signal modeling and Entity Refs](#signal-modeling-and-entity-refs) and
[Entity Events](#entity-events).

### Entity Containment Chain

The DFE distinguishes between two kinds of entities:

- **Ambient entities** describe the environment the DFE runs in: the host,
  container, process, service instance, and so on. The DFE does not create or
  own them. They are discovered by resource detectors, provided through
  [entity propagation][otel-entity-propagation] (`OTEL_ENTITIES`), or
  configured explicitly under `engine.telemetry.resource`.
- **Internal entities** are created, owned, and deleted by the DFE: the
  engine, pipeline groups, pipelines, pipeline instances, nodes, channels, and
  extensions. All of the entity types in [Entity types](#entity-types) are
  internal.

Internal entities form a containment tree rooted at the `engine`. Each internal
entity other than the engine has exactly one containing entity (the engine may
have no detected ambient containing entities):

```text
engine
  +-- pipeline.group
        +-- pipeline
              +-- pipeline.instance
                    +-- node
                    +-- channel
                    +-- extension
```

Channels are contained by the pipeline instance, not by the nodes or
extensions they connect. This holds for both kinds of channel. A pdata channel
spans several nodes, so no single node can own it. A control channel has a
single owning node or extension, but it is modeled the same way so that every
channel has the same containing entity type. The owner of a control channel
is still visible through its `channel.id` (see
[Channel metric sets](#channel-metric-sets) for its format).

The **containment chain** of an entity is the path from the engine to that
entity. For example, the containment chain of a node is
`engine -> pipeline.group -> pipeline -> pipeline.instance -> node`.

Identifying and navigational attributes accumulate along the chain. The full
description of a node is its own attributes (`node.id`, `node.urn`,
`node.type`) together with those of its pipeline instance, pipeline, pipeline
group, and engine. The chain is used for navigation and for inferring which entity
a signal belongs to (see
[Signal modeling and Entity Refs](#signal-modeling-and-entity-refs)).

### Signal modeling and Entity Refs

Ambient entities and the `engine` are carried on the resource. All other
internal entities are carried on the instrumentation scope.

#### Resource

The `Resource` contains the union of the attributes of all resource entities:
every ambient entity plus the `engine`. Every signal the DFE emits shares the
same resource.

Each resource entity is described with an [`EntityRef`][otlp-entity-ref] in
`Resource.entity_refs`. An entity ref names the entity type and lists which
keys in `Resource.attributes` are identifying and which are descriptive.

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

The DFE already places internal entity attributes on the scope rather than repeating
them on every log record or data point. This RFC keeps that approach and further
specifies what goes into the scope.

Every signal belongs to an entity. The scope attributes for a signal are the union,
over every internal entity in the containment chain of:

- the entity's identifying attributes, and
- the entity's navigational attributes.

In other words, the scope carries every attribute in the chain that is fixed
for the lifetime of its entity. A composite identity adds only the local id,
because the containing entity's `uid` is already on the scope.

For example, a log emitted by a node, or a metric from a node's metric set,
carries:

```text
InstrumentationScope:
  name: <metric set name or tracing target>       # unchanged
  attributes:
    # pipeline.group
    pipeline.group.uid: 0193a1c2-...
    pipeline.group.id: ingest
    # pipeline
    pipeline.uid: 0193a1c3-...
    pipeline.id: otlp_to_kafka
    # pipeline.instance
    pipeline.instance.uid: 0193a1d7-...
    core.id: 3
    numa.node.id: 0
    deployment.generation: 2
    # node
    node.id: batch                # identifying, with pipeline.instance.uid
    node.urn: urn:otel:processor:batch
    node.type: processor
```

A metric from a pipeline instance metric set (for example the existing
`pipeline` metric set) carries the same attributes without the `node.*`
attributes.

The scope `name` keeps its current meaning: the name of the metric set for
metrics, or the tracing target for logs. It describes the kind of telemetry,
not the entity.

#### Attaching signals to entities

Each log record or metric belongs to exactly one entity: the **narrowest**
entity in its scope's containment chain whose identifying attributes are all
present on the scope. A consumer can infer that entity from the DFE
containment chain and the identifying attributes present.

| Scope identifying attributes present | Signal belongs to |
| --- | --- |
| None | `engine` (on the resource) |
| `pipeline.group.uid` | `pipeline.group` |
| ... `pipeline.uid` | `pipeline` |
| ... `pipeline.instance.uid` | `pipeline.instance` |
| ... `node.id` | `node` |
| ... `channel.id` | `channel` |
| ... `extension.id` | `extension` |

This works because containment is a tree, and a scope only ever carries the
identifying attributes of a single containment chain. Each type has its own
identifying keys, and sibling types such as `node`, `channel`, and `extension`
never appear on the same scope, so the deepest entity present is unambiguous.
Only scope attributes are considered: a data point attribute such as the
`node.id` endpoint dimension on channel metrics does not change which entity
the signal belongs to. Whether the scope should also mark the
entity type explicitly is an open question (see
[Unresolved questions](#unresolved-questions)).

Signals with an empty scope, such as the existing `engine` metric set, belong
to the `engine`.

Only attributes of the entities in the containment chain appear on the scope.
When a signal describes a population inside its entity, such as the endpoints of a
channel, the attributes that distinguish the population are data point
attributes, not scope attributes.

#### Channels and their endpoints

Entities and metrics reflect the channels the engine actually builds, not the
connections as written in configuration. Operators care about the resolved
queues: how many there are, which nodes share them, and how full they are.

The engine resolves the configured connections of a pipeline into
**hyperedges**. A hyperedge is a set of source `(node, output port)` pairs, a
set of destination nodes, and a dispatch policy. Connections with the same
destination set and dispatch policy are merged into one hyperedge, so a single
hyperedge can collect output ports from several nodes. Each output port
belongs to exactly one hyperedge, so a node with several output ports takes
part in one hyperedge per port.

Each hyperedge is backed by exactly **one queue**, and a `channel` entity is
that queue. When a hyperedge has several destinations, they are competing
consumers of the same queue (`one_of` dispatch). Broadcast to more than one
destination is not supported today, so no hyperedge creates a queue per
destination. A control channel is likewise one queue, owned by its node or
extension.

Channel metrics belong to the `channel`. The scope of a channel metric carries
the channel's containment chain (`engine -> pipeline.group -> pipeline ->
pipeline.instance -> channel`) and nothing else. In particular, it no longer
carries the attributes of the node or extension at either end of the channel.

The endpoints of a channel are not entities. They are populations within the
channel, identified by the data point attributes `node.id` and `node.port`.
Whether an endpoint is a sender or a receiver is given by the metric set name
(`channel.sender` or `channel.receiver`), as it is today. Adding the data
points across endpoints gives a meaningful total for the channel, for example
all messages sent into it.

Metrics that describe a queue, such as its depth and capacity, carry exactly
the dimensions that distinguish the channel's physical queues. Today every
channel has one queue, so these metrics have no endpoint attributes and are
reported once per channel. If a future dispatch policy gives a channel one
queue per destination, its queue metrics would gain the receiving `node.id`
as a dimension.

The changes to the existing channel metric sets are described in
[Changes to existing metric sets](#changes-to-existing-metric-sets).

#### State attributes are never placed on the scope

State attributes are reported only in entity events and never on
the scope. Scope attributes are part of the identity of a metric stream. If a
value that changes, such as a pipeline instance's phase, were placed on the
scope, every change would start a new metric stream (and, for Prometheus, a
new series with different `otel_scope_*` labels). Navigational
attributes do not have this problem because they never change during the
lifetime of the entity, and the entity's identifying attributes already scope
the stream.

### Entity Events

Logs and metrics give each internal entity's identity and stable description,
but they leave three things out:

- the state attributes,
- relationships other than containment, and
- an explicit signal that the entity has been created or deleted.

The DFE reports these using OpenTelemetry [entity events][otel-entity-events].
Entity events are log records with a well-known `EventName`, and they flow
through the ITS logs pipeline like any other DFE log.

Entity events are emitted for **internal scope entities**. Ambient entities are
owned by other observers (the OpenTelemetry SDK, a host agent, an
orchestrator), and the internal telemetry system does not report events for them.

Entity events carry the same resource as all other DFE telemetry. Their scope
is empty: the subject of each event is in the event's attributes, not on the
scope.

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
  Resource: <shared DFE resource>
  Attributes:
    entity.type: pipeline.instance
    entity.id:
      pipeline.instance.uid: 0193a1d7-...
    entity.description:
      pipeline.group.id: ingest
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

For an entity with a composite identity, `entity.id` contains every
identifying attribute, including the containing entity's `uid`. For example,
a node:

```text
    entity.type: node
    entity.id:
      pipeline.instance.uid: 0193a1d7-...
      node.id: batch
```

The `part_of` relationship to the containing entity is still reported
explicitly, even though it can be derived from the composite identity.

- TODO: Define the exact state attributes for each entity type,
  and how they map from the status that `/api/v1/status` reports today.

The description includes the navigational attributes of the entity itself.
It also includes the navigational attributes of the containing
entities that it already carries on its scope today (for example
`pipeline.id` on a pipeline instance). This keeps entity events self-contained
for consumers that do not reconstruct the containment chain.

- TODO: Decide whether the description includes only the entity's own stable
  attributes, or the entity's attributes plus all attributes inherited along
  its containment chain.

#### Entity delete events

When an internal entity is removed, the DFE emits an `entity.delete` event
with its `entity.type`, its `entity.id`, and an `entity.delete.reason`.
Possible reasons include:

| Reason | Example |
| --- | --- |
| `deleted` | A pipeline or pipeline group was removed from configuration. |
| `replaced` | A pipeline instance was superseded by a newer deployment generation. |
| `scaled` | A pipeline instance was removed because the pipeline's core allocation shrank. |
| `failed` | A pipeline instance stopped because of an unrecoverable error. |
| `shutdown` | The engine is shutting down. |

Deleting an entity implicitly deletes all relationships that involve it. The
DFE does not emit separate delete events for entities contained by a deleted
entity unless they are torn down independently.

- TODO: Decide whether contained entities (nodes, channels) get their own
  delete events when their pipeline instance is deleted, or whether consumers
  infer their deletion through containment.

The OpenTelemetry specification
[does not guarantee][otel-entity-delete] that delete events are delivered.
For example, they are lost if the process crashes. Consumers must also expire
entities whose state events stop arriving, using the last reported
`entity.report.interval`.

#### Entity relationships

Relationships are reported in the `entity.relationships` attribute of the
entity state event. OpenTelemetry recommends placing a relationship on the
entity with the
[shorter lifespan or higher churn][otel-relationship-placement], so that
creating or deleting short-lived entities does not force state events for
long-lived ones. The DFE follows this rule.

**Containment** is reported as a `part_of` relationship from each internal
entity to its containing entity:

| Source | Relationship | Target |
| --- | --- | --- |
| `pipeline.group` | `part_of` | `engine` |
| `pipeline` | `part_of` | `pipeline.group` |
| `pipeline.instance` | `part_of` | `pipeline` |
| `node` | `part_of` | `pipeline.instance` |
| `channel` | `part_of` | `pipeline.instance` |
| `extension` | `part_of` | `pipeline.instance` |

**Across the boundary**, the `engine` reports a `runs_on` relationship to the
`service.instance` it runs in. This is the only relationship from an internal
entity to an ambient entity. It lets consumers join the DFE's internal
entities with the infrastructure graph that other observers report. Entity
events can reference any entity by type and id, so the target's attributes do
not need to be present on the event's resource.

- TODO: Decide which ambient entity the `engine` relates to (`service.instance`,
  `process`, or both), given that `service.instance.id` can be overridden by
  users.

**Placement** relationships, between pipeline instances and the CPU cores and
NUMA nodes they run on, are expected to be reported the same way (for example
`pipeline.instance -> runs_on -> core` and `core -> part_of -> numa.node`).
They are deliberately left unspecified here.

- TODO: Define placement relationships once cores, NUMA nodes, and engine
  worker threads are modeled. See [Unresolved questions](#unresolved-questions).

**Channels** connect nodes. In the future this topology could be reported as
relationships between channels and the nodes they connect (for example
`channel -> connects -> node`).

- TODO: Decide whether channel topology is reported as entity relationships,
  and in which direction. This includes the owner of a control channel, which
  today is only encoded in its `channel.id`.

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
`entity.report.interval`. A value of `0` matches the OpenTelemetry meaning:
no periodic state events will be sent.

The way periodic events are spread over the interval, the behavior under
backpressure, and the ordering guarantees are described in the
[Reference-level explanation](#reference-level-explanation).

### `/api/v1/status`

- TODO: Explain how we're going to represent status. Should we change the shape
  of engine status to be aligned to the entity model more explicitly? This could
  maybe be in the "unresolved questions" or "future possibilities" section

## Reference-level explanation

The technical portion of the RFC. Explain the design in enough detail that:

- Its interaction with existing features (crates, macros, factories, the
  engine, xtask, CI) is clear.
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
- TODO: Backpressure and bounded resources. Specify what happens when the logs
  path cannot accept entity events: drop, coalesce, or block. Specify the
  memory bounds on pending events.
- TODO: Ordering. Define how delete and state events are ordered and
  timestamped, given that `metric_unregister_grace_period` delays entity
  unregistration and that replacement instances can overlap with the instances
  they replace.
- TODO: Configuration plumbing and validation for
  `engine.telemetry.entities`, and its interaction with the internal telemetry
  receiver.

### Changes to existing metric sets

#### Channel metric sets

Today, channel metrics are registered per endpoint. Each endpoint registers a
channel entity whose attribute set composes the full attributes of the node or
extension at that end (`NodeChannelAttributeSet`,
`NodeWithCustomChannelAttributeSet`, `ExtensionChannelAttributeSet` in
`crates/engine/src/attributes.rs`). Under this RFC the channel entity carries
only its own attributes, and the endpoint becomes a data point dimension (see
[Channels and their endpoints](#channels-and-their-endpoints)).

**Scope.** The channel attribute set drops the composed node and extension
attributes and `node.port`, and composes the pipeline instance attributes
instead. The channel is identified by `pipeline.instance.uid` and
`channel.id`. The remaining attributes (`channel.kind`, `channel.mode`,
`channel.type`, `channel.impl`) are navigational. All endpoints of a channel
build an identical attribute set, so the entity registry deduplicates them
into a single entity key with a reference count per endpoint.

**Endpoint dimensions.** Each endpoint registers its metric sets against the
shared channel entity and supplies `node.id` and `node.port` as fixed data
point attributes. The `registration_attributes` option of `#[metric_set]`
already supports this: registration attributes are fixed for the lifetime of a
registration, may be strings, and are combined with the existing measurement
attributes (`signal`, `outcome`, `error.type`). The scope already carries
`pipeline.instance.uid`, so `node.id` on the data point completes the node's
identifying attributes (see [Identity](#identity)). Each endpoint dimension
therefore refers to its node by the node's identity. It is also the attribute
users query on, and it stays stable across deployment generations.

```text
InstrumentationScope:
  name: channel.sender
  attributes:
    # pipeline.group, pipeline, pipeline.instance attributes as for a node
    channel.id: hyperedge:3f9a...   # identifying, with pipeline.instance.uid
    channel.kind: pdata
    channel.mode: shared
    channel.type: mpmc
    channel.impl: flume
Metric: channel.sender.messages
  data points:
    - {node.id: otlp_in, node.port: default, signal: logs, outcome: success}
    - {node.id: filelog_in, node.port: default, signal: logs, outcome: success}
```

**Queue state.** `queue_depth` and `capacity` describe the queue, not an
endpoint. Today each receiving endpoint of a shared multi-consumer channel
registers its own `ChannelReceiverStateMetrics` (`crates/engine/src/lib.rs`)
while all receivers read the same queue. With endpoints as dimensions, adding
these gauges across receivers would report a multiple of the real queue.
Queue state is therefore registered once per channel, without endpoint
dimensions, matching the single queue behind each hyperedge. Only the
`messages` counters keep per-endpoint dimensions.

- TODO: A node that is a destination of more than one hyperedge (for example
  `a -> [c]` and `b -> [c, d]`) passes configuration validation but is not
  supported by the engine. A node holds a single pdata receiver, so the
  second hyperedge's queue replaces the first: debug builds panic during
  pipeline build, and release builds silently fail every send into the
  replaced queue. This is tracked separately from this RFC. The channel model
  is unaffected by the fix: either such topologies are rejected, or the node
  appears as a receiver endpoint of several channels.

**Control channels.** Both the sender and receiver control channel metric sets
are registered on the owner's endpoint today, with `node.port` fixed to
`input` (`crates/engine/src/channel_mode.rs`). Individual senders are not
distinguished. Control channel metrics therefore need no endpoint dimensions:
they belong to the channel, and the owner remains identifiable through
`channel.id`. Extension-hosted control channels are handled the same way.

**Control channel ids.** A control channel's id is currently `{owner}:control`
(`control_channel_id` in `crates/engine/src/channel_metrics.rs`), where the
owner is a node id or an extension id. Configuration validation rejects
duplicate node ids and duplicate extension ids, but not a node and an
extension that share an id. Today the two control channels are still told
apart by the composed node or extension attributes on their scopes. Under this
RFC those attributes leave the channel scope, and the channel is identified by
`pipeline.instance.uid` and `channel.id` alone, so the two channels would
collide. The control channel id therefore includes the owner kind:
`node:{node.id}:control` or `extension:{extension.id}:control`. This change
rides on the channel metric change, which is already breaking, and it does not
reject any existing configuration. The alternative is a validation rule that
rejects a node and an extension sharing an id.

**Custom node attributes.** Today the custom attributes configured under a
node's `entity.extend.identity_attributes` are copied onto that node's channel
scopes. Because the channel scope no longer carries node attributes, they are
dropped from channel metrics. Consumers can join them in through `node.id`.

- TODO: Investigate replacing `entity.extend.identity_attributes` with
  `entity.extend.descriptive_attributes`. Under this RFC identifying
  attributes are determined by the identity rule, so user-supplied attributes
  can only ever be descriptive, and it is not clear that user-supplied identity is ever
  necessary.

**Compatibility.** Moving `node.*` and `node.port` from the scope to data
point attributes changes the identity of every channel metric stream. For stable
signals the [stability guide][dfe-stability-guide] treats this as a breaking
change that requires a migration plan. The
on-the-wire size is expected to stay roughly the same: each data point gains
`node.id` and `node.port`, while the per-endpoint scopes of a channel collapse
into one.

## Drawbacks

- TODO: Explain that consumers have to interpret events to get internal DFE
entities
- TODO: Explain that there is still redundancy on scope attributes, but it's not
avoidable in existing OpenTelemetry signals
- TODO: Explain that signals cannot be tied to entities above the boundary. This is
not actually a problem in the current engine design, but for opentelemetry more
generally this seems like a hole because what we're doing with scope is not
a standard
- TODO: Explain the expected byte size difference due to additional uid attributes
on a typical signal. Summarize per entity how many extra bytes and also for a typical
resource log how many bytes
- Channel metrics change stream identity, because endpoint attributes move from
  the scope to data point attributes. This is a breaking change for existing
  dashboards and alerts on channel metrics. See
  [Changes to existing metric sets](#changes-to-existing-metric-sets).

## Rationale and alternatives

### A `uid` on every entity

An earlier draft of this RFC assigned a `uid` to every internal entity,
including nodes, channels, and extensions. This was rejected for three
reasons:

- **It adds nothing to uniqueness.** Nodes, channels, and extensions are never
  recreated inside their pipeline instance, so the instance `uid` plus their
  local id already identifies them.
- **It is wasteful where it matters most.** Nodes and channels are the most
  numerous entities (cores x pipelines x nodes and channels). A `uid` adds about
  45 bytes of key and value to every node and channel scope before
  compression, and it must be generated, stored, and kept consistent across
  every registration of the same entity.
- **It weakens references by id.** Channel metrics identify their endpoints
  with `node.id`, the attribute users query on. With a `node.uid`, that
  dimension would refer to a node by a non-identifying attribute. With a
  composite identity, `node.id` on the data point and `pipeline.instance.uid`
  on the scope are exactly the node's identifying attributes.

- TODO: Explain whether `engine.uid` is necessary. It is kept because the
  engine's containing entities are ambient and `service.instance.id` can be
  overridden, and because it lets consumers track engine restarts over time.

### When the identity rule must be revisited

The rule in [Identity](#identity) is applied per containing entity, not per
type: an entity needs a `uid` only if it can be recreated with the same
natural key while its container is alive. This holds today for every type in
this RFC. The following changes would require revisiting it:

- **Per-node restart.** Today a node failure fails its pipeline instance, and
  recovery replaces the instance with a new deployment generation and a new
  `pipeline.instance.uid`. If individual nodes could be restarted inside an
  instance, nodes would need a `uid`.
- **Live topology changes inside an instance.** Adding or removing nodes or
  channels without a new deployment generation would have the same effect.
- **Extensions hosted outside pipelines.** Engine-level controller extensions
  already exist. They start once with the controller and are never restarted,
  so `engine.uid` plus `extension.id` would identify them. They are not
  reported as entities today. If engine- or group-hosted extensions become
  live add and remove, extensions hosted there would need a `uid`.
- **Topics.** If topics become entities (see
  [Redefining the entity hierarchy](#redefining-the-entity-hierarchy)) and can
  be added and removed live, they would need a `uid`.

In-place node reconfiguration does not trigger the rule. The plumbing for it
exists (`NodeControlMsg::Config`) but is not used in production. A node
reconfigured in place stays the same entity, because its configuration is not
one of its attributes and `node.urn` and `node.type` cannot change in place.
A configuration change may still warrant a state attribute or an
`entity.state` event.

### Other considerations

- TODO: We should consider prefixing our entities with a `dfe.`. This is
disallowed in our documentation today, but there is a compelling reason in a lot
of systems where a dfe and an otc, both with "pipelines" exists. All k8s entities
have a prefix, so why not us?

- Why is this design the best in the space of possible designs?
- What other designs were considered, and why were they not chosen?
- What is the impact of not doing this?

## Prior art

Discuss prior art, both good and bad. What can we learn from how other
projects (Rust, OpenDAL, the OpenTelemetry Collector, etc.) solved this?
If there is no prior art, say so -- new ideas are welcome.

## Unresolved questions

- What parts of the design do you expect to resolve through the RFC process
  before merge?
- What related issues are out of scope for this RFC and could be addressed
  independently later?

### Explicit scope entity marker

Today, the entity a signal belongs to is inferred from the deepest entity
whose identifying attributes are on its scope (see
[Attaching signals to entities](#attaching-signals-to-entities)).
Two alternatives make it explicit:

- **Scope level entity refs.** Propose to OpenTelemetry an equivalent of
  `Resource.entity_refs` on `InstrumentationScope`. Every scope would only
  ever reference one entity, because the signals in a scope logically belong
  to a single entity. That makes a general list of refs heavier than needed.
- **An `entity.type` scope attribute.** Add a single attribute naming the type
  of the entity the scope belongs to. This is simpler, but it adds an
  attribute to every scope when the entity can already be inferred from the
  identifying attributes and the containment chain.

Either option would make the model self-describing for generic consumers that
do not know the DFE's containment chain. Both have to be weighed against
verbosity on the wire.

### Redefining the entity hierarchy

The entity types in this RFC are a first pass. The existing hierarchy needs to
be redefined against the principles here:

- How CPU cores, NUMA nodes, and engine worker threads are modeled. Are they
  internal entities (an engine worker pinned to a core), ambient entities (a
  host CPU), or only descriptive attributes of a pipeline instance?
- The naming clash between the process-wide `engine` entity and the existing
  per-core `controller.attrs` attribute set (`core.id`, `numa.node.id`), which
  `docs/telemetry/entity-model.md` also calls the "execution engine".
- The full per-type attribute tables, including channel and extension
  entities. This includes how `thread.id`, documented but not emitted today,
  fits in.
- TODO: Classify the existing scope attributes that this RFC does not yet
  cover:
  - `scope.kind` on the extension scope (`ExtensionScopeAttributeSet`). It
    names the kind of scope hosting the extension, which may be redundant once
    the host can be inferred from the identifying attributes present.
    Extensions hosted by the engine or a pipeline group would also give
    `extension` more than one containing entity type.
  - `extension.variant` (`local` or `shared`), likely navigational.
  - `topic` on node metric sets (`NodeWithTopicAttributeSet`). Topics are
    declared at the engine or group level and span pipelines, so `topic` may
    need to be its own entity. Whether a node's per-topic metrics then belong
    to the node or to the topic depends on whether a node can bind more than
    one topic.
  - The map-valued `custom` attribute from
    `entity.extend.identity_attributes`. See the related TODO in
    [Channel metric sets](#channel-metric-sets).

### Telescoping identity on the wire

This RFC places every navigational attribute on the scope. A deployment
could instead choose to send only the identifying attributes and rely on
entity events to join
the navigational attributes back in. That would mean moving toward the
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
[otel-entity-data-model]: https://opentelemetry.io/docs/specs/otel/entities/data-model/
[otel-minimal-identity]: https://opentelemetry.io/docs/specs/otel/entities/data-model/#minimally-sufficient-identity
[otel-repeatable-identity]: https://opentelemetry.io/docs/specs/otel/entities/data-model/#repeatable-identity
[otel-entity-propagation]: https://opentelemetry.io/docs/specs/otel/entities/entity-propagation/
[otel-entity-events]: https://opentelemetry.io/docs/specs/otel/entities/entity-events/
[otel-relationship-placement]: https://opentelemetry.io/docs/specs/otel/entities/entity-events/#relationship-placement
[otel-entity-delete]: https://opentelemetry.io/docs/specs/otel/entities/entity-events/#entity-delete-event
[otel-telescoping]: https://github.com/open-telemetry/opentelemetry-specification/blob/a4e05d9704fc5218d85a4154fb5b143ea3f5340e/specification/resource/README.md#telescoping
[otel-navigation]: https://github.com/open-telemetry/opentelemetry-specification/blob/a4e05d9704fc5218d85a4154fb5b143ea3f5340e/specification/resource/README.md#navigation
[otlp-entity-ref]: https://github.com/open-telemetry/opentelemetry-proto/blob/main/opentelemetry/proto/common/v1/common.proto
[uuid-v7]: https://www.rfc-editor.org/rfc/rfc9562#name-uuid-version-7
