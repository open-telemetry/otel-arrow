# Tenant Tokens

## Overview

A **tenant token** names the identity a request belongs to. It is a small set
of `key: value` identifiers, resolved once at the receiver from request-scoped
material -- transport headers today, peer address and authorization data as
those land -- and carried with the request for the rest of its life in the
engine.

Tokens replace the general-purpose transport header map the engine used to
carry. Instead of copying every captured header into an owned name/value pair
per request, the engine copies only the values the operator declared, packed
into one allocation with the key names compiled out.

Downstream nodes use the resolved token to make decisions:

- exporters re-emit retained values as outbound headers,
- the partition processor names partitions from them,
- the Kafka exporter routes topics and sets partition keys from them.

Tenant tokens are **engine scoped**. One declaration covers every pipeline
group, so a value resolved in one pipeline is readable by the same key in the
next hop, and two pipelines cannot disagree about what `tenant_id` means.

## Configuration Scope

Tokens are declared once, under `engine:`:

```yaml
engine:
  tenant_tokens:
    edge_tenant:
      extractors:
        - key: tenant_id
          transport_header: x-tenant-id
          retain: true
        - key: project_id
          transport_header: x-project-id
          retain: true
```

There is no per-group, per-pipeline, or per-node override. A token is a shared
vocabulary; scoping it would defeat the purpose.

## Tokens and Extractors

A token is a list of extractors. **The token resolves only when every one of
its extractors resolves.** `edge_tenant` above therefore requires both headers
to be present; a request carrying only `x-tenant-id` resolves nothing.

Give independent keys their own tokens when they should resolve
independently:

```yaml
engine:
  tenant_tokens:
    tenant:
      extractors:
        - key: tenant_id
          transport_header: x-tenant-id
          retain: true
    project:
      extractors:
        - key: project_id
          transport_header: x-project-id
          retain: true
```

### Extractor kinds

| Field | Source of the value |
| ----- | ------------------- |
| `transport_header: <name>` | An inbound header, matched case-insensitively |
| `generic_key: <value>` | A static value the pipeline mints for itself |
| `remote_address: true` | The network peer's address |
| `imported_key: <name>` | A key offered by an upstream topic boundary |

Every extractor names the token `key` it fills, and every one accepts the two
modifiers below.

### `retain`

By default an extracted value is available for **matching only**: conditions
can test it, but it costs no bytes in the request and cannot be read back.

`retain: true` keeps the value, which is what lets an exporter re-emit it or a
processor name a partition from it.

### `bag`

`bag: true` implies `retain`, and additionally carries the key's *name*
alongside the value. This is the only way a name travels with a request --
every other use of a key name is compiled out at startup.

The bagged run is encoded as a complete OTLP `repeated KeyValue` field, so
instrumentation can append the whole request's tenant identity to span or log
attributes by copying bytes, with no per-entry walk and no re-encoding.

Use it when something downstream needs the identity as attributes; leave it off
otherwise, since names cost wire bytes per request.

## Egress: naming the wire

**A token says nothing about the header name its retained value is re-emitted
under.** The token is the portable identity; the wire name is a site-specific
decision belonging to whichever node does the emitting.

Exporters therefore declare the mapping themselves:

```yaml
exporter:
  type: urn:otel:exporter:otlp_grpc
  config:
    grpc_endpoint: http://backend:4317
    tenant_headers:
      - key: tenant_id
        header: x-acme-customer
      - key: trace_state
        header: x-trace-state
        binary: true
```

Consequences worth knowing:

- The same token can go out as `x-acme-customer` from one exporter and
  `x-customer-id` from another. Neither is "the" name.
- `binary: true` selects gRPC binary metadata. The `-bin` suffix gRPC requires
  is appended at startup if the configured name lacks it, and the raw bytes are
  emitted rather than a base64 wire form, so a value cannot be double-encoded
  by a second hop.
- **Static config wins on collision.** A tenant header configured under a name
  the exporter also sets statically -- `authorization`, typically -- is
  dropped. Tenant material can never override or shadow a backend credential.
- A key that no token retains is reported and skipped at startup. Nothing is
  emitted for it.

## Conditions

Nodes that make decisions from tenant tokens evaluate ordered conditions,
first match wins. Each condition is a set of entries that must **all** match:

```yaml
routes:
  - entries:
      - key: tenant_id
        value: acme
      - key: tier          # no `value`: present with any value
    topic: acme_priority
  - entries:
      - key: tenant_id
        value: acme
    topic: acme_bulk
```

Matching is **exact byte equality**. Values are compiled to dense symbols at
startup and conditions to packed signature words, so the hot path is a single
hash-map probe -- but the probe compares whole values, exactly as a database
hash join verifies equality after the hash narrows the candidates. A hash
collision cannot route one tenant's data to another tenant's destination.

A value that no condition mentions resolves to a reserved "unknown" symbol, so
it matches nothing rather than colliding with something.

## Consumers

### OTLP gRPC exporter

`tenant_headers` maps keys to outbound metadata, as shown above.

### Partition processor

`partition_key` names the token key whose value the processor writes, and the
partition name is derived from the packed context. The layout is positional --
a key's slot is fixed by the registry, not by arrival order -- so two requests
with equal tenant values produce byte-equal contexts and therefore identical
partition names, independent of header ordering or producer architecture.

### Kafka exporter

Topic routing, partition keys, and record headers can all read the tenant
context. Note the deliberate asymmetry:

- an unresolvable **topic-routing** key fails the exporter at startup, since
  falling back to the static topic would deliver one tenant's data to another
  tenant's topic;
- an unresolvable **header** key only drops decoration, so it is reported and
  skipped.

### Kafka receiver

Message headers are resolved into a tenant context exactly as gRPC metadata is,
so a Kafka hop is not a hole in the identity chain.

## Example: capture, carry, re-emit

```yaml
engine:
  tenant_tokens:
    tenant:
      extractors:
        - key: tenant_id
          transport_header: x-tenant-id
          retain: true

  groups:
    default:
      pipelines:
        main:
          nodes:
            receiver:
              type: "receiver:otlp"
              config:
                protocols:
                  grpc:
                    listening_addr: 0.0.0.0:4317
            exporter:
              type: "exporter:otlp_grpc"
              config:
                grpc_endpoint: http://backend:4317
                tenant_headers:
                  - key: tenant_id
                    header: x-acme-tenant
          connections:
            - from: receiver
              to: exporter
```

An inbound `x-tenant-id: acme` is resolved at the receiver, carried through the
pipeline as one packed word list, and re-emitted to the backend as
`x-acme-tenant: acme`. Any other inbound header is dropped, because no token
declared it.

## Limits

The packed context trades breadth for density. The compiler enforces the
budget at startup and fails the configuration rather than truncating:

| Declared keys | Distinct values per key |
| ------------- | ----------------------- |
| up to 4 | about 65,000 |
| up to 8 | 254 |
| up to 16 | 14 |

Values are counted per key across all conditions that mention it, not per
request; a key that is retained but never matched against consumes no symbol
space.

Other limits:

- Header names are matched exactly, case-insensitively. Regex and glob
  patterns are not supported.
- Only OTLP gRPC, OTLP HTTP, and Kafka receivers resolve tenant tokens today.
  The OTAP receiver does not yet.

## See also

- [Configuration reference](configuration.md)
- [Design notes](multitenant-token-propagation.md) -- the reasoning behind the
  packed layout, the encoding, and the matching discipline.
