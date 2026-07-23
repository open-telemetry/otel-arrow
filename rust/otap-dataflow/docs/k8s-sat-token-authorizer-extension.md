# Kubernetes SAT Authorizer Extension

<!-- markdownlint-disable MD013 -->

**Status:** Draft

**Extension URN:** `urn:otel:extension:k8s_sat_token_authorizer`

**Capability exposed:** `BearerTokenAuthorizer`

**Execution model:** Passive + Shared

**Target crate:** `crates/contrib-extensions`

**Target module:** `crates/contrib-extensions/src/k8s_sat_token_authorizer/`

This document describes the design of the **Kubernetes SAT authorizer
extension** (`k8s_sat_token_authorizer`) for the OTAP dataflow engine. The
extension authenticates and admits inbound Kubernetes service-account tokens
(SATs) presented on data-path requests and exposes the verdict to data-path
nodes through a `BearerTokenAuthorizer` capability.

It builds on the extension system foundations:

- [Extension System Proposal](extension-requirements.md) - the *what* and *why*
  of the capability-based extension system.
- [Extension System Architecture](extension-system-architecture.md) - the
  Phase 1 *how* (capability proc macro, registry, Active/Passive lifecycle,
  local/shared execution models).
- [Design Principles and Constraints](design-principles.md) - thread-per-core
  execution, minimal synchronization, security/privacy first.

## Problem

A receiver that accepts telemetry inside a Kubernetes cluster often needs to
authenticate its callers by their Kubernetes service-account token: the caller
presents a projected SAT in the request's `Authorization` header, and the
receiver must verify the token is authentic and valid for its audience, and that
the identity behind it is permitted to send.

Doing this inline in each receiver couples transport handling to Kubernetes API
access, duplicates token-validation and caching logic, and spreads
credential-handling code across nodes. The
[`BearerTokenAuthorizer`](../crates/engine/src/capability/auth/bearer_token_authorizer.rs)
capability exists precisely to move this behind a single call; this extension is
its first production implementation.

## Goals

- Validate an inbound SAT via the Kubernetes `TokenReview` API
  (**authentication**).
- Admit the resulting service account against a configured audience and optional
  service-account allow-list (**admission**).
- Return a single `AuthzDecision` (`Allow` carrying an `AuthorizedIdentity`, or a
  coarse `Deny`), behind one `authorize` call, so receivers do not orchestrate
  the steps themselves.
- **Fail closed**: any undetermined outcome (API server unreachable) is an
  `Err`, which callers must treat as a deny.
- Bound load on the API server by caching reached decisions, keyed by the opaque
  token.

## Non-goals

- Contextual, per-request authorization (route, tenant, signal, or action
  scoping). That needs request context this capability never receives and
  belongs downstream, consuming the `AuthorizedIdentity` this extension emits.
- Non-Kubernetes token schemes (OIDC/JWT validated against a JWKS, opaque OAuth
  introspection). Those are separate authorizer implementations.
- Issuing or refreshing tokens (the outbound side is `BearerTokenProvider`).

## Design

### Capability and execution model

The extension registers into `OTAP_EXTENSION_FACTORIES` via `linkme` when the
`k8s-sat-token-authorizer-extension` feature is enabled, and advertises the
`bearer_token_authorizer` capability on its **shared** (`Send`) variant:

```rust
extension_capabilities!(
    shared: K8sSatTokenAuthorizerExtension => [BearerTokenAuthorizer]
)
```

It is a **Passive + Shared** extension: it runs no event loop. Every consumer
receives a clone that shares the same `Arc<Inner>` state, so they share one
Kubernetes client and one decision cache.

The Kubernetes client is built **lazily on the first `authorize()` call**, not
up front. `kube::Client::try_default()` is async (it reads the projected
service-account token and cluster CA and resolves the API server), so it cannot
run in the synchronous `create()` factory hook; a `tokio::sync::OnceCell` builds
it once, on demand. A build failure is undetermined -- the request fails closed
and the empty cell lets the next request retry -- so no separate readiness/warmup
gate is needed (an early request simply pays the one-time construction latency,
and an inability to construct fails closed rather than allowing).

