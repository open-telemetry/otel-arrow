# OTLP Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:otlp` (`urn:otel:receiver:otlp`)
- Feature gate: Default
- Stability: Experimental

## Overview

The OTLP receiver accepts OTLP/gRPC, OTLP/HTTP, or both. It forwards received
logs, metrics, and traces into the pipeline as pdata and can wait for immediate
downstream ACK/NACK outcomes before responding to clients.

## Getting Started

Enable at least one OTLP protocol. For gRPC only:

```yaml
type: receiver:otlp
config:
  protocols:
    grpc:
      listening_addr: "127.0.0.1:4317"
```

## Configuration

```yaml
type: receiver:otlp
config:
  # At least one protocol must be configured.
  protocols:
    grpc:
      # Enables and configures OTLP/gRPC.
      listening_addr: "127.0.0.1:4317"
      wait_for_result: true
      timeout: 30s
    http:
      # Enables and configures OTLP/HTTP.
      listening_addr: "127.0.0.1:4318"
      wait_for_result: true
      timeout: 30s
```

Authorization is optional. Binding the `bearer_token_authorizer` capability
makes the receiver require a bearer token for OTLP export requests accepted on
the configured gRPC and HTTP export endpoints. The receiver bounds each
authorization call with that protocol's existing `timeout`. When the gRPC
timeout is unset, authorization still has a `10s` limit; HTTP already defaults
to a `30s` request timeout. Provider operation timeouts, such as a Kubernetes
review timeout, remain independently configurable:

```yaml
extensions:
  k8s_authz:
    type: extension:k8s_service_account_token_auth
    config:
      audiences:
        - audience: "otlp-collector"

nodes:
  otlp_in:
    type: receiver:otlp
    capabilities:
      bearer_token_authorizer: k8s_authz
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
```

The authorizer validates the token before the receiver reads or forwards the
request payload. Authentication failures return `UNAUTHENTICATED`/HTTP 401 and
policy denials return `PERMISSION_DENIED`/HTTP 403. An authorizer that cannot
reach a decision fails closed with `UNAVAILABLE`/HTTP 503.

Receivers with no `bearer_token_authorizer` binding accept traffic unchanged.

Common gRPC protocol fields include:

- `listening_addr`
- `request_compression`
- `response_compression`
- `max_concurrent_requests`
- `max_concurrent_streams`
- TCP and HTTP/2 tuning fields
- `wait_for_result`
- `timeout`
- `tls`

Common HTTP protocol fields include:

- `listening_addr`
- `max_concurrent_requests`
- `max_request_body_size`
- `wait_for_result`
- `timeout`
- `accept_compressed_requests`
- `tls`

## Examples

gRPC and HTTP:

```yaml
type: receiver:otlp
config:
  protocols:
    grpc:
      listening_addr: "127.0.0.1:4317"
    http:
      listening_addr: "127.0.0.1:4318"
```

### Pressure-aware rate admission

Bind one named limiter at the receiver node with `rate_limiters: [name]`.
Use `rate_limiters: []` to opt out of an inherited limiter. With several
effective limiters, an explicit single-name binding is required.

```yaml
otlp:
  type: receiver:otlp
  rate_limiters: [ingress]
  config:
    protocols:
      grpc:
        listening_addr: "127.0.0.1:4317"
```

The limiter observes request bytes while memory is normal. Under configured
pressure, `observe_only` reports `would_throttle` without rejecting, while
`enforce` rejects over-limit requests. V1 creates one bucket per receiver
instance; it does not implement tenant or group-wide fairness.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.otlp.requests`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.requests.started` | `{request}` | `signal`, `protocol` | Number of requests admitted to the pipeline send path. |
| `receiver.otlp.requests.completed` | `{request}` | `signal`, `protocol` | Number of admitted requests whose receiver work terminated. |
| `receiver.otlp.requests.payload_size` | `By` | `signal`, `protocol` | Decompressed payload bytes for requests admitted to the pipeline send path. |

#### `receiver.otlp.rejections`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.rejections.requests` | `{request}` | `protocol`, `error.type` | Number of requests rejected before pipeline admission. |

Rate-admission outcomes are reported by the engine metric set
`admission.rate_limiter`. Its `refusals` counter uses the bounded attributes
`dimension=bytes` and `refusal=would_throttle|throttle|oversized`. The metric is
scoped to the configured node entity, but tenant or request identities are
never measurement attributes. Protocol-specific enforced rejections remain in
`receiver.otlp.rejections`.

#### `receiver.otlp.acknowledgements`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.acknowledgements.responses` | `{response}` | `signal`, `outcome` | Number of routed or invalid acknowledgement responses. |

#### `receiver.otlp.transport`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.transport.errors` | `{error}` | `protocol` | Number of transport-level server errors. |

Attribute values are bounded: `signal` is `traces`, `metrics`, or `logs`;
`protocol` is `grpc` or `http`; `outcome` is `success`, `failure`, or
`refused`; and `error.type` is `memory_pressure`, `concurrency_limit`,
`rate_limit`, `authentication`, `permission_denied`,
`authorization_unavailable`, `payload_too_large`, `invalid_request`, or
`internal`.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otlp.receiver.grpc.start` | `info` | OTLP/gRPC listener startup. |
| `otlp.receiver.http.start` | `info` | OTLP/HTTP listener startup. |
| `otlp.receiver.drain_ingress` | `info` | Receiver ingress drain started. |
| `otlp.receiver.shutdown` | `info` | Receiver shutdown completed. |

## Limits

- At least one of `protocols.grpc` or `protocols.http` is required.
- HTTP request body limits apply to both compressed and decompressed payload
  size.
- V1 rate limiting measures decompressed request bytes. A request larger than
  the configured burst is rejected as non-retryable while pressure gating is
  active: HTTP returns 413 without `Retry-After`, and gRPC sends negative retry
  pushback.
- An exhausted receiver may reject before decompressed request weight is known.
  This early HTTP 503 or gRPC `RESOURCE_EXHAUSTED` response has no retry hint.
  Exact retry guidance or non-retryable oversized classification is available
  only after the weighted admission point.
- `wait_for_result` reflects the immediate downstream node, not necessarily the
  final exporter.

## Related Docs

- [OTLP receiver design](../../../../../docs/otlp-receiver.md)
- [Configuration model](../../../../../docs/configuration-model.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
