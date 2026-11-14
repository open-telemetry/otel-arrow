---
Proposal Name: entity-reporting
Start Date: 2026-11-12
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC 0012: Entity Reporting

## Summary

The Dataflow Engine (DFE) has a well-defined entity model. The goal of this RFC is
to formalize that hierarchy in OpenTelemetry terms and outline how the DFE will 
communicate it using OpenTelemetry signals.

For the sake of completeness, this RFC consolidates some information already
present about entities in the following documents:

- TODO(agent): Link to relevant ITS/Entity documents
- TODO(agent): Link to relevant OpenTelemetry Entity documentation

## Motivation

Dataflow Engine entities have status information that is valuable for monitoring
and troubleshooting. Entity status is available via the `/api/v1/status` admin
endpoint, but it's in a bespoke format. This data is also not exported via the
Internal Telemetry System (ITS).

We want entity information to be readily available for DFE users like any other
internal telemetry. This includes:

- Status of existing entities and their types
- Lifecycle events including deletion
- Relationships between entities

The natural thing to do is to align ourselves with OpenTelemetry Entities, which
are in the process of being defined. To that end we place the following
restrictions on ourselves:

- DFE Entities are defined according to the abstract notion of OpenTelemetry
entities e.g. type, identifying attributes, and descriptive attributes.
- 

There are also certain problems with the existing entity model in the engine:

- Can't dismbiguate between a pipeline/node that was deleted and recreated :( 
- TODO: consider removing this section

## Guide-level explanation

### Entity modeling

- TODO(agent): Explain entity types
- TODO(agent): Explain that all entities inside the engine are assigned a UUID V7 as
their sole identifying attribute
- TODO(agent): Explain that remaining attributes are descriptive and mention the signal
modeling section

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

| Type | Lifetime | Description |
| --- | --- | --- |
| `engine` | Process | The DFE instance running inside a process. Root of all entities the DFE owns. |
| `pipeline.group` | Group creation to deletion | A named group of pipelines. |
| `pipeline` | Pipeline creation to deletion | A logical pipeline as defined in configuration. Survives rollouts, resizes, and replacements. |
| `pipeline.instance` | One deployment generation on one core | A runtime instance of a logical pipeline. Replaced on every rollout that changes the pipeline. |
| `node` | Same as its pipeline instance | A receiver, processor, or exporter running inside a pipeline instance. |
| `channel` | Same as its pipeline instance | A control or pdata channel connecting nodes in a pipeline instance. |
| `extension` | Same as its hosting scope | An extension, currently hosted by a pipeline instance. |

The distinction between `pipeline` and `pipeline.instance` is new. The existing
entity model conflates the logical pipeline with its runtime instances: the
`pipeline` attribute set carries `core.id` and `deployment.generation`, which
makes it describe an instance, while nothing describes the logical pipeline
that outlives those instances. Separating them gives each entity a single,
well-defined lifetime, which is a prerequisite for assigning it a stable
identity.

#### Identity

Every entity created by the DFE is assigned a [UUID v7][uuid-v7] when it is
created, and that UUID is its **sole identifying attribute**. The attribute is
named `<type>.uid`, for example `engine.uid`, `pipeline.uid`,
`pipeline.instance.uid`, and `node.uid`.

