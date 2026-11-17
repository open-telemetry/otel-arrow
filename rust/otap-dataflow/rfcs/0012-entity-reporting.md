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

- DFE: [Entity model][dfe-entity-model],
  [Attributes guide][dfe-attributes-guide],
  [Events guide][dfe-events-guide], and
  [Implementation gaps][dfe-implementation-gaps]
- OpenTelemetry: [Entity data model][otel-entity-data-model],
  [Entity events][otel-entity-events], and
  [Entity propagation][otel-entity-propagation]

Reviewers familiar with the existing DFE entity model may want to focus on the
following sections which highlight important changes:

- [Entity types](#entity-types): the logical `pipeline` is separated from its
  runtime `pipeline.instance`, and a node's output ports become `port`
  entities so that the channel topology can be reported.
- [Example pipeline](#example-pipeline): one small pipeline used throughout
  the RFC and for every measurement.
- [Identity](#identity): internal entities that can be recreated under the
  same name get a UUID v7 `uid`. Nodes, channels, and extensions are
  identified by their pipeline instance's `uid` plus their own id.
- [Descriptive attributes](#descriptive-attributes): existing identifiers become
  navigational attributes, distinct from state attributes.
- [Scope](#scope) and
  [Attaching signals to entities](#attaching-signals-to-entities): which
  attributes go on the scope and how a signal is tied to one entity.
- [Entity-typed scopes](#entity-typed-scopes): every scope belongs to exactly
  one entity, is named after the entity type, and holds all metrics of that
  entity. This is a generic encoding change that moves no attributes.
- [Changes to existing entities and metric sets](#changes-to-existing-entities-and-metric-sets):
  per-entity changes that remove entities which are not real entities (channel
  endpoints, per-decision flows) and move their distinguishing attributes to
  data points.
- [Entity Events](#entity-events): state, delete, and relationship reporting,
  and the new `engine.telemetry.entities` setting.

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
model and it's representation in OTel Signals is still in the process of being
defined.

Under those conditions, this RFC:

- Re-defines the DFEs existing entity model in OpenTelemetry terms
- Proposes changes to the model for closer alignment to entity semantics
- Extends the model to communicate the full list of entity-related information
  above

## Guide-level explanation

### Entity modeling

Every entity reported by the DFE follows the abstract OpenTelemetry
[entity data model][otel-entity-data-model]. Every entity has a **type**, a set
of **identifying attributes**, and a set of **descriptive attributes**.

#### Entity types

TODO: Work on the guidelines for when to define an entity type and when not to
TODO: Decide whether pipeline.group should be pipeline_group and similar for other
      entities

Entity types are major components of the dataflow engine that are present at
runtime. An entity model is an abstraction like any other and care must be
exercised in choosing what to model and what not to.

An new Entity type is generally warranted when:

- There is a need for a logical owner for a group of telemetry signals
- A component has a lifetime that can be created/deleted
- A relationship needs an entity to be properly described

Defining a new entity type should generally not be done when:

- The concept is replaced at runtime i.e. config "connections" are replaced by
  channels
- A new metrics set or event needs attributes

Here are example entitiy types in the DFE:

| Type | Lifetime | Identifying attributes | Description |
| --- | --- | --- | --- |
| `engine` | Process | `engine.uid` | The DFE instance running inside a process. Root of all entities the DFE owns. |
| `pipeline.group` | Group creation to deletion | `pipeline.group.uid` | A named group of pipelines. |
| `pipeline` | Pipeline creation to deletion | `pipeline.uid` | A logical pipeline as defined in configuration. Survives rollouts, resizes, and replacements. |
| `pipeline.instance` | One deployment generation on one core | `pipeline.instance.uid` | A runtime instance of a logical pipeline. Replaced on every rollout that changes the pipeline. |
| `node` | Same as its pipeline instance | `pipeline.instance.uid`, `node.id` | A receiver, processor, or exporter running inside a pipeline instance. |
| `port` | Same as its pipeline instance | `pipeline.instance.uid`, `node.id`, `node.port` | An output port of a node. It exists so that each port's connection to a channel can be reported. It has no metrics of its own: the node records per-port metrics with `node.port` as a data point attribute. |
| `channel` | Same as its pipeline instance | `pipeline.instance.uid`, `channel.id` | A resolved runtime channel (one queue) contained by a pipeline instance. Its endpoints are not entities of their own (see [Channels and their endpoints](#channels-and-their-endpoints)). |

#### Entity Containment Chain

Containment is one type of relationship where one entity is `part_of` another.

The DFE distinguishes between two kinds of containing entities:

- Ambient - Entities that define the environment the DFE runs in: the host,
  container, process, service instance, and so on. The DFE does not create or
  own them. They are discovered by resource detectors, provided through
  [entity propagation][otel-entity-propagation] (`OTEL_ENTITIES`), or
  configured explicitly under `engine.telemetry.resource`.
- Internal - Entities that are created, owned, and deleted by the DFE: the
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
                    |     +-- port
                    +-- channel
                    +-- extension
```

Note: In addition to being contained by a pipeline instance, channels have
additional relationships to the other entities (see
[Entity relationships](#entity-relationships)).

The **containment chain** of an entity is the path from the engine to that
entity. For example, the containment chain of a node is
`engine -> pipeline.group -> pipeline -> pipeline.instance -> node`.

#### Identity Scope

Entities IDs consiste of a set of attributes and a scope. The scope is either
`local` or `global`. `local` IDs are unique only within their containing
entity whereas `global` IDs uniquely identify an entity within the system
being observed.

A `global` ID for an entity with a `local` one can be constructed by continuing
to union its ID with the ID of next entity in its containment chain until an
entity with a `global` ID is reached.

#### Identifying Attributes

Compound navigational attributes form the IDs of all DFE entities today. The
term "navigational" comes from the [Navigation][otel-navigation] section of
the OpenTelemetry Resource specification, and refer to attributes that help
users find an entity in their infrastructure. For example, (`node.name`,
`node.port`) are navigational attributes that point to a particular node in a
pipeline and then a port in that node.

Some entities, however, are not uniquely described by their navigational
attributes. This happens when the lifetime of an entity is shorter than the
lifetime of the next entity in its containment chain.

For these entities, we assign an `<entity>.uid` (a [UUID v7][uuid-v7]) when the
entity is created. This allows us to track different incarnations of entities 
with the same navigational attributes across time. A UUID v7 is also 
[minimally sufficient][otel-minimal-identity] and entities with one need no
other identifying attributes

For example:

- `pipeline.group` and `pipeline`. A group or pipeline can be deleted and
  recreated with the same attributes.
- `pipeline.instance`. A rollback relaunches the previous deployment
  generation on a core, so the same `core.id` and `deployment.generation` can
  belong to two different instances of one pipeline.

However, we still keep the existing navigational identifiers of these entities
as [Descriptive attributes](#descriptive-attributes)).

This gives identities useful properties:

- Delete and recreate is unambiguous - Every entity that can be recreated
  under the same name gets a fresh `uid`, and every other entity inherits one
  from its container.
- Creation order is recoverable - UUID v7 embeds a millisecond timestamp, so
  consumers can order instances of the same logical entity without an extra
  attribute.
- It is minimally sufficient - The OpenTelemetry data model asks that the
  identifying attributes be the
  [minimal set sufficient][otel-minimal-identity] to uniquely identify the
  entity. A `uid` is minimal for entities whose natural key can be reused. For
  nodes, ports, channels, and extensions, a `uid` of their own would add nothing that
  the instance `uid` and their local id do not already provide. The data model
  accepts identities that are only unique in context.
- It matches how users query - Composite identities use the ids that users
  write in configuration, such as `node.id`, and those stay stable across
  deployment generations.

#### Descriptive attributes

All attributes other than the identifying attributes are descriptive and belong
to one of two classes:

- Navigational attributes - Attributes that never change during the lifetime of
  the entity and are helpful in finding the entity. These are used for entities
  assigned a UID as their sole identifier. Examples: `pipeline.group.id`,
  `pipeline.id`, `core.id`, `deployment.generation`, `node.urn`, `node.type`.
- State attributes - Attributes indicating status or state that may change during
  the lifetime of the entity. Examples: the lifecycle phase of a pipeline instance
  (`Running`, `Draining`, ...), its conditions, or its last heartbeat time.

The identifying and navigational attributes of an entity, combined with those
of the entities that contain it (see [Entity Containment Chain](#entity-containment-chain)),
make up its [telescoping identity][otel-telescoping].

### Example pipeline

The rest of this RFC uses one small pipeline as a running example. It is also
the workload behind every measurement (see
[Measurement method](#measurement-method)). Two receivers feed a signal type
router, which sends logs and metrics to separate exporters:

```yaml
groups:
  default:
    pipelines:
      main:
        nodes:
          r1: { type: receiver:traffic_generator, ... }
          r2: { type: receiver:traffic_generator, ... }
          p1: { type: processor:type_router, outputs: ["logs", "metrics"] }
          e1: { type: exporter:noop }
          e2: { type: exporter:noop }
        connections:
          - { from: r1, to: p1 }
          - { from: r2, to: p1 }
          - { from: p1["logs"], to: e1 }
          - { from: p1["metrics"], to: e2 }
```

The configuration has four connections, but the engine builds three pdata
channels. Connections with the same destinations are merged into one channel
(see [Channels and their endpoints](#channels-and-their-endpoints)), so
`r1 -> p1` and `r2 -> p1` share a single queue. Each node also gets a control
channel. On one core, the pipeline produces these entities:

| Type | Entity | Local id |
| --- | --- | --- |
| `pipeline.group` | `default` | `pipeline.group.id: default` |
| `pipeline` | `main` | `pipeline.id: main` |
| `pipeline.instance` | `main` on core 0, generation 0 | `core.id: 0`, `deployment.generation: 0` |
| `node` | `r1`, `r2`, `p1`, `e1`, `e2` | `node.id` |
| `port` | `r1/default`, `r2/default`, `p1/logs`, `p1/metrics` | `node.id`, `node.port` |
| `channel` | A: pdata, `r1`, `r2` to `p1` | `channel.id: hyperedge:d59f2e2fafb41d70` |
| `channel` | B: pdata, `p1/logs` to `e1` | `channel.id: hyperedge:b33de11f167e5236` |
| `channel` | C: pdata, `p1/metrics` to `e2` | `channel.id: hyperedge:b55632c4f3d0511d` |
| `channel` | Control channels, one per node | `channel.id: node:{r1,r2,p1,e1,e2}:control` |

Every entity below the pipeline group also carries the identity of its
containing entities, as described in [Identity](#identity). The receivers
and the router use the default port unless the configuration names one, and
the exporters have no output ports.

The entities are connected like this:

```text
r1 --(r1/default)--+
                   +--> [A] --> p1 --(p1/logs)----> [B] --> e1
r2 --(r2/default)--+            |
                                +----(p1/metrics)-> [C] --> e2

pipeline instance --> [node:<id>:control] --> <id>   (for each node)
```

The relationships between them (see
[Entity relationships](#entity-relationships)) are:

| Source | Relationship | Target |
| --- | --- | --- |
| `default` (group) | `part_of` | engine |
| `main` (pipeline) | `part_of` | `default` (group) |
| `main` on core 0 | `part_of` | `main` (pipeline) |
| `r1`, `r2`, `p1`, `e1`, `e2` | `part_of` | `main` on core 0 |
| A, B, C, and the control channels | `part_of` | `main` on core 0 |
| `r1/default`, `r2/default` | `part_of` | `r1`, `r2` |
| `p1/logs`, `p1/metrics` | `part_of` | `p1` |
| `r1/default`, `r2/default` | `connected_to` | A |
| A | `connected_to` | `p1` |
| `p1/logs` | `connected_to` | B |
| B | `connected_to` | `e1` |
| `p1/metrics` | `connected_to` | C |
| C | `connected_to` | `e2` |
| `node:<id>:control` | `connected_to` | `<id>` |

### Signal Modeling and Entity Refs

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

For metrics, every scope belongs to exactly one entity instance:

- The scope `name` is the entity type, for example `node` or `channel`.
- The scope attributes are the attributes of that entity's containment chain,
  as described above, and nothing else.
- All metric sets reported for the same entity instance share that one scope.
- Each metric name is qualified with its metric set name, for example
  `node.output.messages`, because metrics from different metric sets now share
  a scope.

See [Entity-typed scopes](#entity-typed-scopes) for the encoding and its
measured impact.

For example, a metric from any of a node's metric sets carries:

```text
InstrumentationScope:
  name: node
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
attributes, under a scope named `pipeline.instance`.

Logs keep the tracing target as their scope `name`, with the same entity
attributes as metrics.

- TODO: Decide whether log scopes should also be named after the entity type.
  The tracing target identifies the emitting code, which has no equivalent on
  the metrics side once metric names are qualified.

#### Attaching signals to entities

Each log record or metric belongs to exactly one entity. For metrics, the
scope `name` gives the entity type directly. The identifying attributes on
the scope select the instance. For logs, and for consumers that ignore the
scope name, the entity can also be inferred from the DFE containment chain
and the identifying attributes present.

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
the signal belongs to.

Signals whose scope has no attributes, such as the existing `engine` metric
set, belong to the `engine`. For metrics their scope is named `engine`.

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
destination. A control channel is likewise one queue, which delivers to one
node or extension.

Channel metrics belong to the `channel`. The scope of a channel metric carries
the channel's containment chain (`engine -> pipeline.group -> pipeline ->
pipeline.instance -> channel`) and nothing else. In particular, it no longer
carries the attributes of the node or extension at either end of the channel.

The endpoints of a channel are not entities of their own. They are
populations within the channel, identified by data point attributes that
refer to other entities:

- A sending endpoint is a port. `channel.sender` metrics carry `node.id` and
  `node.port`, which together with the scope's `pipeline.instance.uid` are
  the port's identifying attributes.
- A receiving endpoint is a node. `channel.receiver` metrics carry `node.id`.

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
[Channel](#channel).

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
| `port` | `part_of` | `node` |
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

**Connections** describe how data moves between nodes, in the direction it
flows. They are reported as `connected_to` relationships:

| Source | Relationship | Target | Meaning |
| --- | --- | --- | --- |
| `port` | `connected_to` | `channel` | The port sends into the channel. Every port has exactly one. |
| `channel` | `connected_to` | `node` | The channel delivers to the node. A pdata channel has one per destination node; a control channel has one, to the node it controls. |
| `channel` | `connected_to` | `extension` | The control channel delivers to the extension. |

Channels report the actual runtime channels after connections are merged, so
these relationships describe the topology the engine runs, not the
configuration as written. See [Example pipeline](#example-pipeline) for a
complete set.

A control channel has no sending port. It is fed by the pipeline instance's
runtime controller, and the pipeline instance already contains it, so no
separate relationship is needed for its sender.

All of these entities share the lifetime of their pipeline instance, so the
placement rule does not choose a side. The DFE reports `port -> channel` on
the port and `channel -> node` on the channel, so that each relationship is
reported by the entity it starts from.

- TODO: Confirm the relationship type name. `connected_to` is a placeholder
  until it is checked against OpenTelemetry semantic conventions for entity
  relationships.

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

### Entity-typed scopes

This is a generic change to how metrics are encoded. It applies to every
metric set and moves no attribute between the scope and the data points.

**Today.** The OTLP encoder (`crates/telemetry/src/metrics/otlp.rs`) emits
one scope per registered metric set:

- The scope `name` is the metric set name, for example `node.output`.
- The scope attributes are the attributes of the entity the metric set is
  registered against.
- Each metric name is the bare field name, for example `messages`.
- Scopes are keyed by metric set descriptor and entity, so an entity with N
  metric sets is reported in N scopes that repeat the same attributes. Metric
  sets that share a name do not merge either: a node's `node.output`
  messages, items, and size metric sets produce three scopes with identical
  names and attributes.

This shape was inherited rather than designed. Internal metrics used to be
exported through the OpenTelemetry SDK with one meter per metric set, named
after the set. Entity attributes were moved onto that meter's scope later,
and when the SDK was removed the encoder kept the shape. The only information
the per-set scope carries beyond the entity attributes is the metric set
name, which namespaces the bare field names.

**Change.**

- An entity attribute set declares its entity type:
  `#[attribute_set(scope, name = "node.attrs", entity = "node")]`. The type is
  exposed through `AttributeSetHandler::entity_type()` and is not part of the
  entity's identity in the entity registry.
- The encoder names each scope after the entity type and keys scopes by
  entity instance, so every metric set registered against one entity is
  reported in one scope.
- Each metric name is qualified with its metric set name, for example
  `node.output.messages`. A metric renamed by a view keeps the view's name.
- The following attribute sets are typed: `engine`, `pipeline.instance`
  (`PipelineAttributeSet`), `node` (`NodeAttributeSet`,
  `NodeWithCustomAttributeSet`), `channel`, `extension`
  (`ExtensionAttributeSet`, the controller monitor), and `flow`.

Some existing entity attribute sets do not describe a single entity, so they
cannot satisfy "one scope per entity" on their own. The channel endpoint sets
and the per-decision flow set are replaced in
[Changes to existing entities and metric sets](#changes-to-existing-entities-and-metric-sets).
The topic and syslog protocol sets stay untyped for now and keep one scope
per metric set (see the TODO in [Node](#node)).

**Measured impact.** See [Measurement method](#measurement-method) for the
workload. In this measurement the channel endpoint sets were typed
`channel`, so each endpoint's metric sets merge, but the endpoints of a
channel remain separate scopes until the [Channel](#channel) change.

| Raw bytes | Gzip bytes | Scopes | Data points |
| --- | --- | --- | --- |
| 58,212 -> 42,921 (-26.3%) | 8,409 -> 8,056 (-4.2%) | 61 -> 19 | 164 -> 164 |

The series and metric names are unchanged. The saving comes entirely from
no longer repeating the entity attributes once per metric set: the scope
count drops by about 70%. Gzip already removes most of that
repetition, so the compressed saving is much smaller than the raw one.

**Compatibility.**

- Every OTLP scope name and metric name changes, which changes the identity
  of every exported metric stream. For stable signals the
  [stability guide][dfe-stability-guide] treats this as a breaking change
  that requires a migration plan.
- Metric views keep matching on the metric set name (`scope_name`) and field
  name (`instrument_name`), so existing view configuration is unaffected.
- The admin endpoint renders Prometheus and JSON output from the metric set
  descriptors directly, so it keeps `otel_scope_name` set to the metric set
  name and is unaffected.

- TODO: Decide whether the admin Prometheus output should follow the same
  scope naming, so that both exports agree.

### Measurement method

The measurements in this RFC come from an ignored validation test that runs
the engine with the [Example pipeline](#example-pipeline) and captures every
internal metrics export request. The test lives on an experiment branch, with
one commit per change in this RFC.

- **Engine settings.** One core, a 1 second reporting interval, pipeline,
  Tokio, and detailed runtime metrics, and the internal telemetry receiver
  exporting metrics to a capture exporter. Each run lasts 12 seconds, and
  each step is run 5 times.
- **Workload.** `r1` and `r2` are traffic generators that each send 3000
  signals at 500 per second, half logs and half metrics, so both of `p1`'s
  ports carry traffic. They stop after 6 seconds, so the second half of each
  run shows idle telemetry. `e1` and `e2` are noop exporters.
- **Coverage.** The example pipeline exercises fan-in into one channel and a
  node with several output ports. It has no flows and no channel with several
  destinations, so the [Flow](#flow) change and the single report of queue
  state for shared queues are justified in text but not measured.
- **Sizes.** The tables show one complete export: the export request in
  which every series of the run is reported. Raw is its encoded protobuf
  size, and gzip compresses it at level 6. Each value is the median over the
  runs. Totals over a whole run move in the same direction, but they also
  depend on how many export windows fall within the run. The run-to-run
  spread of a complete export is up to about 3% of its raw size and 2% of
  its gzip size, so smaller differences are noise.
- **Parity.** Each series is reduced to a canonical key: the qualified metric
  name plus the union of its scope and data point attributes. Attributes
  that only change placement are normalized: for example, `channel.*`
  attributes are dropped from node metrics because each output port maps to
  exactly one channel. Each step must keep the same canonical series and
  metric names. Item counters must match exactly. Every other value must
  agree within the run-to-run spread plus 5% (10% for timing values). Every
  step met these criteria: 166 series, 83 metric names, and exact item
  counts throughout.

### Changes to existing entities and metric sets

Each change below is independent of [Entity-typed scopes](#entity-typed-scopes)
and of the others, and has its own justification. The changes were measured
one at a time, in this order: entity-typed scopes, `node.input` and
`node.output`, channel, flow, and `node.completion`. Each table is relative to
the previous step, so the steps add up to the total in
[Combined impact](#combined-impact).

- TODO: Decide whether to list here every registered entity that is not a
  single real entity, so the scope of this section is visibly complete:
  node channel endpoints (`NodeChannelAttributeSet`), extension channel
  endpoints (`ExtensionChannelAttributeSet`), per-decision flows
  (`FlowAttributeSet` with `flow.node.decision`), node plus topic
  (`NodeWithTopicAttributeSet`), node plus syslog protocol
  (`NodeWithProtocolAttributeSet`), and possibly extension variants (see
  [Redefining the entity hierarchy](#redefining-the-entity-hierarchy)).

#### Node

Today, a node's own input, output, and completion metrics are not registered
against the node. They are registered against **channel endpoint** entities
(`NodeChannelAttributeSet` in `crates/engine/src/attributes.rs`), which
compose the full node attributes, the channel attributes, and `node.port`.
There is one such entity per node and channel:

- **Sending endpoint.** One per output port, with `node.port` set to the
  port name (`default` unless the configuration names the port).
- **Receiving endpoint.** One for the node's input, with `node.port` set to
  the literal `input`. Nodes have a single input and no input port names, so
  `input` is a label rather than a port.

So node metrics carry channel identity they do not need, and the node's
identity is repeated once per port. These endpoints are not entities in the
sense of this RFC: they are two views of one channel. They are removed here
and in [Channel](#channel).

**`node.input` and `node.output` move to the node.**

- `node.input.*` (messages, items, size) is registered against the node with
  no extra dimension, because a node has one input.
- `node.output.*` is registered against the node once per output port, with
  `node.port` as a registration attribute. Recording is unchanged: the
  engine already selects the per-port metric set by the output port index
  carried on the message.
- This is lossless. Each output port feeds exactly one channel and no two
  ports of a node share one, so `node.port` identifies the channel a node
  sent on.

| Raw bytes | Gzip bytes | Scopes | Data points |
| --- | --- | --- | --- |
| 42,921 -> 43,441 (+1.2%) | 8,056 -> 8,075 (+0.2%) | 19 -> 19 | 164 -> 164 |

This change is about attribution, not size. `node.port` is now repeated on
every `node.output` data point, and the node's metrics were already merged
into one scope per endpoint by [Entity-typed scopes](#entity-typed-scopes),
so moving them saves no scopes. The size difference is within the run-to-run
spread.

**`node.completion` uses the real output port.**

`node.completion.duration` measures the time from when a node takes
responsibility for a message until the terminal ack or nack for that message
is routed back to it. Today:

- Receivers register one completion metric set per output port, against
  their sending endpoints.
- Processors and exporters register one completion metric set against their
  receiving endpoint, so it carries `node.port=input`.
- The engine records into the input-side metric set whenever it exists. For
  a processor, the output port the message left on is carried on the message
  but discarded, so all ports of a multi-output processor such as a router
  collapse into one series labeled `input`.

Under this RFC, every node with outputs registers one completion metric set
per output port, with `node.port` as a registration attribute, and records by
the message's output port. A node without outputs (an exporter) registers one
completion metric set without a port.

| Raw bytes | Gzip bytes | Scopes | Data points |
| --- | --- | --- | --- |
| 41,100 -> 40,625 (-1.2%) | 8,030 -> 8,015 (-0.2%) | 15 -> 15 | 164 -> 164 |

The series count does not change. `p1` sends each signal on its own port,
so its per-signal `input` series become per-port series one to one, and the
exporters' series simply lose `node.port=input`. A processor that sends one
signal on several ports would gain one series per extra port. The size
difference is within the run-to-run spread.

- TODO: Topic receivers and exporters register their metrics against a node
  plus `topic` entity (`NodeWithTopicAttributeSet`), and the syslog receiver
  registers its shared receiver metrics against a node plus `protocol`
  entity (`NodeWithProtocolAttributeSet`). Both are fixed for the lifetime of
  the node: a topic node binds exactly one topic, and a syslog receiver
  listens on exactly one protocol. Decide whether `topic` and `protocol`
  become registration attributes, navigational node attributes, or are
  dropped from metrics. Until then these sets stay untyped and keep one scope
  per metric set.

#### Channel

Today, channel metrics are registered per endpoint. Each endpoint registers a
channel endpoint entity whose attribute set composes the full attributes of
the node or extension at that end (`NodeChannelAttributeSet`,
`NodeWithCustomChannelAttributeSet`, `ExtensionChannelAttributeSet` in
`crates/engine/src/attributes.rs`). A channel with several senders and
receivers is therefore reported as several entities. Under this RFC the
channel entity carries only its own attributes, and the endpoint becomes a
data point dimension (see
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

**Measured impact.**

| Raw bytes | Gzip bytes | Scopes | Data points |
| --- | --- | --- | --- |
| 43,441 -> 40,520 (-6.7%) | 8,075 -> 8,006 (-0.9%) | 19 -> 15 | 164 -> 164 |

Each channel now has one scope instead of one per endpoint. In the example
pipeline, channel A had three endpoint scopes (`r1`, `r2`, and `p1`), and
channels B and C had two each. The series are unchanged. The example has no
channel with several destinations, so the single report of queue state is
not exercised: with two competing receivers, today's encoding reports the
queue depth and capacity twice.

#### Flow

Today, `flow.dropped.items` is registered against a separate flow entity
for each decision node: `FlowAttributeSet` (`crates/engine/src/flow_metrics.rs`)
includes `flow.node.decision`. Every other flow metric set is registered
against the same attribute set with `flow.node.decision` set to an empty
string. So one flow is reported as one entity per decision node plus one
with an empty placeholder.

Under this RFC `FlowAttributeSet` drops `flow.node.decision` and is typed
`flow`, and `flow.dropped.items` carries `flow.node.decision` as a
registration attribute. Summing across decision nodes stays meaningful:
drops at different decision nodes are disjoint, so the sum equals the flow's
total removed items.

| Raw bytes | Gzip bytes | Scopes | Data points |
| --- | --- | --- | --- |
| 40,520 -> 41,100 (+1.4%) | 8,006 -> 8,030 (+0.3%) | 15 -> 15 | 164 -> 164 |

Each flow now has one scope instead of one per decision node plus one for the
rest of its metrics. The example pipeline has no flows, so this change is
not exercised, and the measured difference is run-to-run noise.

#### Extension

Extension attribute sets are typed `extension`, including the controller
monitor (`ControllerMonitorAttributes` in
`crates/controller/src/controller_monitor.rs`), which only carries
`extension.id`. No other extension metrics change. The placement of
`scope.kind` and `extension.variant` is still open (see
[Redefining the entity hierarchy](#redefining-the-entity-hierarchy)).
Neither workload runs an extension that reports metrics, so this change is
not measured.

### Combined impact

All changes together, against today's encoding:

| Raw bytes | Gzip bytes | Scopes | Data points |
| --- | --- | --- | --- |
| 58,212 -> 40,625 (-30.2%) | 8,409 -> 8,015 (-4.7%) | 61 -> 15 | 164 -> 164 |

Most of the saving comes from [Entity-typed scopes](#entity-typed-scopes).
Of the per-entity changes, only the channel change saves a noticeable
amount, because it collapses all endpoints of a channel into one scope. The
others remove scopes but add data point attributes, and come out close to
size neutral.

**Compatibility.** Each change above changes the identity of the affected
metric streams: attributes move from the scope to data points, or, for
`node.completion`, one series per processor becomes one per output port.
They are breaking changes for stable signals in the same way as
[Entity-typed scopes](#entity-typed-scopes), which already renames every
metric, so they should ship together with it.

### Example: p1 logs to e1

This section follows one path through the
[Example pipeline](#example-pipeline): logs that `p1` sends on its `logs`
port, through channel B, to `e1`. The metrics involved and where each change
puts them:

- **`p1`, the sending node.** `node.output.*` (messages, items, size) and
  `node.completion.duration` are reported on `p1`'s `node` scope with
  `node.port: logs` on each data point. `p1`'s own metric set
  (`processor.type_router`) and its `node.input.*` share that scope.
- **Channel B.** `channel.sender.messages` carries the sending port
  (`node.id: p1`, `node.port: logs`), and `channel.receiver.messages` carries
  the receiving node (`node.id: e1`). The queue depth and capacity carry no
  endpoint attributes. All four share channel B's `channel` scope.
- **`e1`, the receiving node.** `node.input.*` and `node.completion.duration`
  are reported on `e1`'s `node` scope. `e1` is an exporter, so its completion
  has no `node.port`.

The export, as captured from the experiment. Values are omitted, the shared
`Resource` is omitted, and the data points for `p1`'s `metrics` port are left
out:

```text
ScopeMetrics:
  Scope:
    name: node
    attributes:
      pipeline.group.id: default
      pipeline.id: main
      core.id: 0
      numa.node.id: 0
      deployment.generation: 0
      node.id: p1
      node.urn: urn:otel:processor:type_router
      node.type: processor
  Metrics:
    processor.type_router.signals.decision    {signal: logs, outcome: success, reason: named_route}
    node.input.messages                       {signal: logs, outcome: success}
    node.input.items                          {signal: logs, outcome: success}
    node.input.size                           {signal: logs, outcome: success}
    node.output.messages                      {node.port: logs, signal: logs, outcome: success}
    node.output.items                         {node.port: logs, signal: logs, outcome: success}
    node.output.size                          {node.port: logs, signal: logs, outcome: success}
    node.completion.duration                  {node.port: logs, signal: logs, outcome: success}
ScopeMetrics:
  Scope:
    name: channel
    attributes:
      pipeline.group.id: default
      pipeline.id: main
      core.id: 0
      numa.node.id: 0
      deployment.generation: 0
      channel.id: hyperedge:b33de11f167e5236
      channel.kind: pdata
      channel.mode: local
      channel.type: mpsc
      channel.impl: internal
  Metrics:
    channel.sender.messages                   {node.id: p1, node.port: logs, signal: logs, outcome: success}
    channel.receiver.messages                 {node.id: e1, signal: logs}
    channel.receiver.queue.depth              {}
    channel.receiver.capacity                 {}
ScopeMetrics:
  Scope:
    name: node
    attributes:
      pipeline.group.id: default
      pipeline.id: main
      core.id: 0
      numa.node.id: 0
      deployment.generation: 0
      node.id: e1
      node.urn: urn:otel:exporter:noop
      node.type: exporter
  Metrics:
    node.input.messages                       {signal: logs, outcome: success}
    node.completion.duration                  {signal: logs, outcome: success}
```

Each scope belongs to exactly one entity, and every data point attribute
that refers to another entity (`node.port` on `p1`, `node.id` and
`node.port` on channel B) completes that entity's identifying attributes
with the `pipeline.instance.uid` on the scope.

Today the same data points are spread over many more scopes. Each metric set
is its own scope, named after the metric set, so `p1`'s `logs` port alone
produces three `node.output` scopes (messages, items, size). Each of them
repeats `p1`'s node attributes together with channel B's attributes and
`node.port: logs`. `p1`'s completion is reported on a scope for channel A
labeled `node.port: input`, and `e1`'s input metrics on scopes for channel B
that also carry `node.port: input`.

- TODO: The example shows the experiment's scope attributes. Under this RFC
  the scope also carries the `uid` attributes from [Identity](#identity),
  which the experiment does not implement.

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
- Every metric name and scope name changes with
  [Entity-typed scopes](#entity-typed-scopes). This is a breaking change for
  existing dashboards and alerts on every internal metric.
- Node, channel, and flow metrics also change stream identity, because
  attributes move from the scope to data point attributes. See
  [Changes to existing entities and metric sets](#changes-to-existing-entities-and-metric-sets).

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

### Marking the entity type of a scope

The scope `name` names the entity type (see
[Entity-typed scopes](#entity-typed-scopes)). Two other ways of making the
entity explicit were considered:

- **Scope level entity refs.** Propose to OpenTelemetry an equivalent of
  `Resource.entity_refs` on `InstrumentationScope`. Every scope only ever
  references one entity, because the signals in a scope belong to a single
  entity, so a general list of refs is heavier than needed. This remains an
  option to propose upstream, and the scope name would map onto it directly.
- **An `entity.type` scope attribute.** This adds an attribute to every scope,
  while the scope name is free: today it repeats the metric set name, which
  qualified metric names already carry.

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
  - `extension.variant` (`local` or `shared`). A single extension id may
    register both variants (`ExtensionVariant` in
    `crates/engine/src/extension/wrapper.rs`), which produces two live
    entities that differ only in this attribute. So it cannot be
    navigational: either each variant is its own entity and the attribute is
    identifying, or it is a distinguishing attribute that belongs on data
    points.
  - `topic` on topic receiver and exporter metric sets
    (`NodeWithTopicAttributeSet`) and `protocol` on the syslog receiver
    (`NodeWithProtocolAttributeSet`). See the TODO in
    [Node](#node). Topics are declared at the engine or group level and span
    pipelines, so `topic` may also need to be its own entity.
  - The map-valued `custom` attribute from
    `entity.extend.identity_attributes`. See the related TODO in
    [Channel](#channel).

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
