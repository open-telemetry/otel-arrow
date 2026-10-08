# Flat File SASL Authentication Extension

Experimental, opt-in SASL credentials for Kafka and other consumers of
`sasl_credential_provider`. Enable the `flat-file-sasl-auth` feature or the
`contrib-extensions` aggregate.

The extension is registered as `urn:otel:extension:flat_file_sasl_auth`. It is
passive and shared, with immutable credentials and no background task.

## Configuration

Declare the extension in a pipeline's `extensions` map:

```yaml
extensions:
  kafka_credentials:
    type: urn:otel:extension:flat_file_sasl_auth
    config:
      username: kafka-user
      password_secret_file: /var/run/secrets/kafka/password
```

Both fields are required and must be non-empty. Unknown fields are rejected.
`username` is an inline string, redacted in typed configuration debug output.
There is no username file or inline password option. SASL usernames are not
subject to HTTP Basic Auth restrictions; mechanism-specific validation belongs
to the consuming node.

`password_secret_file` must point to a readable UTF-8 file no larger than the
shared 4 MiB file limit. Only trailing CR and LF characters are removed; spaces
and other whitespace are preserved. An empty password after trimming, invalid
UTF-8, or a file-read error fails extension construction with field and path
context, without exposing the password.

For a Kubernetes Secret volume, the mounted file contains the decoded real
password, not its base64 representation. Mount it read-only and restrict file
access to the collector process.

## Startup-only behavior

Each extension instance reads and validates the password once during startup,
before exposing its capability. Subsequent requests clone the cached credential
without file I/O. Every independent stream subscription immediately emits that
credential, then stays pending. The credential has no expiry.

Changing or replacing the file has no effect on an existing instance. Restart
the collector to adopt a changed password. There is no polling, refresh interval,
retry loop, rotation handling, or Azure Key Vault integration.

## Kafka integration

This extension only supplies credentials. Kafka receiver capability binding is
a separate integration tracked by
[#4276](https://github.com/open-telemetry/otel-arrow/issues/4276).
This change does not add that binding or change live Kafka connections.
The consumer owns SASL mechanism selection and connection behavior.
