<!-- markdownlint-disable MD013 -->

# OAuth 2.0 Client Auth Extension

**Status:** Draft

| | |
| --- | --- |
| **URN** | `urn:otel:extension:oauth2_client_auth` |
| **Feature gate** | `oauth2-client-auth-extension` |
| **Capability** | `bearer_token_provider` |
| **Execution model** | Active + Shared |

Acquires and refreshes OAuth 2.0 access tokens using the client-credentials
grant and exposes them to data-path nodes through the `BearerTokenProvider`
capability, so nodes never construct credentials or manage token refresh
themselves. The token endpoint is reached over TLS via the engine's shared
`TlsClientConfig`, and `client_id` / `client_secret` may be supplied inline or
via files that are re-read on each acquisition for rotation without a restart.
Tokens are cached and refreshed ahead of expiry in a background task, concurrent
cache misses are coalesced onto a single token request, and startup is gated on
the first successful token publish.

For the full design -- problem, goals, lifecycle, configuration reference, and
security considerations -- see
[`docs/oauth2-client-auth-extension.md`](../../../../docs/oauth2-client-auth-extension.md).

## Crypto provider requirement

The OAuth 2.0 Client Auth extension talks to the token endpoint over TLS via a
`reqwest`/`rustls` client, which requires a process-wide `rustls` crypto
provider to be installed. The deployed binary **must** enable exactly one
`crypto-*` feature (`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`, or
`crypto-symcrypt`, forwarded to `otap-df-otap`); the workspace binary's default
build includes `crypto-ring`. A build that enables
`oauth2-client-auth-extension` without any `crypto-*` feature installs no
provider, and token acquisition panics at runtime with "No provider set".
