---
Proposal Name: context-lookups-edits-and-merges
Start Date: 2026-11-24
RFC PR: open-telemetry/otel-arrow#4914
Tracking Issue: open-telemetry/otel-arrow#4541
---

# RFC NNNN: Context Lookups, Edits, and Merges

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
   entries edits and merges replace, keep, or form.

This document designs the context lookups, edits, and merges that
steps 6 and 7 of
[#4541](https://github.com/open-telemetry/otel-arrow/issues/4541)
need. It does not cover the other step 6 decisions (a new crate or an
evolved compiler, the packed representation, live-control
signatures), the ledger rows for live control and propagation `named`
lookup, or the step 7 import binding, durable buffer context, and
`network_info` composites.

Step 6 adds the lookup primitive and changes how edits and merges form
contexts. The merge change resolves the batch merge and reaggregation
pass-through rows that the #4541 ledger defers to
[#4809](https://github.com/open-telemetry/otel-arrow/issues/4809),
using the `context.keep` setting that #4809 asks for. It adds three
rows to that ledger:

- An edit replaces any existing values of the header it adds, instead
  of appending.
- An edit re-forms the composites that read the header it adds.
- A merge forms absent composites over the entries it keeps.

Each row lands as its own pull request, on the old compiler where it
allows, so that the swap's equivalence review stays clean.

Topic dispatch, the first node to evaluate a lookup key, changes the
topic exporter and receivers, so it lands with step 7, alongside topic
transfer and graph analysis across topics. The RFC 0006 dispatch
examples are therefore acceptance tests for step 7, although #4541
lists the RFC 0006 examples for step 6.

## User stories

See [RFC 0004](./0004-pdata-context.md#user-stories).

For example, User A routes thousands of tenants by the authorized
`customer_id`, and the cost of routing one publication does not depend
on the number of tenants.

For example, User C routes by customer and workspace header together,
and falls back to any known customer when the workspace does not
match.

For example, User G batches before forwarding, and expects a batch of
one customer's requests to keep the customer, and a batch of one
request's fragments to keep its correlation ID.

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

The examples in this document serve as acceptance tests. The routing
example tests topic dispatch in step 7; in step 6, an engine test
compiles its conditions over the publisher's context and checks each
decision. The partition and batch examples test step 6.

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
  keeping only the peer address when every input agrees. Each buffered
  input retains only its peer address until the merge. The batch
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
last`. Names match as entries of their domain do: transport header
names ASCII case-insensitively, other names exactly. The key compares
the selected values with a table of rows; a hit yields the row's
action, which for topic dispatch is a port.

### Rows

Each RFC 0006 value condition is a row, with one value per field.
Conditions with the same ordered fields and selections share one key.
Beyond the RFC 0006 rules, a condition is a configuration error when
it gives `value` for some fields but not others, or when a field gives
only one of `value` and `take`. More than 65536 conditions on one
topic, the most a `u16` position can number, is also a configuration
error.

- A row matches when each field's selected value equals the row's
  value, as exact bytes. Whether a value is text or binary is not
  compared, as for `transport_header_match` today, so a `-bin` header
  matches the UTF-8 bytes of a configured value. A field with no value
  matches no row.
- The engine probes each key at most once, in the order of the key's
  first condition, and keeps the hit with the earliest condition. It
  stops when no remaining key has an earlier condition. This relies on
  RFC 0006 rejecting duplicate value tuples for one key.
- The presence-only condition is checked last, without a probe.

A lookup therefore costs at most one probe per key, independent of the
number of rows.

### Hashing

The context compiler defines the hash and hashes the rows of each key
once per compile. The engine hashes the selected values when the key
is evaluated, in field order, each with its length; nodes never see
hashes. The topic exporter is the only reader, so a key stored with
the context would be computed once and read once, and would have to be
recomputed after every edit.

The hasher is seeded once per process, and lookup hashes are never
exported, transferred, or persisted. A hit is confirmed by comparing
the exact values, so a collision cannot select a row.

### Topic dispatch

Topic dispatch from [RFC
0006](./0006-transport-header-propagation.md#conditional-topic-dispatch)
is a set of lookup keys whose rows select ports. Each field names a
transfer field of the topic. The topic exporter builds the transfer
record through its export binding and evaluates the keys on that
record before it publishes, so dispatch reads the fields as
published. The topic's transfer schema types the fields, so the
topic's publishers share one compiled lookup.

Each publisher's `context.export` decides what a field carries: an
ordered list of sources, a composite member, or a value of another
domain downgraded into a `transport_header` field. A live pipeline
update that changes `context.export` therefore changes what dispatch
reads, without a restart. Without overrides, a field carries the
entry of the same name and domain.

The RFC 0006 port rules are checked whenever a configuration is
validated. Live control already checks topic wiring across pipelines
for each update; with `dispatch`, it also rejects an update that
removes the last subscriber of a port. `dispatch` is part of the
topic's configuration, so changing it requires a restart, as changing
a topic's backend or policies does today.

### Lookup declarations

A node declares the lookup keys it evaluates. In step 7, the engine
supplies them for a topic exporter whose topic has `dispatch`, as it
supplies header propagation policy for exporters today. The compiler
uses them to validate the conditions and check [reads after a
merge](#reads-after-a-merge):

```rust
pub enum ContextDeclaration {
    // ... existing variants ...
    /// Declares a lookup key the node evaluates.
    Lookup {
        /// Fields in key order, each with its selection.
        fields: Box<[(ContextField, ContextEntryTake)]>,
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

pub enum ContextEntryTake {
    Exists,
    First,
    /// New: the last value of each member, in source order.
    Last,
    All,
}
```

A lookup field accepts only `First` and `Last`. The node receives one
compiled `ContextLookup` for its declarations. In step 6,
`evaluate(&Metadata)` returns `Result<ContextDecision, ContextError>`,
like `ValueRead::read`, failing with `ContextMismatch` on a context
from another plan. Step 7 adds the same evaluation over a transfer
record. `ContextDecision` is the RFC 0004 decision:

- `Matched`, with the position of the first matching condition as the
  action;
- `Missing`, when no condition matched and a field that some condition
  reads is absent;
- `NoMatch`, otherwise.

Topic dispatch selects `otherwise` for both `Missing` and `NoMatch`.

## Context formation

<!-- markdownlint-disable MD013 -->
| Formation | RFC 0004 binding | Example nodes | Composites formed |
| --- | --- | --- | --- |
| Produce | Context arrival source | Receivers, traffic generator | Every selected composite |
| Edit | 1:1 or 1:N projector source | Partition processor | Composites that read an added header |
| Merge | N:1 or N:M projector source | Batch, temporal reaggregation | Absent selected composites over kept entries, without random members |
<!-- markdownlint-enable MD013 -->

### Edited contexts

An edit replaces any existing values of each header it adds, matching
names ASCII case-insensitively. The engine then re-forms the
composites that read an added header, which the compiler finds at
compile time:

- An absent composite may become present. It generates its random
  values once per input, so all outputs of one input share them, as
  RFC 0004 requires of split and partitioned outputs.
- A present composite is updated and keeps its random values.
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
`x-tenant: acme` and `k8s-ns` set to its namespace. Today, for a
client that does not send `k8s-ns`, `shard` is evaluated only at the
receiver, before `k8s-ns` exists, so it is absent and no partition
carries either header.

### Merged contexts

A merge node lists the entries it keeps under `context.keep`, using
RFC 0006 selectors
([#4809](https://github.com/open-telemetry/otel-arrow/issues/4809)).
Its output keeps the peer address and each listed entry that every
contributing input holds equally, then forms the absent composites
that a downstream node selects from the entries it kept. `keep` does
not create batch partitions, as RFC 0004, amended in
[#4421](https://github.com/open-telemetry/otel-arrow/pull/4421),
requires. The
`authorized_identity` policy is as in the first example.

```yaml
policies:
  context:
    entries:
      request:                           # Correlation ID
        - type: randomness
          name: id
          value: uuid7

pipelines:
  ingress:
    nodes:
      batch:
        type: processor:batch
        config:
          context:
            keep:
              - {type: authorized_identity, named: [customer_id]}
              - {type: composite, named: [request]}
```

A batch of `acme` requests keeps `customer_id: acme`, and a batch of
fragments of one request keeps its `request` ID. A batch that mixes
customers or requests drops the entry on which they differ.

- **Equal entries.** Inputs hold an entry equally when its values
  match as bytes, in the same order, with the same kind and original
  wire name, which `name: preserve` sends. A composite must also come
  from the same declaration, as RFC 0004 requires after a topic hop.
- **Retention.** Each buffered input retains only its peer address and
  the entries `keep` lists, which the compiler fixes per merge node.
- **Formation.** A formed composite reads only kept entries. A merge
  generates no random values: they are generated only where a context
  is materialized, at receivers as in RFC 0004 and at edits as above.
  So a composite with a random member is either kept with its values
  or absent. Every output that keeps it, including the pieces of one
  split input, shares its value.
  A `required_out` entry on a merge node is a configuration error
  unless the node can form it or `keep` lists it.
- **Pass-through.** Every output of a merge node is formed by the
  merge, including inputs it passes through. A single input agrees
  with itself, so a pass-through keeps every listed entry, and its
  context does not depend on the size or content of the request.
  Inputs of a signal the node never merges, such as logs and traces
  at temporal reaggregation, keep their whole context, as today: they
  never meet other inputs.

The batch processor does not group inputs by context, so it merges
requests of different tenants together. To keep tenants apart,
dispatch before batching.

### Reads after a merge

The compiler checks each field that a dispatch condition reads on
every compile, including live updates, following paths back through
topics with the step 7 graph analysis. This answers the RFC 0006 open
question of warning when a merge removes a dispatch key:

- A field that no path can supply is a configuration error, as the
  #4541 ledger makes other reads that can never be satisfied. Sources
  include receivers, upstream edits that add the field, and merges
  that keep it.
- A field that some paths cannot supply logs a warning. So does a
  field that a merge keeps on a path with no later edit that adds it,
  because merged requests whose inputs disagree lose it.
- When the topic has no `otherwise`, that warning is a configuration
  error instead: a merged request that matches no condition is
  permanently Nacked.

A merged output keeps a value only when every input holds it, so every
input also satisfies the condition the merged request matches. A merge
can move a request to a later condition or to `otherwise`, but never
to a port whose condition one of its inputs fails, so it never routes
one tenant's data to another tenant's dedicated port.

Other declared reads, such as header propagation and the Kafka
exporter's topic header, are not checked.

## Drawbacks

- Inputs that a merge node passes through keep only the entries its
  `keep` lists, where today the batch processor and some temporal
  reaggregation metrics pass-throughs keep the whole context.
  Migration: list the entries that downstream nodes read under `keep`.
- A batch drops a correlation ID unless `keep` lists it, although RFC
  0006 expects split outputs to share it.
- Buffered inputs retain their kept entries until flush, and the batch
  `sizer` does not count them.
- Whether a merged output keeps an entry depends on which inputs a
  batch happens to combine, so the same configuration can route a
  merged request differently from one batch to the next.
- An edit replaces a client-supplied header of the same name, which is
  then no longer propagated. Today an edit appends after the client's
  value, so a `take: first` read sees the client's value rather than
  the edit's; replacement closes that gap.
- Changing a topic's `dispatch` requires a restart.

## Alternatives considered

- **Reject edits whose header a receiver may capture.** Checking the
  pipeline's own receivers misses headers carried through topics, and
  checking every publishing pipeline is fragile under live updates.
- **Composite conditions.** Each condition could name one composite
  entry, so every lookup reads one entry. RFC 0006 lists fields
  instead, so conditions may read different fields and routing needs
  no composite.
- **Lookup fields with ordered sources.** Each field could list the
  sources that export reads. That repeats export's resolution in every
  publisher's lookup; evaluating the exported record shares one lookup
  per topic.
- **Keep every agreed entry by default.** No configuration would be
  needed, but each buffered input would retain its whole context, and
  #4809 asks for the kept entries to be configured.
- **Keep what downstream nodes read.** The compiler could keep the
  agreed entries that some downstream node reads, without `keep`. This
  needs demand analysis across topics, which arrives with step 7.
- **New random values at a merge.** A merge could generate values
  for the composites it forms, but that replaces a correlation ID the
  inputs disagree on instead of dropping it, and gives the pieces of
  one split input different IDs.
- **Random values made at the receiver for later edits.** The receiver
  could generate the values of composites that a downstream edit may
  complete, at a cost to every request.

## Unresolved questions

- **Stored keys.** If more nodes read the same fields, should the
  engine hash each value once when it forms the context, and build
  keys from the stored hashes, as in the name and hash pairs suggested
  in the RFC 0006 review?
- **Rows under live update.** Adding a tenant changes `dispatch` and
  requires a restart. Should a change to rows only, with the same
  fields and ports, rebuild the publishers' lookups through a pipeline
  rollout?
- **Default `keep`.** Once demand reaches across topics, should `keep`
  default to the entries that downstream nodes read?
- **Batch partitioning.** Should the batch processor support RFC
  0004's `partition_by`, grouping inputs by entries instead of
  dropping entries they disagree on?
- **Other declared reads.** Should reads other than lookup keys get the
  same check?

## Related context

This RFC is part of a series beginning with [RFC
0004](./0004-pdata-context.md), the ongoing work described in
[#3872](https://github.com/open-telemetry/otel-arrow/issues/3872). It
follows [RFC 0006](./0006-transport-header-propagation.md) and
designs context lookups, edits, and merges for step 6 of
[#4541](https://github.com/open-telemetry/otel-arrow/issues/4541),
and topic dispatch for step 7, covering
[#3931](https://github.com/open-telemetry/otel-arrow/issues/3931) and
[#4809](https://github.com/open-telemetry/otel-arrow/issues/4809).
