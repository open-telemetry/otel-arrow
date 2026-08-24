<!-- markdownlint-disable MD013 -->

# Contrib Extensions

Contrib extensions are optional, feature-gated extensions that extend the
default OTel Arrow Dataflow Engine build. Extensions provide cross-cutting
capabilities (such as authentication) that data-path nodes bind to via their
`capabilities:` map, rather than processing pipeline data themselves.

For help writing runtime YAML, start at
[`docs/configuration.md`](../../docs/configuration.md). For exact runtime
configuration semantics, see
[`docs/configuration-model.md`](../../docs/configuration-model.md).

## Extensions

| Extension | URN | Feature gate | Capability | Docs |
| --- | --- | --- | --- | --- |
| Azure Identity Auth | `urn:microsoft:extension:azure_identity_auth` | `azure-identity-auth-extension` | `bearer_token_provider` | [usage](./src/azure_identity_auth/README.md), [design](./src/azure_identity_auth/design.md) |
| Kubernetes Service Account Token Auth | `urn:otel:extension:k8s_service_account_token_auth` | `k8s-service-account-token-auth-extension` | `bearer_token_authorizer` | [usage](./src/k8s_service_account_token_auth/README.md), [design](../../docs/k8s-service-account-token-auth-extension.md) |
| OAuth 2.0 Client Auth | `urn:otel:extension:oauth2_client_auth` | `oauth2-client-auth-extension` | `bearer_token_provider` | [usage](./src/oauth2_client_auth/README.md), [design](./src/oauth2_client_auth/design.md) |

Each extension's README is the authoritative configuration reference for that
extension. Nodes that bind an extension document only the binding and how they
use the capability, and link here for the provider's own options.

Extensions are enabled through individual feature gates or the aggregate
`contrib-extensions` feature gate. An extension documented as `Experimental`,
`Alpha`, or `Draft` has no stable compatibility guarantee yet, and its behavior
or configuration can change between releases.
