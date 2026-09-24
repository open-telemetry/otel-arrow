# Windows Event Forwarding (WEF) Receiver -- Design

Status: implementation started; configuration, deterministic IDs, source identity,
mandatory-mTLS transport helpers, bounded in-memory bookmark transitions, URL
routing, bounded SOAP envelope decoding, action dispatch, bounded UTF-16 Ack
encoding, and optimized Enumerate/Subscribe serialization are implemented and
unit tested.
An authenticated HTTP endpoint is registered as a dataflow receiver. It advertises
the configured subscription when a public endpoint is provided, accepts bounded
rendered-text event batches, converts them to Arrow logs, and acknowledges only
after whole-batch downstream Ack and in-memory bookmark commit. Heartbeats and
validated synthetic-bookmark-only batches receive Ack without emitting pdata.
Enumeration replays only the authenticated source's
committed bookmark. Queue refusal, Nack, cancellation, and timeout do not advance
progress. Shared receiver-local metrics and engine output/completion metrics are
complemented by bounded WEF protocol diagnostics. Framework admission policies
and deadline-bounded graceful drain are implemented.
SOAP destinations are checked against the public origin and exact incoming path.
Validated SubscriptionEnd notifications are recorded and acknowledged without
changing bookmarks, pending feedback, or future delivery eligibility.
SLDC and robust replay remain pending.
Real Windows testing has verified mTLS,
enumeration, and end-to-end Application event ingestion after installing the issuer
policy, default XML omission, network interruption recovery, downstream outage
recovery, protocol/output metrics, and graceful drain. Marker-only delivery was
observed without emitted logs; message-less structured bodies still need targeted
live verification. Forced-deadline testing also exposed a separate engine processor
cancellation bug; the successful retest includes that engine fix (section 13.3).
The fix is tracked separately in
[issue #4141](https://github.com/open-telemetry/otel-arrow/issues/4141) and
[PR #4142](https://github.com/open-telemetry/otel-arrow/pull/4142) and is not included
in the receiver changes. Until that fix is integrated, pending processor work can
outlive a forced shutdown deadline and fail when sending to closed channels.
The receiver enforces
one pipeline core and stops on engine drain or shutdown without acknowledging
unfinished batches.
Component URN: `urn:otel:receiver:windows_event_forwarding`
Metric sets: `receiver.windows_event_forwarding`, shared `receiver.received`,
`receiver.processing`, and engine `node.*`
Crate: `contrib-nodes`
Folder: `crates/contrib-nodes/src/receivers/windows_event_forwarding_receiver/`

This document proposes a native OTAP-Dataflow receiver that acts as a Windows Event
Collector (WEC): Windows machines push their event-log records to it using
source-initiated Windows Event Forwarding (WEF), and the receiver converts them
into OTAP/OTLP log records and emits them downstream.

ASCII-only per repository conventions (`AGENTS.md`). All arrows are `->`,
ellipses are `...`, and diagrams use `+`/`-`/`|`.

## 1. Scope and goals

In scope:

- Terminate the WEF **source-initiated (Push)** protocol as a collector.
- Require mutual TLS (mTLS) authentication for all WEF connections.
- Accept event batches and acknowledge them after successful downstream
  feedback; retain bookmarks in memory for the current receiver lifetime.
- Decode Windows event XML into pdata log records with OTel semantics.

Out of scope:

- Permanent storage, durable checkpoints, and guaranteed resume across collector
  restarts. Losing in-memory bookmarks on restart is an accepted tradeoff.
- Kerberos/SPNEGO, NTLM, and anonymous authentication.
- WEF **collector-initiated** (pull) subscriptions.
- Acting as a WEF **source** (forwarding events out).
- Emulating every Windows WEC configuration knob; we expose a pragmatic subset.

Non-goal: reimplementing ETW. The existing `urn:otel:receiver:etw` node is a
**local, in-box** tracing consumer. This receiver is a **remote, networked**
event-log collector. They are complementary and share no code.

## 2. Protocol summary (what we must implement)

Transport: **WS-Management / WinRM (MS-WSMV, derived from DSP0226)** over
HTTPS, conventionally on port 5986, with mandatory mTLS. Source-initiated push
flow, per subscription:

1. Source POSTs to `/wsman/SubscriptionManager/WEC` and authenticates
  using a TLS client certificate.
2. Source sends WS-Enumerate; collector returns an `EnumerateResponse` whose
   items each embed a WS-Eventing `Subscribe` (delivery URL, `Events` mode,
   `Compression=SLDC`, `ContentFormat=RenderedText|Raw`, XPath `Filter`,
   `Heartbeats`, `MaxElements`/`MaxTime`/`MaxEnvelopeSize`, `Bookmark`).
3. Source POSTs `End`; collector returns `204`.
4. Source connects to the per-subscription delivery URL
   (`/wsman/subscriptions/<uuid>/<ver>`) and authenticates again.
5. Source sends `Heartbeat` and/or `Events` batches (SLDC-compressed,
   UTF-16 SOAP, events as CDATA `<Event>`), each with `AckRequested` and an
   updated `Bookmark`.
6. Collector replies `Ack` (`RelatesTo` = message id) **after** the batch is
  accepted according to downstream feedback. Without an ack, Windows may retry
  within its retry policy; this does not guarantee recovery across restarts.
7. Every `Refresh` seconds the source re-enumerates. On shutdown it sends
   `SubscriptionEnd`.

Reliability contract: **only ack after successful downstream feedback**, then
retain the source-provided bookmark in memory. Enqueueing a pipeline message is
not sufficient acceptance. There is no durable restart-recovery guarantee.

## 3. Platform and licensing notes

- Unlike the ETW receiver, this node is **not** Windows-only: it is an
  HTTP/WS-Man server and can run on Linux/macOS collectors. No
  `target_os = "windows"` gate; guard only behind a cargo feature.
- This is a clean-room implementation against public specs (MS-WSMV, DSP0226)
  and does **not** copy GPL-3.0 code (e.g. OpenWEC). OpenWEC and its packet
  analysis may be used as behavioral references only.

## 4. Module layout

Implemented module layout:

```text
windows_event_forwarding_receiver/
  DESIGN.md              // this document
  metadata.yaml          // component metadata (type, class, codeowners)
  mod.rs                 // URN const, factory registration, module wiring
  config.rs              // serde config + validation
  receiver.rs            // Receiver trait impl, control loop, effect handler
  http.rs                // HTTPS listener, routing, mandatory mTLS setup
  identity.rs            // verified certificate -> authorized SourceIdentity
  tls.rs                 // trust snapshot, issuer selection, mTLS configuration
  metrics.rs             // shared boundary metrics and WEF protocol diagnostics
  runtime.rs             // bounded delivery bridge and downstream feedback slots
  wsman.rs               // URL routing and bounded SOAP envelope decoding
  wsman/messages/        // typed actions and response encoders
  event.rs               // Windows event XML -> OTAP Arrow logs
  bookmark.rs            // source-owned in-memory bookmarks
```

Feature wiring in `receivers/mod.rs`:

```rust
/// Windows Event Forwarding (WEF/WS-Man) collector receiver.
#[cfg(feature = "windows-event-forwarding")]
pub mod windows_event_forwarding_receiver;
```

## 5. Configuration surface

```yaml
windows_event_forwarding:
  type: receiver:windows_event_forwarding
  config:
    # Listener
    endpoint: "0.0.0.0:5986"          # HTTPS only
    public_endpoint: "https://collector.contoso.com:5986"
    include_event_original: false   # opt in to retain event.original XML attributes
    tls:
      cert_file: "/etc/wef/collector.pem"
      key_file: "/etc/wef/collector.key"
      client_ca_files: ["/etc/wef/source-ca.pem"]  # validates source certs

    # Mandatory mTLS; no other authentication modes
    auth:
      allowed_sources: ["host.contoso.com"]  # required exact lowercase DNS SANs

    # Subscriptions advertised to sources on Enumerate
    subscriptions:
      - name: "security-audit"
        # Level-1 XPath query list (as exported by Event Viewer)
        query: |
          <QueryList>
            <Query Id="0">
              <Select Path="Security">*</Select>
              <Select Path="Application">*[System[(Level=1 or Level=2 or Level=3)]]</Select>
            </Query>
          </QueryList>
        content_format: "rendered_text"   # only format in the initial milestone
        initial_read: "new_events"        # or all_existing_events without a bookmark
        heartbeat: "60s"
        max_time: "30s"                    # "Minimize Latency"
        max_envelope_size_bytes: 512000
        connection_retry_count: 5
        connection_retry: "60s"

    limits:
      max_request_bytes: 1048576
      max_decompressed_bytes: 8388608
      max_event_bytes: 1048576
      max_bookmark_bytes: 65536
      max_sources: 1024
      max_in_flight_batches: 64
      feedback_timeout: "30s"
      request_timeout: "30s"
      handshake_timeout: "10s"

    # Bookmarks are always in memory; no storage configuration.
    # Preserve incoming WEF batch boundaries; no additional batcher initially.
```

Validation rules (in `config.rs`): exactly one subscription with a nonempty name
and query; mandatory TLS with nonempty credential paths; nonempty `client_ca_files`
and `allowed_sources`; only `rendered_text`
are accepted. All size/capacity limits and whole-millisecond durations must be
positive. Optional `public_endpoint` must be an HTTPS origin without credentials,
custom path, query, or fragment. Without it, enumeration remains empty.
When set, factory validation also requires well-formed query XML with a `QueryList`
root and no DTD; Windows validates the query's event-log semantics.
Credential loading validates file contents. The initial milestone
requires one receiver replica, enforced at factory creation. Request, decoded
text, event, bookmark, source, and in-flight capacity limits are enforced by their
owning runtime paths. Compression remains unsupported.

## 6. Authentication strategy

**Decision: mTLS is the only supported authentication method**, not a temporary
MVP choice ahead of Kerberos. No Active Directory service account, SPN, keytab,
domain membership, or personal username/password is required. Certificates can
be issued by a private CA independent of Active Directory.

Deployment requirements:

- The collector has a server certificate and private key. Windows must trust its
  issuing CA and validate that the certificate matches the destination hostname.
- Each Windows source has its own client certificate and private key, with
  client-authentication usage. Configure Windows WEF to use certificate
  authentication and an HTTPS SubscriptionManager URL, including the appropriate
  issuing-CA selection and private-key access for the forwarding service.
- The collector requires a client certificate on every TLS connection and
  validates its chain against `client_ca_files`, validity, and authentication
  usage. Missing or invalid certificates fail the handshake; there is no
  anonymous or plaintext HTTP fallback.
- Certificate validation and source authorization are separate: apply the
  configured source allow-list before advertising subscriptions or accepting
  deliveries. Certificate renewal, trust rotation, and revocation handling must
  be defined before production use.

Derive `SourceIdentity` from exactly one DNS SAN in the verified TLS peer leaf
certificate, not its subject CN, an HTTP header, or SOAP `MachineID`. Normalize
the DNS SAN to lowercase, reject wildcard or malformed names, and match it exactly
against `allowed_sources`. Multiple DNS SANs are rejected, even if identical.
Other SAN types do not provide a source identity. Renewal preserves identity
when the DNS SAN remains unchanged. The issuing CA must ensure each source has
a unique DNS identity; reusing it on different machines conflates their state.

The transport helper reuses the OTAP TLS utilities but advertises HTTP/1.1 ALPN.
It trusts only configured client CAs, not system roots. Client CA files are loaded
at startup with a combined 4 MiB bound; updating trust requires restart. A bounded
handshake completes before source identity extraction and authorization.
Production trust rotation/revocation remains pending.

mTLS provides transport authentication and encryption. SOAP is processed after
TLS decryption, without Kerberos wrapping or GSSAPI dependencies. WEF compression
is independent of authentication: SLDC must still be supported when advertised.
Certificate authentication does not eliminate SOAP, UTF-16 decoding, or SLDC;
the full exchange must be tested against Windows.

### 6.1. Running the receiver example

Run from `rust/otap-dataflow` after setting the certificate paths and exact allowed
source DNS name in `configs/windows-event-forwarding-console.yaml`. Replace
`public_endpoint` with the collector's certificate-matching HTTPS origin. The
example binds to loopback by default; for a remote Windows source, set `endpoint`
to a reachable local interface and restrict inbound access to intended sources.

```sh
cargo run --bin df_engine --features windows-event-forwarding -- \
  --config configs/windows-event-forwarding-console.yaml
```

This is a dataflow engine configuration with one pipeline core and a console
exporter. Accepted event batches appear as logs in the console exporter.
The receiver logs `wef.source.authenticated` with the normalized
`source` DNS identity after certificate verification and allow-list checks.
`wef.manager.request` records the authenticated source and accepted SOAP action;
SOAP MachineID and event Computer never supply the identity.

Configure Windows' source-initiated SubscriptionManager policy with the collector's
certificate-matching hostname, for example:

```text
Server=https://collector.contoso.com:5986/wsman/SubscriptionManager/WEC,Refresh=60,IssuerCA=<client-issuer-CA-thumbprint>
```

Windows must trust the collector's server certificate. Install the source's
client-authentication certificate and private key in the appropriate machine
store, allow the forwarding service to use its private key, and ensure its one
DNS SAN matches `allowed_sources`. The collector must trust the issuing client
CA. Open the configured TCP port only to the intended sources. Check the Windows
`Microsoft-Windows-Forwarding/Operational` log alongside the collector logs.

For a quick transport-only check with PEM client credentials:

```sh
curl --cacert collector-ca.pem --cert source.pem --key source.key \
  -i https://collector.contoso.com:5986/wsman/SubscriptionManager/WEC
```

An authenticated GET returns `405` with `Allow: POST`; missing/untrusted client
certificates fail TLS, and unauthorized DNS identities are disconnected before
HTTP. Do not use `--insecure`: server authentication is part of the test.

Authenticated SOAP Enumerate returns a correlated UTF-16LE response; End returns
`204`. With `public_endpoint` set, optimized enumeration includes the configured
subscription. Without it, basic and optimized enumeration return no subscriptions.
Remove `public_endpoint` from the sample to return to handshake-only testing.
Positive `MaxElements` is supported. Nonoptimized subscription enumeration,
other enumeration options, and mandatory replay headers are rejected.

The sample requests only new Application events. Heartbeats return a correlated
Ack. Valid event batches wait for downstream feedback before returning Ack; queue
refusal and downstream Nack return `503`, and feedback timeout returns `504`
(the outer request/connection deadline may expire first). Invalid events fail the
whole batch. Delivery versions remain opaque and are retained in log attributes;
unknown subscription URLs return `404`. Bookmarks are memory-only and restart loses
progress. This remains experimental, not a production event collector.

The endpoint accepts uncompressed SOAP, UTF-8 or explicit UTF-16LE/BE; generic
UTF-16 requires a BOM. Unsupported media/encoding returns `415`, malformed SOAP
returns `400`, and oversized bodies return `413`. SOAP faults and full Windows
interoperability remain follow-up work. `max_sources` also bounds concurrent
connections; `request_timeout` bounds the entire HTTP connection as well as body
handling in this test endpoint. Ctrl+C initiates engine ingress drain: the listener
closes while admitted requests can finish before the shutdown deadline. Unresolved
connections are cancelled at the deadline.

## 7. WS-Man state machine

`http.rs` dispatches the typed messages decoded by `wsman.rs` and
`wsman/messages/` according to SOAP `Action`:

```text
POST /wsman/SubscriptionManager/WEC
  Enumerate            -> build EnumerateResponse (embed Subscribe per sub)
  End                  -> 204 No Content

POST /wsman/subscriptions/<uuid>/<ver>
  Heartbeat            -> Ack (RelatesTo = msg id)
  Events               -> hand batch to pipeline; on success -> Ack
  SubscriptionEnd      -> validate status, record notification, Ack
  End                  -> 204 No Content
```

Responsibilities:

- Generate/track `MessageID`/`OperationID`/`RelatesTo`, `SequenceId`, and the
  robust-connection/full-duplex handshake fields.
- Build the embedded WS-Eventing `Subscribe` from each configured subscription,
  including the current per-source `Bookmark` (fetched from the bookmark store).
- Enforce `MaxEnvelopeSize` and map oversize handling to the `Notify` policy.

SOAP parsing uses a bounded, receiver-local document view over `quick-xml`,
enabled only with the receiver feature. Message
bodies are UTF-16; decode to UTF-8 for parsing and re-encode responses to UTF-16.

Namespace strings are interned rather than copied into every expanded name.
A per-document budget of eight times the UTF-8 input length charges copied names,
attribute values, text, and each resolved namespace use before decoding. This
rejects excessive namespace expansion before SOAP validation or event admission.
The budget bounds string expansion and repeated decoding work, not exact total
memory use; arena and reader overhead are additional.

The initial `wsman.rs` helpers route the manager and configured subscription
paths, retaining the version path segment verbatim without UUID validation or
comparison. The caller supplies the HTTP URI path without its query string;
percent escapes in the version remain unchanged.

Text decoding supports UTF-8 and UTF-16 in both byte orders, with optional matching
BOMs. Invalid Unicode and oversized input or decoded text are rejected. The
transport must select the encoding and bound wire bytes and decompression before
calling the decoder. The namespace-aware SOAP 1.2 parser disables DTDs, bounds XML
size and node count, requires one Header followed by one Body, and rejects missing,
duplicate, empty, or nested WS-Addressing Action and MessageID values. MessageID
case is preserved for later Ack correlation; CDATA remains available to event
decoding.

Typed dispatch checks action/route combinations and payload shape, including an
empty `w:Events` for Heartbeat. Delivery messages require `e:Identifier` and
`w:AckRequested`; Identifier is independent of the opaque URL version. Direct
mandatory headers must be understood; mandatory robust-connection replay headers
are rejected until response caching exists. Replies must use the anonymous
address. Source `MaxEnvelopeSize` narrows the caller's response limit.

Before any manager response, delivery admission, or bookmark reservation, SOAP
`To` must be an absolute HTTPS URL without user information, query, or fragment.
Its path must match the incoming HTTP route exactly, including the opaque delivery
version and percent-escape spelling; dot-segment normalization is not accepted as
an alternate path. Its origin must match `public_endpoint`, using URL host
normalization and effective ports (omitted HTTPS port and port 443 are equivalent).
The configured version is never compared with the incoming version. HTTP `Host`
and forwarding headers do not supply the trusted public origin. In handshake-only
mode without `public_endpoint`, URL shape and the manager path are checked, but
there is no configured hostname to compare. Mismatches return HTTP 400 without Ack
or progress changes. Configure the externally visible origin for proxy deployments.

Ack encoding uses an XML writer and emits uncompressed UTF-16LE with a BOM,
an empty SOAP Body, and exact MessageID correlation through `RelatesTo`.
It bounds the encoded response including the BOM and does not advertise
robust-connection headers. The caller supplies a fresh response UUID. Encoding
does not authorize delivery: Events may be acknowledged only after whole-batch
downstream Ack and bookmark advancement.

The embedded Subscribe includes stable subscription/version IDs, a fresh message
ID, HTTPS delivery and termination addresses, rendered text with CDATA, configured
heartbeat/batching/retry settings, and the mutual-TLS authentication profile.
No compression or robust replay support is advertised. The delivery policy uses
`Policy/ExactlyOne/All` and the Microsoft authentication namespace
`http://schemas.microsoft.com/wbem/wsman/1/authentication`, matching the
[documented Windows packet capture][subscription-capture]. The `Authentication`,
`ClientCertificate`, and `Thumbprint` elements use that namespace.
The required certificate constraint described in
[MS-WSMV AuthenticationType][authentication-type] contains a
`Thumbprint Role="issuer"` derived from the verified TLS leaf's issuing CA.
Issuer candidates come from the startup client CA snapshot and the peer's supplied
intermediates. Selection checks CA status, issuer name, and the leaf signature;
missing or ambiguous issuers fail closed when advertising subscriptions.
SHA-1 identifies the issuer certificate for Windows selection only; TLS validation
and DNS authorization remain mandatory. Leaf renewal under the same issuing CA
preserves this hint. No additional issuer lookup is required in handshake-only mode.

Windows tracing confirmed that enumeration reaches the EventLog Subscribe plugin.
The previous generic WS-Man namespace assertion was rejected with fault 2150859101
(unsupported delivery WS-Policy). The type documentation names that generic
namespace, unlike the packet capture. Restoring the Microsoft namespace changed
the fault to 2150859102 (no compatible policy option), explicitly requiring mutual
HTTPS certificate authentication using issuer thumbprints. This supports retaining
the existing policy nesting and replacing leaf constraints with issuer constraints.
With the issuer constraint, a real Windows source reached authenticated delivery
on September 22, 2026. A subsequent live run confirmed Heartbeat and Events
acceptance, including the rendered Application:100 test message and an ordinary
Application:16384 event in the same batch, with downstream Ack.

New-events mode omits the initial bookmark; all-existing-events mode uses the
reserved `bookmark/earliest` URI. Both request bookmarks in future deliveries.
After downstream Ack, the source's committed bookmark overrides that initial-read
choice on later enumerations. Bookmark XML retains its in-scope namespaces.

[authentication-type]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wsmv/906e0d6a-0147-416b-9686-b1de0a84bfa6
[subscription-capture]: https://github.com/cea-sec/openwec/blob/main/doc/protocol.md

SubscriptionEnd handling follows the [WS-Eventing termination outline][eventing-end]
and the [published Windows example][windows-end]. This receiver requires the
advertised delivery-route Identifier and AckRequested header, as in that example;
requests without them fail without Ack. The body requires one SubscriptionManager
with an absolute Address and one scalar, absolute Status URI. An optional scalar
Reason is validated but not logged. Standard statuses are classified as
DeliveryFailure, SourceShuttingDown, or SourceCancelling; extension URIs use the
bounded Other category. Duplicate fields, nested scalar values, and unsupported
termination-body elements are rejected. Existing request, decoded XML, and node
limits bound the entire notification.

An optional Microsoft `WSManFault` extension in the exact
`http://schemas.microsoft.com/wbem/wsman/1/wsmanfault` namespace is accepted.
Its unsigned 32-bit Code and Machine attributes are required, following the
[published fault schema][windows-fault]. At most one Message and its optional
ProviderFault are allowed. Provider-specific content remains opaque within the
existing XML limits. Fault text and attributes are neither logged nor used for
identity, bookmark updates, or acknowledgement of pending event batches.

The SubscriptionManager endpoint reference is descriptive, not an authenticated
identity or an instruction to contact that address. Its source-side reference
properties need not equal the collector Identifier. After mTLS, destination,
Identifier, and body validation, the receiver emits `wef.subscription.ended` with
the authenticated source, subscription ID, and status category. It returns the
bounded, correlated SOAP Ack requested by the sender, consistent with
[WS-Management acknowledgement rules][wsman-standard] (section 10.7).
It emits no event pdata, reserves no bookmark or feedback slot, and does not
cancel pending batches, erase committed progress, or disable future deliveries.
Receipt acknowledges the notification only, never an outstanding Events batch.

SLDC, nonoptimized enumeration, and robust replay remain pending. Parsing alone
does not permit an event acknowledgement.

[eventing-end]: https://www.w3.org/Submission/WS-Eventing/#Subscription_End
[windows-end]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wsmv/f4880091-50be-49e7-a120-2df3fe2bf1f3
[windows-fault]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wsmv/2b6ab0b1-4d5c-4c13-9f28-0f04716e5fa4
[wsman-standard]: https://www.dmtf.org/sites/default/files/standards/documents/DSP0226_1.2.0.pdf

### 7.1. Subscription identity

**Decision: the stable collector namespace is the configured receiver instance
name**, not the component type or URN. Subscription identity is computed as:

```text
application_namespace = UUIDv5(NAMESPACE_URL, "urn:otel:receiver:windows_event_forwarding")
receiver_namespace = UUIDv5(application_namespace, receiver_name)
subscription_id = UUIDv5(receiver_namespace, subscription_name)
```

All replicas of a receiver use the same name and derive identical subscription
IDs across threads, process restarts, and deployment generations. Names are
encoded as their exact UTF-8 bytes without case folding or trimming. Neither
source identity nor bookmark contents participate in this calculation.

Receiver names must be unique within the shared checkpoint and routing scope.
If separate pipelines reuse a receiver name, their state must be isolated or the
name qualified with pipeline identity. Renaming the receiver or subscription
changes the subscription ID and must be treated as a new logical identity, not
as transparent continuity of the existing bookmark state.

A subscription ID identifies a configured subscription, not a Windows machine.
Two sources receiving `security-audit` share its subscription ID but have
independent bookmarks keyed by `(SourceIdentity, subscription_id)`. Subscription
version is neither the owner-thread key nor part of the bookmark key.

### 7.2. Enumeration and source ownership

**Decision: source-sensitive operations use source-owned pipeline-thread state.**
Resolve the owner using the stable, authenticated `SourceIdentity`, scoped to
this receiver. Do not route by subscription ID: that would place every source
sharing a subscription on the same thread.

```text
Complete mTLS and derive SourceIdentity
  -> Resolve the source's owner thread
  -> Determine subscriptions authorized for that source
  -> Ask the owner for each (SourceIdentity, subscription_id) bookmark
  -> Build EnumerateResponse with embedded Subscribe messages
```

Each advertised subscription includes its query and delivery settings, identity
and version, HTTPS event-delivery URL, and the source's saved bookmark when one
is available. With no saved bookmark, use the configured initial-read policy.
A failed bookmark lookup must not be treated as a missing bookmark.

The HTTP connection need not move to the owner thread: its handler can request
the lookup and await a response. Subsequent deliveries, downstream feedback, and
bookmark updates must reach the same source owner, even over new connections.
The owner must serialize progress updates and coordinate enumeration reads with
them; a single-thread runtime alone does not prevent asynchronous update races.

Immutable subscription definitions can be replicated independently. The owning
thread coordinates mutable source progress; affinity itself does not provide
durability. The cross-thread request mechanism and ownership transfer during
scaling or overlapping pipeline generations still require design. The initial
milestone has one replica, so all sources have the same owner and no cross-thread
transport is needed. Permanent checkpoint storage is outside project scope.

### 7.3. Subscription version and delivery

Compute the advertised version once at startup from the subscription definition:

```text
version = UUIDv5(subscription_id, canonical_subscription_settings)
```

Include the query, content format, and advertised delivery settings. Exclude
bookmarks, source identity, timestamps, thread IDs, and the generated version
itself. Identical settings yield identical versions across replicas and restarts;
changed settings yield a different version, and reverting settings restores the
original version. No counter or permanent storage is required.

Canonical settings are compact UTF-8 JSON with defaults expanded, in this fixed
field order: `query`, `content_format`, `initial_read`, `heartbeat_ms`,
`max_time_ms`, `max_envelope_size_bytes`, `connection_retry_count`,
`connection_retry_ms`. Durations are integer milliseconds. Normalize CRLF and CR
in query XML to LF but preserve other spelling and whitespace. Cosmetic XML
changes can therefore change the version. Generated URLs are excluded.

**Decision: retain the version in the advertised delivery URL:**

```text
POST /wsman/subscriptions/<subscription-id>/<version>
```

Treat the received version as opaque metadata. Do not compare it with the current
version, require it to be known, or reject delivery because it differs. For each
emitted log record, copy the delivery URL's version into the string attribute
`wef.subscription.version`. Do not replace it with the current configuration's
version or use it to select a historical configuration.

Authenticate and authorize the source and resolve the configured subscription
ID independently of version. Unknown or removed subscriptions are still errors;
normal request-size limits and SOAP/event validation still apply. Process events
as received, including events selected under an older query, without reapplying
the current query or maintaining a version history.

After a configuration change and collector restart, Windows may continue using
the old delivery URL. A subsequent successful enumeration at the source's
`Refresh` interval advertises the new version. Old and new deliveries are both
accepted for an authorized, configured subscription and retain their respective
version attributes. There is no proactive version notification.

## 8. Event decoding to pdata logs

Pipeline: SOAP `Events` body -> list of `<Event>` XML fragments -> intermediate
`WinEvent` model -> OTAP log records (`OtapPdata`).

Event decoding in `event.rs` captures `System` (Provider,
EventID, Level, Task, Opcode, Keywords, TimeCreated, EventRecordID, Channel,
Computer, Security/UserID), `EventData`/`UserData`, and `RenderingInfo`
(Message, Level text, Keywords) when `content_format = rendered_text`.

Mapping to an OTLP LogRecord (`event.rs`):

| OTLP field | Source |
| --- | --- |
| `time_unix_nano` | `System/TimeCreated/@SystemTime` |
| `observed_time` | collector receive time |
| `severity_number` | mapped from `System/Level` (1..5 -> OTLP) |
| `severity_text` | `RenderingInfo/Level` when present |
| `body` | Rendered message or structured event map (see below) |
| `event_name` | `winlog.event_id` convention (e.g. `Security:4624`) |
| attributes | see below |

The body uses `RenderingInfo/Message` when present, including an explicitly empty
message. Without a message, the receiver emits a native map body, encoded as CBOR
in Arrow, rather than XML or a JSON string:

- `EventData` is an ordered array of `{name, value}` entries for `Data` elements.
  Names use the `Name` attribute or the existing positional `paramN` convention.
  Values remain strings, including embedded JSON. Duplicate names, empty values,
  and element order are preserved. Other EventData elements are not mapped.
- `UserData` is a tree of `{name, namespace, attributes, content}` maps. Attributes
  are ordered `{name, namespace, value}` entries; content is an ordered array of
  text strings and child-element maps. Repeated elements, namespace URIs, and mixed
  text content survive conversion. Comments, processing instructions, and prefix
  spelling are not retained. Nesting beyond 32 levels below UserData fails the batch.
- An event with neither section has an empty map body. System metadata remains in
  the log fields and `winlog.*` attributes.

Receiver-level `include_event_original` defaults to `false`; enabling it retains
the complete XML in `event.original`, including fields not mapped individually.
This flag does not change the body, parsed fields, subscription ID, or advertised
version. Without retention, unmapped XML fields are not preserved; UserData is
included in the fallback body only when no rendered message is present.
XML with DTDs, malformed
timestamps, duplicate mapped scalar fields, or unexpected event roots is rejected.
A batch may contain at most 65,535 events to fit the Arrow log identifier width.

Synthetic subscription-bookmark events are protocol control data, not emitted logs.
Recognition requires provider `Microsoft-Windows-EventForwarder`, event ID 111,
no nonempty Channel, and only System and a Windows-event-namespace
`SubscriptionBookmarkEvent` payload containing one scalar `SubscriptionId`.
Other events with ID 111 are retained. Every event is decoded before accepting the
batch, so a malformed trailing event cannot commit a bookmark from an earlier marker.

Attribute conventions (align with OTel `winlog.*` where they exist):

- `winlog.channel`, `winlog.provider.name`, `winlog.provider.guid`
- `winlog.event_id`, `winlog.record_id`, `winlog.task`, `winlog.opcode`,
  `winlog.keywords`, `winlog.computer`
- `winlog.event_data.*` (named `<Data>` elements; anonymous ones as
  `winlog.event_data.paramN`)
- Source/collector metadata: `source.address` (client IP),
  `source.principal` (stable identity mapped from the verified client certificate),
  `wef.subscription.name`, `wef.subscription.uuid`, `wef.subscription.version`
  (opaque version from the delivery URL). Bookmarks remain opaque protocol state,
  not a parsed record-ID attribute.

EventData attribute keys are unique per log. Values with the same explicit or
generated name are grouped in source order: a singleton is a string, and a
collision produces an array of strings. This includes an explicit `paramN` name
colliding with an unnamed field. The fallback body's ordered EventData entries
remain unchanged, retaining the original entry order independently of attributes.

SID resolution (e.g. `S-1-5-...` -> account name) is explicitly out of scope for
the MVP and can be a later processor/option.

Emit as OTAP Arrow log batches using the same builder approach the ETW receiver
uses (`OtapPdata::new_todo_context(payload.into())`). Preserve incoming WEF batch
boundaries initially, without an additional batching layer. The first milestone
supports rendered text only; raw content is a later extension.

## 9. In-memory bookmarks and acknowledgement

**Decision: bookmarks are memory-only for this project.** Losing them on a
collector restart is acceptable. No file store, database, or durable checkpoint
service is planned.

Design:

- The source owner maintains a bounded in-memory map keyed by
  `(SourceIdentity, subscription_id)`. Store the source's bookmark XML as an
  opaque cursor. Do not add a persistence abstraction.
- Serialize delivery progress per source/subscription, including deliveries with
  different version attributes. Do not attempt to order opaque bookmarks by
  comparing version strings or bookmark contents.
- Delivery ordering per `Events` batch:
  1. Decode records and attach the received subscription version.
  2. Subscribe to downstream Ack/Nack feedback before sending the pdata message;
     await successful feedback for the entire batch. Awaiting
     `effect_handler.send_message(...)` alone only confirms enqueueing.
  3. Update the in-memory bookmark with the accepted batch's cursor.
  4. Send WS-Man `Ack`.
- Synthetic markers are removed before constructing pdata. Mixed batches retain
  whole-batch downstream Ack/Nack semantics for all remaining records. A validated
  marker-only batch emits no pdata and can commit its opaque header bookmark
  locally before Ack. It still reserves source progress under the same bounds and
  cannot overtake a pending batch. The marker's SubscriptionId is not a cursor.
- On decode failure, downstream Nack, or feedback timeout, do not advance the
  bookmark or acknowledge acceptance. Lost acknowledgements can cause duplicate
  delivery. Windows retries are bounded; retention and retry exhaustion can
  prevent recovery. No deduplication is implemented initially.
- On `Enumerate`, embed the in-memory bookmark when available. After restart,
  no saved cursor exists: a surviving Windows subscription may continue, but a
  newly created subscription uses `initial_read`. With `new_events`, earlier
  events may be skipped; with `all_existing_events`, retained matching history
  may be replayed. These policies map to WS-Man `ReadExistingEvents` values
  `false` and `true`, respectively. Count-limited replay is not supported yet.
  Neither policy guarantees lossless restart recovery.

Backpressure: bound in-flight batches and wait for downstream feedback before
acking. Bound bookmark entries as well; refuse additional source/subscription
state at capacity rather than silently evicting active progress.

The implemented `BookmarkStore` belongs to one replica and its single configured
subscription. It enforces `max_sources`, `max_in_flight_batches`, and
`max_bookmark_bytes`, with one pending batch per source. Batch tickets correlate
feedback; stale feedback cannot resolve a newer batch. Ack commits the candidate
bookmark, while Nack (including enqueue failure or timeout) preserves the previous
cursor. Ack without a bookmark also preserves the previous cursor. The HTTP bridge
resolves each ticket before replying to Windows, using receiver control feedback
for emitted batches. A dropped request reservation resolves as Nack. Unit and TCP
tests cover these transitions, mixed batches, and marker-only bookmark replay.

Sources remain allocated even after a failed first delivery. No inactive-state
eviction is implemented; restart clears all entries. Create a new store if the
configured subscription identity changes. Delivery URL versions never select
separate progress state.

Ack/Nack correlation uses the same bounded, generation-safe slot allocator as
OTLP. Waiter drop cancels its slot immediately; stale feedback cannot resolve a
replacement waiter. Bookmark reservations remain WEF-specific and resolve as
Nack when dropped. No sequence-ID map or periodic cleanup scan is needed.

The receiver binds the framework byte-admission policy and initializes admission
from the process memory-pressure state. Hard-pressure enforcement rejects HTTP
requests before body collection with HTTP 503 and Retry-After. Classified Events
requests recheck pressure and the byte-rate gate before event extraction or
bookmark reservation. Observe-only policies remain nonblocking. Effective
in-flight capacity is capped by the downstream PData channel capacity.

## 10. Control loop and lifecycle

Use the local receiver trait and the graceful ingress-drain pattern used by OTLP:

- `local::Receiver<OtapPdata>::start` spawns the HTTPS listener and runs a
  `tokio::select!` loop with `biased` control priority.
- Handle `NodeControlMsg`:
  - `CollectTelemetry` -> report metrics.
  - `MemoryPressureChanged` -> update shared receiver admission state.
  - `DrainIngress { deadline }` -> close the listener and initiate graceful HTTP
    shutdown while continuing to route downstream Ack/Nack feedback. At the
    deadline, cancel unresolved connections and join them. Notify receiver drain
    only after serving tasks finish, then return final metric snapshots. Never
    acknowledge unresolved batches to meet the deadline.
  - `Shutdown { deadline }` -> cancel and join connections immediately, snapshot,
    and return.
- The listener uses the engine's TCP helper; accepted sockets use the shared OTLP
  TCP-option helper and OTLP defaults (NODELAY, keepalive 45s, interval 15s, and
  five retries where supported). TLS uses shared Rustls configuration/handshake
  helpers, with WEF-specific mandatory client authentication and identity checks.
- Connection tasks use `JoinSet::spawn_local` on the pipeline's LocalSet, keeping
  request processing on the owning runtime thread without a Send requirement.
- `CancellationToken` coordinates drain/forced stop; the bounded connection
  `JoinSet` guarantees forced tasks are cancelled and joined.
- Terminal snapshots are not durable bookmark storage; restart starts with an
  empty bookmark map.

## 11. Telemetry

Implemented metrics use the receiver node's stable entity scope. No source DNS,
subscription identifier/version, message ID, URL, or error text is a dimension.

- `receiver.received.messages`: classified Events requests that complete local
  policy checks, batch extraction, log conversion, bookmark reservation, and
  feedback-slot allocation, with `signal=logs` and
  `outcome=success|refused`. Success here is not downstream acknowledgement.
- `receiver.received.payload.size` and `receiver.processing.duration`: shared
  optional boundary measurements, enabled by engine telemetry interests. Payload
  size is the encoded SOAP body size; duration covers local processing/admission,
  not HTTP body read, earlier SOAP dispatch, or downstream feedback wait.
- Engine-owned `node.output.messages` counts PData by signal and Ack/Nack outcome;
  optional `node.output.items` and `.size` supply record counts and logical size.
- Optional engine-owned `node.completion.duration` measures completion latency.
  Framework channel metrics describe the PData/control channels.

WEF-specific diagnostics use the `receiver.windows_event_forwarding` prefix:

- `requests`: authenticated HTTP requests handled or cancelled, with bounded
  `action=enumerate|heartbeat|events|subscription_end|end|other` and
  `outcome=success|refused|failure`. The action is classified after structural
  SOAP parsing; earlier rejections and unknown actions use `other`. Success means
  a successful response was prepared, not delivered to Windows. Invalid requests
  and admission refusals are refused; downstream rejection, feedback timeout,
  and cancellation are failures. TLS/authentication failures never reach this
  counter.
- `subscription_advertisements`: advertisements encoded in successful Enumerate
  responses. Repeated enumeration counts again; empty enumeration does not count.
  These are not subscription creations or activation confirmations.
- `marker_only_batches`: validated Events batches committed locally without
  emitting logs. Mixed batches and rejected marker-only batches do not count.
- `feedback_timeouts`: Events requests whose downstream feedback deadline
  expires. An earlier HTTP/connection timeout, disconnect, or forced shutdown
  does not increment this counter.

No custom log-count counter, pending gauge, heartbeat counter, or duration is
registered. Heartbeats and subscription-management requests emit no PData and do
not count as log messages. Validated marker-only Events requests count as local
receiver messages but emit no PData. Malformed events and classified policy or
capacity rejections count as local refusals. Earlier TLS, routing, SOAP dispatch,
and pre-body memory-pressure refusals are not classified log messages.

The shared helper honors engine telemetry interests for optional measurements.
Periodic collection runs every second and exports deltas. Shutdown joins HTTP
tasks before taking final snapshots. A downstream outcome does not prove Windows
received the HTTP response; a disconnected waiter does not prove downstream
rejected the records.
Retries count as additional attempts; there is no retry detector or deduplication
counter, and these metrics must not be interpreted as unique event totals.

## 12. Dependencies

- HTTP/TLS: `hyper` + `rustls`/`tokio-rustls` (workspace already uses rustls).
- XML: `quick-xml` (already in the workspace), shared by parsing and serialization.
- Compression: unsupported and not advertised. A future SLDC implementation
  would require independent verification against ECMA-321.
- No GPL dependencies.

## 13. Testing strategy

- Unit: SOAP parse/build round-trips; `WinEvent` XML -> pdata mapping golden
  tests; severity/level mapping; config validation (ETW-style table tests with
  the `Scenario:`/`Guarantees:` doc comments required by `AGENTS.md`).
- Protocol: replay captured WEF exchanges (Enumerate -> Subscribe -> Events ->
  Ack) against the state machine; assert correct `RelatesTo`/bookmark handling.
- Identity: verify identical subscription IDs across replicas and restarts,
  changed IDs on receiver/subscription rename, and isolation of repeated receiver
  names in separate pipeline scopes.
- Version: verify deterministic generation, changes on settings changes, and
  restoration on config rollback. Accept old and unknown version values for a
  known authorized subscription and preserve them in every emitted record;
  reject unknown subscription IDs independently of version.
- Source ownership: enumerate two sources sharing a subscription and verify
  independent bookmarks; deliver through another connection and verify lookup,
  feedback, and updates still reach the correct source owner.
- Reliability: inject downstream backpressure/failure and assert **no Ack** is
  sent and the bookmark is not advanced. Enqueue success without downstream
  feedback must not acknowledge acceptance. Restart with an empty bookmark map
  and verify the configured initial-read policy on new subscriptions.
- Authentication: reject missing, untrusted, expired, and unauthorized client
  certificates; verify source separation and identity continuity on renewal.
- E2E (mTLS): a real Windows source configured with an HTTPS
  `SubscriptionManager` and client certificate, without Kerberos or an AD service
  identity; verify enumeration, delivery, negotiated compression, pdata logs,
  in-memory bookmark reuse during the receiver lifetime, and accepted bookmark
  loss across restart. Synthetic sources supplement this coverage.

### 13.1. Live Windows recovery evidence (2026-09-22)

Both exercises used a real Windows source and mandatory mTLS. Times below are
those shown in the captured collector logs.

- Network interruption: after baseline record 92588, outbound TCP 5986 to the
  collector was temporarily blocked. Three test events created at 21:58:05
  (records 92589, 92590, 92591) all arrived in the accepted batch at 22:01:22
  after connectivity returned. No duplicate test records appeared in the shown
  output. This does not distinguish queued retry from bookmark replay.
- Downstream outage: WEF continued running while only the local OTLP sink was
  stopped. Application event 103, record 92602, was created at 22:46:14.450.
  At 22:46:44.507 the receiver rejected its batch after the OTLP exporter's
  connection failure. The sink restarted at 22:47:14.467; at 22:47:44.573 the
  retried Events request was accepted and the sink printed record 92602 once in
  the shown output. The WEF receiver was not restarted.

The downstream exercise used temporary pipelines: WEF -> OTLP exporter on port
14317, and a separate OTLP -> console sink. These local test configurations are
not shipped; the supported example is `configs/windows-event-forwarding-console.yaml`.
These observations establish recovery for the tested outages, not exactly-once
delivery, durable bookmarks, or a general guarantee against duplicate delivery.

### 13.2. Live metrics and instrumented recovery (2026-09-22/23)

Prometheus scraping uses the engine admin endpoint `/api/v1/metrics`; no separate
Prometheus exporter node is required. Enable receiver node telemetry `messages`
and `item_counts` to compare downstream batches and records. Exported metric names
such as `requests_total` and `items_total` are distinguished by `otel_scope_name`
and their other labels, not a WEF scope prefix in the metric name.

- Initial metrics run: two successful Events requests comprised one marker-only
  batch and one emitted batch. `marker_only_batches` increased once; the marker
  emitted no output message. Three heartbeats and two subscription advertisements
  were counted independently; no feedback timeout was recorded.
- Item-count run `c0a8981e-a069-4395-82b4-429f5666ed8b`: five tagged records
  92614-92618 plus two background records produced one successful output message
  and seven successful output items. The Application query includes background
  events, so generated test-event counts alone are not expected to match metrics.
- Instrumented outage: keep WEF running while stopping only the OTLP sink. The
  baseline produced one successful batch with six items. Background record 92625
  failed twice while the sink was unavailable; five tagged records 92626-92630
  from run `bcf18054-069d-4ffc-ae02-7ecaa965af73` queued behind it. After sink
  restart, record 92625 and the five tagged records arrived in two accepted
  batches. Final cumulative output counts were three successful batches/twelve
  items and two failed batches/two items. Events request outcomes were three
  successes and two failures; feedback timeouts stayed zero.

The tagged outage records appeared once in the captured sink output. The failed
attempts concerned the preceding background record, not necessarily each tagged
record. These measurements count attempts, not unique records, and do not directly
prove bookmark replay or exactly-once delivery.

### 13.3. Live graceful and forced drain (2026-09-23)

These runs used a temporary WEF -> debug capture -> ten-second delay -> console
pipeline. A watcher triggered shutdown when the tagged batch reached the capture
before the delay. The temporary harness is not part of the receiver implementation.

- Graceful drain, run `4fc01764-3200-433c-8cc2-c57b36b24dc8`: SIGINT at
  00:06:35.580 UTC, Events acceptance at 00:06:45.585, 10.005 seconds later.
  Tagged records 92633-92637 each appeared once at the console and the collector
  exited successfully. The initial watcher assertion mishandled ANSI log
  formatting; saved evidence passed after stripping terminal control sequences.
- Initial forced-deadline run: the admin endpoint accepted a two-second global
  shutdown and the deadline fired after 2.003 seconds. No tagged output or Events
  acceptance was logged, but the delay processor continued its active handler.
  At about ten seconds it attempted a send to a closed channel and the process
  exited with code 1. This exposed a separate engine processor-cancellation bug,
  not a demonstrated premature WEF acknowledgement. Keep that fix in its own PR.
- Retest with the engine fix, run `7c104658-baa5-4ef7-bd85-c7a42480d804`: all
  five tagged records 92650-92654 reached the capture. Admin shutdown was accepted
  with HTTP 202 at 00:41:34.889 UTC; the deadline fired 2.006 seconds after the
  request. No tagged console delivery or Events acceptance after shutdown was
  logged. The collector exited with code 0 and released both listeners.

The retest watcher failed only because its final event-count matcher retained the
previous test tag. Corrected matching against the saved capture verified all five
records and the successful collector exit. Exact process-exit duration was not
saved because that assertion failed before timing validation. Terminal telemetry
flushes reported expired deadlines; forced shutdown does not guarantee delivery
of final metric snapshots. No packet capture established the exact HTTP connection
close time, and these observations do not prove restart recovery.

The first retest batch was generated before initial enumeration and did not arrive
under `initial_read: new_events`. Generate live test events after subscription
activation; listener readiness alone does not establish an active Windows
subscription. The second batch arrived about thirty seconds after generation,
consistent with the configured delivery interval.

### 13.4. Current validation boundary

The 53 receiver-scoped tests cover configuration, identity, bounded SOAP/event
decoding, bookmark transitions, metrics, real mTLS TCP handling, and receiver
drain behavior. Scoped contrib-nodes compilation and Clippy have passed. These
checks and the live exercises above are not a production-readiness claim.
Destination validation has unit, real-mTLS TCP, and live Windows coverage.
On September 23, enumeration succeeded at 00:58:43.198 UTC and Heartbeat at
00:58:43.281. Run `34a622ad-2b93-44ed-85b5-1ebd1109ecc9` produced five tagged
Application records (92659-92663), each present once in the console capture.
Events acceptance was logged at 00:59:32.857; no destination mismatch was logged.
One background Application record accompanied the tagged batch. Admin shutdown
completed successfully. The initial capture checker expected debug-processor
formatting instead of console-exporter formatting; direct validation of the saved
console capture confirmed the result. Acceptance logs do not prove wire-level
receipt of the Ack, and console/protocol output ordering is not an Ack timestamp.

SubscriptionEnd has parser and real-mTLS TCP coverage, including malformed status,
wrong route, Identifier and destination rejection, and notifications during pending
Events that subsequently Ack, Nack, or time out. These tests verify that termination
does not emit pdata or alter committed bookmarks or pending feedback, and later
delivery remains possible. Live testing on September 23 found a compatibility
failure: at 01:17:06.307 UTC, an authenticated Windows SubscriptionEnd was rejected
with `unsupported SubscriptionEnd element`. The source subsequently recreated its
subscription and delivered tagged Application record 92667; its Events batch was
accepted at 01:20:29.856 UTC. A later reproduction at 01:38:43.488 UTC
was also rejected, but telemetry truncated the combined QName error. The
diagnostic now records the unexpected namespace and local name as separate bounded
fields (up to 128 characters each, escaped), without attributes or contents.
At 01:45:51.425 UTC, a temporary nonexistent-channel subscription reproduced a
rejected Microsoft `WSManFault` extension; Windows recorded subscription error
15007. The parser now recognizes that exact extension and validates its basic
structure. Regression tests cover malformed and duplicate faults, wrong
namespaces, and preservation of pending feedback and bookmarks across both plain
and fault-bearing notifications. After restoring an active Application subscription,
the fixed collector repeated the nonexistent-channel transition and accepted
SourceCancelling at 01:55:23.230 UTC. Windows recorded the expected channel error
15007 at 01:55:23.195; receiver metrics recorded one successful SubscriptionEnd
and no refused termination. This validates the reproduced fault scenario, not
source-side receipt of the Ack. The original 01:17 notification body was not
retained and cannot be compared verbatim.
After restoring the normal Application configuration, enumeration succeeded at
01:56:36.078 UTC and Heartbeat at 01:56:36.153 without another WinRM restart.
The collector then shut down cleanly through the admin endpoint. This recovery
check did not include a new tagged application event.

A controlled Windows `Restart-Service WinRM` subsequently verified the standard
shutdown variant without relaxing parser acceptance. The receiver accepted
SourceShuttingDown at 01:25:15.089 UTC, followed by enumeration at 01:25:16.317
and Heartbeat at 01:25:16.398. Tagged Application record 92670, generated at
01:26:04.410, appeared once; its Events batch was accepted at 01:26:34.525.
The final metrics showed one successful SubscriptionEnd, two successful Events
requests (one marker-only), and one output log item, with no refused termination
in this run. The collector shut down cleanly through the admin endpoint afterward.
This confirms shutdown-notification handling and resumed forwarding, not all
termination variants or source-side receipt of the Ack. Bookmark preservation
across termination remains covered by the deterministic mTLS tests; this live
diagnostic collector had been restarted before the WinRM exercise.

Certificate lifecycle and message-less live event coverage also remain follow-up
work. Compression and robust
replay are unsupported; durable bookmarks and exactly-once delivery are non-goals.

## 14. Phased plan

### 14.1. Initial minimal implementation

Goal: one real Windows source forwards one rendered-text subscription over mTLS
into a single-replica pipeline, with observable logs and protocol acknowledgements.

- Register the receiver URN and validate HTTPS, certificates, source authorization,
  one subscription, and the single-replica restriction.
- Use a simple explicit mapping from verified client certificate identity to
  `SourceIdentity`; do not trust SOAP machine names as authentication.
- Implement enumeration and its associated handshake, deterministic subscription
  ID/version generation, and the version-bearing delivery endpoint. Handle
  `Events`, `Heartbeat`, and `SubscriptionEnd` without version validation.
- Decode UTF-16 SOAP and event XML; support SLDC when advertised. Omitting
  compression requires demonstrated Windows interoperability first.
- Emit pdata logs with timestamps, severity, rendered body, source, channel,
  provider, event/record IDs, and the received `wef.subscription.version`.
- Preserve incoming batch boundaries, serialize source/subscription progress,
  correlate acknowledgements with `MessageID`, and gate them on downstream
  feedback. Keep bookmarks only in bounded memory.
- Bound request bodies, decompressed data, and in-flight batches; implement
  backpressure, graceful shutdown, and basic authentication/decode/delivery
  telemetry.

Acceptance: a real Windows source successfully enumerates, delivers events,
receives acknowledgements, and produces pipeline logs. Verify certificate
rejection, per-delivery version attributes, and no bookmark advancement on
downstream failure. No guarantee of bookmark recovery after restart is required.

Initially exclude cross-thread routing, raw content, SID resolution,
deduplication, and an additional batcher. Single-replica execution preserves the
source-owner model without cross-thread infrastructure. Multi-replica support is
later work; permanent storage is out of scope for the entire project.

### 14.2. Implementation sequence

1. Skeleton: crate wiring, URN + factory registration, config + validation,
  metrics, `metadata.yaml`, mandatory-mTLS listener and source authorization.
2. Protocol MVP: WS-Man state machine (Enumerate/Subscribe/Events/Ack/
  Heartbeat/SubscriptionEnd) over HTTPS, including SLDC when advertised;
  event decode -> pdata; memory bookmarks; downstream-gated ack.
3. Complete the minimal milestone: drain/shutdown, bounded state, basic telemetry,
  and the real-Windows acceptance test above.
4. Later hardening: certificate lifecycle, fuzz SOAP/SLDC, multi-replica source
  ownership, full telemetry, oversize handling,
  dedup guidance, docs +
   changelog entry.

## 15. Alternative considered: OpenWEC out-of-process

Instead of a native receiver, run `openwecd` (GPL-3.0) as a separate process
with an output that ships to the pipeline (e.g. an OTLP output driver, or a
socket/Kafka output consumed by an existing receiver). This avoids implementing
the protocol at all and keeps license separation (separate program). It is the
recommended way to get events flowing quickly and to validate the pdata mapping
before committing to the native path. The native receiver in this document is
the longer-term, dependency-light, Apache-2.0 option.

## 16. Open questions

- Finalize production trust rotation and revocation policy.
- Decide whether inactive-source cleanup is needed beyond the implemented bounded
  in-memory state; currently source entries remain allocated until restart.
- Finalize `winlog.*` attribute names against current OTel semantic conventions.
- Choose listener topology and cross-thread request transport while preserving
  source ownership. Define draining/fencing for ownership transfer during
  scaling and overlapping pipeline generations.
- Do we need collector-initiated (pull) mode for any target environment?