This follows the Kubernetes convention, where `k8s.pod.uid` identifies a pod
and `k8s.pod.name` is a human-friendly, descriptive name. The DFE keeps its
existing human-friendly identifiers (`pipeline.id`, `node.id`, ...) as
navigational attributes (see [Descriptive attributes](#descriptive-attributes))
alongside the new `uid` attributes.

Using a UUID v7 as the identity has useful properties:

- **Delete and recreate is unambiguous.** Today, a pipeline that is deleted and
  recreated with the same configuration produces exactly the same attribute
  values as the original (the deployment generation counter restarts at `0`).
  A fresh UUID distinguishes the two.
- **Creation order is recoverable.** UUID v7 embeds a millisecond timestamp, so
  consumers can order instances of the same logical entity without an extra
  attribute.
- **It is minimally sufficient.** The OpenTelemetry data model asks that the
  identifying attributes be the
  [minimal set sufficient][otel-minimal-identity] to uniquely identify the
  entity. A UUID is unique on its own, without depending on any containing
  context.
- **The DFE is the only observer.** OpenTelemetry asks for
  [repeatable identity][otel-repeatable-identity] across observers. The
  internals of the DFE (pipeline instances, nodes, channels) cannot be
  observed from outside the process, so the DFE is the source of truth for
  these entities and is free to choose its own identifiers.

#### Descriptive attributes

All attributes other than the `uid` are descriptive, but are divided into two
categories:

- **Navigational attributes** are descriptive attributes that are fixed for
  the lifetime of the entity. They let users find the entity and act on it.
  Examples: `pipeline.id`, `pipeline.group.id`, `node.id`, `node.urn`,
  `node.type`, `core.id`, `deployment.generation`.
- **State attributes** are descriptive attributes that may change during the
  lifetime of the entity. Examples: the lifecycle phase of a pipeline instance
  (`Running`, `Draining`, ...), its conditions, or its last heartbeat time.

The term "navigational" comes from the [Navigation][otel-navigation] section of
the OpenTelemetry Resource specification, which distinguishes attributes that
identify an entity from attributes that help users find it in their
infrastructure.

The navigational attributes of an entity, combined with those of the entities
that contain it (see [Entity Containment Chain](#entity-containment-chain)),
make up its [telescoping identity][otel-telescoping].

> [!NOTE]
> Many of the navigational attributes look like identifiers, and some of
> them (for example `pipeline.group.id` + `pipeline.id` + `core.id` +
> `deployment.generation`) were used as a compound identity before
> this RFC. Because the UUID is already minimally sufficient, the
> [minimally sufficient identity][otel-minimal-identity] rule requires every
> one of these attributes to be descriptive rather than identifying. Adding
> them to the identity would add nothing to uniqueness.

Navigational attributes appear on every signal related to the entity, while state
attributes appear only in entity events. The reasons are explained in
[Signal modeling and Entity Refs](#signal-modeling-and-entity-refs) and
[Entity Events](#entity-events).

### Entity Containment Chain

- TODO(agent): explain the boundary between Resources and Entities that enclose the DFE
and are injected via resource/entity detectors and entities that are owned and
operated by the DFE
  - TODO(agent): Need words to describe entities above and below this boundary
- TODO(agent): introduce the concept of entity containment within the DFE and how descriptive

#### Containment

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

The **containment chain** of an entity is the path from the engine to that
entity. For example, the containment chain of a node is
`engine -> pipeline.group -> pipeline -> pipeline.instance -> node`.

Navigational attributes accumulate along the chain. The full description of a
node is its own navigational attributes (`node.id`, `node.urn`,
`node.type`) together with those of its pipeline instance, pipeline, pipeline
group, and engine. The chain is used for navigation and for inferring which entity
a signal belongs to (see
[Signal modeling and Entity Refs](#signal-modeling-and-entity-refs)).

### Signal modeling and Entity Refs

- TODO(agent): Explain how entities above the boundary and the engine itself are unioned
up into the resource
- TODO(agent): Explain how entity refs are only used for resources above the boundary
- TODO(agent): Explain how entities internal to the engine are represented via scope
- TODO(agent): Explain how descriptive and identifying attributes for internal entities
are unioned into each scope
- TODO(agent): Explain how logs and metrics are attached to explicit entities based on
the narrowest containing entity in the scope

The DFE distinguishes between two types of entities:

- **Ambient entities** describe the environment the DFE runs in: the host,
  container, process, service instance, and so on. The DFE does not create or
  own them. They are discovered by resource detectors, provided through
  [entity propagation][otel-entity-propagation] (`OTEL_ENTITIES`), or
  configured explicitly under `engine.telemetry.resource`.
- **Internal entities** are created, owned, and deleted by the DFE: the
  engine, pipeline groups, pipelines, pipeline instances, nodes, channels, and
  extensions. All of the entity types in [Entity types](#entity-types) are
  internal.

Entities are modeled in OpenTelemetry signals as follows.

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

- the entity's identifying attributes (`uid`)
- the entity's navigational attributes.

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
    node.uid: 0193a1d7-...
    node.id: batch
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
entity in its scope's containment chain. A consumer can infer that entity by
looking at the dfe containment chain and referencing the uid attributes present.

| Scope `uid` attributes present | Signal belongs to |
| --- | --- |
| None | `engine` (on the resource) |
| `pipeline.group.uid` | `pipeline.group` |
| ... `pipeline.uid` | `pipeline` |
| ... `pipeline.instance.uid` | `pipeline.instance` |
| ... `node.uid` | `node` |
| ... `channel.uid` | `channel` |
| ... `extension.uid` | `extension` |

This works because containment is a tree: each entity type has one parent
type, so the deepest `uid` present identifies the entity unambiguously.
Whether the scope should also mark the entity type explicitly is an open
question (see [Unresolved questions](#unresolved-questions)).

Signals with an empty scope, such as the existing `engine` metric set, belong
to the `engine`.

#### State attributes are never placed on the scope

State attributes are reported only in entity events and never on
the scope. Scope attributes are part of the identity of a metric stream. If a
value that changes, such as a pipeline instance's phase, were placed on the
scope, every change would start a new metric stream (and, for Prometheus, a
new series with different `otel_scope_*` labels). Navigational
attributes do not have this problem because they never change during the
lifetime of the entity, and the entity's `uid` already scopes the stream.

### Entity Events

- TODO(agent): Explain how entity events are used to communicate internal entities below
the boundary
- TODO(agent): Explain how entities are emitted as entity status events
- TODO(agent): Describe entity relationships
  - TODO(agent): This might need it's own section before this similar to entity containment
  chain or embedded therein. Trying to describe NUMA nodes and cores is probably 
  going to be a doozy here
- TODO(agent): Explain deletion entity events
- TODO(agent): Address periodic emission, probably introduce ITS settings for entity
event emission rates

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

- `entity.type` and `entity.id` (the `uid`),
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
  and in which direction.

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

### api/v1/metrics

- TODO: Explain how we're going to represent status. Should we change the shape
of engine status to be aligned to the entity model more explicitly? This could maybe
be in the "unresolved questions" or "future possibilities" section

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

## Drawbacks

- TODO: Explain that consumers have to interpret events to get internal DFE
entities
- TODO: Explain that there is still redundancy on scope attributes, but it's not
avoidable in existing OpenTelemetry signals
- TODO: Explain that signals cannot be tied to entities above the boundary. This is
not actually a problem in the current engine design, but for opentelemetry more
generally this seems like a hole because what we're doing with scope is not
a standard

## Rationale and alternatives

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

Today, the entity a signal belongs to is inferred from the deepest `uid` on
its scope (see [Attaching signals to entities](#attaching-signals-to-entities)).
Two alternatives make it explicit:

- **Scope level entity refs.** Propose to OpenTelemetry an equivalent of
  `Resource.entity_refs` on `InstrumentationScope`. Every scope would only
  ever reference one entity, because the signals in a scope logically belong
  to a single entity. That makes a general list of refs heavier than needed.
- **An `entity.type` scope attribute.** Add a single attribute naming the type
  of the entity the scope belongs to. This is simpler, but it adds an
  attribute to every scope when the entity can already be inferred from the
  `uid` attributes and the containment chain.

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

### Telescoping identity on the wire

This RFC places every navigational attribute on the scope. A deployment
could instead choose to send only the `uid`s and rely on entity events to join
the navigational attributes back in. That would mean moving toward the
UUID-only end of the [telescoping identity][otel-telescoping] spectrum. Should
this be configurable, and at what granularity?

## Future possibilities

What natural extensions or follow-ups does this enable? This is a place to
record ideas that are out of scope now but worth remembering.

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
