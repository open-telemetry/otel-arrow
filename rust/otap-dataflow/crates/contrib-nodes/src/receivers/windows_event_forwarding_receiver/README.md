# Windows Event Forwarding Receiver

## Metadata

- Type: `receiver:windows_event_forwarding`
  (`urn:otel:receiver:windows_event_forwarding`)
- Feature gate: `windows-event-forwarding` (also enabled by `contrib-receivers`)
- Crate: `otel-arrow-dfe-contrib-nodes`
- Stability: Experimental
- Signals: Logs
- Feature tracking: [#4156](https://github.com/open-telemetry/otel-arrow/issues/4156)
  covers the initial implementation and future enhancements.

## Overview

This receiver acts as a Windows Event Collector for source-initiated Windows
Event Forwarding (WEF). Windows sources push rendered event XML over HTTPS with
mandatory mutual TLS (mTLS). The receiver converts events to OTAP logs and waits
for whole-batch downstream acceptance before acknowledging delivery to Windows.

The collector is not Windows-only; it can run on Linux or macOS as well. Unlike
the ETW receiver, it receives remote event logs rather than local tracing events.
The initial implementation supports exactly one subscription per receiver and
requires a pipeline with exactly one core.

### Protocol References

This receiver is implemented following Microsoft's published protocol
specifications and Windows Event Forwarding documentation. The primary protocol
reference is [MS-WSMV: Web Services Management Protocol Extensions][ms-wsmv],
which describes Microsoft's extensions to WS-Management used for subscription
discovery, event delivery, and acknowledgements. Microsoft's
[Windows Event Collector documentation][wec] describes the forwarding and
subscription model, and its [subscription content format reference][content-format]
defines the distinction between event data and rendered text.

The implementation supports the subset described in this README; these references
do not imply full protocol conformance or support for every Windows WEC feature.

[ms-wsmv]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wsmv/
[wec]: https://learn.microsoft.com/en-us/windows/win32/wec/windows-event-collector
[content-format]: https://learn.microsoft.com/en-us/windows/win32/api/evcoll/ne-evcoll-ec_subscription_content_format

## Getting Started

Use the complete
[WEF-to-console example](../../../../../configs/windows-event-forwarding-console.yaml).
It configures one core, one Application subscription, and a console exporter.

Before starting:

1. Set `tls.cert_file`, `tls.key_file`, and `tls.client_ca_files` to your PEM files.
2. Set `public_endpoint` to the collector's externally reachable HTTPS origin,
   using a hostname covered by its server certificate.
3. Set `auth.allowed_sources` to the exact lowercase DNS SAN of each source's
   client certificate.
4. For remote Windows sources, change the example's loopback `endpoint` to a
   reachable local interface and restrict inbound TCP access to intended sources.
5. Configure the Windows source as described below.

From `rust/otap-dataflow`, run:

```sh
cargo run --bin df_engine --features windows-event-forwarding -- \
  --config configs/windows-event-forwarding-console.yaml
```

The following is a receiver node fragment for an existing one-core pipeline,
not a complete engine configuration:

```yaml
windows_event_forwarding:
  type: receiver:windows_event_forwarding
  config:
    endpoint: "127.0.0.1:5986"
    public_endpoint: "https://collector.example.com:5986"
    tls:
      cert_file: "/etc/wef/collector.pem"
      key_file: "/etc/wef/collector.key"
      client_ca_files: ["/etc/wef/source-ca.pem"]
    auth:
      allowed_sources: ["source.example.com"]
    subscriptions:
      - name: "application"
        query: |
          <QueryList>
            <Query Id="0">
              <Select Path="Application">*</Select>
            </Query>
          </QueryList>
        content_format: rendered_text
        initial_read: new_events
```

### Windows Source Setup

Windows must trust the collector's server certificate and resolve its advertised
hostname. Install a client-authentication certificate and its private key in the
source's machine certificate store, and grant the forwarding service access to
the private key. The source also needs permission to read the selected event
channels.

Configure the source-initiated SubscriptionManager policy for HTTPS certificate
authentication, for example:

```text
Server=https://collector.example.com:5986/wsman/SubscriptionManager/WEC,Refresh=60,IssuerCA=<client-issuer-CA-thumbprint>
```

`IssuerCA` selects the issuing CA for the source's client certificate, not the
collector's server certificate. The collector must trust that client CA through
`tls.client_ca_files`. Ensure the Windows forwarding service is configured and
running; consult its operational log alongside the collector logs when diagnosing
certificate selection or subscription failures.

### Authentication Rules

- Every connection requires a valid client certificate. There is no plaintext,
  anonymous, Kerberos, or NTLM fallback, and no `tls.enabled` or `auth.mode` setting.
- The verified client leaf certificate must contain exactly one DNS SAN.
  Multiple DNS SANs, including duplicates, are rejected. CN and IP SAN values
  cannot supply the source identity.
- The DNS SAN is normalized to lowercase and matched exactly against
  `auth.allowed_sources`; wildcard names are not supported.
- Each machine must have a unique authorized DNS identity. Reusing one identity
  across machines combines their bookmark state. Certificate renewal preserves
  identity when the DNS SAN stays the same.
- SOAP machine names and event `Computer` fields do not grant access.
- Only configured client CAs are trusted, not the collector's system root store.
  Client CA bundles have a combined 4 MiB bound and are loaded at startup;
  changing client trust requires a restart.

## Configuration

Unknown fields are rejected. All size and capacity limits must be positive.
Durations use strings such as `"30s"` or `"1500ms"` and must be positive whole
milliseconds.

### Receiver Fields

| Field | Default | Description |
| --- | --- | --- |
| `endpoint` | Required | Local socket address for the HTTPS listener. |
| `public_endpoint` | Unset | Advertised HTTPS origin, such as `https://collector.example.com:5986`. |
| `include_event_original` | `false` | Include each event's original XML in `event.original`. |
| `tls.cert_file` | Required | PEM server certificate chain. |
| `tls.key_file` | Required | PEM server private key. |
| `tls.client_ca_files` | Required | Nonempty list of PEM client CA bundle paths. |
| `auth.allowed_sources` | Required | Nonempty list of exact lowercase client DNS SAN identities. |
| `subscriptions` | Required | A list containing exactly one subscription. |
| `limits` | See below | Request, state, and timeout bounds. |

Set `public_endpoint` for event collection. When omitted, the receiver runs in
handshake-only mode and advertises no subscriptions. The origin cannot contain
credentials, a custom path, a query, or a fragment. Incoming SOAP destinations
must match this origin and the exact HTTP request path.

The receiver's node name is the collector namespace used to derive subscription
IDs. Keep it and the subscription name stable to preserve subscription identity.
Changing subscription settings changes the advertised version. Incoming delivery
versions remain opaque: they are retained verbatim in log attributes and never
compared with the configured version.

### Subscription Fields

| Field | Default | Description |
| --- | --- | --- |
| `name` | Required | Nonempty name used with the receiver node name to derive the subscription ID. |
| `query` | Required | Windows Event Log query XML with a `QueryList` root. |
| `content_format` | `rendered_text` | Event representation requested from Windows; see [Content Format](#content-format). |
| `initial_read` | `new_events` | Starting policy without a saved bookmark: `new_events` or `all_existing_events`. |
| `heartbeat` | `"60s"` | Advertised source heartbeat interval. |
| `max_time` | `"30s"` | Advertised maximum event delivery latency. |
| `max_envelope_size_bytes` | `512000` | Advertised envelope bound; also bounds encoded responses locally. |
| `connection_retry_count` | `5` | Advertised source connection retry count. |
| `connection_retry` | `"60s"` | Advertised source connection retry interval. |

The collector checks query XML structure; Windows validates event-log query
semantics. A committed in-memory bookmark takes precedence over `initial_read`.
With `new_events`, generate test events after the source activates the
subscription. `all_existing_events` can replay all retained matching history;
it does not provide count-limited replay or a restart-recovery guarantee.

### Content Format

`content_format` controls how the Windows source prepares events before sending
them, not the downstream OTAP/OTLP encoding or transport compression.

The only accepted value today is `rendered_text`, which maps to Windows
`RenderedText`. It includes structured event data plus localized, human-readable
information rendered on the source, such as the event message and level name.
It is still event XML, not just a plain-text message. Source-side rendering lets
a Linux or macOS collector receive readable messages without installing the event
provider's Windows message resources. Those strings may vary with source locale.

The receiver uses the rendered message as the log body when present. If no message
is supplied, it produces a structured body from `EventData` and `UserData` rather
than trying to reconstruct the provider's message locally.

Microsoft also defines an `Events` format containing raw event data without the
localized rendered information. Supporting that alternative is deferred work,
not an available option: values such as `events` or `raw` are currently rejected.
A future implementation would need a defined structured log mapping and Windows
interoperability tests. Producing equivalent readable messages could additionally
require provider-specific message resources; raw data alone does not supply them.
No configuration name or release timeline is committed for this alternative.

`include_event_original: true` is independent of this setting. It retains the
received XML in the `event.original` attribute while still requesting
`rendered_text`; it does not enable a raw subscription format.

### Resource Limits

| Field under `limits` | Default | Description |
| --- | --- | --- |
| `max_request_bytes` | `1048576` | Maximum HTTP request body bytes. |
| `max_decompressed_bytes` | `8388608` | Bound on input and decoded UTF-8 SOAP text; compression is currently unsupported. |
| `max_event_bytes` | `1048576` | Maximum individual event XML fragment bytes. |
| `max_bookmark_bytes` | `65536` | Maximum retained bookmark XML bytes, including required namespaces. |
| `max_sources` | `1024` | Retained source identities; also caps concurrent connections and handshakes. |
| `max_in_flight_batches` | `64` | Maximum unresolved batches across all sources. |
| `feedback_timeout` | `"30s"` | Maximum wait for downstream batch feedback. |
| `request_timeout` | `"30s"` | Overall HTTP connection deadline and request handling timeout. |
| `handshake_timeout` | `"10s"` | Maximum time establishing mutual TLS. |

The effective in-flight batch limit is also capped by the receiver's output
channel capacity. Only one batch per source may await feedback at a time, even
across different delivery versions. Source entries are not evicted until restart.

XML parsing also enforces an expanded-byte budget of eight times each document's
UTF-8 input length. Copied names, attribute values, text, and each resolved
namespace use consume that budget before decoding. Namespace strings share
storage; excessive namespace expansion is rejected even within the request byte
limit. This is not an exact total-memory ceiling: tree and parser overhead remain.

`request_timeout` includes reading, processing, and waiting for feedback; it can
expire before `feedback_timeout`. Increase it above the feedback budget with room
for request transfer and processing when slower downstream acceptance is expected.
Subscription `max_time` controls source batching, not these collector deadlines.

## Delivery and Log Mapping

Each incoming batch is converted atomically. The receiver sends a protocol Ack
only after downstream Ack for the whole batch and bookmark commit. Queue refusal,
decode failure, downstream Nack, cancellation, and timeout do not advance the
bookmark. Validated synthetic bookmark markers are not emitted as logs; a
marker-only batch can commit locally under the same ordering and capacity bounds.
Subscription termination notices are informational and do not clear progress or
cancel pending delivery feedback.

Bookmarks exist only in memory. Restart loses them, and lost acknowledgements can
produce duplicates. Windows retries and event retention are finite. Neither
exactly-once delivery nor lossless restart recovery is guaranteed.

Logs contain the event timestamp, observed timestamp, mapped severity, and
available Windows metadata. Rendered messages become string bodies; when no
message exists, structured `EventData` and `UserData` become a map body.
Attributes include `source.principal`, `source.address`, `wef.subscription.*`,
and available `winlog.*` fields such as event ID, channel, provider,
and record ID.
EventData attributes use `winlog.event_data.<name>`, with positional `paramN`
names for unnamed fields. A unique name produces a string; repeated names,
including collisions with generated names, produce one array-valued attribute
containing every value in source order. Attribute keys remain unique per log.
Raw XML is omitted unless `include_event_original: true` is configured.

## Diagnostics

The receiver emits `wef.source.authenticated`, `wef.manager.request`, and
`wef.delivery.accepted` events for successful protocol steps. Rejections are
reported through `wef.connection.rejected`, `wef.request.invalid`,
`wef.admission.refused`, and `wef.delivery.rejected`.

Metrics use `receiver.windows_event_forwarding`, shared `receiver.received` and
`receiver.processing`, and engine `node.*` sets. WEF diagnostics count requests
by bounded action/outcome, subscription advertisements, marker-only batches, and
feedback timeouts. Configure engine telemetry export to collect these metrics.

An authenticated GET to the SubscriptionManager endpoint returns `405` with
`Allow: POST`; this can check transport authentication but does not test SOAP
delivery. Do not disable server certificate verification during testing.

## Pending and Planned Work

These items are not supported in the initial version and are not configuration
options or delivery commitments:

- SLDC compression, with bounded decompression and protocol fuzz testing.
- WS-Man robust-connection response caching and replay in the receiver, as
  explained below. Mandatory replay requests are currently rejected.
- Broader WS-Man interoperability, including nonoptimized subscription
  enumeration and SOAP fault responses.
- Production certificate lifecycle guidance, trust rotation, and revocation
  handling beyond the current restart-based client trust snapshot.
- Multiple subscriptions per receiver, with enumeration and delivery routing for
  each subscription and independent bookmark and pending-batch state per
  source/subscription pair. Exactly one subscription is supported today.
- Multi-thread/multi-core execution through multiple receiver replicas, with
  explicit source/subscription ownership, cross-core routing, and safe ownership
  transfer during scaling or reconfiguration. The current one-core restriction
  remains required. This is separate from supporting multiple subscriptions on
  a single core.
- Additional Windows interoperability coverage, including targeted live testing
  of events without rendered messages. Structured fallback has automated tests.

### Protocol Replay Versus Pipeline Retry

Here, "robust connection" names a WS-Man protocol capability, not a general
engine reliability feature. Consider an event batch that is accepted downstream,
but whose SOAP Ack is lost when the HTTPS connection breaks. Windows cannot tell
whether the collector accepted the batch. Protocol response caching and replay
would let the receiver return the retained response for a recognized retry
without submitting the same request's events to the pipeline again.

This belongs at the receiver's protocol boundary: it understands SOAP message
identities, authenticated sources, subscriptions, and the response expected by
Windows. An implementation would need bounded cache retention, source-scoped
request correlation, handling of retries while feedback is still pending, and
the required WS-Man negotiation and replay semantics. It must never replay a
successful Ack before the original batch has been accepted downstream.

Engine buffering, exporter retries, and downstream Ack/Nack propagation address
delivery inside the pipeline. They provide the acceptance result used by this
receiver, but do not by themselves remember or resend a SOAP response lost on
the Windows-facing connection. A shared cache utility could support the work;
the protocol decisions would still belong to the receiver.

Protocol replay is also narrower than event deduplication: recognizing a retry
of one request does not identify the same event in a different batch or an
overlapping subscription. General event deduplication is not implemented.
Bounded response caching would not guarantee exactly-once delivery or recovery
after cache expiry or collector restart. Neither protocol replay nor general
event deduplication is an available configuration option today.

### Other Deferred Work and Non-Goals

Raw event content format, SID resolution, count-limited history replay, inactive
source cleanup, and final `winlog.*` semantic-convention alignment remain
deferred work or open design questions, not promised features.

Durable bookmarks, permanent checkpoint storage, alternative authentication,
collector-initiated subscriptions, acting as a WEF source, and exactly-once
delivery are outside the current scope. Memory-only bookmarks are intentional,
not a placeholder for a planned persistence backend.

Forced-deadline testing also identified a separate engine processor cancellation
issue: pending processor work can outlive shutdown and send to closed channels.
Its fix is tracked independently in
[issue #4141](https://github.com/open-telemetry/otel-arrow/issues/4141) and
[PR #4142](https://github.com/open-telemetry/otel-arrow/pull/4142);
it is not part of this receiver implementation.

See [DESIGN.md](DESIGN.md) for protocol details, design decisions, and recorded
Windows interoperability evidence.
