# Flat File SASL Authentication Extension

Experimental, opt-in SASL credentials for Kafka and other consumers of
`sasl_credential_provider`. Enable the `flat-file-sasl-auth` feature or the
`contrib-extensions` aggregate.

The extension is registered as `urn:otel:extension:flat_file_sasl_auth`. It is
active and shared, using the existing background-provider lifecycle, cache,
watch subscriptions, readiness, retry policy, and metrics tracking.

## Configuration

Declare the extension in a pipeline's `extensions` map. Existing inline-username
configurations remain supported:

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

To keep both credential values outside pipeline configuration, use two files:

```yaml
extensions:
  kafka_credentials:
    type: urn:otel:extension:flat_file_sasl_auth
    config:
      username_file: /var/run/secrets/kafka/username
      password_secret_file: /var/run/secrets/kafka/password
      password_secret_file_refresh: 1m
      startup_timeout: 30s
```

Either `username` or `username_file` is required. `username_file` takes precedence
when both are present; the unused inline username is not validated. A selected
file failure never falls back to the inline value. File paths must be non-empty.
`password_secret_file` remains required; inline passwords are not supported.
`password_secret_file_refresh` defaults to `1h` and accepts human-readable
durations from `10s` through `365d`, inclusive. Despite its historical name, this
setting refreshes both credential files when `username_file` is configured.
There is no separate username polling interval. Unknown fields are rejected.
`startup_timeout` defaults to `30s` and accepts nonzero human-readable durations.
An inline `username` must be non-empty and is redacted in typed configuration
debug output. SASL usernames, including file-backed usernames with colons, are not
subject to HTTP Basic Auth restrictions; mechanism-specific validation belongs
to the consuming node.

Each credential file must be readable UTF-8 no larger than the shared 4 MiB
per-file limit. Only trailing CR and LF characters are removed; spaces and other
whitespace are preserved. An empty value after trimming, invalid UTF-8, or a
file-read error fails credential acquisition with the selected `username_file`
or `password_secret_file` field and path context, without exposing either value.

For a Kubernetes Secret volume, the mounted file contains the decoded real
username/password, not their base64 representations. Mount it read-only and
restrict file access to the collector process. Secret volume updates propagate
eventually, so polling does not guarantee immediate adoption. A `subPath` mount
does not automatically receive Secret updates.

## Acquisition and refresh

The active extension asynchronously reads the selected username source, then the
password file, and validates the complete pair at startup and at the configured
refresh interval. It becomes ready only after a successful acquisition.
Initial failures publish no credential; the shared provider retries
with bounded backoff. The engine waits up to `startup_timeout` for the first
successful acquisition; if none succeeds, pipeline startup fails. The default
`30s` leaves time for the first retry, which waits `5s` to `10s` after a failed
read. A shorter timeout can intentionally fail startup before that retry.
This timeout does not change the refresh interval or limit retries after startup.

Successful acquisitions publish the complete credential to the shared cache.
Reading and validating both values before publication is **not an atomic
cross-file snapshot**: separately updated files can yield a mixed-version pair
even when both reads succeed. Coordinate credential rotation with the file
publisher and authentication system; the provider does not verify matching
versions or make two-file replacement atomic.

Credential requests clone the cached value without file I/O; concurrent cache
misses use the shared
provider's coalesced acquisition. Each independent stream subscription immediately
emits the latest cached credential, if available, then receives subsequent
publications. Credentials have no expiry.

Every successful acquisition publishes a credential, even when the file and
credential values are unchanged. A stream item indicates successful acquisition,
not necessarily credential rotation; consumers deciding whether to reconnect
should compare credential values. Watch subscriptions expose the latest value,
so slow subscribers may coalesce multiple publications.

A failed read or validation of either file retains the last good complete
credential, keeps streams open, and retries using the existing shared backoff
policy. File replacement or content
changes are picked up on the next successful acquisition without restarting the
provider. Changing the inline username or refresh configuration requires a
collector restart. The poll interval is not a credential expiry or a maximum
cache age: failures may retain the last good pair beyond that interval.
There is no Azure Key Vault integration.

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
