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
            stored_names: [request_id]
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
  capitaliztaion) or `stored_name` for the name configured by `store_as`.

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

### Propagation limited to one selector

Propagation rules `default` and `override` apply to each captured
header, the first matching override wins. By design, current propagation
policies apply only to transport header names. 

This document proposes to leave the existing `default` and `override`
policies in place while migrating to a new form of rule-based selector
list, `rules`:


```yaml
header_propagation:
  rules:
    # propagate transport headers by stored_name 
    # workspace_id and environment.
    - selector:
        type: named
        named:
          - workspace_id
          - environment
      action: propagate
      header: 
        name: stored_name

    # propagate authorized identity using x-customer-id
    - selector:
        type: authorized_identity
        named:
          - customer_id
      action: propagate
      header:
        name: literal
        literal: x-customer-id

    # propagate a randomness value as x-idempotency
    - selector:
        type: named
        named:
          - delivery:id
      action: propagate
      header:
        name: literal
        literal: x-idempotency

    # propagate all captured headers
    - selector:
        type: all_captured
      action: propagate
      header: 
        name: preserve
```

### Proposed selectors

New selectors support the missing operations:

<!-- markdownlint-disable MD013 -->
| Selector                                            | Meaning                                                                                       |
|-----------------------------------------------------|-----------------------------------------------------------------------------------------------|
| `{type: all_captured}`                              | _Existing:_ All captured transport headers are propagated.                                    |
| `{type: named, named: [workspace_id]}`              | _Existing:_ Select specific transport headers by unqualified name.                            |
| `{type: named, named: ["product_user:workspace"]}`  | _Extend:_ Only the transport header domain is supported, extend to match any composite entry. |
| `{type: authorized_identity, named: [customer_id]}` | _New:_ Export an authorized-identity field.                                                   |

The selection logic is: 

- `all_captured` applies only to transport headers
- `named` applies to transport headers or whole composites (unqualified name) or composite entries (qualified names). This uses the stored name (all varieties) or preserved name (transport headers only).
- Other names refer to non-transport-header primtive domains, e.g., `authorized_identity` or `network_info`.

### Whole composite and selected fields

The whole-composite rule emits multiple n headers, only when `product_user` is present:

```yaml
header_propagation:
  rules:
    - selector:
        type: composite
        name: product_user
      header_name:
        fields:
          customer: x-customer-id
          workspace: x-workspace-id
```

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
          required: true
        delivery_id:
          type: randomness
          required: true
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
        selector: {type: authorized_identity, named: [customer_id]}
      delivery_id:
        selector: {type: named, named: ["delivery:id"]}
      workspace:
        selector: {type: named, named: ["product_user:workspace"]}
      source_addr:
        selector: {type: named, named: ["source_addr"]}
      route:
        selector: {type: named, named: ["delivery:route"]}
```

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

## Durable buffer processor

The durable buffer processor will embed both export and import
Pdata Context transfer configuration.

```yaml
type: processor:durable_buffer
config:
  context:
    partition_by:
    export:
      customer:
        selector: {type: authorized_identity, named: [customer_id]}
      workspace:
        selector: {type: transport_header, named: [workspace_id]}
    import:
      customer:   # import the configured name, "customer"
      workspace:  # import the configured name, "workspace"
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
  rules:
    # proagate all transport headers in the context
    - selector:
        type: all_captured
      action: propagate
      header: 
        name: preserve
```

## Related context

This RFC is part of a series beginning with [RFC
0004](./0004-pdata-context.md), the ongoing work described in
[#3872](https://github.com/open-telemetry/otel-arrow/issues/3872).
