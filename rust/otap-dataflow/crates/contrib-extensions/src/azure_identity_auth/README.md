<!-- markdownlint-disable MD013 -->

# Azure Identity Auth Extension

**Status:** Draft

| | |
| --- | --- |
| **URN** | `urn:microsoft:extension:azure_identity_auth` |
| **Feature gate** | `azure-identity-auth-extension` |
| **Capability** | `bearer_token_provider` |
| **Execution model** | Active + Shared |

Acquires and refreshes Azure OAuth access tokens via the `azure_identity` SDK
and exposes them to data-path nodes through the `BearerTokenProvider`
capability, so nodes never construct credentials or manage token refresh
themselves. Supported flows: Managed Identity, developer tooling, and Workload
Identity Federation.

For the full design -- problem, goals, lifecycle, configuration reference, and
security considerations -- see
[`docs/azure-identity-auth-extension.md`](../../../../docs/azure-identity-auth-extension.md).

## Crypto provider requirement

The Azure Identity Auth extension talks to Azure over TLS via the Azure SDK's
`reqwest`/`rustls` client, which requires a process-wide `rustls` crypto
provider to be installed. The deployed binary **must** enable exactly one
`crypto-*` feature (`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`, or
`crypto-symcrypt`, forwarded to `otap-df-otap`); the workspace binary's default
build includes `crypto-ring`. A build that enables
`azure-identity-auth-extension` without any `crypto-*` feature installs no
provider, and token acquisition panics at runtime with "No provider set".
