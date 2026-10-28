# Microsoft JSON Receiver

## Metadata

- Type: `urn:microsoft:receiver:json`
- Full URN: `urn:microsoft:receiver:json`
- Feature: `microsoft-json`
- Stability: In progress; not operational
- Signals: Logs
- Audience: Browser telemetry clients
- Authentication: None, by design

## Overview

The Microsoft JSON receiver accepts formatted JSON events in Microsoft Common
Schema (CS) or Application Insights / Azure Monitor sent over HTTP as sent by
the Microsoft SDKs and converts them to OpenTelemetry log records. The complete
design supports OneCollector newline-delimited JSON and Application Insights /
Azure Monitor JSON payloads.

The receiver is browser-focused. It is designed to provide the cross-origin
request controls required by browser clients through CORS (Cross-Origin
Resource Sharing).

Browser telemetry is accepted without authentication. Credentials embedded in
a web page cannot be kept secret from users or scripts running in that page,
so the receiver does not claim that API keys or other browser-provided values
authenticate the sender.

The component is registered and validates its receiver-level configuration,
but intentionally returns a `NotImplemented` error during construction.
Construction does not open a listener, read endpoint files, process requests,
or emit telemetry.

A standalone endpoint compiler and bounded JSON-to-log mapper are available.
They are not invoked by receiver construction yet. Arrow construction and
HTTP/runtime integration are not implemented.

## Standalone mapping implementation

The [endpoint compiler](field_mapping.rs) loads endpoint YAML files, resolves
environment attributes once, compiles iKey filters, flattens nested rules and
merges event overrides. Endpoint and external filter file references are
confined to the pipeline configuration directory, including symlink resolution.
Startup loading returns either the complete route set or an explicit error.

The [event mapper](event_mapping.rs) produces a typed intermediate log, not an
Arrow record batch. It implements validation/coercion, required defaults,
event-name transforms, iKey verdicts and remapping, severity, timestamps, trace
context, resource/log attributes, instrumentation scope, additional targets,
unmapped-field policies and retained-source `filtered_item_size`.

Mapping is transactional per event: a rejected event or exceeded safeguard
returns an error, never a partially accepted log. Override `reject` remains a
distinct request-payload rejection for the future runtime to propagate.
Explicit drops remain distinguishable from errors.

Internal safeguards bound each mapping operation:

Reusable traversal and file-read safeguards live in the configuration crate's
[`data_limits`](../../../../config/src/data_limits.rs) module; receiver error
translation remains local. This does not wire endpoint loading into the engine
or introduce a new runtime policy.

| Resource | Limit |
| --- | --- |
| Endpoint routes per receiver | 64 |
| Each endpoint or external filter YAML file | 1 MiB |
| JSON/rule nesting depth | 64 |
| JSON value nodes per logical event | 65,536 |
| Compiled rules, including expanded overrides | 4,096 |
| Traversal work units per event or compilation | 16,777,216 |
| Cumulative emitted data and final intermediate JSON size | 3 MiB |
| Distinct resource plus log attributes per event | 8,192 |

Work accounting includes inspected bytes, rule/path comparisons and emitted
data. Fan-out and overwritten destinations consume the cumulative output
budget too. The output budget is deliberately conservative: an event can be
refused before its final representation reaches 3 MiB. The configured
`max_event_size` additionally caps the compact source JSON representation.
When omitted, it does not impose an input cap equal to the output budget;
request-body limits and independent traversal limits still apply.
Encoded request size, decompression limits and request-wide batch limits must
be enforced by the future HTTP/runtime implementation.

## Ingestion modes

| Method | Path | Content type | Description |
| -------- | ------ | ------------- | ------------- |
| POST | `/OneCollector/1.0` | `application/x-json-stream` or `application/json` | Newline-delimited JSON. Primary path for the 1DS Web SDK via `XMLHttpRequest`, `fetch()` or `sendBeacon()`. |
| GET | `/OneCollector/1.0[/t.js]?<values>` | URL-encoded query string | Browser beacon / `<image>` / `<script>` tag fallback. |
| POST | `/v2/track` | `application/json` | JSON array of event objects (Application Insights / Azure Monitor protocol). |
| POST | `/v2.1/track` | `application/json` | JSON array of event objects (Application Insights / Azure Monitor protocol). |
| OPTIONS | *(any configured path)* | - | CORS preflight with protocol-specific headers. |

