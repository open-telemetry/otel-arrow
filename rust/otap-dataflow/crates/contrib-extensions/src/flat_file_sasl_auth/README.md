# Flat File SASL Authentication Extension

Experimental, opt-in SASL credentials for Kafka and other consumers of
`sasl_credential_provider`. Enable the `flat-file-sasl-auth` feature or the
`contrib-extensions` aggregate.

The extension is registered as `urn:otel:extension:flat_file_sasl_auth`. It is
active and shared, using the existing background-provider lifecycle, cache,
watch subscriptions, readiness, retry policy, and metrics tracking.

## Configuration

Declare the extension in a pipeline's `extensions` map:

```yaml
extensions:
  kafka_credentials:
    type: urn:otel:extension:flat_file_sasl_auth
    config:
      username: kafka-user
      password_secret_file: /var/run/secrets/kafka/password
      password_secret_file_refresh: 1h
      startup_timeout: 30s
```

`username` and `password_secret_file` are required and must be non-empty.
`password_secret_file_refresh` defaults to `1h` and accepts human-readable
durations from `10s` through `365d`, inclusive. Unknown fields are rejected.
`startup_timeout` defaults to `30s` and accepts nonzero human-readable durations.
`username` is an inline string, redacted in typed configuration debug output.
There is no username file or inline password option. SASL usernames are not
subject to HTTP Basic Auth restrictions; mechanism-specific validation belongs
to the consuming node.

`password_secret_file` must point to a readable UTF-8 file no larger than the
shared 4 MiB file limit. Only trailing CR and LF characters are removed; spaces
and other whitespace are preserved. An empty password after trimming, invalid
UTF-8, or a file-read error fails credential acquisition with field and path
context, without exposing the password.

For a Kubernetes Secret volume, the mounted file contains the decoded real
password, not its base64 representation. Mount it read-only and restrict file
access to the collector process.

## Acquisition and refresh

The active extension asynchronously reads and validates the password at startup
and at the configured refresh interval. It becomes ready only after a successful
acquisition. Initial failures publish no credential; the shared provider retries
with bounded backoff. The engine waits up to `startup_timeout` for the first
successful acquisition; if none succeeds, pipeline startup fails. The default
`30s` leaves time for the first retry, which waits `5s` to `10s` after a failed
read. A shorter timeout can intentionally fail startup before that retry.
This timeout does not change the refresh interval or limit retries after startup.

Successful acquisitions pair the latest file password with the configured
username and publish the credential to the shared cache. Credential requests
clone the cached value without file I/O; concurrent cache misses use the shared
provider's coalesced acquisition. Each independent stream subscription immediately
emits the latest cached credential, if available, then receives subsequent
publications. Credentials have no expiry.

A failed refresh retains the last good credential, keeps streams open, and
retries using the existing shared backoff policy. File replacement or content
changes are picked up on the next successful acquisition without restarting the
provider. Changing the inline username or refresh configuration requires a
collector restart. There is no username-file or Azure Key Vault integration.

The SASL source is a thin protocol-specific adapter over the same bounded,
zeroizing file reader and background provider used by the Basic Auth extension;
it does not expose a Basic Auth capability or add separate synchronization.

## Metrics

The `extension.flat_file_sasl_auth` metric set records `auth_successes`,
`auth_failures`, `auth_publishes`, and `auth_success_latency` (milliseconds).
These describe credential acquisition and publication, not Kafka login success.

## Kafka integration

This extension only supplies credentials. Kafka receiver capability binding is
a separate integration tracked by
[#4276](https://github.com/open-telemetry/otel-arrow/issues/4276).
That integration is startup-only. Provider refresh does not automatically rotate
credentials on, or reconnect, live Kafka connections. This change does not add
receiver binding or live connection rotation; the consumer owns SASL mechanism
selection and connection behavior.
