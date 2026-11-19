# RFC 0006: Context Capture, Transfer, and Propagation

## Status and scope

This is a proposal extending [RFC 0004: Pdata Context](./0004-pdata-context.md)
in two directions:

1. The transport headers feature found in
`policies::transport_headers::header_capture` and
`policies::transport_headers::header_propagation` were designed before
context entries, and currently we have not established configuration
structures that would permit propagating other context fields that may
be valuable to users.
2. The prior RFC did not define how to transfer context to and from
persistent storage or across topic boundaries.

## User stories

See [RFC 0004](./0004-pdata-context.md#user-stories).

For example, User A has authenticated a user and wishes to pass claim
values or mTLS subject names as outbound transport header names.

For example, Users A/B/C wants to use the randomness source to attach
a deduplication identifier to outbound request batches before they
enter the durable buffer processor's persistent queue.

For example, Users A/B/C wants to batch by a two-dimensional composite
consisting of a claim and source network address and propagate both
context entry fields to the the destination as plain headers over a
trusted connection.

## Detailed design

We will extend propagation as follows:

- Outbound HTTP and gRPC transport propagation will be extended to
  support composite entries as a whole or by list of members.
- Topic exporter and receiver will be extended with fine control
  over contexts as they cross between pipelines.
- The durable buffer processor will be extended with control over
  contexts that are stored and retrieved on the disk.

### Baseline

The current state this document reflects is defined by existing
transport headers policy for capture and propagation. Here is an
example:

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-workspace-id]
          store_as: workspace_id
        - match_names: [x-request-id]
          store_as: request_id
    header_propagation:
      default:
        selector:
          type: named
          named: [workspace_id, request_id]
        action: propagate
        name: preserve
      overrides:
        - match:
            # "named" is a new alias for stored_names
            named: [request_id]
          action: drop
```

Header capture policies lets users control how transport headers are
captured and whether they retain their original header
name. `match_names` is a list of exact header names.  `store_as` lets
users rename the transport header context entry.  Additional capture
controls cover `sensitive` and value kind, binary or text.

Header propagation consists of `default` settings and a list of
`overrides`. The default settings include the selector:

- `selector`: what to select
  - `type`: type of selection including `all_captured`, `none`, or `named`
  - `named`: list of transport header names, only the `named` selector type
- `action`: whether to `propagate` or `drop`
- `name`: can be `preserve` for the original header name (including
  capitalization) or `stored_name` for the name configured by `store_as`.

The overrides section features the same `action` and `name`
clauses. Unlike defaults, overrides include a `match` section with a
list of `stored_names` for the action.

Since the addition of composite context entries, transport header propagation
has been extended to support propagation of transport headers within
composites, however this support is limited.

[RFC 0004](./0004-pdata-context.md) defines node entry and exit
requirements for Pdata Context which are not currently implemented.

## Pdata Context propagation

Context features are partially incorporated into DFE receivers and
exporters, however `header_propagation` is still limited to the
transport headers domain. The proposal below supports propagating
authorized identity fields, constant and randomness fields, and whole
composite entries.

### Full context example

The follow context stores two transport headers, one authorized
identity field, a composite with a conditional match, and a
fully-synthetic composite containing idempotency token and route name.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-workspace-id]
          store_as: workspace_id
        - match_names: [x-environment]
          store_as: environment
  authorized_identity:
    - claim: sub
      store_as: customer_id
  context:
    entries:
      product_user:
        - type: authorized_identity
          name: customer_id
          store_as: customer
        - type: transport_header
          name: workspace_id
          store_as: workspace
        - type: transport_header_match
          name: environment
          value: production
      delivery:
        - type: randomness
          name: id
          value: uuid7
        - type: constant
          name: route
          value: primary
```

### Extending defaults and overrides

This proposal retains the existing `default` and ordered `overrides`
form.

For example, drop captured headers by default, propagate a composite
entry by its stored name, an authorized identity field with a custom
header name, and a randomness value:

