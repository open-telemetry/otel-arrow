# Pipeline Data Context

**Status:** Draft

## Overview

A **Pdata Context** is message-scoped metadata attached to an
`OtapPdata` and carried through the pipeline. We use a policy
declaration language consisting of optional **context entries**
containing various structured data. This design presents features
meant to address requirements in multitenant environments

The use of context entries is an optional feature of the DFE, and the
explicit configuration mechanisms discussed here can be safely omitted
in configurations that do not require them. Configuration that
interacts with context entries remains in the associated domains, for
example:

- Authorized identity entries can be placed into context entries using
  authorization configuration. This design calls for new
  `authorized_identity` policies to set context entries from standard
  information returned by authorization extensions.
- One or more transport headers can be placed into context entries
  using transport header configuration. The existing
  `policies::transport_headers` section already has a mechanism for
  defining context entries though its `store_as` verb.
- Network information such as source address can be placed into 
  context entries using network configuration, and so on.

For users with multitenancy requirements, we introduce a form of
**composite context entry** capable of binding multiple context
entries into a single context entry. Composite context entries are
subject to conditions, so these entries are conceptually present or
absent. When present composite entries are defined by multiple
primitive context entries. A new section `entries` will be introduced
under `policies::context` for declaring user-defined and
builtin-function context entries.

Users will not be required to configure or learn underlying concepts
used in the implementation of context entries. This design includes
the outline of a technical approach that encodes the set of context
entries in a compact byte array including an index for fast lookup.

## User stories

The term "tenant" is used in these stories to describe use-cases for
the context entries and composite context entries. No surface of the
DFE configuration refers to tenants--there is no tenant configuration
here--instead the concept of tenancy derives from however the Pdata
context is used.

### User A(uthenticated)

As a DFE user with untrusted inputs, I want to configure a telemetry
pipeline with authenticated credentials with strong security
controlled through bearer tokens (or mTLS certificates). Authorization
is configured to return specific claims tied to access roles. A single
context entry carries the authorized claim value, corresponding with a
tenant in this scenario. I may want to:

- Route messages to tenant-specific pipelines
- Batch messages into single-tenant batches
- Store messages in tenant-specific volumes
- Limit concurrent messages by tenant
- Limit message memory usage by tenant
- Limit message rate of bytes/items/size/message by tenant
- Observe pipeline metrics by tenant.

### User B(asic)

As a DFE user in a trusted environment, I want to configure a pipeline
using a single HTTP header to easily identify the source of the
message. A single content entry holds the selected transport header,
corresponding with a tenant in this scenario.  I have all the same
desires as User A but fewer security requirements.

In exchange for operational simplicity, the user understands accepts
that transport headers are not secured by the authorization extension.

### User C(omplex)

As a DFE user, I want to configure authenticated credentials like User
B, then I want a simple subdivision of telemetry data into separate
units using HTTP headers like User A. I have all the same desires as
User A, but with two content entries that identify messages in the
pipeline. In this scenario, tenants are identified by one
authenticated credential entry and one HTTP header entry. In addition
to the A/B desires, I may want to:

- Configure routing differently by tenant
- Configure batching differently by tenant
- Configure storage differently by tenant
- Configure concurrent message limits differently by tenant
- Configure memory limits differently by tenant
- Configure message rates differently by tenant

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
- Configure routing/batching/storage/limits grouped by context entry vaules
- Configure routing/batching/storage/limits with a set of conditions over context entry values

### User L(oad-balance) and P(artition)

As a DFE user with a data processing telemetry pipeline, I want
control to split and join messages with control over partitions based
on load characteristics, by distinct values, or by ranges of values.
I have the desires as A/B/C/G, in addition I may wish to:

- Route messages by using dimension(s) of the context entries for load balance
- Subdivide messages using aspects of the data and assign new context entries
- Batch by continuous range(s) of an aspect of the data (e.g., TraceID) for optimization
- Batch by properties of the OpenTelemetry resource
- Partition by property of the items (e.g., metric_name) to ensure consistency.

While these partitioning steps are orthogonal from context, the
partition function must support configuration that preserves context
entries in its output. For example, the partition procsesor
configuration will support partitioning by metric name, TraceID, and
other aspects including context entry values, so that the partition
function produces the necessary context for the next node in the
pipeline.

## Configuration model

### Transport header context entries

The transport headers model is the basis of this design, it includes a
`store_as` designator and use of `type: named` for propagating named
context entries as in this design. The configuration structures for
transport headers are preserved unmodified.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_regexp: "x-info-.+"    # Capture unnamed headers
        - match_names: [x-request-id]  # Capture a named header
          store_as: request_id         # Header into context entry

    header_propagation:
      default:
        selector:
          type: named
          named: [request_id]          # Refer to a context entry
        action: propagate
        name: preserve                 # Original name preserved
      overrides:
        ...
```

### Authorized Identity context entries

The new policies match `otap_df_engine::capability::auth::models::authorized_identity`.

```yaml
policies:
  authorized_identity:
    - claim: xyzmon-acct
      store_as: customer_id            # Claim into context entry
