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