```yaml
header_propagation:
  default:
    selector:
      type: all_captured
    action: drop
  overrides:
    - match:
        # Here, a named composite entry using the stored name
        type: named
        named: ["product_user:workspace"]
      action: propagate
      name: stored_name

    - match:
        # Select an authorized-identity field explicitly.
        type: authorized_identity
        named: [customer_id]
      action: propagate
      name: literal
      literal: x-customer-id

    - match:
        # Here, a composite randomness entry using a literal name
        type: named
        named: ["delivery:id"]
      action: propagate
      name: literal
      literal: x-idempotency
```

### Proposed selectors

New selectors support the missing operations:

<!-- markdownlint-disable MD013 -->
| Selector | Meaning |
| --- | --- |
| `{type: all_captured}` | _Existing:_ Select all captured transport headers. |
| `{type: none}` | _Existing:_ Select no sources; overrides may still select sources. |
| `{type: named, named: [workspace_id]}` | _Existing:_ Select transport headers by stored name. |
| `{type: named, named: ["product_user:workspace"]}` | _Extend:_ Select a composite member from any domain. |
| `{type: composite, named: [product_user]}` | _New:_ Select a whole composite. |
| `{type: authorized_identity, named: [customer_id]}` | _New:_ Select an authorized-identity field. |
| `{type: randomness, named: [id]}` | _New:_ Select a randomness entry. |
| `{type: network_info, named: [peer_addr]}` | _New:_ Select a network-info field. |

These selectors are available in both `default.selector` and
`overrides[].match`. An override's `match` object is the selector
itself, without a nested `selector` key. In override matches, `named`
is an alias for the legacy `stored_names`; the examples use `named`
consistently.

A selector's `type` names a primitive
domain or `composite`, and `named` lists names in that namespace. The
selection logic is:

- `all_captured` applies only to transport headers
- `named` applies to transport headers (unqualified names) or composite members (qualified names). Transport headers are selected by stored name; original wire names are used by `name: preserve`, not as an additional matching namespace.
- `composite` selects whole composites by name.
- Other types select primitive domains explicitly, e.g., `authorized_identity`, `randomness`, or `network_info`. Constants are specific to their composite and are selected only as qualified composite members.

### Whole composite and selected fields

The whole-composite override emits multiple headers when the composite
entry is valid. It uses a `composite` selector in the match clause:

```yaml
header_propagation:
  default:
    selector:
      type: none
  overrides:
    - match:
        type: composite
        named: [product_user]
      action: propagate
      name: fields
      fields:
        customer: x-customer-id
        workspace: x-workspace-id
```

This form of selector expands into the same configuration as an
explicit match clause for every entry in the composite.

### Propagation semantics

Transport-header values retain their text or binary kind, and their
original wire name when capture policy requires it. Constant,
randomness, and network-info values have no original wire name, so
`name: preserve` falls back to the entry or member name. Repeated
source values retain source order.

## Context transfer

Context values may be exported and imported between DFE nodes or
between a node and itself, cases such as the topic exporter/receiver
and the durable buffer processor. These cases will declare Pdata Context
transfer bindings during registration:

