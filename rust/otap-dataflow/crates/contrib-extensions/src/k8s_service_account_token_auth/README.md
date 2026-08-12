<!-- markdownlint-disable MD013 -->

# Kubernetes Service Account Token Auth Extension

**Status:** Draft

| | |
| --- | --- |
| **URN** | `urn:otel:extension:k8s_service_account_token_auth` |
| **Feature gate** | `k8s-service-account-token-auth-extension` |
| **Capability** | `bearer_token_authorizer` |
| **Execution model** | Passive (Shared + Local variants) |

Authenticates and admits inbound Kubernetes service-account tokens for data-path
nodes (typically receivers) through the `BearerTokenAuthorizer` capability, so a
node depends on this single capability rather than validating tokens itself. Each
presented token is validated via the Kubernetes `TokenReview` API
(authentication) and the resulting identity is admitted (admission) **per
audience**: each configured audience binds to one strategy -- a static
service-account allow-list or a Kubernetes RBAC `SubjectAccessReview` check --
so an identity is only trusted for the audience it was minted for (no
cross-tenant admission), behind one `authorize` call.

Reached decisions are cached, keyed by the token's SHA-256 digest (no plaintext
credential retained) and bounded by a
configurable TTL and entry cap, to bound `TokenReview` calls to the API server. A
reached deny (missing, invalid, or not-permitted token) is a normal outcome; when
the API server is unreachable or does not respond within `review_timeout`, the
decision is undetermined and surfaced as an error so callers fail closed.

For the full design -- problem, goals, lifecycle, configuration reference, and
security considerations -- see
[`docs/k8s-service-account-token-auth-extension.md`](../../../../docs/k8s-service-account-token-auth-extension.md).

## Crypto provider requirement

The extension talks to the Kubernetes API server over TLS via `kube`'s
`hyper`/`rustls` client, which requires a process-wide `rustls` crypto provider
to be installed. The deployed binary **must** enable exactly one `crypto-*`
feature (`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`, or `crypto-symcrypt`,
forwarded to `otap-df-otap`); the workspace binary's default build includes
`crypto-ring`. A build that enables `k8s-service-account-token-auth-extension` without
any `crypto-*` feature installs no provider, and the `TokenReview` call fails at
runtime.
