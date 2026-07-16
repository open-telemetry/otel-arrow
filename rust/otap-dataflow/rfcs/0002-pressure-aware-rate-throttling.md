---
Proposal Name: pressure-aware-rate-throttling
Start Date: 2026-07-14
RFC PR: open-telemetry/otel-arrow#3484
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC 0002: Pressure-Aware Rate Throttling

## Summary

Add pressure-aware throttling at receiver admission points. Receivers measure
ingress rate at the configured throttling scope and throttle traffic that
exceeds its configured rate limit when process memory is under pressure. The
rate unit is receiver-specific: a receiver may use request bytes, telemetry
items, or another unit it can measure on its admission path.

This proposal is an admission-control policy. It does not require retained-work
memory attribution across queues, topics, batchers, retry buffers, exporters, or
external components. The existing process-wide memory limiter remains the final
safety guard.

The first version is pipeline-local and per-core, with
`aggregation: receiver_instance`. The configured rate limit applies to each
receiver instance independently, not to the process-wide total. A scope's
effective process-wide rate therefore depends on both the number of receiver
instances admitting its traffic and how that traffic is distributed across them.
Operators who need an exact process-wide per-tenant rate should read this
version as a pressure-relief mechanism rather than as a precise rate limit.

## Motivation

The current process-wide memory limiter protects the process when RSS or cgroup
memory reaches hard pressure, but its action is broad: it sheds normal ingress
without tenant-specific fairness.

Operators often already know the expected ingest rate for a tenant, pipeline
group, pipeline, or source. When memory is healthy, short bursts above that rate
may be acceptable. When memory is under pressure, scopes exceeding their
configured rate should be throttled before scopes that are staying within their
configured rate.

This gives the engine a simple first admission-control step:

- use the existing process memory signal to decide when throttling is needed,
- use scoped rate to decide which traffic is excess,
- apply throttling at receiver admission before more work enters the engine.

## Guide-level explanation

Each receiver measures incoming telemetry rate at its configured throttling
scope. In a shared pipeline, the buckets can be keyed by tenant tokens. If each
tenant already has its own pipeline, the traffic is already separated, so the
policy needs no tenant token to distinguish scopes. Each receiver instance
keeps its own rate state in both cases.

When process memory is normal, the receiver records the configured rate signal
but does not reject only because a scope is over its configured rate. When
process memory enters soft pressure, receivers throttle scopes whose measured
rate is above their configured limit. Scopes within their configured rate
continue normally. If memory reaches hard pressure, the existing global
hard-pressure shedding remains the final backstop.

The basic behavior is:

<!-- markdownlint-disable MD013 -->
| Condition | Action |
| --- | --- |
| Memory normal | Observe scoped rate only. |
| Memory soft and scope over rate | Throttle or reject new work for that scope. |
| Memory soft and scope within rate | Continue admitting that scope's work. |
| Memory hard | Global hard-pressure shedding applies; the scope gate also stays active for requests the global limiter admits. |
<!-- markdownlint-enable MD013 -->

This is a fairness heuristic, not evidence that an over-limit scope caused or
owns the process memory pressure. It only decides which excess ingress is
removed first while memory is scarce.

Each request is evaluated only against its own scope, which fixes the behavior
in the two boundary cases. If several scopes are over their configured rate at
the same time, requests belonging to each of them are rejected independently;
scopes are not ranked against one another, and no scope is spared because
another scope is further over its limit. If no scope is over its configured rate,
this policy admits everything and provides no relief, so pressure keeps building
until the existing global hard-pressure shedding takes over. That second case is
intended rather than a gap in the mechanism: the policy removes excess ingress,
and when there is no excess by the operator's own definition, it has nothing to
act on. Pressure-scaled limits, listed under "Future possibilities", are the
natural escalation if operators need relief in that state.

## Reference-level explanation

### Tenant Tokens

