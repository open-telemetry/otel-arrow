# RFC 0006: Context Capture, Transfer, and Propagation

## Status and scope

This is a proposal extending [RFC 0004: Pdata Context](./0004-pdata-context.md).

The transport headers feature found in
`policies::transport_headers::header_capture` and
`policies::transport_headers::header_propagation` were designed before
context entries, and currently we have not established configuration
structures that would permit propagating other context fields that may
be valuable to users.

This proposal defines how we will extend propagation features to
account for new features of Pdata Context.

## User stories

See [RFC 0004](./0004-pdata-context.md#user-stories).

For example, User A has authenticated a user and wishes to pass claim
values or mTLS subject names as outbound transport header names.

For example, Users A/B/C wants to use the randomness source to attach
a deduplication identifier to outbound request batches before they
enter the persistent queue.

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

New features are partially incorporated into DFE receivers and
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
form. It extends source selection and output naming without introducing
a separate `rules` form or requiring existing configurations to migrate.

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
        stored_names: [customer:workspace_id]
      action: propagate
      name: stored_name

    - match:
        # This field is added, defaults to transport_header.
        type: authorized_identity
        stored_names: [customer_id]
      action: propagate
      name: literal
      literal: x-customer-id

    - match:
        # Here, a composite randomness entry using a literal name
        stored_names: ["delivery:id"]
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
| `{type: named, named: [product_user]}` | _Extend:_ Select a whole composite. |
| `{type: authorized_identity, named: [customer_id]}` | _New:_ Select an authorized-identity field. |

These selectors are available in both `default.selector` and
`overrides[].match.selector`. The selection logic is:

- `all_captured` applies only to transport headers
- `named` applies to transport headers or whole composites (unqualified name) or composite members (qualified names). Transport headers are selected by stored name; original wire names are used by `name: preserve`, not as an additional matching namespace.
- Other types select primitive domains explicitly, e.g., `authorized_identity` or `network_info`.

### Whole composite and selected fields

The whole-composite override emits multiple headers when the composite
entry is valid. It uses a `composite` selector and field in the match
selector:

```yaml
header_propagation:
  default:
    selector:
      type: none
  overrides:
    - match:
        type: composite
        composite: product_user
      action: propagate
      name: fields
      fields:
        customer: x-customer-id
        workspace: x-workspace-id
```

This form of selector expands into the same configuration as an
explicit match clause for every entry in the composite.

## Context transfer

Context values may be exported and imported between DFE nodes or
between a node and itself, cases such as the topic exporter/receiver
and the durable buffer processor. These cases will declare Pdata Context
transfer bindings during registration:

- Export transfer: these bindings define a function from Pdata Context
  to a flat namespace of Context entry. Each entry is associated with
  the a primitive kind. Composite entries are not exportable.
- Import transfer: these bindings define a function from the flat list
  of name and domain values.

As an example, the following topic declares a Pdata Context transfer
with one entry from each of the five current domains.

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
        selector: {type: network_info, named: [peer_socket_addr]}
      route:
        selector: {type: named, named: ["delivery:route"]}
```

Topic exporter configuration has the option to set `required: true`
for any of the fields. An absent required value causes an immediate
permanent Nack.

Note that the selector `named` list supports multiple names per
selector. This is useful in cases where more than one composite may
supply the value. The `transport_header` domain is the lowest grade of
context entry. The other domains may be degraded into transport
headers, but transport headers cannot be upgraded into other domain
values. In such cases, the exporter will list names referring to
different types. For example:

```yaml
# export a constant or authorized identity into a transport header
dest_name:
  selector:
    type: named
    named:
      - "product:dest_name"               # authorized_identity
      - "internal_normal:destination"     # constant
      - "internal_alternate:destination"  # constant
```

### Context import

Consumers of the transfer context declare a mapping from transfer
names to name using `store_as` clauses.

```yaml
type: receiver:topic
config:
  topic: handoff
  subscription:
    mode: balanced
    group: workers
  context:
    import:
      customer:
        store_as: customer_id
      workspace:
        store_as: workspace_id
      delivery_id:
        store_as: request_id
      route:
        store_as: route_name
```

Like other receivers, the topic receiver will construct Pdata Context
from the transfer context using their bindings, which results in
evaluating composite entry presence as needed by its pipeline.

#### Topic receiver composites

The fields of a topic transfer are used to define new composite
entries.  Topic receivers make use of materialization bindings to
construct new contexts like other receivers, but whereas ordinary
receivers use authorized identity and transport header domains as the
input, topic context-materialization bindings make use of the flat
entries defined for context transfer as their primitive input.

Using this mechanism, composite context entries with `type: transfer`
can be used to derive new conditional composite entries from
transferred context in the receiving pipeline.

To derive conditional transfer composites, use `type: transfer_match`,
which is a conditional element analogous to `transport_header_match`
for the transport header domain. Transfer composite entries include
an additional `topic` field, naming the topic to which they apply.

As an example policy,

```yaml
policies:
  context:
    entries:
      tenant_base:
        - type: transfer
          topic: handoff
          name: customer
        - type: transfer
          topic: handoff
          name: workspace
      tenant_one:
        - type: composite
          name: tenant_base
        - type: transfer_match
          topic: handoff
          name: customer
          value: "1"
      tenant_two:
        - type: composite
          name: tenant_base
        - type: transfer_match
          topic: handoff
          name: customer
          value: "2"
```

Using the `required_in` conext policy, users may configure that a
topic receiver requires a certain composite in order for requests to
pass. Because the engine establishes both the topic receiver and
exporter bindings, the exporter is able to evaluate receiver conditions.
Receiver bindings will only receive requests that meet the conditional
requirements of the listed composites.

```yaml
# This receiver is for tenant one.
bigfish:
  type: receiver:topic
  policies:
    context:
      required_in:
        - tenant_one
  config:
    topic: handoff
    ...

# This receiver is for tenant two.
smallfish:
  type: receiver:topic
  policies:
    context:
      required_in:
        - tenant_two
  config:
    topic: handoff
    ...

# This receiver is for the remaining tenants.
catchall:
  type: receiver:topic
  policies:
    context:
      required_in:
        - tenant_base
  config:
    topic: handoff
    ...
```

## Durable buffer processor

The durable buffer processor will embed both export and import
Pdata Context transfer configuration.

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
    import:
      customer:   # import the configured name, "customer"
      workspace:  # import the configured name, "workspace"
    ...
```

## All captured transfer

A capture policy for all headers:

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-a, x-b, x-c]
          store_as: all_headers
```

All captured transport headers can be transferred

```yaml
config:
  topic: handoff
  context:
    export:
      all_headers:
        selector:
          type: all_captured
```

They can be imported

```yaml
type: receiver:topic
config:
  topic: handoff
  context:
    import:
      all_headers:
```

and propagated:

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
