---
Proposal Name: pressure-aware-eps-throttling
Start Date: 2026-07-14
RFC PR: open-telemetry/otel-arrow#3484
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC NNNN: Pressure-Aware EPS Throttling

## Summary

Add pressure-aware throttling at receiver admission points. Receivers measure
event rate at the configured throttling scope and throttle traffic that
exceeds its configured events-per-second (EPS) limit when process memory is
under pressure.

This proposal is an admission-control policy. It does not require retained-work
memory attribution across queues, topics, batchers, retry buffers, exporters, or
external components. The existing process-wide memory limiter remains the final
safety guard.

The first version is pipeline-local and per-core: the configured EPS applies to
each receiver instance independently, so a scope's effective process-wide rate
scales with the number of cores and pipelines admitting its traffic. Aggregating
one scope's rate across receiver instances requires either routing that scope to
a single pipeline or adding a shared limiter extension, and is deferred.
Operators who need an exact process-wide per-tenant rate should read this
version as a pressure-relief mechanism rather than as a precise rate limit.

## Motivation

The current process-wide memory limiter protects the process when RSS or cgroup
memory reaches hard pressure, but its action is broad: it sheds normal ingress
without tenant-specific fairness.

Operators often already know the expected ingest rate for a tenant, pipeline
group, pipeline, or source. When memory is healthy, short bursts above that rate
may be acceptable. When memory is under pressure, scopes exceeding their
configured EPS should be throttled before scopes that are staying within their
configured rate.

This gives the engine a simple first admission-control step:

- use the existing process memory signal to decide when throttling is needed,
- use scoped EPS to decide which traffic is excess,
- apply throttling at receiver admission before more work enters the engine.

## Guide-level explanation

Each receiver measures incoming telemetry rate at its configured throttling
scope. In a shared pipeline, the buckets can be keyed by tenant tokens. If each
tenant already has its own pipeline, the traffic is already separated, so the
policy needs no tenant token to distinguish scopes. Each receiver instance
keeps its own rate state in both cases.

When process memory is normal, the receiver records EPS but does not reject only
because a scope is over its configured rate. When process memory enters soft
pressure, receivers throttle scopes whose EPS is above their configured limit.
Scopes within their configured EPS continue normally. If memory reaches hard
pressure, the existing global hard-pressure shedding remains the final backstop.

The basic behavior is:

<!-- markdownlint-disable MD013 -->
| Condition | Action |
| --- | --- |
| Memory normal | Observe scoped EPS only. |
| Memory soft and scope over EPS | Throttle or reject new work for that scope. |
| Memory soft and scope within EPS | Continue admitting that scope's work. |
| Memory hard | Existing global hard-pressure shedding applies, unchanged. |
<!-- markdownlint-enable MD013 -->

This is a fairness heuristic, not evidence that an over-limit scope caused or
owns the process memory pressure. It only decides which excess ingress is
removed first while memory is scarce.

Each request is evaluated only against its own scope, which fixes the behavior
in the two boundary cases. If several scopes are over their configured EPS at
the same time, requests belonging to each of them are rejected independently;
scopes are not ranked against one another, and no scope is spared because
another scope is further over its limit. If no scope is over its configured EPS,
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

The EPS limiter should not define a new hard-coded `tenant_id` field. It should
use the resolved tenant token as the bucket key. The design must not rely on a
raw client-controlled header such as `x-tenant-id` unless an upstream
authentication or trust boundary has already validated it and mapped it into an
operator-owned tenant token.

### Throttling Scope

Three things are easy to conflate here, and this RFC keeps them separate:

- **Policy declaration** is where the limiter's configuration is defined. It
  controls inheritance.
- **Receiver binding** is which receiver applies a named limiter at its
  admission point.
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
validation to match. Pressure-aware EPS should follow the same pattern: state
which scopes accept the policy, and enforce that at startup.

#### Binding and bucket key

A limiter bucket is keyed by the tuple *(resolved policy, optional tenant
token)*. The tenant token, when present, subdivides that policy's traffic into
per-tenant buckets. Both parts are operator-owned.

