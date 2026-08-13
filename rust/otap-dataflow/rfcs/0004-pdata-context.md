# Pipeline Data Context

**Status:** Draft

## Overview

A **Pdata context** is request-scoped metadata attached to an
`OtapPdata` and carried through the pipeline. We use a policy
declaration language consisting of optional **context fields** and
**identities**.

Context fields hold individual or multiple values, including transport
headers, authorized identity fields, and other contextual information
(e.g. peer IP address, idempotency key, constant
information). Identities holds qualified sets of specific fields used
to distinctly identify the request. Requests carry multiple identities.

A **Pdata context compiler** computes a set of tables describing the
engine configuration, containing information needed by:

- **extractors** which configure how to form a context field from a carrier
- **injectors** which configure how to place a context field into a carrier
- **predicate** which can parition or condition by identities and fields.

For an efficient implementation, nodes and extensions are required to
declare policy bindings in their respective configuration areas. For
example, the batch processor after validating its configuration will
bind an identity policy. The Pdata context compiler, taking all of the
policy bindings together, computes or recomputes the tables used in
the implementation provided for the binding.

## User stories

### User A(uthenticated)

As a DFE user with untrusted inputs, I want to configure a telemetry
pipeline with authenticated credentials with strong security
controlled through bearer tokens (or mTLS certificates). Authorization
is configured to return specific claims tied to access roles. Tenants
are identified by a single authorized claim value. I may want to:

- Route requests to tenant-specific pipelines
- Batch requests into single-tenant batches
- Store requests in tenant-specific volumes
- Limit concurrent requests by tenant
- Limit request memory usage by tenant
- Limit request rate of bytes/items/size/request by tenant
- Observe pipeline metrics by tenant.

### User B(asic)

As a DFE user in a trusted environment, I want to configure a
multitenant pipeline using a single HTTP header to easily identify the
source of the data.  I have all the same desires as User A but fewer
security requirements.

In exchange for operational simplicity, the user understands accepts
that transport headers are not secured by the authorization extension.

### User C(omplex)

As a DFE user, I want to configure authenticated credentials like User
B, then I want a simple subdivision of telemetry data into separate
units using HTTP headers like User A. I have all the same desires as
User A, but with two fields that identify requests in the
pipeline. Tenants are identified by one authenticated credential and
one HTTP header. In addition to the A/B desires, I may want to:

- Configure routing differently by tenant
- Configure batching differently by tenant
- Configure storage differently by tenant
- Configure concurrent request limits differently by tenant
- Configure memory limits differently by tenant
- Configure request rates differently by tenant

### User G(ateway)

As a DFE user acting as a gateway or proxy, I want a configuration
that forwards telemetry data while passing other transport headers,
possibly with fanout to different destination based on tenant
profile. Tenants are defined as with A/B/C, both complex and
differentiated. I may wish to:

- Assign dedicated CPU and memory resources to individual tenants
- Assign dedicated CPU and memory resources to shared groups of tenants
- Pass through and/or rename a specific list of HTTP headers
- Pass through a wildcarded list of HTTP headers
- Configure routing/batching/storage/limits grouped using dimension(s) of the identity(ies)
- Configure routing/batching/storage/limits with a set of conditions over dimension(s) of the identity(ies)

### User P(artition)

As a DFE user with a data processing telemetry pipeline, I want
control to split and join requests with control over partitions.  I
have the same desires as A/B/C/G, in addition I may wish to:

- Route requests by using dimension(s) of the identity(ies) for load balance
- Subdivide requests using aspects of the data and assign them new identities
- Batch by continuous range(s) of an aspect of the data (e.g., TraceID) for optimization
- Batch by properties of the OpenTelemetry resource
- Partition by property of the items (e.g., metric_name) to ensure consistency.

## Configuration model

### Transport headers

The transport headers model is the basis of this design, it includes a
`store_as` designator and use of `type: named` for propagating named
context fields as in this design. The configuration structures for
transport headers are preserved unmodified.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_regexp: "x-info-.+"    # Capture unnamed headers
        - match_names: [x-request-id]  # Capture a named header
          store_as: request_id         # Header into context field

    header_propagation:
      default:
        selector:
          type: named
          named: [request_id]          # Refer to a context field
        action: propagate
        name: preserve                 # TODO: clarify
      overrides:
        ...
```

### Identity

Identities can be defined at the pipeline group or the engine
level. Pipeline group identity names should not conflict with
engine-level identity names.

#### Identity fields

An identity is an optional set of fields, of varying predefined
types. Identities are named. Identity features are not required and
cost nothing not used.  Some common field types are `transport_header`
and `authorized_claim`:

```yaml
policies:
  identity:
    customer_workspace:                # Name of identity
      fields:
        customer:
          type: authorized_claim       # Authorization claim
          name: customer_id            # Claim name
        workspace:                     # Name of field
          type: transport_header       # Transport header
          name: workspace_id           # Header name
