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
   dispatch over ordered fields, but neither defines how conditions
   compile into keys, how keys are hashed, or what a lookup costs.
2. RFC 0004 leaves it to projector bindings to "dictate which context
   entries survive into the output context", but does not say which
   composites edits and merges form.

This is the design for steps 6 and 7 of
[#4541](https://github.com/open-telemetry/otel-arrow/issues/4541).
Step 6 compiles lookup keys and changes composite formation, whether
it adds a new `pdata-context` crate or evolves the existing compiler.
The merge change resolves the batch merge and reaggregation
pass-through entries that the #4541 ledger defers to
[#4809](https://github.com/open-telemetry/otel-arrow/issues/4809),
and adds three entries to that ledger:

- A merge keeps every entry its inputs agree on, in every domain, not
  only headers.
- An edit replaces any existing values of the header it adds, instead
  of appending.
- An edit re-forms the composites that read the header it adds.

Neither change adds node configuration.

Topic dispatch, the first node to evaluate a lookup key, changes the
topic exporter and receivers, so it lands with step 7, alongside topic
transfer, durable buffer context, and graph analysis across topics.
The RFC 0006 dispatch examples are therefore acceptance tests for step
7, although #4541 lists the RFC 0006 examples for step 6.

## User stories

See [RFC 0004](./0004-pdata-context.md#user-stories).

For example, User A routes thousands of tenants by the authorized
`customer_id`, and the cost of routing one publication does not depend
on the number of tenants.

For example, User C routes by customer and workspace header together,
and falls back to any known customer when the workspace does not
match.

For example, User G batches before forwarding, and expects a batch of
one customer's requests to keep the customer and correlation ID that
the requests share.

For example, User P partitions by namespace and sends each partition
with tenant and namespace headers taken from one composite, both or
neither.

### Example: routing tenants

**Scope of this example.** The YAML is a group-level excerpt.
`dispatch` and `subscription.port` use the syntax proposed in RFC
0006. A complete deployment also supplies an authenticated
`receiver:otlp` named `authenticated_receiver`, an exporter in each
delivery pipeline, and `shared_delivery` and `unattributed_delivery`
pipelines that differ from `dedicated_delivery` only in their port.

```yaml
policies:
  authorized_identity:
    - claim: sub
      store_as: customer_id
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-workspace]

topics:
  handoff:
    backend: in_memory
    dispatch:
      conditions:
        - condition:
            - type: authorized_identity
              name: customer_id
              take: first
              value: acme
            - {name: x-workspace, take: last, value: blue}
          port: dedicated
        - condition:
            - {type: authorized_identity, name: customer_id}
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
With workspace `green`, or without the header, it goes to `shared`,
because the second condition reads only `customer_id`. A request
without a customer goes to `unattributed`. Adding conditions for
more tenants over the same two fields adds rows, not lookups.

## Detailed design

The examples in this document serve as acceptance tests: the routing
example for step 7, and the partition example for step 6.

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
  keeping only the peer address when every input agrees. The batch
  processor passes in-range inputs through with their whole context.
  Temporal reaggregation keeps the whole context for inputs it does
  not aggregate and for partial pass-throughs that need no completion
  tracking, and forms other partial pass-throughs by the merge, from
  their single input.
- Topics pass the whole context through, dropping only Ack/Nack
  routing state. The durable buffer processor drops the context.
- Nodes read context values directly, for example the Kafka exporter
  takes its topic from a transport header, but no node evaluates a
  condition against a configured table.

## Context lookup keys

A **lookup key** is an RFC 0004 [predicate
binding](./0004-pdata-context.md#pdata-context-predicates) over an
ordered list of fields. Each field names a primitive context entry by
domain and name, with the RFC 0006 selection `take: first` or `take:
last`. The key compares the selected values with a table of rows; a
hit yields the row's action, which for topic dispatch is a port.

### Rows

Each RFC 0006 value condition is a row, with one value per field.
Conditions with the same ordered fields and selections share one key.
Beyond the RFC 0006 rules, a
condition is a configuration error when it gives `value` for some
fields but not others, or when a field gives only one of `value` and
`take`.

- A row matches when each field's selected value equals the row's
  value, as exact bytes. Whether a value is text or binary is not
  compared, as for `transport_header_match` today, so a `-bin` header
  matches the UTF-8 bytes of a configured value. A field with no value
  matches no row.
- The engine probes each key at most once, in the order of the key's
  first condition, and keeps the hit with the earliest condition. It
  stops when no remaining key has an earlier condition.
- The presence-only condition is checked last, without a probe.

A lookup therefore costs at most one probe per key, independent of the
number of rows.

### Hashing

The context compiler defines the hash and hashes every row once per
key, at startup. The engine hashes the selected values when the key
is evaluated, in field order, each with its length; nodes never see
hashes. The topic exporter is the only reader,
so a key stored with the context would be computed once and read
once, and would have to be recomputed after every edit.

The hasher is seeded once per process, and lookup hashes are never
exported, transferred, or persisted. A hit is confirmed by comparing
the exact values, so a collision cannot select a row.

### Topic dispatch

Topic dispatch from [RFC
0006](./0006-transport-header-propagation.md#conditional-topic-dispatch)
is a set of lookup keys whose rows select ports. Each field names a
transfer field, and the topic exporter evaluates the keys before it
publishes. The compiler resolves each transfer field to the context
entry it carries through the export binding, so evaluation reads the
publisher's context. A derived transfer field keeps the domain and
name of the entry it carries. An explicit `context.transfer` decides
what each field carries, so changing it can change what an unchanged
condition reads.

The RFC 0006 port rules are checked whenever a configuration is
validated. Live control already checks topic wiring across pipelines
for each update; with `dispatch`, it also rejects an update that
removes the last subscriber of a port. `dispatch` is part of the
topic's configuration, so changing it requires a restart, as changing
a topic's backend or policies does today.

### Lookup declarations

The engine supplies lookup declarations for a topic exporter whose
topic has `dispatch`, as it supplies header propagation policy for
exporters today. The compiler uses them to validate the conditions and
check [reads after a merge](#reads-after-a-merge):

```rust
pub enum ContextDeclaration {
    // ... existing variants ...
    /// Declares a lookup key the node evaluates.
    Lookup {
        /// Fields in key order, each with its selection.
        fields: Box<[(ContextField, ContextTake)]>,
        /// Value rows, one value per field, each with the position
        /// of its condition.
        rows: Box<[(u16, Box<[Box<[u8]>]>)]>,
    },
    /// Declares a presence-only condition, checked last.
    Presence { fields: Box<[ContextField]> },
}

pub struct ContextField {
    pub domain: ContextDomain,
    pub name: ContextEntryName,
}

pub enum ContextTake {
    First,
    Last,
}
```

The node receives one compiled `ContextLookup` for its declarations.
Its `evaluate(&Metadata)` returns the RFC 0004 `ContextDecision`:
`Matched` with the position of the first matching condition as the
action, `Missing` when no condition matched and a field that some
condition reads is absent, and `NoMatch` otherwise. Topic dispatch
selects `otherwise` for both `Missing` and `NoMatch`.

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
- A present composite becomes absent when the edited header no longer
  satisfies it, for example when a `transport_header_match` on that
  header fails, as the receiver decides presence today.
- Other composites keep their snapshots, and an edit with no
  dependents costs nothing extra.

A dependent composite therefore reads only the edit's value, wherever
a header of that name came from. This narrows RFC 0004's rule that
later source changes "do not update an existing snapshot or make an
absent composite present".

### Example: propagating partitions

A `shard` composite combines the tenant header from the client with
the namespace header that the partition processor adds, and the
exporter sends both members as headers. A complete deployment also
supplies a `receiver:otlp` named `otlp_receiver`.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-tenant]
    header_propagation:
      default:
        selector:
          type: named
          named: [shard:tenant, shard:namespace]
  context:
    entries:
      shard:                             # Composite name
        - type: transport_header         # Sent by the client
          name: x-tenant
          store_as: tenant
        - type: transport_header         # Added by the partition
          name: k8s-ns
          store_as: namespace

pipelines:
  ingress:
    nodes:
      partition:
        type: processor:partition
        config:
          partition_by:
            opl_expression: resource.attributes["k8s.namespace.name"]
          partition_header_name: k8s-ns
      export:
        type: exporter:otlp_grpc
        config:
          grpc_endpoint: "http://backend:4317"
    connections:
      - {from: otlp_receiver, to: partition}
      - {from: partition, to: export}
```

Each partition of a request with `x-tenant: acme` leaves with
`x-tenant: acme` and `k8s-ns` set to its namespace. Today `shard` is
evaluated only at the receiver, before `k8s-ns` exists, so it is
absent and no partition carries either header.

### Merged contexts

A merge forms no composites. Its output keeps every entry that every
contributing input holds with the same values, in the same order, and
drops the rest. This is the RFC 0004 accumulator, which "will erase
conflicting entries", and it applies to every domain:

- A composite is kept whole when every input holds it with the same
  members; a merge never forms a composite from parts.
- Random values of different requests never agree, so merged requests
  lose them. Fragments of one split input agree, so they keep the
  correlation ID that RFC 0004 and RFC 0006 expect split outputs to
  share.
- Every output of a merge node is formed by the merge, including
  inputs it passes through. A single input agrees with itself, so a
  pass-through keeps its whole context, as the #4541 development
  branch did.

The #4541 development branch kept agreed headers for a batch; this
extends the rule to every domain. The batch processor does not
group inputs by context, so it merges requests of different tenants
together and drops the entries on which they differ. To keep tenants
apart, dispatch before batching.

### Reads after a merge

A merge keeps a field only when its inputs agree, so whether a
dispatch condition can read the field after a merge depends on the
traffic. This answers the RFC 0006 open question of warning when a
merge removes a dispatch key:

- The engine logs a warning at startup when a field that a dispatch
  condition reads reaches the topic exporter through a merge node. The
  check follows paths back through topics to the publishing pipelines,
  using the step 7 graph analysis.
- Merged requests whose inputs disagree on the field go to the next
  matching condition or to `otherwise`.

Other declared reads, such as header propagation and the Kafka
exporter's topic header, get no warning.

## Drawbacks

- Merged outputs gain the entries their inputs agree on, so exporters
  downstream of a batch may propagate headers they do not send today,
  and downstream nodes see authorized identity and composites they do
  not see today.
- A merge compares every entry of its inputs, not only the peer
  address.
- Whether a merged output keeps an entry depends on which inputs a
  batch happens to combine, so the same configuration can route a
  merged request differently from one batch to the next. Today a
  merged output keeps nothing but an agreed peer address.
- An edit replaces a client-supplied header of the same name, which is
  then no longer propagated.
- Changing a topic's `dispatch` requires a restart.

## Alternatives considered

- **Reject edits whose header a receiver may capture.** Checking the
  pipeline's own receivers misses headers carried through topics, and
  checking every publishing pipeline is fragile under live updates.
- **Composite conditions.** Each condition could name one composite
  entry, so every lookup reads one entry. RFC 0006 lists fields
  instead, so conditions may read different fields and routing needs
  no composite.
- **Merges form composites.** Forming composites at a merge would read
  entries the inputs disagree on.
- **A narrow default with `keep`.** A merge could keep only the peer
  address and agreed headers, as the development branch did, and list
  other entries in a `keep` setting on each merge node. Fragments of a
  split input would then lose their correlation ID unless every batch
  listed it.

## Unresolved questions

- **Stored keys.** If more nodes read the same fields, should the
  engine hash each value once when it forms the context, and build
  keys from the stored hashes, as in the name and hash pairs suggested
  in the RFC 0006 review?
- **Randomness on edit.** A composite completed by an edit generates
  random values per output, so partitions of one input differ. Should
  they derive from the input instead?
- **Dropping agreed entries.** Should a merge node be able to drop
  entries its inputs agree on, for example to stop sending a header
  after a batch?
- **Batch partitioning.** Should the batch processor support RFC
  0004's `partition_by`, grouping inputs by entries instead of
  dropping entries they disagree on?
- **Other declared reads.** Should reads other than lookup keys get the
  same startup warning?

## Related context

This RFC is part of a series beginning with [RFC
0004](./0004-pdata-context.md), the ongoing work described in
[#3872](https://github.com/open-telemetry/otel-arrow/issues/3872). It
follows [RFC 0006](./0006-transport-header-propagation.md) and
defines lookup keys and composite formation for step 6 of
[#4541](https://github.com/open-telemetry/otel-arrow/issues/4541),
and topic dispatch for step 7, covering
[#3931](https://github.com/open-telemetry/otel-arrow/issues/3931) and
[#4809](https://github.com/open-telemetry/otel-arrow/issues/4809).
