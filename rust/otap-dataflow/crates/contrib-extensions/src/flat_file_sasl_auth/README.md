# Flat File SASL Auth Extension

## Metadata

- URN: `urn:otel:extension:flat_file_sasl_auth`
- Feature gate: `flat-file-sasl-auth` (or `contrib-extensions`)
- Capability provided: `sasl_credential_provider`
- Execution model: Active + Shared
- Stability: Draft

## Overview

Reads both username and password from local UTF-8 files and exposes them through
`SaslCredentialProvider`. Files may be ordinary files or Kubernetes Secret
volume mounts; the extension does not call the Kubernetes API or decode base64.
Each file contains the actual credential value, not a secret reference.

File acquisition is protocol-neutral and reuses the bounded secret-file reader
used by the existing Basic Auth extension. The provider constructs
`SaslCredential` directly, rejecting empty values without applying HTTP Basic
restrictions. The consumer owns SASL mechanism selection and mechanism-specific
validation. Existing Oracle and Basic Auth configurations remain supported;
the Basic Auth extension also supports an optional `username_file` through the
same acquisition helper.

## Configuration

Declare an extension instance in the pipeline's `extensions:` map:

```yaml
extensions:
  kafka_credentials:
    type: urn:otel:extension:flat_file_sasl_auth
    config:
      username_file: /run/secrets/auth/username
      password_secret_file: /run/secrets/auth/password
      credentials_file_refresh: 30m
```

| Field | Default | Description |
| --- | --- | --- |
| `username_file` | Required | Non-empty path to the UTF-8 username file. |
| `password_secret_file` | Required | Non-empty path to the UTF-8 password file. |
| `credentials_file_refresh` | `1h` | Interval for reading both files; between `10s` and `365d`, inclusive. |

Both paths must be accessible to the collector process. Absolute paths avoid
dependence on its working directory. Inline credential fields and unknown fields
are rejected. Each file uses the shared 4 MiB read limit. Only trailing CR/LF
characters are stripped; other whitespace is preserved. Empty values after
stripping, invalid UTF-8, and read failures are reported without file contents.

A consumer that supports the SASL capability binds the extension through:

```yaml
capabilities:
  sasl_credential_provider: kafka_credentials
```

Kafka receiver integration is tracked separately in
[#4276](https://github.com/open-telemetry/otel-arrow/issues/4276). This extension
does not add that receiver integration or live Kafka connection rotation.

## Acquisition and rotation

The active extension acquires credentials at startup and signals readiness only
after both files have been read and the pair validated. Startup failures do not
publish a credential; retries follow the shared bounded backoff policy and
startup remains subject to the engine's extension-readiness timeout.

Each successful refresh publishes a complete pair to a shared cache. New
subscribers receive the current pair immediately; existing subscriptions remain
open across failures. Subscribers observe the latest value, not an ordered
history of every update. If either read or validation fails, no replacement is
published, the last good pair remains available, and the failure is logged and
retried with bounded backoff. File credentials have no known expiry; the polling
interval is not an expiry or a maximum cache age.

Two file reads are **not an atomic snapshot**. Updating files separately can
produce a mixed-version pair even if both reads succeed. Coordinate credential
rotation, for example by keeping old/new combinations valid during the update
window or restarting the collector after provisioning a complete pair. Atomic
replacement of each individual file does not make the two reads atomic.

For Kubernetes Secret volumes, allow for Kubernetes propagation plus the polling
interval. `subPath` Secret mounts do not receive automatic updates. The provider
publishes refreshed values but the consumer must explicitly apply them; the
initial Kafka capability integration covers startup acquisition only.

## Security and telemetry

Restrict file permissions and mount secrets read-only. Credentials are held in
redacted, zeroizing secret types, not logged or placed in telemetry attributes.
Failure messages identify the configuration field and path.

The metric set `extension.flat_file_sasl_auth` reports `auth_successes`,
`auth_failures`, `auth_publishes`, and `auth_success_latency` (milliseconds).
The warning event `flat_file_sasl_auth.credential_refresh_failed` reports failed
background acquisitions without exposing credential values.

## Building

From the `rust/otap-dataflow` directory:

```sh
cargo build --release --features flat-file-sasl-auth
```

The extension is opt-in and also included in the aggregate `contrib-extensions`
feature.