```

#### Conditional identity fields

Fields may include conditional elements. If a field definition fails
due to any condition, the identity will not be defined in the context,
for example:

```yaml
policies:
  identity:
    <name>:
      ...
      fields:
        environment:                          # Named identity field
          type: transport_header_match
          name: environment                   # Named transport header
          match_values: [production, staging]
```

#### Other identity fields

Another useful field type is `peer_address`, for applying generic
labels:

```yaml
policies:
  identity:
    <name>:
      ...
      fields:
        peer:                     # Named identity field
          type: peer_address
```

New field types can be added as needed, for example `randomness`,

```yaml
policies:
  identity:
    <name>:
      ...
      fields:
        idempotency:              # Named identity field
          type: randomness
          value: uuid7
```

and more, in particular derived from `AuthorizedData`.

#### Identity dimensions

An identity listing N keys is said to have N dimensions. Placing
multiple fields an identity with N>1 as opposed to the use of multiple
identities enables fine configuration control. For example, with two
fields we have several options:

- Two independent identities, one field each
- Single identity, two dimensions
- Take both, a one-dimensional identity and a two-dimensional identity.

The choice lets users determine the cardinality of the identity at the
point(s) where it is used. As an example of the "both" case, the batch
processor can form batches over two dimensional while a rate limiter
or router in the same pipeline distinguishes over a single dimension.

```yaml
policies:
  identity:
    customer_workspace:                # Name of identity
      fields:
        customer:
          type: authorized_claim       # Authorization claim
          name: customer_id            # Claim name
        workspace:                     # Name of field
          type: transport_header       # Transport header
          name: workspace_id           # Header name
    customer_wide:                     # Name of identity
      fields:
        account:
          type: authorized_claim       # Authorization claim
          name: customer_id            # Claim name
groups:
  default:
    pipelines:
      main:
        nodes:
          batch:
            type: processor:batch
            config:
              partition_by:
                - source:
                    identity: customer_workspace
                    field: customer
                    value: acme
                  with:
                    otap:
                      min_size: 10000
                      sizer: items
                - source:
                    identity: customer_workspace
                  with:
                    otap:
                      min_size: 1000
                      sizer: items
          route:
            type: processor:router
            config:
              routes:
                - source:
                    identity: customer_wide
                    field: account
                    value: acme
                  destination: dedicated
                - source:
                    identity: customer_wide
                  destination: shared
```

### Identity requirements

Another policies section controls the identities the are required at
which nodes in the pipeline. These policies can be defined at the
engine, group, pipeline, or node level, the most-specific wins. For
example, to indicate that the `customer_wide` identity is required at
while the `customer_workspace` identity is optional:

```yaml
policies:
  pdata_context:
    required: 
      - source: 
          identity: customer_wide
    optional: 
      - source: 
          identity: customer_workspace
```

In this example, repeating the batch processor from above

```yaml
groups:
  default:
    pipelines:
      main:
        nodes:
          batch:
            type: processor:batch
            config:
              partition_by:
                - source:
                    identity: customer_workspace
                    field: customer
                    value: acme
                  with:
                    ...
                - source:
                    identity: customer_workspace
                  with:
                    ...

              # The default configuration is reached when 
              # customer_workspace is not defined.
              otap:
                min_size: 100
                sizer: items
```

## Developer interfaces

In this section, we describe the interfaces that nodes will use to
bind to context fields and identities. These describe the different
kinds of contracts between Pdata context and its users.

### Pdata context sources

For receivers and for processors that form new Pdata context values, a
**Pdata source binding** is registered. The relevant optional and
required identity policies will be evaluated through an
interface. Multiple methods of formation may be provided, for example
the receiver that constructs new Pdata context values from arriving
requests will use a different binding than a processor that partitions
by a specific context field.

For receivers, the source address, the HTTP headers, and the
`AuthorizedData` value will be passed to the binding.

```rust
TODO e.g.,
```
In other cases, where there is an incoming Pdata context being
extended or projected, the input Pdata context will be used
through a different binding method.

```rust
TODO e.g.,
```

### Pdata context sinks

For exporters that produce external request context encoding from
Pdata context fields and identities, use the **Pdata sink binding**,
(e.g., as done in `header_propagation`) with `type: identity`.

```yaml
groups:
  default:
    pipelines:
      main:
        nodes:
          batch:
            type: exporter:otlp
            policies:
              transport_headers:
                header_propagation:
                  default:
                    action: propagate
                    selector:
                      type: identity
                      identity: network
                      field: peer_addr
                  overrides: 
                    ...
```

This sort of binding used here will support the node in injecting
Pdata context identity fields into outgoing requests.

### Pdata context predicates

A variety of nodes will configure behavior based on Pdata context
identities and fields for partitioning, load-balancing, table lookup
and various applications of per-tenant configuration.

These bindings will refer to fields and identities in effect for the
nodes. Such an interface will support fast table lookup and equality
checking.

```yaml
TODO e.g.
```

### Conslusion

The users A/B/C/G/P each meet their needs, here.
