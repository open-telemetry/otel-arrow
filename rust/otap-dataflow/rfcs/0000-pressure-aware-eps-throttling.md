---
Proposal Name: pressure-aware-eps-throttling
Start Date: 2026-07-14
RFC PR: open-telemetry/otel-arrow#0000
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
tenant already has a separate pipeline or pipeline group, policy placement can
define the scope without extracting a tenant token.

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

Each scope's decision depends only on its own rate, which fixes the behavior in
the two boundary cases. If several scopes are over their configured EPS at the
same time, all of them are throttled in the same admission pass; there is no
ordering or ranking between them, and no scope is spared because another scope
is further over its limit. If no scope is over its configured EPS, this policy
admits everything and provides no relief, so memory pressure keeps building
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

#### Scope model

A limiter bucket is keyed by the tuple *(placement scope, optional tenant
token)*. Placement scope is the configuration level the policy is attached
to — receiver, pipeline, or pipeline group. The tenant token, when present,
subdivides that placement into per-tenant buckets. Both parts are
operator-owned.

This tuple is the extensibility contract. One mechanism then covers a
receiver-level limit with no tenant token, a per-tenant limit inside a shared
pipeline, and a group-level limit spanning several pipelines, without a separate
design for each. Tenant limits are not a special case: they are this tuple with
the token component populated. A later process-wide scope adds a new placement
level rather than a new bucket concept.

#### Composition across levels

A single admission decision evaluates exactly one bucket in the first version.
When policies at more than one placement level could apply to the same admission
point, the most specific placement wins: receiver over pipeline, pipeline over
group. Policy precedence follows the existing
[configuration model](../docs/configuration-model.md).

Nested limits are deliberately out of scope for the first version. A nested case
is one where a tenant is within its own configured EPS while the group
containing it is over the group limit, or the reverse. Evaluating every
applicable level and throttling on any violation is a natural extension of the
same bucket model, but it raises questions this RFC does not answer: whether an
over-limit group should throttle tenants that are within their own limits, how a
throttled scope reports which level rejected it, and how per-level rate state is
kept consistent when the levels live on different receiver instances.
Configuration validation should reject a multi-level placement combination that
the first version cannot evaluate at startup, rather than silently applying only
the most specific one.

#### Placement in practice

The first implementation is pipeline-local and targets receiver admission using
the existing policy scope. In a shared pipeline, the limiter bucket can be
selected by tenant token. In a deployment where tenants are already separated by
pipeline or pipeline group, the policy can be placed at that scope.

Process-wide per-tenant EPS is a separate scope choice. It requires either
routing each tenant to one pipeline or adding a shared limiter extension that
aggregates rate state across receiver instances. This RFC does not require that
shared limiter in the first step.

### EPS Measurement

Receivers maintain EPS measurements for the limiter scope that applies at the
admission point. If the policy is pipeline-local, the EPS limit is local to that
pipeline instance, per core, so the effective process-wide rate scales with the
number of cores. If the policy is shared by a later extension, the shared
implementation owns aggregation across receiver instances.

The design must define the counted unit. A first version can use normalized
telemetry items:

- log records for logs,
- spans for traces,
- metric data points for metrics.

The first pre-decode implementation is limited to tenant tokens resolved from
trusted transport metadata, receiver identity, or static configuration.
Resource-attribute extractors require decoded request context and are outside
that path unless the receiver explicitly accepts the cost of decoding before
admission.

If the receiver cannot count decoded items before admission, it may use recent
accepted-request history for that tenant to make the next pre-decode decision.
Request or body bytes may be tracked as an additional signal, but this RFC's
required policy is EPS-based. The implementation must define how that history is
aged or reset, and whether rejected requests update it.

### Admission Decision

Receiver admission combines process memory pressure and scoped rate state. It
should follow the existing memory-limiter admission pattern: the engine
maintains shared pressure or limiter state, propagates admission state to
receivers, and receivers consult local admission state on their ingress hot
paths. The engine does not call receiver-specific throttle APIs; receivers own
the protocol-specific response.

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

Rate state should use a rolling window or token-bucket style calculation.
Existing token-bucket terminology may use `burst` for the maximum single request
weight; any separate time-window smoothing should be named separately. The
policy should also use admission recovery hysteresis before unthrottling so the
same scope does not rapidly switch between admitted and rejected on every sample.
This is separate from the memory limiter's own hysteresis for leaving soft
pressure. Rate measurements should continue while memory is normal so entering
soft pressure uses current history instead of starting from an empty window.

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
illustrative only. It assumes the tenant-token and `rate_limits` schema proposed
in [otel-arrow#3389](https://github.com/open-telemetry/otel-arrow/pull/3389);
if that design changes, this shape changes with it.

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
        # Proposed policy-level gate: apply this rate limit only while process
        # memory is at or above soft pressure. The final schema must define
        # how this composes with tenant-token conditions.
        token_bucket:
          allow: 10000
          burst: 20000 # maximum single request weight
          interval: 1s
          mode: nonblocking
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
already separated by pipeline or pipeline group, the same policy can be applied
at that scope without extracting a tenant token. The receiver remains the
admission point in both cases. Both shapes are the same bucket key described in
"Scope model", with and without the tenant-token component.

This example is not accepted by the current v1 schema. The eventual schema must
keep strict unknown-field rejection, validate policy placement and receiver
binding, verify that the receiver supplies the configured weight unit, reject
unsupported pressure-gate combinations at startup, and reject multi-level
placements that the first version cannot evaluate. Per-tenant overrides should
use ordered conditions from the tenant-token policy model if that model is
adopted.

A later implementation should define:

- tenant-token extractors or descriptors,
- default EPS limit,
- per-tenant-token EPS overrides,
- which placement levels accept the policy, and how a multi-level placement is
  validated or rejected,
- pressure gating and how it composes with conditions,
- request weight and burst semantics,
- rolling-window or token-bucket parameters,
- soft-pressure threshold for selective throttling,
- admission recovery hysteresis,
- live-update behavior for limiter state.

The configuration should be operator-owned and should avoid unbounded
per-request or per-scope label cardinality.

## Drawbacks

- EPS is not memory ownership. A scope can stay within EPS but send large
  events or cause downstream buffering.
- A scope over EPS may not be the scope that caused process memory pressure.
- Process-wide per-tenant EPS requires tenant routing or a shared limiter
  extension; a pipeline-local token bucket does not provide that by itself.
- The counted unit must be defined carefully across logs, traces, and metrics.
- Pre-decode admission may need to use recent history because decoded item count
  is not always known before reading the request body.
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
- How should accepted-request history age, and should rejected requests update
  it?
- Should the first version limit only EPS, or also request/body bytes per
  second?
- What is the exact response code and retry guidance for each receiver type?
- What rolling-window or token-bucket parameters should be configurable?
- Should selective EPS throttling start at process soft pressure only, or at a
  separate threshold below soft pressure?
- Should configuration validation reject multi-level placements outright in the
  first version, or accept them with most-specific-wins and warn at startup?
- Is pressure gating a policy-level receiver gate, rather than a new tenant
  condition?
- How should limits be represented for mixed signal traffic from one scope?
- Should unidentified traffic share one default bucket, or should missing tenant
  identity reject immediately?
- Does changing limits reset or preserve limiter state during live
  reconfiguration?

## Future possibilities

- Add bytes-per-second limits alongside EPS to handle large events better.
- Add per-signal EPS limits for logs, traces, and metrics.
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
