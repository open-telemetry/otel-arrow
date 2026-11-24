---
Proposal Name: context-lookup-keys-and-composite-formation
Start Date: 2026-11-24
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#4541
---

# RFC NNNN: Context Lookup Keys and Composite Formation

## Status and scope

This is a proposal extending [RFC 0004: Pdata
Context](./0004-pdata-context.md) and [RFC 0006: Context Capture,
Transfer, and Propagation](./0006-transport-header-propagation.md):

1. RFC 0004 predicate bindings "will support fast table lookup and
   equality checking", and RFC 0006 defines conditional topic
   dispatch, but neither defines how conditions compile, what they
   cost, or how they apply to composites.
2. RFC 0004 leaves it to projector bindings to "dictate which context
   entries survive into the output context", but does not say which
   composites edits and merges form.

This is the design for step 6 of
[#4541](https://github.com/open-telemetry/otel-arrow/issues/4541),
whether that step adds a new `pdata-context` crate or evolves the
existing compiler. Topic dispatch reads the publisher's context, so it
lands with step 6; topic transfer mappings, export, and import remain
step 7.

## User stories

See [RFC 0004](./0004-pdata-context.md#user-stories).

For example, User A routes thousands of tenants by the authorized
`customer_id`, and the cost of routing one publication does not depend
on the number of tenants.

For example, User C routes by customer and workspace header together,
through one composite entry.

For example, User G batches before forwarding, and expects a request
to be routed the same way whether it was merged or passed through.

For example, User P partitions by namespace and routes each partition
by a composite that includes the partition header.

### Example: routing tenants by a composite key

**Scope of this example.** The YAML is a group-level excerpt.
`dispatch` and `subscription.port` use the syntax proposed in RFC
0006; the `composite` condition is proposed here. A complete
deployment also supplies an authenticated `receiver:otlp` named
`authenticated_receiver`, an exporter in each delivery pipeline, and
`shared_delivery` and `unattributed_delivery` pipelines that differ
from `dedicated_delivery` only in their port.

```yaml
policies:
  authorized_identity:
    - claim: sub
      store_as: customer_id
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-workspace]
  context:
    entries:
      tenant:                            # Composite name
        - type: authorized_identity      # Authorization claim
          name: customer_id
          store_as: customer             # Member name
        - type: transport_header         # Transport header
          name: x-workspace
          store_as: workspace            # Member name

topics:
  handoff:
    backend: in_memory
    dispatch:
      conditions:
        - condition:
            type: composite
            name: tenant
            value: {customer: acme, workspace: blue}
          port: dedicated
        - condition: {type: composite, name: tenant}
          port: shared
      otherwise: unattributed

pipelines:
  ingress:
    nodes:
      publish:
        type: exporter:topic
        config:
          topic: handoff
    connections:
      - {from: authenticated_receiver, to: publish}

  dedicated_delivery:
    nodes:
      receive:
        type: receiver:topic
        config:
          topic: handoff
          subscription: {port: dedicated, mode: balanced, group: delivery}
```

A request from `acme` with `x-workspace: blue` goes to `dedicated`.
With workspace `green` it matches no value row, but `tenant` is
present, so it goes to `shared`. Without the header, `tenant` is
absent and the request goes to `unattributed`, even though its
customer is known: a composite is present only when all its members
are. Because each lookup reads one entry, routing such requests by
customer alone takes a second hop, in which `unattributed_delivery`
publishes to a topic that dispatches on `customer_id`.

## Detailed design

The examples in this document serve as acceptance tests.

### Baseline

The current state this document reflects is defined by how each kind
of node forms an output context:

- Receivers and the traffic generator materialize the composites that
  some node in the pipeline selects. A composite is a snapshot: later
  changes to its sources in the same pipeline do not update it.
- The partition processor appends the partition value as a transport
  header and keeps the input's composites as they are. Partitions of
  one input share its composites, including random values.
- The batch and temporal reaggregation processors merge contexts,
  keeping only the peer address when every input agrees. Inputs they
  pass through, such as in-range batches, keep their whole context.
- Topics pass the whole context through, dropping only Ack/Nack
  routing state. The durable buffer processor drops the context.
- Nodes read context values directly, for example the Kafka exporter
  takes its topic from a transport header, but no node evaluates a
  condition against a configured table.

## Context lookup keys

A **lookup key** is an RFC 0004 [predicate
binding](./0004-pdata-context.md#pdata-context-predicates). It projects
one context entry, primitive or composite, and compares it with an
ordered list of rows, returning `Matched { action }`, `Missing`, or
`NoMatch`. For topic dispatch, the action is a port.

### Rows

A **value row** gives the value the entry must hold. A **presence
row** gives no value and matches any present entry. In the example
above, the first condition is a value row and the second a presence
row.

- Every row names the same entry; lookups over several fields use a
  composite.
- A value row matches when any of the entry's values equals it, as
  exact bytes, as with `transport_header_match`.
- When several rows match, the lowest row wins, regardless of value
  order.
- At most one presence row is allowed, and it is last.
- Duplicate value rows are a configuration error.
- A missing entry matches no row.

### Composite keys

A value row for a composite gives a map with a value for every member
except constants, and matches when each of those members holds that
value. A composite used as a key may not contain `randomness` members.

A composite key is probed only when each member has exactly one
value. Otherwise only the presence row can match, and the engine
counts the event in a metric. A lookup therefore costs one probe for a
composite and one per value for a primitive entry, independent of the
number of rows.

### Hashing

The context compiler defines the hash and hashes every row once per
binding, at startup. The engine hashes the message's projected values
when the key is evaluated, in member order, each with its kind (text
or binary) and length; nodes never see hashes. The topic exporter is
the only reader, so a key stored with the context would be computed
once and read once, and would have to be recomputed after every edit.

The hasher is seeded once per process, and lookup hashes are never
exported, transferred, or persisted. A hit is confirmed by comparing
the exact values, so a collision cannot select a row.

### Topic dispatch

Topic dispatch from [RFC
0006](./0006-transport-header-propagation.md#conditional-topic-dispatch)
is a lookup key whose rows select ports:

- `Matched` selects the row's port; `Missing` and `NoMatch` select
  `otherwise`. Without `otherwise`, the exporter refuses the
  publication with a permanent Nack.
- Every port, including `otherwise`, must have a subscriber, and every
  subscriber of a topic with `dispatch` names a port. Balanced groups
  are scoped per port, so subscribers of different ports may share a
  group name.

The topic exporter evaluates the key on the publisher's context before
it publishes. Composite conditions extend RFC 0006, whose derived
transfer has no composite fields: each publishing pipeline compiles
them against its own composite declaration, and a pipeline without one
is rejected at startup. Publishers whose `tenant` members read
different sources therefore give the same row different meanings.

### Lookup declarations

The engine supplies a lookup declaration for a topic exporter whose
topic has `dispatch`, as it supplies header propagation policy for
exporters today. The compiler uses it to validate the rows, plan the
composite, and check [reads after a merge](#reads-after-a-merge):

```rust
pub enum ContextDeclaration {
    // ... existing variants ...
    /// Declares a lookup key the node evaluates.
    Lookup {
        entry: ContextLookupEntry,
        /// Value rows in configuration order, one value per
        /// non-constant member.
        rows: Box<[Box<[(ValueKind, Box<[u8]>)]>]>,
        /// Whether a presence row follows the value rows.
        presence: bool,
    },
}

pub enum ContextLookupEntry {
    Primitive { domain: ContextDomain, name: ContextEntryName },
    Composite { name: ContextEntryName },
}
```

The node receives a compiled `ContextLookup` whose
`evaluate(&Metadata)` returns the RFC 0004 `ContextDecision`.

## Composite formation

<!-- markdownlint-disable MD013 -->
| Formation | RFC 0004 binding | Example nodes | Composites formed |
| --- | --- | --- | --- |
| Produce | Context arrival source | Receivers, traffic generator | Every selected composite |
| Edit | 1:1 or 1:N projector source | Partition processor | Composites that read an added header |
| Merge | N:1 or N:M projector source | Batch, temporal reaggregation | None |
<!-- markdownlint-enable MD013 -->

### Edited contexts

An edit replaces any existing values of each header it adds. The
engine then re-forms the composites that read an added header, which
the compiler finds at startup:

- An absent composite may become present. It generates its random
  values then, once per output.
- A present composite is updated and keeps its random values, so
  partitions of one input still share them.
- Other composites keep their snapshots, and an edit with no
  dependents costs nothing extra.

A dependent composite therefore reads only the edit's value, wherever
a header of that name came from. This narrows RFC 0004's rule that
later source changes "do not update an existing snapshot or make an
absent composite present".

### Example: routing partitions

A `shard` composite combines the customer with the namespace header
that the partition processor adds. The `authorized_identity` policy is
as in the first example, and the `acme_payments` and `shared` ports
have subscribers configured like those in the first example.

```yaml
policies:
  context:
    entries:
      shard:                             # Composite name
        - type: authorized_identity      # Authorization claim
          name: customer_id
          store_as: customer
        - type: transport_header         # Added by the partition
          name: k8s-ns
          store_as: namespace

topics:
  handoff:
    backend: in_memory
    dispatch:
      conditions:
        - condition:
            type: composite
            name: shard
            value: {customer: acme, namespace: payments}
          port: acme_payments
      otherwise: shared

pipelines:
  ingress:
    nodes:
      partition:
        type: processor:partition
        config:
          partition_by:
            opl_expression: resource.attributes["k8s.namespace.name"]
          partition_header_name: k8s-ns
      publish:
        type: exporter:topic
        config:
          topic: handoff
    connections:
      - {from: authenticated_receiver, to: partition}
      - {from: partition, to: publish}
```

The `payments` partition of an `acme` request goes to `acme_payments`,
and the others go to `shared`. Today `shard` is evaluated only at the
receiver, before `k8s-ns` exists, so every partition goes to `shared`.

### Merged contexts

A merge forms no composites. By default its output keeps the peer
address and the transport headers that every contributing input
agrees on. This is the batch merge change recorded in the #4541
ledger and deferred to #4809. A merge node lists other entries to
keep, using RFC 0006 selectors
([#4809](https://github.com/open-telemetry/otel-arrow/issues/4809)):

```yaml
batch:
  type: processor:batch
  config:
    context:
      keep:
        - {type: composite, named: [tenant]}
        - {type: authorized_identity, named: [customer_id]}
```

- An entry is kept only when every contributing input holds it with
  the same values, in the same order. This is the RFC 0004
  accumulator, which "will erase conflicting entries".
- `keep` may not select a randomness entry or a composite with a
  randomness member, so a merge never copies a random value into more
  than one output, including the fragments of a split input.
- Every output of a merge node is formed by the merge, including
  inputs it passes through, so its context does not depend on the
  size or content of the request. Signals the node never merges are
  exempt: temporal reaggregation forwards logs and traces unchanged.

The batch processor does not group inputs by context, so it merges
requests of different tenants together. To keep tenants apart,
dispatch before batching.

### Reads after a merge

The compiler checks each lookup key against the pipeline graph:

- When every path to the reading node passes through a node that
  removes the entry, the configuration is rejected. Such nodes are a
  merge that neither keeps the entry by default nor selects it in
  `keep`, and the durable buffer processor until it preserves context
  ([#4058](https://github.com/open-telemetry/otel-arrow/issues/4058)).
- When only some paths remove it, the engine logs a warning at
  startup.

Other declared reads, such as header propagation and the Kafka
exporter's topic header, are not checked, so existing configurations
still start.

## Drawbacks

- Inputs that the batch processor passes through, and metrics that
  temporal reaggregation passes through, lose their authorized
  identity and composites, which they keep today. Migration: list the
  entries that downstream nodes read under `keep` on each such
  processor.
- Merged outputs gain the headers their inputs agree on, so exporters
  downstream of a batch may propagate headers they do not send today.
- An edit replaces a client-supplied header of the same name, which is
  then no longer propagated.
- A composite key whose members repeat matches only the presence row.

## Alternatives considered

- **Reject edits whose header a receiver may capture.** Checking the
  pipeline's own receivers misses headers carried through topics, and
  checking every publishing pipeline is fragile under live updates.
- **A `fields` list for multi-field lookups.** Dispatch could list
  several fields and give rows as value tuples. A composite already
  names several fields, so each lookup keeps to one entry.
- **Merges form composites, or keep every agreed entry by default.**
  Forming composites at a merge would read entries the inputs disagree
  on, and keeping randomness by default would copy random values into
  several outputs. Headers carry no random values, so they are kept by
  default.
- **Probe every combination of repeated composite values.** The number
  of probes would grow with the number of values.

## Unresolved questions

- **Stored keys.** If more nodes read the same key, should the engine
  hash it once when it forms the context and store it there?
- **Randomness on edit.** A composite completed by an edit generates
  random values per output, so partitions of one input differ. Should
  they derive from the input instead?
- **Default identity.** Should merges also keep agreed authorized
  identity by default, like headers?
- **Topic-level keys.** With step 7 export mappings, should a topic
  define its key over transfer fields instead of each publishing
  pipeline?
- **Batch partitioning.** Should the batch processor support RFC
  0004's `partition_by`, grouping inputs by kept entries instead of
  dropping entries they disagree on?
- **Port subscribers under live update.** When is the rule that every
  port has a subscriber checked, given that a live update can remove a
  delivery pipeline?
- **Other declared reads.** Should reads other than lookup keys get the
  same startup check, at the cost of rejecting configurations that
  start today?

## Related context

This RFC is part of a series beginning with [RFC
0004](./0004-pdata-context.md), the ongoing work described in
[#3872](https://github.com/open-telemetry/otel-arrow/issues/3872). It
follows [RFC 0006](./0006-transport-header-propagation.md) and
defines step 6 of
[#4541](https://github.com/open-telemetry/otel-arrow/issues/4541),
covering
[#3931](https://github.com/open-telemetry/otel-arrow/issues/3931) and
[#4809](https://github.com/open-telemetry/otel-arrow/issues/4809).
