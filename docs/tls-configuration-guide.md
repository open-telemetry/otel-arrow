# TLS/mTLS Configuration Guide

This guide provides comprehensive instructions for configuring TLS
(Transport Layer Security) and mTLS (mutual TLS) for OTLP and OTAP
receivers and exporters.

## Table of Contents

- [Overview](#overview)
- [Feature Flag](#feature-flag)
- [Receivers (Server-Side TLS)](#receivers-server-side-tls)
- [Exporters (Client-Side TLS)](#exporters-client-side-tls)
- [Configuration Examples](#configuration-examples)
- [Security Best Practices](#security-best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

TLS/mTLS support enables secure communication for:

- **Receivers**: OTLP and OTAP receivers accepting TLS/mTLS connections
  from clients
- **Exporters**: OTLP and OTAP exporters connecting securely to backend
  collectors

**Key Features:**

- Server and client certificate authentication
- Automatic certificate hot-reload (receivers only)
- Multiple certificate sources (files or inline PEM)
- System CA certificate integration
- Configurable handshake timeouts and concurrency limits
- DoS protection via handshake timeouts

## Feature Flag

TLS/mTLS support requires the `experimental-tls` feature flag:

```bash
cargo build --features experimental-tls
```

*Note: This feature flag is temporary and will be removed in a future
release once TLS support is stabilized and enabled by default.*

## Receivers (Server-Side TLS)

Receivers use TLS to accept secure connections from clients. This section
covers both basic TLS (server authentication only) and mTLS (mutual
authentication).

### Basic TLS Configuration

Basic TLS authenticates the server to clients using a server certificate
and private key.

#### Using Certificate Files

```yaml
receivers:
  otlp:
    config:
      listening_addr: "0.0.0.0:4319"
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
```

#### Using Inline PEM

```yaml
receivers:
  otlp:
    config:
      listening_addr: "0.0.0.0:4319"
      tls:
        cert_pem: |
          -----BEGIN CERTIFICATE-----
          MIIDXTCCAkWgAwIBAgIJAKZqNPPWPqCjMA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNV
          ...
          -----END CERTIFICATE-----
        key_pem: |
          -----BEGIN PRIVATE KEY-----
          MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC9...
          -----END PRIVATE KEY-----
```

### mTLS Configuration

mTLS adds client certificate authentication, requiring clients to present
valid certificates signed by a trusted CA.

#### Basic mTLS with Client CA

```yaml
receivers:
  otlp:
    config:
      listening_addr: "0.0.0.0:4319"
      tls:
        # Server certificate
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"

        # Client CA for verification
        client_ca_file: "/path/to/client-ca.crt"
```

#### mTLS with System CA Pool

Trust both custom CA and system CA certificates:

```yaml
receivers:
  otlp:
    config:
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        client_ca_file: "/path/to/client-ca.crt"
        include_system_ca_certs_pool: true
```

System CAs for client authentication are **not** included by default;
set `include_system_ca_certs_pool: true` to combine them with your custom
CA bundle.

#### mTLS with Inline CA Certificate

```yaml
receivers:
  otlp:
    config:
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        client_ca_pem: |
          -----BEGIN CERTIFICATE-----
          MIIDXTCCAkWgAwIBAgIJAKZqNPPWPqCjMA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNV
          ...
          -----END CERTIFICATE-----
```

### Certificate Hot-Reload

The receiver supports automatic certificate reloading without service
restart or connection drops. This is critical for environments with
frequently rotating certificates (e.g., SPIFFE/SPIRE, cert-manager).

#### Server Certificate Reload

Server certificates use **lazy polling** - certificates are checked during
TLS handshakes after the reload interval expires:

```yaml
receivers:
  otlp:
    config:
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        reload_interval: "5m"  # Check every 5 minutes (default)
```

**How it works:**

1. During each TLS handshake, check if `reload_interval` has elapsed
2. If elapsed, compare file modification times
3. If changed, trigger asynchronous reload in background
4. Current handshake uses existing valid certificate
5. New connections use updated certificate after reload completes

**Best Practices:**

- Set `reload_interval` based on certificate rotation frequency
- For daily rotations: `reload_interval: "5m"` or `"10m"`
- For hourly rotations: `reload_interval: "1m"` or `"2m"`
- Ensure certificate overlap window (deploy new cert before old expires)

*Hot-reload applies only to file-based certificates (`cert_file` and
`key_file`). Inline PEM (`cert_pem`/`key_pem`) is loaded once at
startup.*

#### Client CA Certificate Reload (mTLS)

Client CA certificates support two reload mechanisms:

##### File Watching Mode (Recommended)

Uses OS-native file notifications for immediate reload:

```yaml
receivers:
  otlp:
    config:
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        client_ca_file: "/path/to/client-ca.crt"
        watch_client_ca: true  # Enable file watching
```

**Characteristics:**

- **Detection time:** 50-500ms after file change
- **CPU overhead:** Near-zero (event-driven)
- **Best for:** Frequent certificate rotation, production deployments
- **Reliability:** Works with most standard file systems

##### Polling Mode (Fallback)

Uses interval-based checking:

```yaml
receivers:
  otlp:
    config:
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        client_ca_file: "/path/to/client-ca.crt"
        watch_client_ca: false  # Use polling
        reload_interval: "1m"   # Check every minute
```

**Characteristics:**

- **Detection time:** Based on `reload_interval`
- **CPU overhead:** Minimal periodic checks
- **Best for:** Network file systems (NFS, CIFS), environments where
  file watching is unreliable
- **Compatibility:** Works with all file systems

**Inline CA:** `client_ca_pem` is supported but does not hot-reload;
changes require a restart.

#### Reload Mechanism Summary

| Certificate Type | Configuration | Reload Method | When Checked |
|------------------|---------------|---------------|--------------|
| Server Cert | `reload_interval` | Lazy polling | Handshake (if interval expired) |
| Client CA | `watch_client_ca: true` | File watching | Immediate (~50-500ms) |
| Client CA | `watch_client_ca: false` | Polling | Every `reload_interval` |

### Advanced Settings

#### Handshake Timeout

Protects against slowloris-style DoS attacks by limiting TLS handshake
duration:

```yaml
receivers:
  otlp:
    config:
      tls:
        cert_file: "/path/to/server.crt"
        key_file: "/path/to/server.key"
        handshake_timeout: "10s"  # Default: 10 seconds
```

#### Concurrent Handshake Processing

The receiver processes up to 64 concurrent TLS handshakes to prevent
head-of-line blocking from slow clients. When this limit is reached,
new connections wait in the TCP accept queue until a handshake slot
becomes available.

## Exporters (Client-Side TLS)

Exporters use TLS to connect securely to downstream collectors or backends.
**Note: Unlike receivers, exporters do not support automatic certificate
hot-reload; a restart is required for certificate changes.**
Hot-reload is not implemented for exporters; `reload_interval` is ignored
and certificate changes require a restart.

### Exporter TLS Configuration

#### Scheme-Driven TLS (Default Behavior)

TLS is automatically enabled based on the endpoint scheme:

```yaml
exporters:
  otlp:
    config:
      # https:// enables TLS with system CAs
      grpc_endpoint: "https://backend:4317"
```

**Behavior:**

- `https://` -> TLS is enabled with system root CAs
- `http://` -> Plaintext (no TLS)

#### Custom CA Certificate

Trust a specific CA (common for self-signed or internal CAs):

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://backend:4317"
      tls:
        ca_file: "/path/to/ca.crt"
```

#### Inline CA Certificate

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://backend:4317"
      tls:
        ca_pem: |
          -----BEGIN CERTIFICATE-----
          MIIDXTCCAkWgAwIBAgIJAKZqNPPWPqCjMA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNV
          ...
          -----END CERTIFICATE-----
```

### Exporter mTLS Configuration

mTLS requires the client to present a certificate for authentication:

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://backend:4317"
      tls:
        # Trust anchors (server verification)
        ca_file: "/path/to/ca.crt"

        # Client identity (client authentication)
        cert_file: "/path/to/client.crt"
        key_file: "/path/to/client.key"
```

#### mTLS with Inline Certificates

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://backend:4317"
      tls:
        ca_pem: |
          -----BEGIN CERTIFICATE-----
          ...
          -----END CERTIFICATE-----
        cert_pem: |
          -----BEGIN CERTIFICATE-----
          ...
          -----END CERTIFICATE-----
        key_pem: |
          -----BEGIN PRIVATE KEY-----
          ...
          -----END PRIVATE KEY-----
```

### Trust Anchors

#### System CA Certificates (Default)

System CAs are enabled by default for HTTPS endpoints:

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://public-backend.example.com:4317"
      # No tls block needed - uses system CAs automatically
```

#### Disable System CAs

Only trust custom CAs:

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://backend:4317"
      tls:
        ca_file: "/path/to/custom-ca.crt"
        include_system_ca_certs_pool: false
```

#### Insecure Mode Behavior

The `insecure` configuration field interacts with other settings in specific ways:

- **`insecure: true` (no CA):** If no custom CA is configured, TLS
  configuration is skipped, and the connection depends entirely on the
  scheme (`https://` or `http://`).
- **`insecure: true` + `ca_file`:** If a custom CA is present, TLS is
  **still enabled** and the CA is used. The `insecure` flag does NOT
  disable TLS when certificates are configured.

#### SNI Override

Override the server name for TLS SNI and certificate verification:

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://192.168.1.100:4317"
      tls:
        server_name: "backend.internal.example.com"
```

**Use cases:**

- Connecting by IP address
- Endpoint hostname doesn't match certificate
- Load balancer scenarios

### Limitations

#### Certificate Hot-Reload Not Supported

Unlike receivers, exporters **do not support automatic certificate
reloading**. If certificates change (rotation, expiration), the service
must be restarted.

**Affected certificates:**

- **Client Identity**: Client certificate and key (mTLS)
- **Trust Anchors**: Custom CA certificates (`ca_file`) and system CAs

When these files change (e.g., certificate rotation or CA bundle updates),
the process must be restarted to pick up the changes.

**Why not supported?**

While the Go OpenTelemetry Collector supports hot-reload for exporter
certificates via periodic polling, implementing this for Rust exporters
requires significant changes to how tonic's gRPC channel manages TLS
configuration. This would involve either recreating the channel (risking
disruption to in-flight requests) or implementing a custom TLS connector.

**Workarounds:**

- Plan service restarts during certificate rotation windows
- Implement blue-green deployment strategies
- Use longer-lived certificates for exporters

**Future Support:**

Hot-reload support for exporters may be added in a future release if it
becomes an operational requirement. See the architecture documentation
for technical details.

#### insecure_skip_verify Not Supported

Certificate verification **cannot be disabled**. Setting
`insecure_skip_verify: true` causes startup failure (fail-fast).
This is a deliberate security decision.

**Alternatives:**

- Use self-signed certificates with `ca_file`/`ca_pem`
- Use plaintext with `http://` endpoint (not recommended for production)

## Configuration Examples

### Example 1: Basic TLS Receiver + TLS Exporter

```yaml
receivers:
  otlp:
    config:
      listening_addr: "0.0.0.0:4319"
      tls:
        cert_file: "/etc/certs/receiver-server.crt"
        key_file: "/etc/certs/receiver-server.key"

exporters:
  otlp:
    config:
      grpc_endpoint: "https://upstream-collector:4317"
      tls:
        ca_file: "/etc/certs/upstream-ca.crt"
```

### Example 2: mTLS Receiver with File Watching

```yaml
receivers:
  otlp:
    config:
      listening_addr: "0.0.0.0:4319"
      tls:
        # Server certificate (lazy reload every 5 minutes)
        cert_file: "/etc/certs/server.crt"
        key_file: "/etc/certs/server.key"
        reload_interval: "5m"

        # Client CA (immediate reload with file watching)
        client_ca_file: "/etc/certs/client-ca.crt"
        watch_client_ca: true

        # Security settings
        handshake_timeout: "10s"
```

### Example 3: mTLS Exporter with Custom CA

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "https://mtls-backend:4317"
      tls:
        # Server trust
        ca_file: "/etc/certs/backend-ca.crt"
        include_system_ca_certs_pool: false

        # Client identity
        cert_file: "/etc/certs/client.crt"
        key_file: "/etc/certs/client.key"
```

### Example 4: SPIFFE/SPIRE Integration

For short-lived certificates (e.g., 1-hour rotation):

```yaml
receivers:
  otlp:
    config:
      tls:
        cert_file: "/var/run/spire/certs/svid.crt"
        key_file: "/var/run/spire/certs/svid.key"
        reload_interval: "30s"  # Check frequently for 1-hour certs

        client_ca_file: "/var/run/spire/certs/bundle.crt"
        watch_client_ca: true  # Immediate reload
```

### Example 5: Plaintext Fallback

Let the endpoint scheme decide (no custom CA provided):

```yaml
exporters:
  otlp:
    config:
      grpc_endpoint: "http://localhost:4317"
      tls:
        insecure: true  # Defer to the endpoint scheme when no custom CA is set
```

**Notes:**

- `insecure: true` stops a TLS block from forcing TLS when no CA material
  is provided; HTTPS endpoints still use TLS, while HTTP endpoints stay
  plaintext.
- If a custom CA or client identity is configured, TLS is enabled
  regardless of `insecure`.

## Security Best Practices

### Certificate Management

1. **Use Strong Keys**
   - Minimum 2048-bit RSA or 256-bit ECDSA
   - Prefer ECDSA for better performance

2. **Certificate Rotation**
   - Rotate certificates regularly (30-90 days recommended)
   - Automate rotation with cert-manager, SPIFFE/SPIRE, or similar tools
   - Ensure certificate overlap windows (deploy new before old expires)

3. **Private Key Protection**
   - Restrict file permissions: `chmod 600 /path/to/key.pem`
   - Store keys securely
   - Never commit keys to version control

4. **CA Certificate Security**
   - Limit CA certificate distribution
   - Use separate CAs for different trust domains
   - Implement proper CA key management and rotation

### Network Configuration

1. **Use mTLS for Internal Communication**
   - Prefer mTLS over basic TLS for service-to-service communication
   - Provides defense-in-depth

2. **Firewall Rules**
   - Restrict TLS ports to authorized sources
   - Use network segmentation

3. **Monitor Certificate Expiration**
   - Implement alerts for certificates expiring soon
   - Monitor reload failures in logs

### Deployment Considerations

1. **File Watching Reliability**
   - Use `watch_client_ca: true` for most deployments
   - Fall back to polling for network file systems

2. **Reload Intervals**
   - Set `reload_interval` based on certificate lifetime
   - Balance between reload frequency and performance

3. **Testing**
   - Test certificate rotation in staging environments
   - Verify TLS handshakes with `openssl s_client`
   - Monitor connection errors during rotation

## Troubleshooting

### TLS Handshake Failures

**Symptom:** Clients cannot connect, "TLS handshake failed" errors

**Common Causes:**

1. Certificate chain incomplete
2. Certificate expired
3. Hostname mismatch
4. Clock skew between client and server

**Solutions:**

```bash
# Verify certificate chain
openssl verify -CAfile /path/to/ca.crt /path/to/server.crt

# Check certificate expiration
openssl x509 -in /path/to/server.crt -noout -dates

# Test TLS connection
openssl s_client -connect localhost:4319 -servername localhost

# Check system time
date
```

### Certificate Reload Not Working

**Symptom:** New certificates not picked up

**For Receivers:**

1. Verify `reload_interval` is appropriate
2. Check file modification times are actually changing
3. Review logs for reload errors
4. For client CA: ensure `watch_client_ca` is configured correctly
5. For network file systems: try `watch_client_ca: false`

**For Exporters:**

Certificate reload is not supported - restart the service.

### mTLS Client Authentication Failures

**Symptom:** Clients rejected, "certificate required" or
"certificate verify failed"

**Solutions:**

1. Verify client certificate is signed by configured CA
2. Check client certificate is not expired
3. Ensure certificate chain is complete (including intermediate CAs)
4. Verify `client_ca_file` contains the correct CA certificate
5. Check client is sending certificate (not just server)

```bash
# Test mTLS connection with client certificate
openssl s_client -connect localhost:4319 \
  -cert /path/to/client.crt \
  -key /path/to/client.key \
  -CAfile /path/to/ca.crt
```

### Performance Issues

**Symptom:** High CPU usage, slow connections

**Common Causes:**

1. Too frequent certificate reload checks
2. Large certificate files
3. File watching issues on certain file systems

**Solutions:**

1. Increase `reload_interval` (e.g., from 1m to 5m)
2. Reduce certificate file size (remove unnecessary intermediates)
3. Switch from file watching to polling mode:

   ```yaml
   tls:
     client_ca_file: "/etc/certs/ca.crt"
     watch_client_ca: false
     reload_interval: "5m"
   ```

### File Watching Not Triggering Reload

**Symptom:** CA certificates not reloading with `watch_client_ca: true`

**Common Causes:**

1. File system doesn't support native notifications (e.g., NFS, CIFS)
2. Too many rapid file changes causing event loss
3. File replaced via non-atomic operations

**Solution:** Switch to polling mode

```yaml
receivers:
  otlp:
    config:
      tls:
        client_ca_file: "/etc/certs/ca.crt"
        watch_client_ca: false  # Switch to polling
        reload_interval: "30s"  # Check every 30 seconds
```

### Certificate Size Limit Errors

**Symptom:** "File too large" errors

**Cause:** TLS files exceed 4MB limit

**Solutions:**

1. Remove unnecessary certificates from bundles
2. Split large certificate chains
3. Use separate CA files instead of bundling all CAs

### Debugging Tips

1. **Check Certificate Validity**

   ```bash
   # View certificate details
   openssl x509 -in /path/to/cert.crt -text -noout

   # Verify private key matches certificate
   openssl x509 -noout -modulus -in /path/to/cert.crt | openssl md5
   openssl rsa -noout -modulus -in /path/to/key.key | openssl md5
   # (Outputs should match)
   ```

2. **Test Connectivity**

   ```bash
   # Basic TLS test
   curl -v --cacert /path/to/ca.crt https://localhost:4319

   # mTLS test
   curl -v \
     --cacert /path/to/ca.crt \
     --cert /path/to/client.crt \
     --key /path/to/client.key \
     https://localhost:4319
   ```

3. **Review Logs**

   Look for messages like:
   - "TLS certificate reloaded"
   - "Successfully reloaded client CA certificates"
   - "Failed to reload" (with error details)
   - "TLS handshake failed" (with client info)
   - "TLS handshake timed out"

## Additional Resources

- [OpenTelemetry Collector TLS Documentation](https://opentelemetry.io/docs/collector/configuration/#tls-configuration)
- [SPIFFE/SPIRE](https://spiffe.io/)
- [Certificate Management with cert-manager](https://cert-manager.io/)
- [OpenSSL Documentation](https://www.openssl.org/docs/)

---

For architecture and implementation details, see
[TLS Architecture Documentation](./tls-architecture.md).
