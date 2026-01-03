# HTTP Proxy Support for OTLP/OTAP Exporters

## Overview

This document describes the HTTP CONNECT proxy tunneling implementation for OTLP and OTAP gRPC exporters. The implementation enables telemetry export through corporate HTTP proxies using the standard HTTP/1.1 CONNECT method.

## Motivation

### Why Custom Proxy Implementation?

The OTAP dataflow project contains two categories of exporters with different proxy requirements:

1. **HTTP-based exporters** (Azure Monitor, Geneva)
   - Use `reqwest` HTTP client
   - Built-in proxy support via `reqwest::Proxy::all()`
   - No custom code needed

2. **gRPC-based exporters** (OTLP, OTAP)
   - Use `tonic` for gRPC
   - No built-in proxy support in tonic
   - Require custom TCP connectors via `tower::service_fn`
   - Need manual HTTP CONNECT tunnel implementation

This implementation fills the gap for gRPC-based exporters, enabling them to work in enterprise environments where all outbound traffic must traverse HTTP proxies.

## Architecture

### HTTP CONNECT Tunneling Flow

The implementation uses the HTTP/1.1 CONNECT method to establish a bi-directional tunnel through the proxy:

#### Step 1: Tunnel Establishment (The Handshake)

```
+-----------+                                          +-----------+
|  Exporter | ------- TCP connection ----------------> |   Proxy   |
|           |                                          |           |
|           | ---- CONNECT backend:4317 HTTP/1.1 --->  |           |
|           | ---- Host: backend:4317 ---------------> |           |
|           | ---- Connection: Keep-Alive -----------> |           |          +----------+
|           |                                          |           | - TCP -> | Backend  |
|           |                                          |           |          +----------+
|           | <--- HTTP/1.1 200 Connection established |           |
+-----------+                                          +-----------+
```

#### Step 2: Data Tunnel (Opaque Byte Stream)

Once the 200 response is received, the exporter uses the same TCP socket for actual protocol data. The proxy acts as a transparent TCP relay, forwarding bytes without interpretation:

```
+-----------+       +-----------+       +--------------+
| OTLP/OTAP |  TCP  |   Proxy   |  TCP  |   Backend    |
| Exporter  |======>|  (relays) |======>|   Server     |
|           |<======|           |<======|              |
+-----------+       +-----------+       +--------------+
          ||                                    ||
          +======================================+

     Protocol inside the tunnel (opaque to proxy):

     Case 1 - TLS target (https://backend:4317):
     +-----------------------------------------+
     | TLS Handshake (negotiates HTTP/2)       |
     | |- ALPN: h2                             |
     | |- Encrypted HTTP/2 + gRPC frames       |
     +-----------------------------------------+

     Case 2 - Cleartext target (http://backend:4317):
     +-----------------------------------------+
     | HTTP/2 Cleartext (h2c)                  |
     | |- HTTP/2 + gRPC frames (unencrypted)   |
     +-----------------------------------------+
```

### Key Design Points

1. **Single TCP connection**: The TCP connection to the proxy carries both the CONNECT handshake and the tunneled gRPC traffic
2. **Transparent tunneling**: After CONNECT succeeds, the proxy doesn't inspect or modify the tunneled data
3. **TLS inside tunnel**: For HTTPS targets, TLS handshake happens *inside* the established tunnel
4. **HTTP/2 multiplexing**: Multiple concurrent gRPC calls multiplex over a single HTTP/2 connection
5. **Socket options**: TCP settings (nodelay, keepalive) are applied to the proxy connection and affect the tunneled traffic

## Configuration

### Environment Variables

Standard proxy environment variables are supported:

```bash
# Proxy for HTTP targets
export HTTP_PROXY=http://proxy.corp.com:8080

# Proxy for HTTPS targets (connection to proxy is still HTTP)
export HTTPS_PROXY=http://proxy.corp.com:8080

# Fallback proxy for all targets
export ALL_PROXY=http://proxy.corp.com:8080

# Bypass proxy for specific hosts
export NO_PROXY=localhost,127.0.0.1,*.internal,192.168.0.0/16
```

**Note**: Variable names are case-insensitive. Both `HTTP_PROXY` and `http_proxy` are recognized.

### YAML Configuration

Explicit proxy configuration in YAML overrides environment variables:

```yaml
grpc_client:
  endpoint: "https://api.example.com:4317"

  # Proxy configuration
  proxy:
    http_proxy: "http://proxy.corp.com:8080"
    https_proxy: "http://proxy.corp.com:8080"
    all_proxy: "http://proxy.corp.com:8080"
    no_proxy: "localhost,127.0.0.1,*.internal"

  # TCP socket options (applied to proxy connection)
  tcp_nodelay: true
  tcp_keepalive: 30s
  tcp_keepalive_interval: 10s
  tcp_keepalive_retries: 3
```

### NO_PROXY Patterns

The `NO_PROXY` variable supports multiple pattern types:

| Pattern | Example | Matches |
|---------|---------|---------|
| Wildcard all | `*` | All hosts (disables proxy) |
| Exact hostname | `localhost` | Exactly "localhost" |
| Wildcard domain | `*.example.com` | `api.example.com`, `foo.example.com` |
| Domain suffix | `.example.com` | `api.example.com`, `example.com` |
| Exact IP | `127.0.0.1` | Exactly 127.0.0.1 |
| IPv4 CIDR | `192.168.0.0/16` | 192.168.0.1 - 192.168.255.254 |
| IPv6 CIDR | `fe80::/10` | Link-local IPv6 range |
| Host with port | `example.com:443` | `example.com` on port 443 only |
| IPv6 with port | `[::1]:4317` | IPv6 localhost on port 4317 |

**Example**:
```bash
NO_PROXY="localhost,*.internal,192.168.0.0/16,10.0.0.0/8,example.com:8080"
```

