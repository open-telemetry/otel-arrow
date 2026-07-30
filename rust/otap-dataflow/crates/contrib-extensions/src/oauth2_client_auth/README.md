<!-- markdownlint-disable MD013 -->

# OAuth 2.0 Client Auth Extension

**Status:** Draft

| | |
| --- | --- |
| **URN** | `urn:otel:extension:oauth2_client_auth` |
| **Feature gate** | `oauth2-client-auth-extension` |
| **Capability** | `bearer_token_provider` |
| **Execution model** | Active + Shared |

Acquires and refreshes OAuth 2.0 access tokens and exposes them to data-path
nodes through the `BearerTokenProvider` capability, so nodes never construct
credentials or manage token refresh themselves. Two grants are supported: the
client-credentials grant (RFC 6749 section 4.4, client id + secret) and the
JWT-bearer grant (RFC 7523 section 2.1, a signed JWT assertion). The token
endpoint is reached over TLS via the engine's shared `TlsClientConfig`, and the
`client_id` / `client_secret` / signing-key material may be supplied inline or
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
