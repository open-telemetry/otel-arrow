# Multitenancy Limiter Policies

**Status:** Draft

## Limiter policies

Limits are declared as policies under `policies.resources`, alongside
the process-wide `memory_limiter`. Because policies are already
hierarchical, every limit inherits the engine's policy resolution:
top-level defaults are overridden by pipeline-group policies, which are
in turn overridden by pipeline policies, with precedence applied by
family. The policy hierarchy and its resolution rules are defined in
[configuration-model.md](configuration-model.md). Limits come in two
categories, held in two policy families:

- **Rate limits**, under `policies.resources.rate_limiters`, count
  resources that are limited as a function of time. When the resource
  is not available, the caller can choose to wait a definite amount of
  time, provided they hold a reservation. These resources are consumed
  immediately and not returned by the caller.
- **Resource limits**, under `policies.resources.resource_limiters`,
  count resources that are limited by a current total. When the
  resource is not available, the caller can choose to wait indefinitely
  for the resource to be returned. These apply anywhere in the engine
  there is a resource held in-use by ongoing work, such as queues,
  batches, and topics.

Both kinds of limit can be used with different weight measures, for
example we can limit by request count, by in-memory bytes count, by
compressed bytes count, or by items of telemetry. Rate and resource
limits have distinct runtime interfaces, and of course use different
configuration; however, they generally use the same model for
multitenancy and share a common policy schema:

- **unit**: Describes the units of weight being limited. For rate
  limits this must end with "/second", for example
  `request_count/second` or `memory_bytes/second`. For resource limits,
  omit the rate suffix, for example `memory_bytes`. In the yaml
  configuration, this gives the raw numbers meaning; in the code, a
  verification step ensures that each limit is applied to the correct
  category of weight.
- **tenant_tokens**: Optional, a list of the tenant tokens used by the
  limiter that must be present, otherwise the request is immediately
  failed. These token values are extracted from the request and used
  to evaluate conditions.
- **conditions**: A list of conditions, each of them defined by a name
  and a list of entries with a bucket-specific limit. When all the entries
  are satisfied for all the input tokens, the conditional limit
  is chosen. The first matching condition is selected, otherwise a
  default is used.
- **cardinality**: Determines the limit of unique combinations for
  buckets in the limiter and what happens when the number is exceeded.

The engine provides a built-in implementation for each category, so
the common case needs no custom code: a token bucket for rate limits
and a semaphore for resource limits. A policy selects the built-in by
naming its specific configuration block, so a `token_bucket` block
selects the token-bucket rate limiter and a `semaphore` block selects
the semaphore resource limiter. The general form exposes the shared
schema, one specific setting per condition, and one default value. For
example:

```yaml
policies:
  resources:
    rate_limiters:
      some_rate_limit:
        unit: request_items/second
        tenant_tokens: [tenant_form_a, tenant_form_b]
        conditions:
        - name: first
          entries: { ... }
          token_bucket: { ... } # first condition specifics
          cardinality: { ... }
        - name: second
          entries: { ... }
          token_bucket: { ... } # second condition specifics
          cardinality: { ... }
        token_bucket: { ... }   # default condition specifics
        cardinality: { ... }
```

The engine applies each limit policy at the enforcement points that
handle its declared weight, within the policy's scope. Where a specific
node must be selected, a node references a named policy directly, as
shown in the examples below. Limits that exceed the built-ins are
supplied as policy extensions, described under Shared limiters.

### Resource limiters

The built-in thread-local resource limiter is a semaphore that restricts the
total weight of some concurrent activity, for example the number of
requests in flight, the total memory in use, or the number of items in
a queue. The semaphore acts as a gate, limiting the amount of weight
admitted into a logical-admitted state.

The main resource limiter interface **acquire** returns a reservation
object. If the resource reservation was a success, the caller is given
a **resource hold** object with a corresponding **release**
operation. The engine supports storing the resource hold object in the
request Context, for a reservation to follow the request.

