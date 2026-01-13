# TLS/mTLS Architecture Documentation

This document describes the high-level architecture and design principles
of TLS/mTLS support for OTLP/OTAP receivers and exporters.

## Table of Contents

- [Overview](#overview)
- [Design Principles](#design-principles)
- [Receiver Architecture](#receiver-architecture)
- [Exporter Architecture](#exporter-architecture)
- [Certificate Hot-Reload](#certificate-hot-reload)
- [Performance Characteristics](#performance-characteristics)
- [Security Considerations](#security-considerations)
- [Future Enhancements](#future-enhancements)

## Overview

The TLS/mTLS implementation provides secure communication for OTLP and OTAP
receivers and exporters. It supports both basic TLS and mutual TLS (mTLS),
following industry-standard patterns to ensure data protection with minimal
performance overhead.

### Technology Stack

- **rustls**: Modern TLS 1.2/1.3 implementation (replaces OpenSSL)
- **tokio-rustls**: Async TLS integration with Tokio runtime
- **tonic**: gRPC framework (existing dependency, now configured for TLS)
- **notify**: File system watching for certificate hot-reload
- **arc_swap**: Lock-free atomic pointer swaps

### Key Features

- **Zero-downtime certificate reloading**: Receivers update certificates
  without dropping connections.
- **Wait-free hot path**: TLS handshakes use lock-free atomic pointer loads.
- **Concurrent handshake processing**: Handles up to 64 parallel handshakes
  to prevent head-of-line blocking.
- **DoS protection via timeouts**: Enforces 10s handshake timeout to mitigate
  slowloris attacks.
- **Secure by default**: Rejects insecure configurations and requires
  explicit trust anchors.

## Design Principles

### 1. Zero-Downtime Reloads

Certificate updates don't interrupt active connections. Existing connections
continue using the current certificate while new connections use the updated
certificate after reload completes.

### 2. Wait-Free Hot Path

TLS handshakes involve no blocking operations or locks. Certificate lookups
use atomic pointer loads, providing consistent performance under load.

### 3. Secure by Default

- TLS automatically enabled for `https://` endpoints
- Certificate verification cannot be disabled
- System CA certificates included by default
- Handshake timeouts prevent DoS attacks

### 4. Performance Conscious

- **NUMA Local**: Unique TLS resolver instance per receiver (per-core design)
  to minimize cross-NUMA traffic.
- **Concurrent Processing**: Up to 64 parallel handshakes prevented
  head-of-line blocking.
- **Minimal Overhead**: Hot path involves single atomic pointer loads
  (~3-10ns).
- **Efficient Reload**: Lazy checks during handshakes (server cert) or
  background threads (client CA).

## Receiver Architecture

Receivers implement server-side TLS with specific optimizations for
high-throughput environments. Each receiver instance maintains its own isolated
TLS state, ensuring that certificate management (and any associated lock
contention during reloads) remains local to that receiver's execution context.

The architecture separates server certificate management from client CA management:

### Receiver Component Flow

```text
+-----------------------------------------+
|           Client Connection              |
+--------------+--------------------------+
               |
               v
+-----------------------------------------+
|         TLS Handshake Layer             |
|                                          |
|  +--------------------------------+     |
|  |  Server Certificate Resolver   |     |
|  |  (Lazy Reload)                 |     |
|  +--------------------------------+     |
|                                          |
|  +--------------------------------+     |
|  |  Client CA Verifier (mTLS)     |     |
|  |  (File Watch / Poll)           |     |
|  +--------------------------------+     |
|                                          |
|  * Concurrent Processing (64 parallel)  |
|  * Timeout Protection (10s default)     |
+--------------+--------------------------+
               |
               v
+-----------------------------------------+
|         gRPC Service Layer              |
|  * OTLP/OTAP Request Handling           |
|  * Telemetry Processing                 |
+-----------------------------------------+
```

### Server Certificate Management

**Purpose**: Provide server certificates during TLS handshakes with
automatic reloading.

**Reload Strategy**: Lazy Polling

- Certificates checked during TLS handshakes after reload interval expires
- File modification times compared to detect changes
- Reload triggered asynchronously in background
- Current handshake uses existing certificate (no blocking)

**Configuration**:

```yaml
tls:
  cert_file: "/path/to/server.crt"
  key_file: "/path/to/server.key"
  reload_interval: "5m"  # Default
```

**Key Design Elements**:

1. **Leader Election**: Only one thread checks modification times when
   interval expires (prevents thundering herd)
2. **Async Reload**: File I/O happens in background task, not on
   handshake path
3. **Atomic Swap**: New certificate atomically replaces old certificate
   (lock-free)
4. **Graceful Failure**: Failed reload keeps existing certificate
   (no downtime)
5. **Scope**: Hot-reload applies only to file-based certificates
   (`cert_file`/`key_file`). Inline PEM (`cert_pem`/`key_pem`) is loaded
   once at startup.

### Client CA Management (mTLS)

**Purpose**: Verify client certificates for mutual TLS authentication with
automatic CA reloading.

**Reload Strategies**: Two modes available

#### File Watching Mode (Recommended)

- Uses OS-native file notifications (inotify/kqueue/FSEvents)
- Detection time: 50-500ms after file change
- CPU overhead: Near-zero (event-driven)
- Best for: Production deployments, frequent certificate rotation

**Configuration**:

```yaml
tls:
  client_ca_file: "/path/to/client-ca.crt"
  watch_client_ca: true
```

**Key Design Elements**:

1. **Parent Directory Watching**: Watches parent directory to detect
   atomic renames and symlink swaps
2. **Inode-Based Detection**: Uses file identity (inode) rather than
   just modification time
3. **Debouncing**: Coalesces rapid file changes (1-second window)
4. **Atomic Swap**: New CA store atomically replaces old store

#### Polling Mode (Fallback)

- Periodic file checking based on reload interval
- Detection time: Based on `reload_interval` setting
- CPU overhead: Minimal periodic checks
- Best for: Network file systems (NFS, CIFS)

**Configuration**:

```yaml
tls:
  client_ca_file: "/path/to/client-ca.crt"
  watch_client_ca: false
  reload_interval: "1m"
```

**Static Inline CA**:

- `client_ca_pem` is supported but does not participate in hot-reload.
  Changes require a restart.

### TLS Stream Processing

**Concurrent Handshake Processing**:

- Up to 64 parallel TLS handshakes (prevents head-of-line blocking)
- Handshake timeout protection (default: 10 seconds)
- Failed handshakes logged but don't terminate server
- Backpressure: New connections wait in TCP queue when limit reached

**Flow**:

```text
TCP Accept -> TLS Handshake (with timeout) -> TLS Stream -> gRPC Handler
     |              |                              |
     |              +- Failed: Log & Drop          |
     |                                             |
     +--------------- Parallel (64 max) ----------+
```

## Exporter Architecture

Exporters implement client-side TLS for connecting to downstream collectors.
Unlike receivers, hot-reload is not currently supported.

### Exporter Component Flow

```text
+-----------------------------------------+
|      Exporter (OTLP/OTAP)               |
+--------------+--------------------------+
               |
               v
+-----------------------------------------+
|      TLS Configuration Layer            |
|                                          |
|  * Scheme-Driven TLS (https://)         |
|  * Trust Anchors (System + Custom CAs)  |
|  * Client Identity (mTLS)               |
|  * SNI Override                         |
+--------------+--------------------------+
               |
               v
+-----------------------------------------+
|      gRPC Channel (tonic)               |
|  * Connection Pooling                   |
|  * Load Balancing                       |
|  * Retry Logic                          |
+--------------+--------------------------+
               |
               v
+-----------------------------------------+
|      Backend Collector                  |
+-----------------------------------------+
```

### TLS Configuration Strategy

**Scheme-Driven Defaults**:

- `https://` -> TLS enabled with system CAs
- `http://` -> Plaintext (no TLS)
- Explicit `tls` block overrides scheme defaults

**Trust Anchor Management**:

1. **System CAs**: Loaded once per process, cached for reuse
2. **Custom CAs**: Added to trust store alongside system CAs
3. **Validation**: Ensures at least one trust anchor configured

**Reload Interval Note**: The `reload_interval` field is ignored for
exporters today because client-side hot-reload is not implemented.

**mTLS Support**:

- Client certificate and key provided via configuration
- Both cert and key required (validated at startup)
- Combined with trust anchors for full mTLS

### Why No Hot-Reload for Exporters?

Certificate hot-reload is not currently implemented for exporters. This
affects all certificate types:

- **Client Identity**: Client certificate and key (mTLS)
- **Trust Anchors**: Custom CA certificates (`ca_file`) and system CAs

When these files change (e.g., certificate rotation or CA bundle updates),
the process must be restarted to pick up the changes.

**Technical Challenges**:

While the Go OpenTelemetry Collector's exporters support hot-reload for
client certificates via periodic polling, implementing this for Rust
exporters is significantly more complex:

1. **gRPC Channel Lifecycle**: tonic's `Channel` doesn't support runtime
   TLS reconfiguration. Hot-reload would require either:
   - Recreating the gRPC channel (may disrupt in-flight requests)
   - Implementing a custom TLS connector with lazy certificate loading

2. **Connection Pool Management**: The entire connection pool would need
   to be recreated, not just the TLS configuration

3. **In-Flight Requests**: Risk of disrupting active RPCs during reload,
   requiring careful coordination

4. **Integration Complexity**: Unlike receivers which use
   `LazyReloadableCertResolver`, exporters would need significant
   integration work with tonic's transport layer

**Current Approach**: Service restart required for certificate updates

**Future Considerations**:

This feature can be implemented if it becomes an operational requirement.
Potential approaches include:

- Channel replacement strategy (with graceful shutdown)
- Custom TLS connector with lazy certificate loading
- Connection pool recreation with request draining

## Certificate Hot-Reload

### Reload Mechanism Comparison

| Aspect | Server Cert (Receiver) | Client CA (Receiver) | Exporter |
|--------|------------------------|----------------------|----------|
| **Reload Support** | Yes | Yes | No |
| **Detection Method** | Lazy polling | File watch or poll | N/A |
| **Check Frequency** | During handshake | Immediate or interval | N/A |
| **Hot Path Impact** | ~5-10ns | ~3-5ns | N/A |
| **Blocking Operations** | None | None | N/A |
| **Graceful Failure** | Keep old cert | Keep old CA | N/A |

### Reload Workflow

#### Server Certificate Reload

```text
TLS Handshake -> Check interval expired? -> Compare mtime -> Files changed?
                       | No                      |              | No
                   Use current cert          Return           Use current cert
                                                | Yes
                                         Spawn async reload
                                                |
                                         Load new cert
                                                |
                                         Atomic swap
                                                |
                                         Log success
```

#### Client CA Reload (File Watch)

```text
File System Event -> Is our CA file? -> Wait 50ms (settle)
       | Yes              | No              |
   Continue            Ignore         Check inode changed?
                                             | Yes
                                        Debounce (1s)
                                             |
                                        Acquire lock
                                             |
                                        Load new CA
                                             |
                                        Build verifier
                                             |
                                        Atomic swap
                                             |
                                        Log success
```

#### Client CA Reload (Poll)

```text
Timer tick -> Check mtime -> Changed? -> Acquire lock
   | Yes         | No         | Yes      |
Continue      Return       Load CA    Build verifier
                                          |
                                     Atomic swap
                                          |
                                     Log success
```

## Performance Characteristics

> **Note**: The timing values below are typical estimates based on modern
> hardware. Actual performance varies with hardware, load, and configuration.

### Hot Path Performance

**TLS Handshake Path** (per connection):

| Operation | Time | Notes |
|-----------|------|-------|
| Certificate lookup | ~5-10ns | Atomic pointer load |
| CA verification | ~3-5ns | Atomic pointer load |
| Full TLS handshake | ~1-5ms | Dominated by cryptography |

**Key Insight**: Certificate management overhead is negligible compared
to crypto operations.

### Reload Performance

**Server Certificate**:

| Operation | Time | Frequency |
|-----------|------|-----------|
| Interval check | ~1us | Per handshake (amortized) |
| File mtime check | ~1us | When interval expired |
| Certificate reload | ~1-10ms | When file changed (async) |

**Client CA (File Watch)**:

| Operation | Time | Frequency |
|-----------|------|-----------|
| Event processing | ~100us | Per file change |
| CA reload | ~1-10ms | When file changed |

**Client CA (Poll)**:

| Operation | Time | Frequency |
|-----------|------|-----------|
| Poll check | ~1us | Every interval |
| CA reload | ~1-10ms | When changed |

### Memory Overhead

**Per Receiver Instance**:

- Server certificate resolver: ~100 bytes + certificate size (2-10 KB typical)
- Client CA verifier: ~100 bytes + CA store size (varies by number of CAs)
- File watcher (if enabled): +1 OS thread per receiver

**Exporter**:

- TLS configuration: Negligible (loaded at startup)
- System CA cache: Shared across all exporters (loaded once)

### Concurrency Limits

- **Max concurrent handshakes**: 64 per receiver (fixed constant today)
- **Handshake timeout**: 10 seconds (configurable)
- **Reload coordination**: Single reload at a time (per certificate type)

## Security Considerations

### Certificate Validation

**Server-Side (Receivers)**:

- Basic TLS: Server presents certificate; no client validation
- mTLS (when `client_ca_file` is configured):
  - X.509 certificate chain validation
  - Expiration checking
  - Signature verification
  - Standards-compliant via rustls WebPkiClientVerifier

**Client-Side (Exporters)**:

- Server certificate verification always performed when TLS is enabled
- `insecure_skip_verify` is not supported (fails at startup)
- Trust anchors: system CAs + custom CAs
- Hostname verification via SNI
- Optional SNI override for IP-based connections

### Attack Mitigation

**Handshake Timeout Protection**:

- Prevents slowloris-style DoS attacks
- Limits time slow clients can hold connection slots
- Default: 10 seconds per handshake

**Concurrent Handshake Limiting**:

- Bounds resource consumption
- Prevents handshake exhaustion attacks
- Backpressure via TCP accept queue

**Certificate Reload Safety**:

- Graceful failure: keeps existing certificate if reload fails
- Atomic swap: no intermediate state where cert is invalid
- Validation before swap: new cert tested before replacing old

### TLS Protocol Support

- **TLS Versions**: 1.2 and 1.3 only (via rustls)
- **Cipher Suites**: Modern, secure ciphers only
- **No Weak Ciphers**: No support for deprecated algorithms
- **ALPN**: HTTP/2 (h2) for gRPC

## Configuration Layer

### Configuration Structure

**Common Settings** (TlsConfig):

- Certificate and key (file or inline PEM)
- Reload interval for file-based certificates

**Server Settings** (TlsServerConfig):

- Extends common settings
- Client CA configuration (mTLS)
- Hot-reload settings (watch_client_ca)
- Handshake timeout

**Client Settings** (TlsClientConfig):

- Extends common settings
- Server CA configuration (trust anchors)
- System CA inclusion
- SNI override
- Insecure mode flag

### Configuration Validation

**Startup Validation** (fail-fast):

- Both cert and key must be provided (or both omitted)
- At least one trust anchor for TLS connections
- Certificate files must exist and be readable
- File size limits enforced (4MB max)

**Runtime Validation**:

- Certificate parsing validated before swap
- Failed reload keeps existing certificate
- Errors logged but don't crash service

## Future Enhancements

### Potential Improvements

#### 1. Exporter Certificate Hot-Reload

**Challenge**: Requires recreating gRPC channel or implementing custom
TLS connector

**Possible Approaches**:

- Channel replacement with graceful shutdown
- Custom TLS connector with lazy certificate loading
- Connection pool recreation strategy

**Trade-offs**:

- Complexity vs. operational benefit
- In-flight request handling
- Performance impact of channel recreation

#### 2. TLS Version and Cipher Suite Configuration

**Proposed Fields**:

- `min_version`: Minimum TLS version (default "1.2")
- `max_version`: Maximum TLS version
- `cipher_suites`: Explicit cipher suite list
- `curve_preferences`: ECDHE curve preferences

**Proposed Implementation**:

- Allow users to specify minimum TLS version (1.2 or 1.3)
- Configure allowed cipher suites (rustls-supported subset)
- Validate against rustls capabilities at startup

**Challenges**:

- rustls has a fixed, secure cipher suite list
- Need clear documentation of supported options

#### 3. Certificate Revocation List (CRL) Support

**Purpose**: Check if certificates have been revoked

**Challenges**:

- Performance impact of CRL checking
- CRL hot-reload mechanism
- OCSP stapling alternative

#### 4. Hardware Security Module (HSM) / TPM Integration

**Purpose**: Store private keys in hardware-backed security (TPM/HSM)

**Challenges**:

- rustls HSM/TPM support is limited
- Platform-specific APIs (Linux TPM2, Windows CNG)
- Performance considerations for key operations

#### 5. Per-Core TLS Acceptor Instances

**Purpose**: Further reduce cross-NUMA traffic

**Trade-offs**:

- Memory overhead (N x certificate size)
- Reload coordination complexity
- Marginal performance benefit

### Non-Goals

These are explicitly **not planned** to maintain security:

1. **insecure_skip_verify Support**: Would undermine security model
2. **Weak Cipher Suites**: Only modern, secure ciphers allowed
3. **TLS < 1.2 Support**: Modern versions only

## Design Patterns

### Lock-Free Concurrency

**Pattern**: Arc-Swap

- Hot path uses atomic pointer loads (wait-free)
- Updates use atomic pointer swaps
- No locks, no contention
- Graceful memory cleanup via reference counting

**Benefits**:

- Consistent performance under high concurrency
- No thread blocking
- Simple reasoning about correctness

### Leader Election

**Pattern**: Compare-Exchange

- Multiple threads compete to perform expensive operation
- First thread wins, others skip
- Prevents duplicate work
- No coordination overhead on fast path

**Use Cases**:

- Reload interval checks
- File modification time checks
- Prevents thundering herd

### Graceful Degradation

**Pattern**: Fallback to Current State

- Reload failures don't break existing connections
- Log errors but maintain service availability
- Operator notified but service continues

**Philosophy**: Availability over perfection

### Async Offloading

**Pattern**: Fire-and-Forget Background Tasks

- Expensive operations spawn async tasks
- Caller returns immediately
- Results applied when ready
- No blocking on hot path

**Use Cases**:

- Certificate file I/O
- CA certificate reloading
- Certificate parsing and validation

## References

### External Technologies

- **rustls**: [GitHub Repository](https://github.com/rustls/rustls)
- **tokio-rustls**: Async TLS integration
- **notify**: [GitHub Repository](https://github.com/notify-rs/notify)
- **arc-swap**: [GitHub Repository](https://github.com/vorner/arc-swap)
- **tonic**: [GitHub Repository](https://github.com/hyperium/tonic)

### Standards

- **RFC 8446**: TLS 1.3 Protocol
- **RFC 5280**: X.509 Certificate and CRL Profile
- **RFC 6125**: Domain Name Representation in Certificates

### Related Documentation

- [TLS Configuration Guide](./tls-configuration-guide.md) -
  User-facing configuration instructions
- [OpenTelemetry Collector TLS Documentation](https://opentelemetry.io/docs/collector/configuration/#tls-configuration)
- [SPIFFE/SPIRE](https://spiffe.io/)

---

**Note**: This document focuses on high-level architecture and design
principles. For detailed configuration instructions, see the
[TLS Configuration Guide](./tls-configuration-guide.md).
