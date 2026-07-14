---
Proposal Name: pressure-aware-eps-throttling
Start Date: 2026-07-14
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#3272
---

# RFC NNNN: Pressure-Aware EPS Throttling

## Summary

Add pressure-aware throttling at receiver admission points. Receivers measure
event rate at the configured enforcement scope and throttle traffic that
exceeds its configured events-per-second (EPS) limit when process memory is
under pressure.

This proposal is an admission-control policy. It does not require retained-work
memory attribution across queues, topics, batchers, retry buffers, exporters, or
external components. The existing process-wide memory limiter remains the final
safety guard.

## Motivation

The current process-wide memory limiter protects the process when RSS or cgroup
memory reaches hard pressure, but its action is broad: it sheds normal ingress
without tenant-specific fairness.

Operators often already know the expected ingest rate for a tenant, pipeline
group, pipeline, or source. When memory is healthy, short bursts above that rate
may be acceptable. When memory is under pressure, scopes exceeding their
configured EPS should be throttled before scopes that are staying within their
configured rate.

This gives the engine a simple first enforcement step:

- use the existing process memory signal to decide when throttling is needed,
- use scoped EPS to decide which traffic is excess,
- apply throttling at receiver admission before more work enters the engine.

## Guide-level explanation

Each receiver measures incoming telemetry rate at its configured enforcement
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
| Critical internal telemetry | Use a separate bounded protected class, not normal tenant capacity. |
<!-- markdownlint-enable MD013 -->

This is a fairness rule, not proof of memory ownership. It says that when memory
is scarce, traffic above the configured rate for the selected scope is removed
first.

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

Internal telemetry should not be treated as a normal tenant. It should use a
bounded protected class so it can continue during pressure without having
unlimited memory.

### Enforcement Scope

The first implementation should target receiver admission using the existing
policy scope. In a shared pipeline, the limiter bucket can be selected by tenant
token. In a deployment where tenants are already separated by pipeline or
pipeline group, the policy can be placed at that scope.

Process-wide per-tenant EPS is a separate scope choice. It requires either
routing each tenant to one pipeline or adding a shared limiter extension that
aggregates rate state across receiver instances. This RFC does not require that
shared limiter in the first step.

### EPS Measurement

Receivers maintain EPS measurements for the limiter scope that applies at the
admission point. If the policy is pipeline-local, the EPS limit is local to that
pipeline instance. If the policy is shared by a later extension, the shared
implementation owns aggregation across receiver instances.

The design must define the counted unit. A first version can use normalized
telemetry items:

- log records for logs,
- spans for traces,
- metric data points for metrics.

If the receiver cannot count decoded items before admission, it may use request
history for that tenant to make the next pre-decode decision. Request or body
bytes may be tracked as an additional signal, but this RFC's required policy is
EPS-based.

### Admission Decision

Receiver admission combines process memory pressure and scoped rate state:

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
still the global shedding backstop. If the process memory limiter is configured
in observe-only mode, this policy should observe only too.

The exact rejection response is receiver-specific. HTTP receivers can return a
service-unavailable or too-many-requests response with retry guidance. gRPC
receivers can use `ResourceExhausted` and retry pushback where available.

### Bursts and Recovery

Configured EPS should allow bounded spikes. A tenant should not be throttled
forever because of a short spike.

Rate state should use a rolling window or token-bucket style calculation.
Existing token-bucket terminology may use `burst` for the maximum single request
weight; any separate time-window smoothing should be named separately. The
policy should also use admission recovery hysteresis before unthrottling so the
same scope does not rapidly switch between admitted and rejected on every
sample.

Receivers should continue updating scoped rate state while throttling so the
engine can detect recovery.

### Interaction With Retained-Work Accounting

Pressure-aware EPS throttling and retained-work accounting are complementary.

EPS throttling controls excess input rate. It does not prove which tenant caused
memory pressure, and it does not measure retained memory inside queues, topics,
batchers, retry buffers, or exporters.

Retained-work accounting can later explain where accepted work remains buffered
and which scope owns that retained work. It is not required for this first EPS
enforcement step.

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
        # Proposed extension: apply this rate limit only while process
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
admission point in both cases.

A later implementation should define:

- tenant-token extractors or descriptors,
- default EPS limit,
- per-tenant-token EPS overrides,
- pressure gating and how it composes with conditions,
- request weight and burst semantics,
- rolling-window or token-bucket parameters,
- soft-pressure threshold for selective throttling,
- admission recovery hysteresis,
- internal telemetry class behavior.

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

## Rationale and alternatives

- EPS throttling is a practical first enforcement step because it operates at
  receiver admission and does not require every downstream component to
  participate in retained-memory accounting.
- Using process pressure as the trigger avoids enforcing tenant EPS during
  healthy memory periods where bursts may be harmless.
- Keeping the existing hard-pressure limiter preserves the current safety guard
  when selective throttling is insufficient.
- The alternative of starting with per-tenant retained-memory budgets is more
  precise for memory ownership, but requires broader accounting coverage across
  engine retention sites and cooperative external components.
- The alternative of always enforcing EPS, even during normal memory, is simpler
  but less flexible for controlled bursts.

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
- Should the first version enforce only EPS, or also request/body bytes per
  second?
- What is the exact response code and retry guidance for each receiver type?
- Should the first implementation be pipeline-local, require tenant routing to
  one pipeline, or depend on a shared limiter extension?
- What rolling-window or token-bucket parameters should be configurable?
- Should selective EPS throttling start at process soft pressure only, or at a
  separate threshold below soft pressure?
- How should limits be represented for mixed signal traffic from one scope?

## Future possibilities

- Add bytes-per-second limits alongside EPS to handle large events better.
- Add per-signal EPS limits for logs, traces, and metrics.
- Feed retained-work accounting into later decisions so the engine can throttle
  tenants whose accepted work remains buffered even when current EPS is low.
- Add administrative metrics showing throttled tenants, over-limit duration,
  accepted EPS, rejected EPS, and process pressure at the time of throttling.