```

Context entries can be defined at the pipeline group or the engine
level. Pipeline group context entry names should not conflict with
engine-level context entry names.

### Composite context entries

A composite context entry combines multiple context entries. These
can be used as the basis of multi-dimensional context.

```yaml
policies:
  context:
    entries:
      # A product user consists of two context entries.
      product_user:                      # Composite name
        - type: authorized_identity      # Authorization claim
          name: customer_id              # Claim entry name
        - type: transport_header         # Transport header
          name: workspace_id             # Header entry name
```

The composite entry defined above might be useful to in a batch
processor configuration, for example,

#### Conditional composite context entries

A composite context entry combines multiple context entries with
optional conditional elements. These entries are only defined when
all the composite conditions are met. For example:

```yaml
policies:
  context:
    entries:
      # A product user consists of two context entries
      # under a condition that xyz_environment=production.
      product_user:                      # Composite name
          ...                            # Two entries as above
        - type: transport_header_match   # Condition
          name: xyz_environment          # Header entry name
          value: production              # Match value
```

#### Special context entry types

Singleton context entries can be defined in the same way, for example,

```yaml
policies:
  context:
    entries:
      origin_address:           # Named context entry
        - type: network_info
          name: peer_socket_addr
```

More entry types can be added as needed, for example `randomness`,

```yaml
policies:
  context:
    entries:
      idempotency:              # Named context entry
        - type: randomness
          value: uuid7
```

Or `constant`,

```yaml
policies:
  context:
    entries:
      receiver:                 # Named context entry
        type: constant
        value: otlp-http-json
```

#### Composite context entry dimensions

An composite context entry listing N keys is said to have N
dimensions. Placing multiple fields into a composite with N>1 as
opposed to the use of multiple independent entries enables fine
configuration control. For example, with two fields we have several
options:

- Two independent entries, two independent dimensions
- Composite entry, coordinates have two dimensions
- Take both, a one-dimensional entry and a two-dimensional entry.

The choice lets users determine the cardinality of the context entry
values used in a pipeline, for example to form batches using two dimensions 
while rate-limiting in one dimension, ignoring the other.

Generally, when referring to a composite entry by a specific
associated entry, the engine supports syntax `composite:entry`. For
example, `product_user:customer_id` refers to the `customer_id` entry
associated with the `product_user` composite.

```yaml
policies:
  context:
    entries:
      # A product user consists of two context entries.
      product_user:                      # Composite name
        - type: authorized_identity      # Authorization claim
          name: customer_id              # Claim entry name
        - type: transport_header         # Transport header
          name: workspace_id             # Header entry name
      produce_account:                   # Name of entry
        - type: authorized_identity      # Authorization claim
          name: customer_id              # Claim entry name
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
                    entry: product_user:customer
                    value: acme
                  with:
                    otap:
                      min_size: 10000
                      sizer: items
                - source:
                    entry: product_user
                  with:
                    otap:
                      min_size: 1000
                      sizer: items
          route:
            type: processor:router
            config:
              routes:
                - source:
                    entry: product_account:workspace_id
                    value: acme
                  destination: dedicated
                - source:
                    entry: product_account
                  destination: shared
```

### Entry requirements

Another policies section controls the context entries the are required
at certain nodes in the pipeline. These policies can be defined at the
engine, group, pipeline, or node level. For example, to indicate that
the `product_account` entry is required at while the `product_user`
entry is optional:

```yaml
policies:
  context:
    required_in:
      - node: receiver0
        entry: product_account
    optional_in:
      - node: receiver1
        entry: product_user
```

or for example, to indicate that a processor must produce a certain 
output context entry.

```yaml
policies:
  context:
    required_out:
      - node: batchproc0
        entry: product_account
```

## Developer interfaces

In this section, we describe the interfaces that nodes will use to
bind to context entries. These describe the different kinds of
contracts between Pdata context and its users.

### Pdata context sources

For receivers and for processors that form new Pdata context values, a
**Pdata source binding** is registered. The relevant optional and
required context policies will be evaluated through an
interface. Multiple methods of formation may be provided, for example
the receiver that constructs new Pdata context values from arriving
messages will use a different binding than a processor that partitions
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

For exporters that produce external message context encoding from
Pdata context entries, use the **Pdata sink binding**, (e.g., as done
in `header_propagation`) with `type: context_entry`.

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
                      type: context_entry
                      entry: origin_addr
                  overrides: 
                    ...
```

This sort of binding used here will support the node in injecting
Pdata context entry fields into outgoing messages.

### Pdata context predicates

A variety of nodes will configure behavior based on Pdata context
entries for partitioning, load-balancing, table lookup and various
applications of per-entry configuration.

These bindings will refer to entries in effect for the nodes. Such an
interface will support fast table lookup and equality checking.

```yaml
TODO e.g.
```

### Conslusion

The users A/B/C/G/L/P each meet their needs, here.