This tuple is the extensibility contract. One mechanism covers a limit with no
tenant token and a per-tenant limit inside a shared pipeline, without a separate
design for each. Tenant limits are not a special case: they are this tuple with
the token component populated. A later shared-state scope changes where the
bucket lives, not what the bucket is keyed by.

A receiver binds named limiters with `rate_limits: [...]`. Under the
tenant-token model that list may name several limiters, all of which must grant
for a request to proceed. This RFC constrains the first version to one
pressure-aware EPS limiter per receiver; composing several pressure-aware
limiters at one admission point is out of scope.

#### Runtime aggregation

In the first version, every resolved limiter is instantiated independently in
each pipeline runtime and receiver instance, and its rate state is local to that
instance. The configured EPS applies per instance, so a scope's effective
process-wide rate scales with the number of instances admitting its traffic.

Process-wide or group-wide aggregation is a separate choice and is deferred. It
requires either routing a scope's traffic to a single pipeline, or a shared
limiter extension that aggregates rate state across instances. This RFC does not
require that shared limiter in the first step.

Because the first version has no shared state, nested limits across declaration
levels are out of scope. A nested case is one where a tenant is within its own
configured EPS while the group containing it is over a group limit, or the
reverse. Evaluating every applicable level and throttling on any violation is a
natural extension of the same bucket model, but it raises questions this RFC
does not answer: whether an over-limit group should throttle tenants that are
within their own limits, how a throttled scope reports which level rejected it,
and how per-level rate state is kept consistent across instances. Configuration
validation should reject a multi-level placement that the first version cannot
evaluate, at startup, rather than silently applying only the most specific one.

### EPS Measurement

The counted unit is normalized telemetry items:

- log records for logs,
- spans for traces,
- metric data points for metrics.

Receivers admit whole requests or batches, not individual items inside an
accepted batch. This matters because admission happens before decode, and the
exact item count of a request is only known after decode. The first version
therefore uses **receiver-local accepted-item history** rather than reserving
weight before admission:

1. A request arrives.
2. The receiver resolves the scope from transport metadata, receiver identity,
   or static configuration.
3. If process memory is at soft pressure and that scope's recent accepted item
   rate is over its configured EPS, the receiver rejects the whole request.
4. Otherwise the receiver admits, reads, and decodes the request.
5. The receiver counts the actual items and updates that scope's rate state.

The consequence is that admission decisions are made on the scope's recent
history, not on the request in hand. A scope goes over its limit, and the next
request for that scope pays for it. This is deliberate: it keeps the receiver
from having to know a request's weight before paying the memory cost of
decoding it, which is the cost this policy exists to avoid under pressure.

The first implementation resolves tenant tokens only from trusted transport
metadata, receiver identity, or static configuration. Resource-attribute
extractors require decoded request context and are outside the pre-decode path
unless the receiver explicitly accepts the cost of decoding before admission.

Request or body bytes are known before decode and may be tracked as an
additional signal, but this RFC's required policy is EPS-based. The
implementation must define how accepted-item history is aged or reset, and
whether rejected requests update it.

