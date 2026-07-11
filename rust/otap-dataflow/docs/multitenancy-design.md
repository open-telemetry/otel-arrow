# Agent Multitenancy Design

## Background

Two other open-source systems influence this design:

- [Kubernetes multitenant concepts](https://kubernetes.io/docs/concepts/security/multitenancy/)
- [Envoy rate limit configuration](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/other_features/global_rate_limiting.html)

Both of these systems will be familiar to many users and we aim to
keep our concepts close to theirs.

## Definitions

As telemetry collection agents can be deployed to perform in a wide
range of application scenarios, there is no single definition of, or
data model for, a tenant. _Multitenancy_ describes a set of features
for managing tenancy requirements, not a specific aspect of the
dataflow engine.

Tenancy requirements depend on the use-case, covering what resources
are being shared, what needs to be isolated, and the acceptable level
of operational complexity. Tenancy use-cases are often divided into
categories based on the types of relationship between the principal
and the tenant(s):

- **Multiple teams** that share an administrator boundary (e.g., divisions
  in a company). These are usually small in number, tenants are
  cooperative and share administrative control.
- **Multiple customers** of a SaaS sharing a service endpoint have a
  contractual relationship, compete for shared service resources, and
  may be large in number.

Sometimes there will be more than one concept of tenancy in use at a
time (e.g., SaaS customer account and signed-in user). Sometimes
multitenancy is applied at multiple levels (e.g., both thread-local
and global rate limits).

## Scope

This document covers the design for tenant identity: how the engine
recognizes a tenant and carries that identity through the pipeline. In
scope are:

- Configuration model for tenant identity
- How tenant tokens integrate with the configuration and data model
- Conditional evaluation that resolves tenant tokens in O(1) time

The isolation mechanisms that consume tenant identity, including the
limiter policies and the CPU, memory, and operating-system mechanisms
behind them, are covered in the companion [multitenancy
overview](./multitenancy-overview.md).

## Request and tenant context

System operators require multiple tenants to share pipeline
resources, with fine-grained limits configured to achieve isolation
and fairness between tenants. Tenant identification depends on the
use-case and what is being shared. How the engine identifies a tenant
is a matter of choice. It is often possible to express tenancy
directly through configuration, by associating specific resources with
specific tenants in whole terms. As an example, by giving each tenant
a dedicated port and pipeline, they can physically separate tenants
without the use of specific multi-tenant features. By isolating
whole-ports, whole CPUs, or whole threads to a single tenant or tenant
group, the operator may be able to avoid using tenant tokens
entirely.

However, where there are context-specific attributes used for
limiting, a **tenant token** will be used. A tenant token is
a small vector of key:value identifiers which represent the current
tenant in some context. Usually, the context is a request, however,
custom contexts can be defined as long as a tenant token can be
stored.

Tenant tokens are multi-dimensional to enable tenant assignment
in logically independent ways with user control over cardinality. For
example, a request may be accompanied by an environment name, a
project name, and a user name for three forms of tenancy. Choosing
three forms means a single limiter table with three-dimensions.

Multiple tenant tokens per request are supported, enabling
multiple independent limiters with lower dimensions. For example, a
request may be accompanied by both an end-user tenant identifier and
an acting-on-behalf-of tenant identifier. These are logically
independent, two single-dimensional limiters.

Multiple tenant tokens can be used to express various forms of a
single tenant identity. When there are multiple sets of header
conventions that describe a tenant (e.g., "modern" and "legacy"), they
are listed as alternatives. Generally, the alternatives should be
non-overlapping, but the behavior is well-defined either way: all
tokens of the request must match all the conditions of the
limiter.

To create a new request context, tenant token extractors are
applied; we take the union of key:values of the resolved tenant
tokens and precompute a table lookup key for every distinct set
of condition entries defined in the engine. Later in the pipeline,
these precomputed table-lookup keys will be used to evaluate limiter
conditions in **O(1)** time.

To evaluate which limiter bucket a given request falls in to, the
sequence of conditions is applied in order. For a context to match a
condition, all its entries must match for all tenant token values
to qualify.

### Model terms and request flow

The model uses the following terms, which are developed in more detail below.

| Term            | Definition                                                                                                                                       |
|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| Tenant token    | A list of extractors, possibly conditional, that determine a resolved token value. These can be defined at several levels for restricted access. |
| Token extractor | A configured rule that extracts and/or matches one token key from a request.                                                                     |
| Token value     | The set of keys and values forming a resolved token, with a number of associated table lookup keys.                                              |
| Key / value     | A single dimension of a tenant token value.                                                                                                      |
| Entry           | A single `{ key, value }` term within a condition; no `value` means wildcard.                                                                    |
| Condition       | An ordered list of entries selecting a bucket; the first matching condition wins.                                                                |
| Bucket          | Contains one distinct limiter instance per distinct set of entry keys for the matching condition, up to the cardinality limit.                   |

Token extractors are evaluated and the matching results are placed
in the request context where it passes with the request data in a
pipeline where individual nodes will apply specific limiters.

![A flow diagram showing how tenant token values are computed and flow through the pipeline](./multitenancy-diagram.svg)

### Tenant token extractors

Tenant tokens originate in a series of extractors, configured at the
top level, group-level, and pipeline-level of the dataflow engine
configuration, since these definitions are shared across pipeline
groups. Token values refer to one or more tenant associations,
each association consisting of multiple keys and values and used as a
lookup key for evaluating limiter conditions. Tenant token values
must be erased when they go out of scope to avoid leaking sensitive
data across unintended boundaries.

Token extractors are applied by receivers (or processors) when they
originate new request contexts. They support attribute
extraction in a variety of ways, for example,

- `receiver_id`, `source_node_id`: the first node or preceding node traversed by the request.
- `remote_address`, `masked_remote_address`: origin network address, optionally CIDR-masked.
- `generic_key`: a static, hard-coded key-value.
- `transport_header`: copy a transport header value into a token key.
- `transport_header_match`: token key value must match a condition.

Here, the term _transport header_ is generic. Although we model these as
HTTP headers, receivers for other protocols are responsible for
deriving headers according to the protocol that is in use, using
semantic conventions as needed. The engine reserves the token key
`signal` to identify the type of signal in the request (e.g., "logs").

### Tenant token examples

Here is an example of a single token extractor list based on the
client address, a hard-coded value, and an extracted transport header as
the keys:

```yaml
extractors:
- key: client_address
  remote_address: {}     # use the network peer's socket address as the value
- key: route_name
  generic_key: http/otlp
- key: user_id
  transport_header: x-user-id
```

Here is an example engine configuration defining three tenant
tokens: two end-user forms (modern and legacy) and one
acting-on-behalf-of service token.

```yaml
# Engine top-level tenant tokens, shared across all pipeline groups.
tenant_tokens:
  enduser_tenant_modern:
    extractors:
    - key: customer_id
      transport_header: x-customer-id
    - key: workspace_id
      transport_header: x-workspace-id
    - key: tier
      transport_header: x-customer-tier
  enduser_tenant_legacy:
    extractors:
    - key: customer_id
      transport_header: x-legacy-customer-id
    - key: workspace_id
      transport_header: x-legacy-workspace-id
    # legacy tenants are identified by a version header
    - transport_header_match: x-protocol-version
      value: "very-old"
  onbehalfof_tenant:
    extractors:
    - key: service_id
      transport_header: x-service-id
    - key: subscription_id
      transport_header: x-subscription-id
groups:
  # ... pipeline groups reference the tokens above ...
```

### Engine tenancy support

The Dataflow Engine's pipeline data type (`OtapPdata`) will be
modified to propagate tenant tokens in the context. Receivers and
processors that create new contexts will be upgraded to evaluate the
applicable tenant token extractors based on their transport headers.
When all extractor conditions succeed for a token and request, the
resulting tenant token value is entered into the context.

Any kind of node can apply a limit. When the engine starts, it applies
the resolved limit policies at the nodes and chokepoints where they
take effect. The engine implements the standard limits itself and
provides helpers to apply them in a uniform way, so most limits require
no custom code.

### Resource-level tenants

At the top of the OpenTelemetry data model, the Resource value
describes a single producer of telemetry. When a pipeline request
contains data for a single resource, as typically produced by an OpenTelemetry SDK,
the request's resource attributes can be made to act like transport headers as the input
for tenant token extractors.

Receivers for non-HTTP telemetry protocols that convey single-resource
requests may apply token extractors directly to resource
attributes. Limiters that are applied in those contexts may refer to
single-resource token extractors `resource_attribute` and
`resource_attribute_match` to extract or conditionally extract tenancy
information. These extractors can only be applied in single-resource
contexts. For example to extract a `service_name` tenant for the
production namespace in a single-resource context:

```yaml
extractors:
- key: service_name
  resource_attribute: service.name
- resource_attribute_match: service.namespace
  value: production
```

These extractors do not apply in contexts where multiple resources are
present. Under this proposal, the options for handling multi-resource
requests with these resource token extractors are:

- Nack the request.
- Reject the configuration: callers asserts single-reource context or else
  invalid configuration.
- Do not resolve the extractor, the token will be unresolved; receivers
  and limiters that require this token will reject, otherwise these
  requests will not match conditions and take the default limit.

Routing and batching by tenant token based on resource attributes
generalizes in useful ways. Telemetry collection agents are sometimes
required to aggregate by both tenant-token property and
sub-characteristic such as metric name or TraceId. In general, the
engine must add support splitting requests in these ways to enable
routing, shuffling, grouping and load balancing by tenant token.

### Tenant trust

Pipeline operators are responsible for secure tenant
configurations. Tenant tokens can be defined at multiple levels
of the engine configuration (e.g., global, pipeline group, pipeline)
to avoid leaking tenant details outside their scope. The dataflow
engine is responsible for enforcing this discipline automatically.

Pipeline operators are advised not to use unauthenticated request
headers.

Pipeline operators are advised to configure sensible cardinality
limits to protect the pipeline.

### Routing/batching by tenant token

Processors (e.g., fanout) and exporters (e.g., topic) will be
configurable to route by token condition.

```yaml
nodes:
  tenant_split:
    type: processor:tenant_router
    config:
      optional_tenant_tokens: [enduser_tenant_modern, enduser_tenant_legacy]

      # first-match wins
      routes:
      # customer_id=bigfish
      - entries:
        - key: customer_id
          value: bigfish
        output: bigfish_pipeline
      # for premium-tier customers
      - entries:
        - key: tier
          value: premium
        output: premium_pipeline
      # for any customer
      - entries:
        - key: customer_id
        output: shared_pipeline
      default_output: fallback

    outputs:
      bigfish_pipeline: {...}
      premium_pipeline: {...}
      shared_pipeline: {...}
      fallback: {...}
```

The same applies to the batch processor specifically. Note that
[OpenTelemetry Collector supports batching by selected transport headers
using
`sending_queue::batch::partition::metadata_keys`](https://github.com/open-telemetry/opentelemetry-collector/blob/main/exporter/exporterhelper/README.md#sending-queue-batch-settings)
and a configurable cardinality limit. Tenant tokens and batching
conditions will be used to this effect in the dataflow engine.

## Integration with policies

Tenant tokens and their conditions are consumed by the engine's limiter
policies, which apply rate and resource limits per tenant and per scope.
The limiter-policy model, together with the CPU and memory isolation
mechanisms that build on it, is described in the companion [multitenancy
overview](./multitenancy-overview.md). The remainder of this document
covers how tenant identity is resolved efficiently.

## Implementation details

The tenancy features described above have an efficient, table-driven
implementation. It runs in three steps: a one-time build step, then
two per-request steps: context creation and limiter lookup. Using
these structures, the algorithm reduces to O(1) work per condition. We
assume the use of a fixed-size hash function over ordered key:values,
as in a database group-by hash-join operation.

### Compile tokens and conditions

When loading the engine configuration, produce static data
structures to accelerate the two steps that follow. First, from
the pipeline graph structure, determine a reachability mapping
for all nodes, indicating whether a context created at any given
node can reach any other limiter in the engine.

Next, partition the token extractors into two groups. Conditional
extractors are those whose key is tested by some reachable condition;
their presence and value will resolve the tokens, and they are
indexed here. Wildcard extractors that require a key without a specific
value are indexed to accept all values. Remaining extractors are deferred
until the matching tokens are known.

The index is a map by header key; any-value cases are resolved here,
then a second map by header value. The any-value and specific-value
cases are identical, each a list of _(token, extractor)_ slots that the
corresponding header satisfies.

Finally, precompute the token-by-condition cross-product: for
every token and every reachable condition (the conditions of
limiters reachable from the node), the projection that yields that
condition's table-lookup key.

### Context creation step

Using the structures above, a receiver builds the token values
for a request:

- Take the union of tokens over the applicable limiters. Initialize a
  candidate vector indexed by token, each entry a bit-vector with a
  1-bit for every unsatisfied token key.
- For each header key/value on the request, look up the key, consider
  any-value case, then the specific case. For each (token,
  extractor) slot listed, clear that token's corresponding bit.
- A token whose bit-vector reaches 0 has all keys present and
  matching: construct its value, and with it the precomputed lookup
  key for each reachable condition that references it. A token
  that never reaches 0 is unresolved for this request.

### Limiter lookup step

The context value carries, for each resolved token, a precomputed
lookup key per reachable condition. When a request arrives, each
limiter scans its conditions and performs one table lookup per
condition, per bound token, using the precomputed keys.

### Algorithm Analysis

Let `D_recv` be the number of tokens in the node, `C_reachable`
the reachable limiter conditions from that node, `H` the header count,
and `k_d`, `k_c` the token and condition key counts. A condition
applies to a token only when the token's schema covers the
condition's keys. Let `P` be the number of applicable (token,
condition) pairs: `C_reachable <= P <= D_recv * C_reachable`, the
upper bound reached only when a limiter binds multiple same-schema
tokens.

The space used in this approach:

- resolved token values: `O(D_recv * k_d)`
- precomputed lookup keys: one fixed-size fingerprint per applicable pair,
  total size `O(P)`.

The time used in this approach:

- context creation: `O(H + D_recv * k_d)` to resolve tokens and
  materialize values, plus `O(P * k_c)` to project and hash one lookup
  key per applicable pair
- limiter lookup: one `O(1)` table probe per applicable (token,
  condition) pair, `O(P)` total, independent of `k_d` and `k_c`,
  because projection and hashing cost were paid at context creation.

## Open questions

These things have been considered out of scope:

- Live reconfiguration of tenant tokens has not been considered.
- The topic exporter/receiver will be potentially responsible for
  erasing and/or recomputing tenant tokens as they cross scope
  boundaries. This was not covered above.

Open questions about the limiter policies and the CPU and memory
isolation mechanisms are tracked in the companion [multitenancy
overview](./multitenancy-overview.md).