- Export transfer: these bindings define a function from Pdata Context
  to a flat record of named transfer fields. Each field carries the
  values of one primitive entry or composite member, in the domain
  declared for that field. Whole composites are not exportable, except
  the implicit composite of all captured headers (see [All captured
  transfer](#all-captured-transfer)).
- Import transfer: these bindings define a function from the flat
  record to a Pdata Context in which each field is a primitive entry of
  its declared domain, under its field name (see [Context
  import](#context-import)).

For transfer, this proposal extends RFC 0004 by treating randomness and
constants as primitive domains. A randomness declaration names an entry
and its generator, initially `uuid7`. Composites referring to the same
randomness name share a value and must specify the same generator.
Constants remain composite-specific and are selected for export by
qualified `composite:member` references. Network information is a primitive
domain with named fields, beginning with `peer_addr`.

Transfer carries these values rather than generating randomness, setting
constants, or observing a new peer. Receiving pipelines evaluate their
own composites over the imported entries. These transfer rules supersede
RFC 0004's receiver-boundary materialization description for topics.

As an example, the following topic declares a Pdata Context transfer
with one field of each domain.

```yaml
topics:
  handoff:
    backend: in_memory
    context:
      transfer:
        customer:
          type: authorized_identity
        delivery_id:
          type: randomness
        route:
          type: constant
        source_addr:
          type: network_info
        workspace:
          type: transport_header
```

### Context export

The export side of a context transfer is responsible for selecting
from the available composite and primitive context entries the
corresponding transfer context. The exporter lists a selector
expression for each context transfer field:

```yaml
type: exporter:topic
config:
  topic: handoff
  context:
    export:
      customer:
        required: true
        selector: {type: authorized_identity, named: [customer_id]}
      delivery_id:
        required: true
        selector: {type: named, named: ["delivery:id"]}
      workspace:
        selector: {type: named, named: ["product_user:workspace"]}
      source_addr:
        selector: {type: network_info, named: [peer_addr]}
      route:
        selector: {type: named, named: ["delivery:route"]}
```

Topic exporter configuration has the option to set `required: true`
for any of the fields. An absent required value causes an immediate
permanent Nack.

Note that the selector `named` list supports multiple names per
selector. This is useful in cases where more than one composite may
supply the value. A field's source must belong to the field's declared
domain, with one exception: the `transport_header` domain is the
lowest grade. Values of other domains may be degraded into
`transport_header` fields, but transport headers cannot be upgraded
into other domains. In such cases, the exporter may list names
referring to different types. For example:

```yaml
# topic configuration under context::export
dest_name:
  selector:
    type: named
    # export a constant or authorized identity into a transport header
    named:
      - "product:dest_name"               # authorized_identity
      - "internal_normal:destination"     # constant
      - "internal_alternate:destination"  # constant
```

### Context import

Imported fields become primitive entries of their declared domain in
the receiving pipeline, under their transfer field names. Import never
changes a field's domain. The only change of domain is an explicit
downgrade to `transport_header` at export, so importing never grants a
transferred value more authority than its source had.

Transfer bindings carry randomness and constant entries like entries
of any other domain, and never generate or set their values.

Imported entries follow the same naming rules as other primitive
entries. They are identified by domain and transfer field name.
When an import cannot be resolved unambiguously, the configuration is
rejected and the user must configure the transfer explicitly.

A reference to a randomness, constant, or network-info entry that the
transfer cannot supply is a configuration error. An optional field may
be absent from an individual record; import leaves it absent rather than
synthesizing a replacement.

Importing nodes have no import mapping. The compiler retains exactly
the transfer fields that bindings in the receiving pipeline reference.
Like other receivers, an importing node constructs Pdata Context using
its bindings, which evaluates composite entry presence as needed by
its pipeline.

### Pass-through

Because the engine establishes both the export and import bindings, it
can recognize when a transfer is equivalent to passing the context
through. A field exported under its source's domain and name restores
that source entry exactly, whether the exporter selected the entry
directly or as a composite member. The all-captured field restores the
captured transport headers with their stored and original wire names.

When every entry referenced by the receiving pipeline is restored this
way, and the compiled bindings on both sides are compatible, a topic
may pass the context snapshot through without flattening it. This fast
path is the only difference between topic and durable buffer transfer;
the durable buffer always flattens, because its records outlive the
process.

A pass-through must remain flattenable on demand. When either side's
compiled bindings change while messages are in flight, the receiver
restores context from the transfer record and re-evaluates composites
against the restored entries. Composite evaluation only reads primitive
entries, so re-evaluation never generates a new random value. A
composite whose carried member is missing is absent.

#### Composites over imported entries

Imported entries are ordinary primitive entries, so composites in the
receiving pipeline refer to them by domain, like entries captured by
any other receiver. Importing nodes make use of materialization
bindings to construct new contexts like other receivers, using the
imported entries as their primitive input.

As an example policy,

```yaml
policies:
  context:
    entries:
      tenant_base:
        - type: authorized_identity
          name: customer
        - type: transport_header
          name: workspace
```

### Conditional topic dispatch

The topic lists conditions paired with named output ports, with an
optional `otherwise` port. These ports name topic destinations, not
outputs of the exporter or receiver. Conditions use the predicate
bindings described in [RFC 0004](./0004-pdata-context.md#pdata-context-predicates).
The engine compiles them against each export mapping, using the
topic's transfer field names and domains.

Conditions are evaluated in order before delivery. The first matching
condition selects its port; a missing input does not match. If no
condition matches, the topic selects `otherwise`, or refuses the
publication when no fallback is configured. For example, extend the
`handoff` topic above:

```yaml
topics:
  handoff:
    dispatch:
      conditions:
        - condition:
            type: transport_header_match
            name: workspace
            value: "1"
          port: bigfish
        - condition:
            type: transport_header_match
            name: workspace
            value: "2"
          port: smallfish
      otherwise: catchall
```

Receivers subscribe to a named port. Broadcast or balanced delivery
applies within the selected destination:

```yaml
# This receiver is for tenant one.
bigfish:
  type: receiver:topic
  config:
    topic: handoff
    subscription:
      port: bigfish
    ...

# This receiver is for tenant two.
smallfish:
  type: receiver:topic
  config:
    topic: handoff
    subscription:
      port: smallfish
    ...

# This receiver is for the remaining tenants.
catchall:
  type: receiver:topic
  config:
    topic: handoff
    subscription:
      port: catchall
    ...
```

The `required_in` policy remains a presence assertion, not a dispatch
condition. The `otherwise` port handles no-match cases, not delivery
failures: backpressure, unavailable receivers, and Ack/Nack outcomes
follow the selected destination's policies without selecting another
port. Topics without `dispatch` retain their existing delivery behavior.

### Derived contracts

A topic without a `context.transfer` declaration still has a
transfer contract. The engine derives it from the configuration: the
contract contains one optional field for each primitive entry that the
receiving pipelines reference and the exporting pipelines can supply,
under that entry's own domain and name, and each exporter selects the
same-named entry. A derived contract is exactly the contract a user
could write, and the engine can print it so that users can see, copy,
and modify the configuration that is in effect. Because every derived
field restores its source entry under its domain and name, derived
contracts are eligible for pass-through. Fields referenced by topic
conditions are included on the same basis.

Pipelines on either side of a topic may be reconfigured independently
as long as the topic's contract does not change.

## Durable buffer processor

The durable buffer processor will embed Pdata Context transfer and
export configuration, and import its own stored records.

```yaml
type: processor:durable_buffer
config:
  context:
    transfer:
      customer:
        type: authorized_identity
      delivery_id:
        type: randomness
      route:
        type: constant
      source_addr:
        type: network_info
      workspace:
        type: transport_header
    partition_by:
      - customer
    export:
      customer:
        required: true
        selector: {type: authorized_identity, named: [customer_id]}
      delivery_id:
        required: true
        selector: {type: named, named: ["delivery:id"]}
      workspace:
        selector: {type: transport_header, named: [workspace_id]}
    ...
```

The durable buffer uses the same transfer mechanism as topics, without
the fast path: it always stores the flat transfer record. On retrieval
it imports the record like a topic receiver. When the restore is
handled correctly, the result is as if the context had passed through
the buffer: fields exported under their source's domain and name
restore those entries, and downstream consumers are unaffected.

If the context configuration changes and the buffer restarts, stored
records may no longer match the new bindings. Their fields are still
restored as entries of their stored domains and names, where the user
defines how to handle them with composites, and composites re-evaluate
as described in [Pass-through](#pass-through).

## All captured transfer

Captured transport headers form an implicit composite whose members
are all captured headers, under their stored names. A capture policy
for several headers:

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-a, x-b, x-c]
```

The `all_captured` selector selects this implicit composite as a
whole, so it can be exported as one transfer field:

```yaml
config:
  topic: handoff
  context:
    export:
      all_headers:
        selector:
          type: all_captured
```

On import, the field restores the captured headers, with their
stored and original wire names, in the receiving pipeline's
transport-header domain. The same propagation therefore applies after
the topic:

```yaml
header_propagation:
  default:
    selector:
      type: all_captured
    action: propagate
    name: preserve
```

## Related context

This RFC is part of a series beginning with [RFC
0004](./0004-pdata-context.md), the ongoing work described in
[#3872](https://github.com/open-telemetry/otel-arrow/issues/3872).