Endpoint paths are configuration keys rather than hard-coded routes. The
paths above describe the intended standard configurations.

## Configuration reference

### Core settings

| Field | Type | Default | Description |
| ------- | ------ | --------- | ------------- |
| `listening_addr` | `string` | *(required)* | Socket address to bind, e.g. `"0.0.0.0:9090"`. |
| `allowed_origins` | `string[]` | `["*"]` | CORS allowed origins. Use `["*"]` to allow any. |
| `max_request_body_size` | `u32` | `3145728` (3 MiB) | Hard cap on request body size (bytes). Per-endpoint limits may be lower but never higher. |
| `max_concurrent_requests` | `usize` | `256` | Maximum in-flight requests before back-pressure. |
| `timeout` | `duration` | `30s` | Request processing timeout (humantime format). |
| `header_read_timeout` | `duration` | `5s` | Maximum time to receive HTTP headers. |
| `endpoints` | map | `{}` | URL path prefixes mapped to endpoint YAML files. |
| `user_agent_attr_target` | enum | `resource` | Target for user-agent enrichment. |
| `http_attr_target` | enum | `resource` | Target for HTTP enrichment. |
| `instance_id_attr` | string | unset | Resource key for a generated instance ID. |
| `origin_attr` | string | unset | Resource key for the request origin. |

Durations use humantime syntax. Enrichment targets are `resource` or
`log_attribute`.

The skeleton validates positive limits and timeouts, non-empty origins and
attribute names, endpoint paths beginning with `/`, and non-empty endpoint
file names.

## Example

```yaml
type: urn:microsoft:receiver:json
config:
  listening_addr: "0.0.0.0:4319"
  allowed_origins:
    - "https://example.com"
  max_request_body_size: 3145728
  max_concurrent_requests: 256
  timeout: 30s
  header_read_timeout: 5s
  user_agent_attr_target: resource
  http_attr_target: resource
  endpoints:
    "/OneCollector/1.0":
      config_file: "onecollector-1ds-endpoint.yaml"
    "/v2/track":
      config_file: "azure-monitor-envelope.yaml"
    "/v2.1/track":
      config_file: "azure-monitor-envelope.yaml"
```

This configuration validates, but pipeline construction fails with
`Microsoft JSON receiver is not implemented` until the runtime is added.