This bypasses proxy for:
- `localhost`
- Any host ending in `.internal`
- All private IPs in 192.168.0.0/16 and 10.0.0.0/8
- `example.com` on port 8080 specifically

### Proxy Authentication

Basic authentication is supported via credentials in the proxy URL:

```bash
export HTTP_PROXY=http://username:password@proxy.corp.com:8080
```

**Security note**: Credentials are redacted in logs and error messages using the `SensitiveUrl` type.

## Implementation

### Integration with Tonic

The proxy connector integrates with tonic's endpoint as a custom `tower::Service`:

```rust
let connector = make_proxy_connector(proxy_config);
let channel = endpoint.connect_with_connector(connector).await?;
```

For each connection request, the connector:
1. Checks if proxy should be used (based on target URI and NO_PROXY rules)
2. Establishes TCP connection (to proxy or direct)
3. Performs CONNECT handshake if using proxy
4. Applies TCP socket options
5. Returns the connected stream to tonic

### TCP Socket Options

Socket options (nodelay, keepalive) are applied using `socket2` because tokio's `TcpStream` doesn't expose detailed keepalive configuration. This requires a conversion chain: tokio -> std -> socket2 -> std -> tokio.

**Performance note**: This happens once per connection establishment (not per RPC), so overhead is negligible.

### Security Measures

1. **Credential redaction**: `SensitiveUrl` type automatically redacts credentials in logs and error messages
2. **Structured logging**: Uses structured fields instead of raw request strings
3. **Limited error exposure**: Logs only `ErrorKind` and `raw_os_error` from IO errors

Example log output:
```
[DEBUG] Proxy.Using proxy=[REDACTED]@proxy.corp.com:8080 target=https://api.example.com:4317
[DEBUG] Proxy.ConnectRequest target=api.example.com:4317 has_auth=true
[DEBUG] Proxy.Connected
```

## Limitations

### Current Limitations

1. **No HTTPS to proxy**
   - Only `http://` proxy URLs are supported
   - Connection to the proxy server itself is always plain HTTP
   - TLS is supported for the target endpoint (inside the tunnel)
   - See [#1710](https://github.com/open-telemetry/otel-arrow/issues/1710) for HTTPS proxy support

2. **SOCKS proxy not supported**
   - Only HTTP CONNECT method is supported
   - SOCKS4/SOCKS5 proxies are not supported

### Performance Considerations

- **Connection establishment**: Proxy adds one additional round-trip (CONNECT handshake)
- **Hot path**: Not a hot path - connection is established once and reused for all RPCs via HTTP/2 multiplexing
- **NO_PROXY parsing**: Currently parses patterns on each request ([#1711](https://github.com/open-telemetry/otel-arrow/issues/1711) tracks optimization)

## Future Enhancements

1. **HTTPS proxy support** ([#1710](https://github.com/open-telemetry/otel-arrow/issues/1710))
   - TLS connection to proxy server
   - Required for some enterprise environments

2. **NO_PROXY pre-parsing** ([#1711](https://github.com/open-telemetry/otel-arrow/issues/1711))
   - Parse patterns once at startup
   - Eliminate allocations in request path

3. **SOCKS proxy support**
   - Alternative to HTTP CONNECT
   - Common in some environments

4. **Proxy connection pooling**
   - Reuse CONNECT tunnels across multiple gRPC channels
   - Reduce connection overhead

## References

- [HTTP CONNECT Method (RFC 7231)](https://datatracker.ietf.org/doc/html/rfc7231#section-4.3.6)
- [Proxy Authentication (RFC 7617)](https://datatracker.ietf.org/doc/html/rfc7617)
- [OpenTelemetry Collector Proxy Docs](https://opentelemetry.io/docs/collector/configuration/#proxy-support)
- [Tonic Custom Connector Guide](https://docs.rs/tonic/latest/tonic/transport/struct.Endpoint.html#method.connect_with_connector)

## Example End-to-End Flow

```yaml
# Configuration
grpc_client:
  endpoint: "https://otlp.example.com:4317"
  proxy:
    https_proxy: "http://user:pass@proxy.corp.com:8080"
    no_proxy: "localhost,*.internal"
  tcp_keepalive: 30s
```

**Execution flow**:

1. **Configuration merge**: Explicit config overrides env variables
2. **Target resolution**: Extract `otlp.example.com:4317` from endpoint
3. **Proxy selection**:
   - Check NO_PROXY: `otlp.example.com` doesn't match `localhost` or `*.internal`
   - Select HTTPS_PROXY: `http://user:pass@proxy.corp.com:8080`
4. **Connection**:
   - TCP connect to `proxy.corp.com:8080`
   - Apply TCP keepalive (30s interval)
5. **CONNECT handshake**:
   ```
   CONNECT otlp.example.com:4317 HTTP/1.1
   Host: otlp.example.com:4317
   User-Agent: otap-dataflow
   Proxy-Authorization: Basic dXNlcjpwYXNz
   Connection: Keep-Alive
   ```
6. **Proxy response**: `HTTP/1.1 200 Connection established`
7. **TLS handshake** (inside tunnel):
   - ClientHello with ALPN: h2
   - TLS negotiation
   - Encrypted HTTP/2 connection established
8. **gRPC calls**: Multiplexed over HTTP/2, encrypted via TLS, relayed by proxy

**Logging** (credentials redacted):
```
[DEBUG] Proxy.Using proxy=[REDACTED]@proxy.corp.com:8080 target=https://otlp.example.com:4317
[DEBUG] Proxy.Connecting host=proxy.corp.com port=8080
[DEBUG] Proxy.Connected
[DEBUG] Proxy.ConnectRequest target=otlp.example.com:4317 has_auth=true
```