When a hold is stored in the request Context, ownership transfers with
the context: whichever component consumes the context becomes
responsible releasing it. The detailed Rust ownership mechanics are an
out of scope here (see
[otel-arrow#3316](https://github.com/open-telemetry/otel-arrow/pull/3316),
however the main requirement is that owners are responsible for
releasing resource reservations and sometimes the reservation is
carried by the request, to be dropped at the same time. Additionally,
resource holds do not automatically cross shared boundaries (i.e.,
they are `!Send`). Sub-components within the engine (e.g., channels)
assume responsibility for resources, and special nodes (e.g., topic
exporter and receiver) will require an explicit mechanisms to transfer
resource holds across threads and CPUs.

The resource limiter interface also controls the total number of
waiters and/or the total amount of pending weight through its
reservation interface. The built-in semaphore limiter's specific
configuration, for example:

```yaml
semaphore:
  admitted: 100_000_000  # max admitted weight
  waiting:  20_000_000   # max waiting weight
  waiters:  100          # max number waiting
  mode:     lifo         # prioritize freshness
```

### Rate limiters

The rate limiter interface **limit** returns a reservation object. If
the rate reservation was a success, the caller just continues. If the
reservation was not granted, limiters may return to the
caller an option to wait. The option to wait is cancellable.

The built-in token bucket limiter's specific configuration, for
example:

```yaml
token_bucket:
  allow:    100_000_000  # maximum per interval
  burst:    20_000_000   # maximum individual weight
  interval: 60s          # interval duration
  waiters:  100          # max number waiting
  mode:     fifo         # prioritize fairness
```

### Shared limiters

Sharing a limit at the pipeline-group or engine scope, meaning across
threads or CPUs, requires careful attention to avoid synchronization
costs. The engine's built-in token bucket and semaphore limiters
support thread-local use only, so they resolve at pipeline scope. Wider
scopes are served by **policy extensions**: the policy machinery lets a
limit request a shared implementation at CPU-local (pipeline group),
global (engine), and eventually NUMA-regional scope.

For performance reasons, a shared limiter policy extension should
separate its hot and cold paths. Commonly, this is done by aggregating
requests in the background and using asynchronous or relaxed-memory
mechanisms to refresh hot-path limiter state.

Among open-source global rate limit solutions, Envoy implements the
[gRPC Global Rate Limit
service](https://github.com/envoyproxy/ratelimit); another popular
solution is
[Gubernator](https://github.com/gubernator-io/gubernator). A global
rate limit policy extension will map fields of the tenant token
into rate-limit requests on systems such as these.

Shared resource limits may be implemented using conventional
synchronization primitives, for example a policy extension with
Mutex-wrapped internal state, but as with a shared rate limit, these
implementations should leverage thread-local state and separate their
hot and cold paths to avoid interference with the dataflow engine.

The limits described in previous examples resolve at thread-local
scope, making them per-tenant and per-pipeline. To implement
engine-wide or group-wide limits on a per-tenant basis, there are two
options:

1. Use a group- or engine-shared limiter instance, supplied as a policy
   extension.
2. Route the data by tenant token to a single pipeline, then use
   the built-in thread-local limiter.

Both are reasonable options.

### Example limiter yaml

Completing the example started above, we can use the three tokens
declared above to implement two rate limit policies.

```yaml
tenant_tokens: { ... }
groups:
  main-group:
    pipelines:
      main-pipe:
        # Limit policies at pipeline scope resolve thread-local, per core.
        policies:
          resources:
            rate_limiters:
              # The first rate limit applies to the customer.
              customer_rate:
                unit: network_bytes/second
                tenant_tokens: [enduser_tenant_modern, enduser_tenant_legacy]
                conditions:
                # This gives each workspace with customer_id=bigfish more allowance.
                - name: bigfish
                  entries:
                  - key: workspace_id
                  - key: customer_id
                    value: bigfish
                  token_bucket:
                    allow: 50_000   # 50KB/s allowance per pipeline PER CORE
                    burst: 100_000  # 100KB maximum size PER CORE
                  cardinality:
                    max_count: 1000 # Up to 1000 workspaces (single customer)
                # Every combination of { customer_id != bigfish, workspace_id }
                # uses this limit PER CORE.
                token_bucket: { allow: 25_000, burst: 50_000 }
                # Limit to 10000 buckets, the point where isolation breaks down.
                cardinality:
                  max_count: 10000
                  # When the limit is reached, choose to "break" isolation
                  # or else choose to "reject" the tenant.
                  failure_mode: break

              # Second rate limit applies to the OBO service
              obo_rate:
                unit: network_bytes/second
                tenant_tokens: [onbehalfof_tenant]
                # ...

        nodes:
          otlp:
            type: receiver:otlp
            # The engine applies ingress rate-limit policies at this receiver.
            # Listed order is the evaluation order; all must grant to proceed.
            rate_limiters: [obo_rate, customer_rate]
            config:
              # determine which tenant tokens are evaluated
              tenant_tokens: [enduser_tenant_modern, enduser_tenant_legacy, onbehalfof_tenant]
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
```

In this example, where multiple limits are listed for a node as with
`rate_limiters: [obo_rate, customer_rate]` above, they are evaluated in
listed order and all must grant for the request to proceed. If a limit
failure causes the request to short-circuit, the granted reservations
are cancelled. Specific implementation details are out-of-scope, see
open questions.

### Limiter fairness

The built-in thread-local limiters support a fairness mode enabling
LIFO and FIFO behavior. We choose LIFO as the default for both because
it prioritizes fresh data and because LIFO is more robust for bursty
workloads, especially considering that telemetry data is usually sent
with a timeout in effect. LIFO-based limiters are less likely to enter
states where all requests exceed their deadline because of limits.

A third option, when queueing, is to prevent blocking in which case
`mode: nonblocking` prevents callers from waiting at the limiter
interface. This is the only valid mode setting in cases where the
caller is unable to block. See blocking and queuing implementation
details below.

In addition to queueing mode, a policy extension can take advantage of
tenant token fields. As an example, a tenant token might be
used to implement a notion of priority among waiters. A hypothetical
`priority_semaphore` policy extension could allow higher-priority
requests to jump ahead of lower-priority requests using this
configuration:

```yaml
priority_semaphore:
  admitted: 100_000_000
  waiting:  20_000_000
  # requests are admitted in order by level
  levels:
  # bigfish always first priority
  - name: high_priority
    entries:
    - key: customer_id
      value: bigfish
  # metrics are second priority
  - name: medium_priority
    entries:
    - key: signal
      value: metrics
  # otherwise lowest priority
  # mode applies within levels.
  mode: lifo
```

### Cardinality limits

The common limiter configuration includes a cardinality limit, which
places a hard limit on the number of buckets. This is the point at
which isolation between limiters has to break somehow. When the number
of distinct limiter instances is reached, we expect several
configurable behaviors:

- Block new tenants, hard error
- Try using least-recently-used or random limiter
- Block the heaviest user.

### Limiter observability

Limiters record a primary observability signal with the declared units,
exposing the current value of the limiter.

- For rate limiters, an OpenTelemetry Counter named `otap.rate_limiter.accepted` measuring the accepted weight
- For resource limiters, an OpenTelemetry UpDownCounter named `otap.resource_limiter.admitted` measuring the currently admitted weight

Each of these uses the following dimensions:

- Condition name
- Key:values in condition entries of matching bucket
- Signal name of the request.

To bound metric cardinality, these dimensions use the entry keys of
the limiter condition bucket, which are bounded in cardinality by the
limiter.

Additional common metrics are supported at varying levels of detail:

- Cardinality of limiter instances by bucket name (UpDownCounter)
- Number of waiting requests by bucket name (UpDownCounter)
- Amount of waiting weight by bucket name (UpDownCounter)
- Requests arriving, by outcome (accepted/failed)
- Histogram of arriving request weight by accepted/failed

### Rust async blocking and queueing

The mode configuration described for standard limiter requires a
degree of coordination between the caller of the limiter and the
runtime. Limiters themselves never block the runtime, they are
synchronous interfaces that return reservations. Reservations describe
a contract between the caller and the runtime to enable blocking with
LIFO or FIFO semantics, however it is the caller's responsibility
to delay the request.

This explains why the `nonblocking` mode is sometimes necessary, as it
is the only viable setting for callers that cannot delay a request.
When the receiver is able to delay a request (e.g., in memory) and
unless the `nonblocking` is configured, the reservation object is used
to coordinate with the limiter.