This history-based shape is not one of the two built-in limiters described in
[otel-arrow#3389](https://github.com/open-telemetry/otel-arrow/pull/3389), which
offers a token bucket for rate limits and a semaphore for resource limits. A
token bucket sized in telemetry items assumes the receiver knows a request's
item weight at admission time, which it does not. Pressure-aware EPS is the
first policy to run into that pre-decode weight problem, and it sidesteps it by
using accepted-item history. Whether that arrives as an additional built-in rate
limiter shape or as a policy extension is an open question for the tenant-token
model.

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
if process_pressure == Hard:
    reject normal ingress using the existing global limiter behavior
else if process_pressure == Soft and scope_eps > configured_eps:
    throttle this scope
else:
    admit
```

This gives soft pressure an admission-control meaning for this policy. The
existing phase-1 memory limiter behavior remains unchanged: hard pressure is
still the global shedding backstop. If the process memory limiter is not
configured, this policy has no process-pressure trigger and should observe EPS
without pressure-based throttling. If the process memory limiter is configured
in `observe_only` mode, this policy should observe only too. If it is configured
in `enforce` mode, hard pressure continues to shed normal ingress globally,
while soft pressure can trigger scoped EPS throttling. Adopting this policy
would require updating the phase-1 memory limiter documentation that describes
soft pressure as informational only.

The exact rejection response is receiver-specific. OTLP/HTTP should preserve the
existing memory-pressure response shape: HTTP 503 with `Retry-After`. OTLP/gRPC
should use `ResourceExhausted` with retry pushback metadata. Other receivers
define equivalent protocol-specific refusal behavior.

### Bursts and Recovery

Configured EPS should allow bounded spikes. A tenant should not be throttled
forever because of a short spike.

Spike tolerance comes from the width of the accepted-item history window rather
than from a separate burst allowance. A window wide enough to average over
normal traffic absorbs a short spike; a narrow window reacts faster but throttles
on noise. That trade-off is the main parameter to settle during implementation.

The policy should also use admission recovery hysteresis before unthrottling so
the same scope does not rapidly switch between admitted and rejected on every
sample. This is separate from the memory limiter's own hysteresis, which governs
leaving soft pressure; the two operate on independent axes and both apply. Rate
measurements should continue while memory is normal so entering soft pressure
uses current history instead of starting from an empty window.

Receivers should continue updating scoped rate state while throttling so
receiver-local admission can detect when the scope becomes eligible again.

### Interaction With Retained-Work Accounting

Pressure-aware EPS throttling and retained-work accounting are complementary.

EPS throttling controls excess input rate. It does not prove which tenant caused
memory pressure, and it does not measure retained memory inside queues, topics,
batchers, retry buffers, or exporters.

Retained-work accounting can later explain where accepted work remains buffered
and which scope owns that retained work. It is not required for this first EPS
admission-control step.

### Configuration Shape

This RFC does not define the final configuration schema. The sketch below is
illustrative only. It borrows the tenant-token and `rate_limits` vocabulary
proposed in
[otel-arrow#3389](https://github.com/open-telemetry/otel-arrow/pull/3389); if
that design changes, this shape changes with it. It deliberately does not name a
built-in limiter block, because the accepted-item history described above is not
one of that design's built-ins.

```yaml
tenant_tokens:
  customer_tenant:
    extractors:
    - key: customer_id
      transport_header: x-customer-id
    - key: workspace_id
      transport_header: x-workspace-id

policies:
  resources:
    rate_limits:
      pressure_eps:
        unit: request_items/second
        optional_tenant_tokens: [customer_tenant]
        # Applies per receiver instance, not process-wide.
        allow: 10000
        # Width of the accepted-item history window used to compute the
        # recent rate. Wider absorbs spikes; narrower reacts faster.
        interval: 1s
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
            rate_limits: [pressure_eps]
            config:
              tenant_tokens: [customer_tenant]
```

In a shared pipeline, the EPS buckets come from tenant tokens. If tenants are
already separated by pipeline or pipeline group, the same policy can be declared
at that scope without extracting a tenant token. The receiver remains the
admission point in both cases, and each receiver instance keeps its own rate
state in both cases. Both shapes are the same bucket key described in "Binding
and bucket key", with and without the tenant-token component.

This example is not accepted by the current v1 schema. `rate_limits` does not
exist in the configuration model today, and `Policies` and `ResourcesPolicy`
reject unknown fields. The eventual schema must keep strict unknown-field
rejection, validate declaration placement and receiver binding, verify that the
receiver supplies the configured weight unit, and reject at startup both
unsupported pressure-gate combinations and multi-level placements that the first
version cannot evaluate. Per-tenant overrides should use ordered conditions from
the tenant-token policy model if that model is adopted.

A later implementation should define:

- tenant-token extractors or descriptors,
- default EPS limit,
- per-tenant-token EPS overrides,
- which declaration scopes accept the policy, and how a multi-level placement is
  validated or rejected,
- whether the history-based limiter is an additional built-in shape or a policy
  extension,
- how the pressure gate is expressed, and how it composes with tenant-token
  conditions,
- accepted-item history window width, ageing, and reset,
- admission recovery hysteresis,
- live-update behavior for limiter state.

The configuration should be operator-owned and should avoid unbounded
per-request or per-scope label cardinality.

## Drawbacks

- EPS is not memory ownership. A scope can stay within EPS but send large
  events or cause downstream buffering.
- A scope over EPS may not be the scope that caused process memory pressure.
- Process-wide per-tenant EPS requires tenant routing or a shared limiter
  extension; a receiver-local limiter does not provide that by itself.
- Declaring the policy at group scope gives every pipeline in the group the same
  configuration, not a shared group-wide bucket. Operators who read placement as
  aggregation will be surprised.
- The counted unit must be defined carefully across logs, traces, and metrics.
- Admission decisions use the scope's recent accepted-item history rather than
  the request in hand, because item count is not known before decode. A scope
  that goes over its limit is throttled on its next request, not the one that
  crossed the line.
- If pressure is caused by a stuck exporter, retry backlog, or other downstream
  retention site, throttling a current high-EPS scope may not reduce pressure.
- Unidentified traffic must still be bounded. If tenant identity is optional,
  unresolved traffic falls into a default bucket unless the policy requires a
  tenant token and rejects missing identity.
- Live policy updates need explicit state handling so changed limits, removed
  buckets, and per-core limiter state do not produce surprising behavior.

## Rationale and alternatives

- EPS throttling is a practical first admission-control step because it operates
  at receiver admission and does not require every downstream component to
  participate in retained-memory accounting.
- Using process pressure as the trigger avoids applying tenant EPS limits during
  healthy memory periods where bursts may be harmless.
- Keeping the existing hard-pressure limiter preserves the current safety guard
  when selective throttling is insufficient.
- The alternative of starting with per-tenant retained-memory budgets is more
  precise for memory ownership, but requires broader accounting coverage across
  engine retention sites and cooperative external components.
- The alternative of always applying EPS limits, even during normal memory, is
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
- If there is prior art in related OpenTelemetry components for tenant EPS
  throttling, it should be added during RFC review.

## Unresolved questions

- Which tenant-token extractor source should the first implementation use?
- How should accepted-item history age, and should rejected requests update it?
- How wide should the history window be by default, and should it be
  configurable per policy?
- Should the accepted-item history limiter be an additional built-in rate
  limiter shape, or a policy extension?
- Should the first version limit only EPS, or also request/body bytes per
  second? Bytes are known before decode and would allow a pre-decode weight cap,
  which items do not.
- What response and retry guidance should non-OTLP receivers use? OTLP/HTTP and
  OTLP/gRPC follow the existing memory-pressure responses.
- Should configuration validation reject multi-level placements outright in the
  first version, or accept them with most-specific-wins and warn at startup?
- How should limits be represented for mixed signal traffic from one scope?
- Should unidentified traffic share one default bucket, or should missing tenant
  identity reject immediately?
- Does changing limits reset or preserve limiter state during live
  reconfiguration?

## Future possibilities

- Add bytes-per-second limits alongside EPS to handle large events better.
- Add per-signal EPS limits for logs, traces, and metrics.
- Trigger selective throttling at a threshold below soft pressure, or at a
  separate threshold of its own, instead of reusing the existing soft level.
  The first version uses soft pressure because it already exists and already
  reaches receivers.
- Add pressure-scaled limits, where the effective EPS for each scope shrinks as
  pressure deepens instead of staying at the configured value. This gives the
  engine a graduated response between throttling only over-limit scopes and
  shedding everything at hard pressure, and it is the main answer to soft
  pressure that persists while every scope is within its configured rate.
- Add nested limit evaluation across placement levels, so a group-level limit
  and a per-tenant limit inside it can both apply to one admission decision.
- Add bounded protected handling for internal telemetry, separate from normal
  tenant traffic.
- Add process-wide per-tenant EPS through tenant routing or a shared limiter
  extension.
- Feed retained-work accounting into later decisions so the engine can throttle
  tenants whose accepted work remains buffered even when current EPS is low.
- Add administrative metrics showing throttled tenants, over-limit duration,
  accepted EPS, rejected EPS, and process pressure at the time of throttling.