The extension is *passive* rather than *active* because it has no periodic work
and nothing to drain or flush at shutdown: no background loop, no in-flight
requests to await, and an in-memory-only cache. Dropping it is a clean shutdown.
The trade-off is that a passive extension receives no `CollectTelemetry` control
message, so this version exposes **no metrics** (metric support for passive
extensions is a future enhancement).

### Request path

`authorize(&BearerToken)` proceeds:

1. **Empty credential** short-circuits to `Deny(MissingCredential)` without any
   API call.
2. **Cache hit** (a still-valid decision keyed by the opaque token) is returned
   directly.
3. **Client init** builds the Kubernetes client on first use; a build failure is
   undetermined and returns an `Err` -- fail closed (the next request retries).
4. **`TokenReview`** is submitted for the token with the configured audiences.
   The API server answer maps to:
   - authenticated -> **admission** against the allow-list (below);
   - not authenticated -> `Deny(InvalidCredential)` with the API server's
     message as log-only detail;
   - request failure / missing status -> `Err` (undetermined; not cached).
5. The reached decision (allow or deny) is cached and returned.

No lock is held across the `TokenReview` await; the cache uses short
`std::sync::Mutex` critical sections that never span an `.await`.

### Admission

After authentication, if an allow-list is configured, the returned username
(`system:serviceaccount:<namespace>:<name>`) must be a member or the request is
`Deny(NotPermitted)`. An empty allow-list admits any authenticated account
(audience-only admission). Allow-list entries accept three shapes -- the full
username, `<namespace>/<name>`, and `<namespace>:<name>` -- all normalized to the
canonical username at wiring time for an O(1) set lookup.

An `Allow` carries an `AuthorizedIdentity` whose `subject` is the SA username and
whose `audience` is the confirmed audience, so downstream per-tenant/route
authorization can consume it.

### Decision cache

Reached decisions are cached in a bounded map keyed by the opaque token, with a
configurable TTL (`cache_ttl`) and entry cap (`cache_max_entries`). On insert at
capacity, expired entries are reclaimed first; if the map is still full of live
entries the new decision is returned but not cached, so the cache never exceeds
its bound. `Allow` decisions are cached for up to `cache_ttl`, so a token
revoked at the API server may continue to be admitted until its cached decision
expires -- decision freshness is deliberately the implementation's concern (the
capability's `Allow` carries no validity window).

### Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `audiences` | list of strings | *required* | Audiences requested on `TokenReview`; the token must be valid for at least one. Must be non-empty. |
| `allowed_service_accounts` | list of strings | `[]` | Admitted service accounts (`system:serviceaccount:<ns>:<name>`, `<ns>/<name>`, or `<ns>:<name>`). Empty admits any authenticated account. |
| `cache_ttl` | duration | `5m` | How long a reached decision is cached, keyed by the opaque token. Must be non-zero. |
| `cache_max_entries` | integer | `1024` | Upper bound on cached decisions. Must be greater than zero. |

Example:

```yaml
extensions:
  k8s_authz:
    urn: urn:otel:extension:k8s_sat_token_authorizer
    config:
      audiences:
        - https://my-collector.observability.svc
      allowed_service_accounts:
        - workloads/otlp-sender
      cache_ttl: 5m
```

A receiver binds it via its `capabilities:` map (see
[`docs/configuration-model.md`](configuration-model.md)).

### Telemetry

None in this version. As a passive extension it receives no `CollectTelemetry`
control message, so it registers no metric set. Exposing metrics from passive
extensions is a planned enhancement.

## Security considerations

- **Fail closed.** An unreachable API server yields an `Err`, never an allow.
- **Secret handling.** The token is carried by the secret-protecting
  `BearerToken`; it is exposed only to build the `TokenReview` request. The token
  string is used as a cache key and thus held in memory for up to `cache_ttl`;
  the cache is capped and never logged.
- **No policy leak.** Deny reasons are coarse, low-cardinality
  (`MissingCredential`, `InvalidCredential`, `NotPermitted`); per-request detail
  goes only to logs, never to metric labels or untrusted callers.
- **Crypto provider.** The `kube` client uses `rustls`, so the deployed binary
  must install a process-wide crypto provider via exactly one `crypto-*` feature
  (the workspace default is `crypto-ring`).