Admission decisions should use the tenant-token model proposed by
[otel-arrow#3389](https://github.com/open-telemetry/otel-arrow/pull/3389). That
PR is not merged, so this RFC treats the tenant-token model as design context,
not as current repository behavior. A tenant token is an operator-configured set
of key/value identifiers resolved from trusted request context, such as
validated transport headers, resource attributes, receiver identity, or a static
configured key.

The rate limiter should not define a new hard-coded `tenant_id` field. It
should use the resolved tenant token as the bucket key. The design must not rely
on a raw client-controlled header such as `x-tenant-id` unless an upstream
authentication or trust boundary has already validated it and mapped it into an
operator-owned tenant token.

### Throttling Scope

Three things are easy to conflate here, and this RFC keeps them separate:

- **Policy declaration** is where the limiter's configuration is defined. It
  controls inheritance.
- **Receiver application** is which compatible receivers apply the effective
  policy at their admission point.
- **Runtime aggregation** is whether limiter state is local to one instance or
  shared across several.

Declaration placement does not determine aggregation. These are independent.

#### Declaration and inheritance

Policy declaration follows the existing
[configuration model](../docs/configuration-model.md): top-level policies
provide defaults, group policies override them for a group, and pipeline
policies override them for a single pipeline. Precedence applies by policy
family, and a lower scope replaces an upper scope for that family rather than
deep-merging into it. Receiver binding is not a policy scope and does not add a
precedence level.

Resolution copies configuration down; it does not create shared state. A
group-level declaration therefore gives every pipeline in that group the same
limiter *configuration*. It does not aggregate their traffic into one bucket.

The engine already has a policy that is genuinely process-wide, and it is
instructive: the memory limiter "is process-wide and only supported at top-level
`policies.resources`", and group and pipeline overrides of it are rejected during
engine validation. It is process-wide because its runtime object is process-wide,
not because of where it is declared, and its declaration is then constrained by
validation to match. Pressure-aware rate throttling should follow the same
pattern: state which scopes accept the policy, and enforce that at startup.

#### Policy application and bucket key

A limiter bucket is keyed by the tuple *(resolved policy, optional tenant
token)*. The tenant token, when present, subdivides that policy's traffic into
per-tenant buckets. Both parts are operator-owned.

This tuple is the extensibility contract. One mechanism covers a limit with no
tenant token and a per-tenant limit inside a shared pipeline, without a separate
design for each. Tenant limits are not a special case: they are this tuple with
the token component populated. A later shared-state scope changes where the
bucket lives, not what the bucket is keyed by.

This RFC models `rate_limit` as a singleton policy family, not a named limiter
catalog. Compatible receivers apply the effective `rate_limit` policy
automatically. A receiver can provide an inline override or explicitly disable
the policy when needed. This follows the existing hierarchical policy model:
top-level policy provides the default, group policy replaces it for a group, and
pipeline policy replaces it for a pipeline. The existing `transport_headers`
policy is the closest precedent: an effective policy applies to compatible
nodes, while a node can override it when needed.

This RFC constrains the first version to one effective pressure-aware rate
limiter per compatible receiver. Composing several rate limits at one admission
point is out of scope and belongs to future generic-limiter work.

#### Runtime aggregation

In the first version, every resolved limiter is instantiated independently in
each pipeline runtime and receiver instance, and its rate state is local to that
instance. The configuration makes this explicit with
`aggregation: receiver_instance`. In v1 this field is required, and
`receiver_instance` is the only supported value.

In `receiver_instance` mode, `allow` applies to each receiver-instance bucket,
not to the process-wide total for the scope. A tenant sending 8,000 items/s
through each of four receiver instances is below a local `allow: 10000` in every
bucket while reaching 32,000 items/s process-wide. The same tenant sending
12,000 items/s through one receiver instance is over the local limit and can be
throttled. Distribution may depend on connection topology, core allocation, and
routing, not only tenant behavior.

This makes v1 a local pressure-relief heuristic, not exact process-wide
per-tenant fairness. Exact per-tenant rate semantics require tenant affinity or
shuffle routing to one pipeline, or a shared limiter extension that aggregates
rate state across instances. Tenant affinity gives exact local semantics but
trades away parallelism for that tenant.

Process-wide or group-wide aggregation is a separate choice and is deferred. It
can later be added as another `aggregation` value, such as `process`, without
changing the meaning of `allow` for `receiver_instance`.

Because the first version has no shared state, nested limits across declaration
levels are out of scope. A nested case is one where a tenant is within its own
configured rate while the group containing it is over a group limit, or the
reverse. Evaluating every applicable level and throttling on any violation is a
natural extension of the same bucket model, but it raises questions this RFC
does not answer: whether an over-limit group should throttle tenants that are
within their own limits, how a throttled scope reports which level rejected it,
and how per-level rate state is kept consistent across instances. Configuration
validation should reject a multi-level placement that the first version cannot
evaluate, at startup, rather than silently applying only the most specific one.

### Rate Measurement

Each receiver should declare which rate units it can measure on its admission
path. Configuration may choose from those supported units, and startup
validation should reject unsupported units. If a unit requires extra decoding or
scanning, that cost must be explicit rather than accidental.

Two useful units are:

- `request_bytes/second` for request or body bytes known before decode,
- `request_items/second` for normalized telemetry items.

For `request_items/second`, the counted unit is normalized telemetry items:

- log records for logs,
- spans for traces,
- metric data points for metrics.

OTLP receivers can usually measure request bytes before protobuf inspection, so
bytes per second is the likely default unit for encoded OTLP admission. Item
counting for encoded OTLP is not free and should not be assumed; it requires a
scan or decode the receiver would otherwise avoid. OTAP receivers, and receivers
that already build item-level batches on their admission path, may support item
rate units more naturally.

Receivers admit whole requests or batches, not individual items inside an
accepted batch. This matters for item-based units because admission happens
before decode, and the exact item count of a request may only be known after
decode.

For units known before admission, such as `request_bytes/second`, the receiver
can use a normal token-bucket check because it knows the request weight before
admission. For item-based units whose weight is known only after decode, the
receiver should use a **post-charge token bucket with bounded debt**:

1. A request arrives.
2. The receiver resolves the scope from transport metadata, receiver identity,
   or static configuration.
3. If process memory is at or above soft pressure and the bucket balance is not
   positive, the receiver rejects the whole request.
4. Otherwise the receiver admits, reads, and decodes the request.
5. The receiver charges the actual item count after decode. The bucket balance
   may go negative only to a bounded debt floor.

Rejected requests do not update exact item count because their weight is
unknown. There is no accepted-history decay; token refill replaces it. Admission
resumes when refill brings the balance positive again.

While the pressure gate is active under sustained over-limit offered load, the
admitted rate is bounded by the configured rate because each admitted request
pays its true weight before more work is admitted. For item-based units, this
requires the debt floor to hold the largest chargeable request. A smaller floor
would clamp away part of the charge and allow sustained admitted rate above the
configured rate. Maximum instantaneous overshoot is bounded by the configured
burst and debt limits plus the receiver's existing maximum request size.

The debt floor must also apply while memory is normal. The gate only rejects
when pressure is active, but the bucket should keep charging and refilling in
normal memory so pressure starts from current state. The debt floor keeps a
scope from accumulating unbounded debt before soft pressure and bounds the
worst-case lockout after pressure starts to `max_debt / rate`.

The first implementation resolves tenant tokens only from trusted transport
metadata, receiver identity, or static configuration. Resource-attribute
extractors require decoded request context and are outside the pre-decode path
unless the receiver explicitly accepts the cost of decoding before admission.

Byte-based units do not have the same pre-decode item-count problem because the
receiver can usually measure request or body bytes before accepting more work.
Rejected byte-based requests can still update the bucket when the request weight
is known before refusal.

This aligns with the token-bucket limiter described in
[otel-arrow#3389](https://github.com/open-telemetry/otel-arrow/pull/3389), but
item-based units need one extra capability: post-charge negative balance with a
bounded debt floor. Without that, a telemetry-item bucket would require knowing
the full request weight before admission, which some receivers cannot do
cheaply.

### Admission Decision

Receiver admission combines process memory pressure and scoped rate state. It
should follow the existing memory-limiter admission pattern: the engine
maintains shared pressure state, propagates it to receivers, and receivers
consult local admission state on their ingress hot paths. The engine does not
call receiver-specific throttle APIs; receivers own the protocol-specific
response.

Pressure reaches receivers over the existing control channel. The engine already
delivers `NodeControlMsg::MemoryPressureChanged` carrying the current
`MemoryPressureLevel`, and receivers already consume it. This policy is a
receiver-local gate driven by that message, not a new tenant-token condition:
the pressure level is a property of the process, identical for every scope at a
given moment, while tenant tokens select which bucket a request counts against.
Keeping them as separate inputs avoids putting process state into a per-request
matching path.

```text
if process_pressure == Hard and the global limiter rejects this request:
    reject normal ingress using the existing global limiter behavior
else if process_pressure >= Soft and the scope bucket cannot admit:
    throttle this scope
else:
    admit
```

When the rate policy mode is `observe_only`, the "throttle this scope" branch
records a would-throttle decision instead of rejecting.

This gives soft pressure an admission-control meaning for this policy, and the
gate remains active at higher pressure levels. The existing phase-1 memory
limiter behavior remains unchanged: its mode controls global hard-pressure
shedding. The pressure-aware rate policy consumes the pressure level as an
input signal and has its own `mode`.

If process memory measurement is not configured, a pressure-aware rate policy
has no pressure source and should fail startup validation rather than silently
run a policy that can never trigger.

The policy mode controls scoped throttling:

<!-- markdownlint-disable MD013 -->
| Memory limiter mode | Rate policy mode | Behavior |
| --- | --- | --- |
| `enforce` | `enforce` | Hard pressure sheds globally; soft pressure can throttle scoped traffic. |
| `enforce` | `observe_only` | Hard pressure sheds globally; soft pressure records would-throttle decisions only. |
| `observe_only` | `enforce` | No global hard-pressure shedding; soft pressure can throttle scoped traffic. This is useful for experiments, but it has no global hard backstop. |
| `observe_only` | `observe_only` | Both policies observe; nothing is enforced. |
<!-- markdownlint-enable MD013 -->

Adopting this policy would require updating the phase-1 memory limiter
documentation that describes soft pressure as informational only.

The exact rejection response is receiver-specific. OTLP/HTTP should preserve the
existing memory-pressure response shape: HTTP 503 with `Retry-After`. OTLP/gRPC
should use `ResourceExhausted` with retry pushback metadata. Other receivers
define equivalent protocol-specific refusal behavior.

### Bursts and Recovery

Configured rate limits should allow bounded spikes. A tenant should not be
throttled forever because of a short spike.

Spike tolerance comes from token-bucket burst capacity. Recovery comes from
token refill. For post-charge item buckets, negative balance is bounded by the
debt floor, and admission resumes only after refill brings the balance positive.

This is separate from the memory limiter's own hysteresis, which governs leaving
soft pressure. Rate state should continue to update while memory is normal, so
entering soft pressure uses current bucket state instead of starting from an
empty or full bucket.

### Interaction With Retained-Work Accounting

Pressure-aware rate throttling and retained-work accounting are complementary.

Rate throttling controls excess input rate. It does not prove which tenant
caused memory pressure, and it does not measure retained memory inside queues,
topics, batchers, retry buffers, or exporters.

Retained-work accounting can later explain where accepted work remains buffered
and which scope owns that retained work. It is not required for this first
admission-control step.

### Configuration Shape

This RFC does not define the final configuration schema. The sketch below is
illustrative only. It treats `rate_limit` as a singleton policy family. The
future plural `rate_limits` namespace is reserved for a named limiter catalog if
the generic limiter model is added later.

`rate_limit` is separate from `policies.resources`. The current policy resolver
handles `resources` as one atomic family, so a scoped `resources.core_allocation`
override would replace any broader rate-limit configuration nested under
`resources`.

```yaml
tenant_tokens:
  customer_tenant:
    extractors:
    - key: customer_id
      transport_header: x-customer-id
    - key: workspace_id
      transport_header: x-workspace-id

policies:
  rate_limit:
    mode: enforce
    aggregation: receiver_instance
    unit: request_bytes/second
    optional_tenant_tokens: [customer_tenant]
    # Applies per receiver instance, not process-wide.
    allow: 10485760
    # Token refill interval for the configured rate.
    interval: 1s
    # Burst and debt fields are illustrative. Exact schema names are
    # unresolved and should align with the tenant-token limiter model.
    burst: 10485760
    max_debt: 10485760
    # Proposed policy-level gate: apply this limit only while process
    # memory is at or above soft pressure. Driven by the pressure level
    # the receiver already receives, not by a tenant-token condition.
    pressure: soft
    cardinality:
      max_count: 10000
      failure_mode: reject

groups:
  main:
    pipelines:
      logs:
        nodes:
          otlp:
            type: receiver:otlp
            config:
              tenant_tokens: [customer_tenant]
```

This example uses `request_bytes/second` because OTLP request bytes are
available before protobuf inspection. A receiver that can measure item counts on
its admission path may instead support `request_items/second`. In a shared
pipeline, the rate buckets come from tenant tokens. If tenants are already
separated by pipeline or pipeline group, the same singleton policy can be
overridden at that scope without extracting a tenant token. The receiver remains
the admission point in both cases, and each receiver instance keeps its own rate
state in both cases. Both shapes are the same bucket key described in "Policy
application and bucket key", with and without the tenant-token component.

In the sketch, `optional_tenant_tokens` says which resolved tokens may subdivide
the policy into buckets. The receiver's `config.tenant_tokens` says which tokens
the receiver should resolve from its request context.

The `cardinality` block bounds the number of tracked scope buckets. If that
limit is exceeded, `failure_mode: reject` avoids creating unbounded new bucket
state while the process is already under pressure.

This example is not accepted by the current v1 schema. `rate_limit` does not
exist in the configuration model today, and `Policies` rejects unknown fields.
The eventual schema must keep strict unknown-field rejection, validate
declaration placement and receiver compatibility, verify that the receiver
supports the configured rate unit on its admission path, and reject at startup
both unsupported pressure-gate combinations and multi-level placements that the
first version cannot evaluate. The first version should also reject any
`aggregation` value other than `receiver_instance`, treat `pressure: soft` as
the only supported v1 pressure gate, require an explicit rate policy `mode`,
reject a pressure-aware rate policy when no process pressure source is
configured, and reject configs that set both singleton `rate_limit` and reserved
plural `rate_limits`. Per-tenant overrides should use ordered conditions from
the tenant-token policy model if that model is adopted.

The first version must emit would-throttle counters in `observe_only` mode so
operators can evaluate the policy before enforcing it.

A later implementation should define:

- tenant-token extractors or descriptors,
- default rate limit,
- per-tenant-token rate overrides,
- which declaration scopes accept the policy, and how a multi-level placement is
  validated or rejected,
- how post-charge debt is represented in the token-bucket limiter,
- receiver-level override and disable syntax,
- how the pressure gate is expressed, and how it composes with tenant-token
  conditions,
- burst and debt bounds,
- live-update behavior for limiter state.

The configuration should be operator-owned and should avoid unbounded
per-request or per-scope label cardinality.

This RFC does not define the generic limiter catalog. Same-receiver multi-limit
composition, per-signal selectors, shared aggregation, retained-work activation,
and non-receiver enforcement points are future generic-limiter work. If a
future generic limiter framework is accepted, the singleton `rate_limit` policy
can either remain as shorthand for one inherited receiver admission limiter, or
migrate mechanically into a named `rate_limits` policy plus explicit bindings.

### Performance Validation

Before implementation is accepted, the admission path should be measured against
the latest baseline implementation that does not include this rate limiter. The
checks should cover
throughput, CPU cost, memory used by per-scope rate state, and the cost of
measuring the configured rate unit.

The expected cost factors are tenant or scope extraction, bucket lookup,
per-scope rate-state updates, request byte measurement, optional post-decode
item counting, scope cardinality, and whether the limiter remains
receiver-local or later uses shared state. The first implementation should use
the existing benchmark and performance-test surfaces where possible, including
receiver throughput tests and the existing item-count benchmark for item-based
units.

## Drawbacks

- Rate is not memory ownership. A scope can stay within its configured rate but
  send large telemetry payloads or cause downstream buffering.
- A scope over its configured rate may not be the scope that caused process
  memory pressure.
- Process-wide per-tenant rate limits require tenant routing or a shared limiter
  extension; a receiver-local limiter does not provide that by itself.
- Receiver-local aggregation depends on traffic distribution. A scope spread
  across many receiver instances can admit a higher process-wide rate than the
  same scope concentrated on one receiver instance.
- Declaring the policy at group scope gives every pipeline in the group the same
  configuration, not a shared group-wide bucket. Operators who read placement as
  aggregation will be surprised.
- The singleton policy does not support composing multiple independent rate
  limits at one receiver. That belongs to a future named limiter catalog.
- The configured rate unit must be defined carefully across receivers and
  signals.
- For item-based units, a receiver may admit one request before knowing the
  exact item count. That request is charged after decode, so overshoot is
  bounded but not zero.
- If pressure is caused by a stuck exporter, retry backlog, or other downstream
  retention site, throttling a current high-rate scope may not reduce pressure.
- Unidentified traffic must still be bounded. If tenant identity is optional,
  unresolved traffic falls into a default bucket unless the policy requires a
  tenant token and rejects missing identity.
- Live policy updates need explicit state handling so changed limits, removed
  buckets, and per-core limiter state do not produce surprising behavior.

## Rationale and alternatives

- Rate throttling is a practical first admission-control step because it
  operates at receiver admission and does not require every downstream component
  to participate in retained-memory accounting.
- Using process pressure as the trigger avoids applying tenant rate limits
  during healthy memory periods where bursts may be harmless.
- Keeping the existing hard-pressure limiter preserves the current safety guard
  when selective throttling is insufficient.
- The alternative of starting with per-tenant retained-memory budgets is more
  precise for memory ownership, but requires broader accounting coverage across
  engine retention sites and cooperative external components.
- The alternative of always applying rate limits, even during normal memory, is
  simpler but less flexible for controlled bursts.

## Prior art

- The existing [process-wide memory limiter](../docs/memory-limiter-phase1.md)
  already classifies process memory pressure and sheds ingress under hard
  pressure.
- The unmerged [multitenancy design
  proposal](https://github.com/open-telemetry/otel-arrow/pull/3389) describes
  tenant tokens, limiter policies, and receiver binding.
- Receiver-side admission control is the natural place to reject or throttle
  new work before it creates more retained state.
- If there is prior art in related OpenTelemetry components for tenant rate
  throttling, it should be added during RFC review.

## Unresolved questions

- Which tenant-token extractor source should the first implementation use?
- Which rate units should each receiver support natively?
- How should post-charge debt be represented in the token-bucket limiter model?
- What default burst and debt bounds should be used?
- Should item-based units for encoded OTLP be rejected, or allowed only with an
  explicit decode or scan cost?
- What response and retry guidance should non-OTLP receivers use? OTLP/HTTP and
  OTLP/gRPC follow the existing memory-pressure responses.
- How should limits be represented for mixed signal traffic from one scope?
- Should unidentified traffic share one default bucket, or should missing tenant
  identity reject immediately?
- Does changing limits reset or preserve limiter state during live
  reconfiguration?
- If maintainers want a generic named limiter catalog in this RFC, should this
  RFC depend on that design and land after it is accepted?

## Future possibilities

- Add more receiver-native rate units beyond request bytes and telemetry items.
- Add per-signal item-rate limits for logs, traces, and metrics.
- Add a generic `policies.rate_limits` catalog with named definitions and
  explicit node bindings. The singleton `policies.rate_limit` namespace is
  reserved for v1, and configs should not set both forms.
- Trigger selective throttling at a threshold below soft pressure, or at a
  separate threshold of its own, instead of reusing the existing soft level.
  The first version uses soft pressure because it already exists and already
  reaches receivers.
- Add pressure-scaled limits, where the effective rate limit for each scope
  shrinks as pressure deepens instead of staying at the configured value. This
  gives the engine a graduated response between throttling only over-limit
  scopes and shedding everything at hard pressure, and it is the main answer to
  soft pressure that persists while every scope is within its configured rate.
- Add nested limit evaluation across placement levels, so a group-level limit
  and a per-tenant limit inside it can both apply to one admission decision.
- Add bounded protected handling for internal telemetry, separate from normal
  tenant traffic.
- Add per-runtime retained-work pressure as another local trigger. This would
  use signals such as queue depth, in-flight batches, and estimated pdata bytes
  held by a pipeline runtime, not raw thread allocator memory.
- Add process-wide per-tenant rate limits through tenant routing or a shared limiter
  extension.
- Feed retained-work accounting into later decisions so the engine can throttle
  tenants whose accepted work remains buffered even when current rate is low.
- Add administrative metrics showing throttled tenants, over-limit duration,
  accepted rate, rejected rate, configured unit, and process pressure at the
  time of throttling.