Configuration examples live in this receiver directory, not under
`collector/examples/microsoft-json`. See the
[local console pipeline](pipeline-local-console.yaml), the
[OneCollector / 1DS Common Schema mapping](onecollector-1ds-endpoint.yaml),
and the
[Azure Monitor preserving mapping](azure-monitor-envelope.yaml) with its
[complete Azure Monitor pipeline](azure-monitor-exporter.yaml). The pipeline
includes receiver routes, Azure identity authentication, exporter mappings,
and connection wiring; replace its Azure placeholders before use. The
[endpoint reference](Endpoint.md#azure-monitor-envelope-preservation) explains
the mapping contract and fidelity limits.

Both `/v2/track` and `/v2.1/track` can use either the preserving mapping or the
[Application Insights browser-event
mapping](azure-monitor-browser-events.yaml).
The route does not select the mapping: set `config_file` on both entries to
choose the same behavior for clients using either path.

Use the preserving mapping with the paired Azure Monitor exporter when the
goal is to retain envelope field contents without expansion. It stores fields
in
dedicated attributes and keeps `tags`, `data`, and `ext` intact as JSON strings
that the existing exporter emits unchanged as string columns. This does not
reconstruct the original nested JSON envelope. Use the expanded mapping when
downstream OTel
processors or other exporters need individual properties, measurements,
trace context, severity, and resource/scope fields. That normalization is
intentionally not reversible. Extracting fields also supports queryable keys in
Azure Data Explorer dynamic attribute columns and individual field validation
at the receiver boundary. See
[browser-event extraction and validation](Endpoint.md#browser-event-extraction-for-azure-data-explorer)
for the intended ingestion and query behavior.
The Azure Monitor exporter can export either
representation, but does not automatically reconstruct an Application Insights
envelope from expanded OTel fields. See
[choosing a mapping](Endpoint.md#choosing-an-application-insights--azure-monitor-mapping)
for concrete examples and trade-offs.

## Endpoint files

Each `endpoints` entry references an endpoint YAML configuration. The complete
configuration schema covers:

- endpoint request and event size limits
- accepted HTTP methods, content types, and body formats
- GET query-string handling and query-parameter header promotion
- CORS and additional response headers
- request-header and JSON field mappings
- unmapped-field handling and per-event overrides
- event-name validation and event drop patterns
- minimal, OneCollector, and Application Insights response formats
- inline or external iKey rules, origin restrictions, and iKey remapping
- environment-variable resource attributes

The iKey filtering module includes rule compilation and evaluation, external
YAML loading, origin restrictions, event allow-lists, and key remapping, with
the full contract documented in [iKey filtering](Endpoint.md#ikey-filtering).
The standalone endpoint compiler can load these files, and the mapper evaluates
the filter before emitting a log. Wiring both into receiver startup and request
handling remains follow-up work.

## Telemetry

The planned runtime will opt into the shared `receiver.received` and
`receiver.processing` metric sets rather than duplicate them with
component-specific counters. See the
[shared receiver contract](../../../../../docs/telemetry/metrics-guide.md#shared-receiver-and-exporter-boundary-metrics)
for their definitions and telemetry policy.

All classified ingestion requests use `signal=logs`. One configured ingestion
HTTP request is one external message even when its payload contains
multiple JSON events or produces multiple PData messages. Engine-owned
`node.output` metrics independently count emitted PData, so their cardinality
is not expected to equal `receiver.received.messages`.

Payload size is the encoded POST body size before decompression, or the
encoded GET query-string size.

The processing duration starts when handling begins and includes concurrency
admission, body collection, decompression, request validation, JSON decoding,
field mapping, and Arrow construction. It excludes batching, downstream
channel handoff, and downstream delivery. The terminal boundary observation
is recorded before attempting downstream handoff; a later handoff failure
does not change a locally successful receiver outcome.

Health checks, CORS preflight requests, and requests that do not select a
configured ingestion endpoint are not classified as received telemetry
messages. Payload size is omitted when the receiver rejects a request before
observing an encoded application payload rather than recording a synthetic
zero-byte measurement.

The receiver also defines these non-overlapping protocol diagnostics under
`receiver.microsoft_json`:

| Metric suffix | Unit | Description |
| --- | --- | --- |
| `received.logs.invalid` | `{item}` | JSON events that fail parsing or endpoint validation. |
| `cors.preflight.requests` | `{request}` | CORS preflight requests handled. |
| `compressed.requests` | `{request}` | Requests received with an encoded body. |
| `ikey.rejected` | `{item}` | Events rejected by the iKey filter. |
| `ikey.ignored` | `{item}` | Events silently dropped by the iKey filter. |
| `ikey.unauthorized` | `{item}` | Events rejected because their origin is not allowed. |
| `events.oversized` | `{item}` | Events exceeding the endpoint event-size limit. |
| `config.endpoints.loaded` | `{item}` | Endpoint files loaded at startup. |
| `config.endpoints.failed` | `{error}` | Endpoint files that fail startup loading. |

This skeleton does not emit metric observations because construction
intentionally returns `NotImplemented`. The configuration module composes the
shared `ReceiverMetrics` helper under a `boundary` field with the diagnostic
metric set. Runtime integration must register it and record the terminal outcome
before handing accepted data downstream.

## Security model

The planned browser ingestion endpoint is unauthenticated. CORS controls which
browser origins may read or submit requests according to browser policy, but
CORS is not an authentication mechanism. Non-browser clients are not bound by
browser CORS enforcement and can supply arbitrary `Origin` headers.

Deployments must treat received events as untrusted input and use network
exposure controls, request limits, validation, filtering, and downstream
authorization appropriate to their environment.

## Future enhancements

Authenticated connections could be supported by a future non-browser-focused
mode. That enhancement is not currently planned for this browser-focused
implementation.

## Limits

- Only a direct listener address is supported by the configuration schema.
- Browser telemetry is intentionally accepted without authentication.
- Endpoint runtime behavior is not implemented.

## Related docs

- [Endpoint configuration](Endpoint.md)
- [OTAP Dataflow configuration](../../../../../docs/configuration.md)
- [Receiver and exporter boundary metrics](../../../../../docs/telemetry/metrics-guide.md#shared-receiver-and-exporter-boundary-metrics)
