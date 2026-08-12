# Kubernetes Service Account Token Auth Extension

<!-- markdownlint-disable MD013 -->

**Status:** Draft

**Extension URN:** `urn:otel:extension:k8s_service_account_token_auth`

**Capability exposed:** `BearerTokenAuthorizer`

**Execution model:** Passive (Shared + Local variants)

**Target crate:** `crates/contrib-extensions`

**Target module:** `crates/contrib-extensions/src/k8s_service_account_token_auth/`

This document describes the design of the **Kubernetes service-account-token
auth extension** (`k8s_service_account_token_auth`) for the OTAP dataflow
engine. The
extension authenticates and admits inbound Kubernetes service-account tokens
(SATs) presented on data-path requests -- authenticating via `TokenReview` and
admitting via a service-account allow-list or an RBAC `SubjectAccessReview` --
and exposes the verdict to data-path nodes through a `BearerTokenAuthorizer`
capability.

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
- Admit the resulting identity per audience (**admission**): each audience is
  bound to one strategy -- a static service-account allow-list or a Kubernetes
  RBAC check via `SubjectAccessReview` -- so an identity is only trusted for the
  audience it was minted for (no cross-tenant admission).
- Return a single `AuthzDecision` (`Allow` carrying an `AuthorizedIdentity`, or a
  coarse `Deny`), behind one `authorize` call, so receivers do not orchestrate
  the steps themselves.
- **Fail closed**: any undetermined outcome (API server unreachable) is an
  `Err`, which callers must treat as a deny.
- Bound load on the API server by caching reached decisions, keyed by the token's
  SHA-256 digest.

## Non-goals

- Contextual, per-request authorization (route, tenant, signal, or action
  scoping). That needs request context this capability never receives and
  belongs downstream, consuming the `AuthorizedIdentity` this extension emits.
  The optional RBAC check uses a **fixed** resource/verb from configuration (the
  same for every request), so it is admission "on the token alone", not
  per-request authorization.
- Non-Kubernetes token schemes (OIDC/JWT validated against a JWKS, opaque OAuth
  introspection). Those are separate authorizer implementations.
- Issuing or refreshing tokens (the outbound side is `BearerTokenProvider`).

## Design

### Capability and execution model

The extension registers into `OTAP_EXTENSION_FACTORIES` via `linkme` when the
`k8s-service-account-token-auth-extension` feature is enabled, and advertises the
`bearer_token_authorizer` capability as a **dual variant** -- a `Send` shared
variant and a `!Send` local variant -- sharing one common implementation:

```rust
extension_capabilities!(
    (shared: SharedK8sServiceAccountTokenAuth,
     local: LocalK8sServiceAccountTokenAuth)
        => [BearerTokenAuthorizer]
)
```

It is a **Passive** extension: it exposes a capability and builds its Kubernetes
client on demand. Both variants live side by side in `authorizer.rs` over common
`core`, `config`, and `reviewer` logic, each holding its own cache:

