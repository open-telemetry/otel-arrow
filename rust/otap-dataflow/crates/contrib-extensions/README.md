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
| Azure Identity Auth | `urn:microsoft:extension:azure_identity_auth` | `azure-identity-auth-extension` | `bearer_token_provider` | [`docs/azure-identity-auth-extension.md`](../../docs/azure-identity-auth-extension.md) |

Extensions are enabled through individual feature gates or the aggregate
`contrib-extensions` feature gate. An extension documented as `Experimental`,
`Alpha`, or `Draft` has no stable compatibility guarantee yet, and its behavior
or configuration can change between releases.

## Crypto provider requirement

The Azure Identity Auth extension talks to Azure over TLS via the Azure SDK's
`reqwest`/`rustls` client, which requires a process-wide `rustls` crypto
provider to be installed. The deployed binary **must** enable exactly one
`crypto-*` feature (`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`, or
`crypto-symcrypt`, forwarded to `otap-df-otap`); the workspace binary's default
build includes `crypto-ring`. A build that enables
`azure-identity-auth-extension` without any `crypto-*` feature installs no
provider, and token acquisition panics at runtime with "No provider set".