- **Shared variant** (`SharedK8sServiceAccountTokenAuth`): state shared across clones
  lives behind an `Arc` (required by the shared instance factory's `Send` bound)
  and the decision cache is guarded by a `std::sync::Mutex`. Served to `Send`
  consumers, and to local consumers when no local variant exists (the
  `SharedAsLocal` fallback).
- **Local variant** (`LocalK8sServiceAccountTokenAuth`): state lives behind an `Rc`
  (the local instance factory has no `Send` bound) and the cache is a `RefCell`.
  Thread-per-core consumers therefore hit the cache lock-free, with no
  cross-core contention. Each core gets its own instance and hence its own
  cache, consistent with the engine's thread-per-core model.

Each variant calls its own cache, which supplies the guard and slot handle
suited to its concurrency model; the security decision on a miss comes from the
common `Core`. Registering a native local variant lets local consumers use the
`RefCell` path directly instead of adapting the shared, `Mutex`-guarded instance
via `SharedAsLocal`.

The Kubernetes client is built **lazily on the first `authorize()` call**.
`kube::Client::try_default()` is async (it reads the projected service-account
token and cluster CA and resolves the API server), so it cannot run in the
synchronous `create()` factory hook; a `tokio::sync::OnceCell` builds it once, on
demand. A build failure is undetermined -- the request fails closed and the empty
cell lets the next request retry -- so a separate readiness gate is unnecessary:
an early request pays the one-time construction latency, and a failure to
construct fails closed.

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
2. **Cache hit** (a still-valid decision keyed by the token's SHA-256 digest) is
   returned directly.
3. **Client init** builds the Kubernetes client on first use; a build failure is
   undetermined and returns an `Err` -- fail closed (the next request retries).
4. **`TokenReview`** is submitted for the token with the union of all entry
   audiences and the configured `review_timeout`. The API server answer maps to:
   - authenticated -> **admission** (below);
   - authenticated as a non-service-account identity ->
     `Deny(InvalidCredential)`;
   - not authenticated -> `Deny(InvalidCredential)` with the API server's
     message as log-only detail;
   - request failure / timeout / missing status -> `Err` (undetermined; not
     cached).
5. **Admission** (only when authenticated): every configured entry whose
   audience the API server *confirmed* must admit the identity (a logical AND,
   failing closed on the first denial); each entry's allow-list / audience-only
   admission is decided in-process, while RBAC admission issues a second API call
   (`SubjectAccessReview`, also bounded by `review_timeout`). A token whose
   confirmed audiences include none that are configured is
   `Deny(NotPermitted)`; an RBAC request failure or timeout is `Err`
   (undetermined; not cached).
6. The reached decision (allow or deny) is cached and returned.

No lock is held across the API-server awaits; the cache uses short
`std::sync::Mutex` critical sections that never span an `.await`.

### Admission

Admission is **per audience**. Each configured entry ties one audience to one
admission strategy, and admission uses the entry for the audience `TokenReview`
actually confirmed -- **not** a global list. This binds a service account to the
audience it was minted for and closes cross-tenant admission: with two tenants,
a token minted by tenant A's SA for tenant A's audience cannot be admitted
through tenant B's entry, even if that SA also appears elsewhere. `TokenReview`
still requests the union of audiences; only the admission step keys off the
matched one.

A token can be confirmed for **several** configured audiences at once (a
projected token minted for multiple audiences; `TokenReview` returns the
intersection, whose order Kubernetes does not specify). Admission then requires
**every** matched audience's policy to admit (a logical AND), failing closed on
the first denial. This keeps cross-tenant isolation intact -- a token that also
carries a laxer audience can never bypass a stricter audience's policy -- while
still admitting a token legitimately valid for multiple tenants when each
tenant's policy accepts it. The emitted identity's `aud` claim then lists every
audience the token was admitted for (multi-valued when more than one). A
confirmed audience that is not configured is ignored; if **no** confirmed
audience is configured, the request is denied (`NotPermitted`, "token audience
is not bound").

Admission additionally requires the authenticated identity to be a **service
account**, i.e. to carry the canonical
`system:serviceaccount:<namespace>:<name>` username. `TokenReview` authenticates
every credential type the API server accepts (OIDC, webhook, ...), so any other
authenticated identity is denied (`InvalidCredential`) rather than admitted --
otherwise an audience-only entry would admit a non-service-account caller and
emit it under the `k8s_sat` scheme.

Within an entry, admission uses exactly one strategy (the two fields are
mutually exclusive):

- **Audience-only** (neither field set): any account authenticated for this
  audience is admitted.
- **Allow-list** (`allowed_service_accounts`): the returned username
  (`system:serviceaccount:<namespace>:<name>`) must be a member, else
  `Deny(NotPermitted)`. Entries accept three shapes -- the full username,
  `<namespace>/<name>`, and `<namespace>:<name>` -- all normalized to the
  canonical username at wiring time for an O(1) set lookup.
- **RBAC** (`resource_attributes`): a `SubjectAccessReview` asks the API server
  whether the authenticated subject (user, uid, groups, and extra from the
  `TokenReview`) may perform the configured `verb` on the configured resource.
  `allowed` (and not `denied`) admits; anything else is `Deny(NotPermitted)`
  with the API server's reason as log-only detail. A failed or timed-out
  `SubjectAccessReview` call is undetermined and fails closed.

An `Allow` carries an `AuthorizedIdentity` holding every claim the `TokenReview`
verified, so a downstream tenant / per-route authorization resolver can match on
them without re-parsing anything:

- `scheme` = `k8s_sat`, `principal` = the SA username (best-effort, for logs).
- `sub` = SA username; `aud` = the matched audience.
- `k8s.namespace` / `k8s.serviceaccount` = parsed from the SA username.
- `uid`; `groups` (multi-valued); and any `extra` attributes as `extra.<key>`.

The extension emits the full verified claim set and leaves tenant resolution to
a separate, configurable concern that consumes these claims. Claim names follow
the shared `capability::auth::AuthorizedIdentity` vocabulary (standard
`sub`/`aud`/`groups`, otherwise namespaced), so a resolver written against them
works across authorizers.

### Decision cache

Reached decisions are cached in a bounded map keyed by the token's **SHA-256
digest**, with a configurable TTL (`cache_ttl`) and entry cap
(`cache_max_entries`). Keying on the digest keeps the cache holding only 32-byte
digests, so a memory or core dump exposes no live credential, and lookups
compare unpredictable digests. SHA-256 is collision-resistant, so distinct
tokens always occupy distinct entries.

Each entry holds a shared, lazily-initialized decision slot, which collapses a
**cache stampede**: when several requests bearing the same token arrive together
and miss, the first performs the `TokenReview` while the rest await that same
slot, so one token costs one round-trip no matter how many requests race. This
matters most at TTL expiry, where a single entry's expiry instant would
otherwise release a synchronized burst of identical reviews at the API server
every `cache_ttl` -- enough, under load, to trip API Priority and Fairness and
have legitimate traffic denied by the extension's own fail-closed path. A failed
review publishes the same error to requests already awaiting that flight, then
removes the slot so a later request retries.

Both variants need this. A thread-per-core runtime multiplexes many tasks on one
thread and preempts a task at the `.await` on the `TokenReview`, so interleaved
tasks holding one token each reach the miss path before any of them stores a
result.

The two variants use separate caches, `SharedDecisionCache` and
`LocalDecisionCache`, each implementing its own hot path:

| | `SharedDecisionCache` | `LocalDecisionCache` |
| --- | --- | --- |
| Guard | `Mutex` | `RefCell` |
| Slot handle | `Arc` | `Rc` |
| Hit clones the decision | after releasing the guard | under the borrow |
| Hit touches the slot refcount | yes | never |
| `Send` | yes | `!Send` |

The shared variant runs requests in parallel, so anything done under the `Mutex`
blocks unrelated tokens. It clones the slot under the guard and the decision
only after releasing it, trading one atomic refcount bump for a locked region
that is just a map probe.

In the local variant tasks interleave on one thread, and preemption is possible
only at an `.await`, of which the hit path has none. It clones the decision out
of the entry while the borrow is held, so a hit costs one non-atomic borrow-flag
update and leaves the slot's refcount untouched. Its slots are `Rc`, making them
`!Send`, so thread-confinement is checked at compile time.

The caches share their bookkeeping: TTL, bounds, and eviction. The security
decision on a miss comes from the common `Core`. A slot remains in flight until
its decision completes; its TTL starts at successful completion, so a slow
review cannot expire and start a duplicate request. In-flight slots are not
eviction candidates.

The token's SHA-256 is computed by the caller, before the lock is taken, so it
stays outside the critical section.

On insert at capacity, a completed entry closest to expiry is chosen from a
small bounded sample and evicted. The bounded sample keeps insertion O(1) under
the lock instead of scanning the map. If the sample contains only in-flight
reviews, the new distinct token is decided without retaining its slot; this
preserves the entry cap without interrupting duplicate suppression already in
progress. Eviction protects availability because unauthenticated callers can
mint unlimited distinct cache keys.

`Allow` decisions are cached for up to `cache_ttl`, so a token revoked at the
API server may continue to be admitted until its cached decision expires;
decision freshness is the implementation's concern, as the capability's `Allow`
carries no validity window.

`cache_max_entries` bounds a *single* cache instance. The local
(thread-per-core) variant holds one cache per core, so total memory scales with
core count and the same token is reviewed once per core.

### Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `audiences` | list of audiences | *required* | Per-audience admission audiences. Must be non-empty, with a unique `audience` per entry. |
| `cache_ttl` | duration | `5m` | How long a reached decision is cached, keyed by the token's SHA-256 digest. Must be non-zero. |
| `cache_max_entries` | integer | `1024` | Upper bound on cached decisions. Must be greater than zero. |
| `review_timeout` | duration | `10s` | Maximum duration of each `TokenReview` or `SubjectAccessReview`. Must be non-zero. |

Each **entry**:

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `audience` | string | *required* | The audience this entry admits. A token is admitted through this entry only when `TokenReview` confirms it for this exact audience. |
| `allowed_service_accounts` | list of strings | `[]` | Allow-list admission for this audience (`system:serviceaccount:<ns>:<name>`, `<ns>/<name>`, or `<ns>:<name>`). Empty admits any account authenticated for this audience. Mutually exclusive with `resource_attributes`. |
| `resource_attributes` | object | *unset* | RBAC admission via `SubjectAccessReview`. `resource` and `verb` are required; `group`, `version`, `namespace`, `name`, `subresource` are optional. Mutually exclusive with `allowed_service_accounts`. |

Multi-tenant example (allow-list and RBAC per audience):

```yaml
extensions:
  k8s_authz:
    type: "urn:otel:extension:k8s_service_account_token_auth"
    config:
      audiences:
        - audience: https://tenant-a.observability.svc
          allowed_service_accounts:
            - tenant-a/otlp-sender
        - audience: https://tenant-b.observability.svc
          resource_attributes:
            group: telemetry.opentelemetry.io
            resource: telemetry
            verb: export
            namespace: tenant-b
      cache_ttl: 5m
      review_timeout: 10s
```

A receiver binds it via its `capabilities:` map (see
[`docs/configuration-model.md`](configuration-model.md)). No built-in receiver
invokes `bearer_token_authorizer` yet, so binding it does not itself enforce
authentication.

### Collector RBAC

The collector's own ServiceAccount must be allowed to call the review APIs it
uses: `create` on `authentication.k8s.io/tokenreviews`, plus `create` on
`authorization.k8s.io/subjectaccessreviews` when `resource_attributes` (RBAC
admission) is configured. For example:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: otel-collector-sat-authorizer
rules:
  - apiGroups: ["authentication.k8s.io"]
    resources: ["tokenreviews"]
    verbs: ["create"]
  - apiGroups: ["authorization.k8s.io"]
    resources: ["subjectaccessreviews"]
    verbs: ["create"]
```

### Telemetry

No metrics in this version. As a passive extension it receives no
`CollectTelemetry` control message, so it registers no metric set. Exposing
metrics from passive extensions is a planned enhancement.

The extension does emit structured log events: debug events for authentication
and admission outcomes, and warning events for Kubernetes client and API
failures. Debug events can include the authenticated service-account username
and the confirmed audiences (token-derived identity data, so they are
log-only and never metric labels); the plaintext token is never logged.

## Security considerations

- **Fail closed.** An unreachable or unresponsive API server yields an `Err`,
  never an allow.
- **Secret handling.** The token is carried by the secret-protecting
  `BearerToken`; it is exposed only to build the `TokenReview` request and to
  compute its SHA-256 cache key. No plaintext token is retained -- the cache
  keys on the digest -- and the plaintext token is never logged. Debug events
  may include the authenticated service-account username and audiences.
- **No policy leak.** Deny reasons are coarse, low-cardinality
  (`MissingCredential`, `InvalidCredential`, `NotPermitted`); per-request detail
  goes only to logs, never to metric labels or untrusted callers.
- **Crypto provider.** The `kube` client uses `rustls`, so the deployed binary
  must install a process-wide crypto provider via exactly one `crypto-*` feature
  (the workspace default is `crypto-ring`).
