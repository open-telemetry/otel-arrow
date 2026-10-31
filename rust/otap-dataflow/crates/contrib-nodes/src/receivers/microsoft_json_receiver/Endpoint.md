# Endpoint Configuration

Each URL path in the `endpoints` map references an external YAML file that
defines how the receiver accepts, parses, and maps incoming events for that
route.

## Endpoint file references

```yaml
endpoints:
  "/OneCollector/1.0":
    config_file: "onecollector-1ds-endpoint.yaml"
  "/v2/track":
    config_file: "azure-monitor-envelope.yaml"
  "/v2.1/track":
    config_file: "azure-monitor-envelope.yaml"
```

Each entry requires a `config_file` field pointing to an external YAML file
containing the full `EndpointConfig`. Relative paths are resolved against
the pipeline configuration directory.

`/v2/track` and `/v2.1/track` are both Application Insights / Azure Monitor
ingestion paths in these examples. Neither path forces a mapping strategy:
both can reference `azure-monitor-envelope.yaml` for preservation, or both can
reference `azure-monitor-browser-events.yaml` for expanded browser-event
mapping.
Configure both paths when clients may use either; adding one does not
implicitly register the other. These are inbound receiver paths, not the
destination URL used by the Azure Monitor exporter.

## EndpointConfig fields

| Field | Type | Default | Description |
| ------- | ------ | --------- | ------------- |
| `name` | `string` | `""` | Human-readable label for logging and diagnostics. |
| `allowed_methods` | `string[]` | `["POST"]` | HTTP methods accepted (uppercased at compile time). |
| `accepted_content_types` | `string[]` | `[]` (accept any) | Allowed `Content-Type` values. Parameters (e.g. `;charset=utf-8`) are stripped before matching. |
| `body_format` | `enum` or `enum[]` | `line_separated_json` | Accepted POST body parsing strategies. |
| `get_format` | `enum` | `event_fields` | GET query-string handling. |
| `filtered_item_size` | `boolean` | `false` | Emits the compact JSON byte size of retained source fields as the `filtered_item_size` log attribute. |
| `response_format` | `enum` | `minimal` | HTTP response body format. |
| `event_name_validation` | `enum` | `none` | Validation applied to the final event name. |
| `drop_events` | `string[]` | `[]` | Event name patterns to silently discard. Supports exact names and glob prefixes (`*` suffix). |
| `cors` | `CorsConfig` | *(see below)* | CORS response header configuration. |
| `header_mappings` | `HeaderMapping[]` | `[]` | Maps HTTP headers (or promoted QS params) to log record fields. |
| `query_string_headers` | `map<string, string>` | `{}` | QS parameter names to promote as virtual headers (lowercased key -> canonical header name). |
| `rules` | `FieldRule[]` | `[]` | Ordered field mapping rules for JSON body parsing. |
| `event_overrides` | `map<string, EventOverride>` | `{}` | Per-event-name rule overrides (exact or glob key). |
| `ikey_filter` | `IkeyFilterSource` | `null` | Instrumentation key (iKey) filtering. Supports inline rules or an external file reference. See [iKey filtering](#ikey-filtering). |
| `env_resource_attrs` | `map<string, string>` | `{}` | Maps environment variable names to OTel resource attribute keys. Resolved at compile time. See [Environment variable mapping](#environment-variable-mapping). |

## Body formats

| Value | Description |
| --- | --- |
| `line_separated_json` | Newline-delimited JSON -- one event per line. Used by the 1DS Web SDK. |
| `json_array` | JSON array of objects (or a single object). Used by Application Insights. |

Specify one value for a single accepted format, or a list when an endpoint
accepts multiple formats:

```yaml
body_format:
  - line_separated_json
  - json_array
```

When multiple formats are configured, the receiver identifies the top-level
JSON structure before parsing so the payload is decoded only once.

## Filtered item size

`filtered_item_size` measures one accepted item after field mapping determines
which source values are retained. The receiver preserves the source JSON keys
and nesting, omits dropped or skipped values, serializes the result as compact
UTF-8 JSON, and writes the byte count to the `filtered_item_size` integer log
attribute. This is an emitted log attribute, not an additional endpoint
configuration field. The generated size attribute is not included in its own
measurement.

```yaml
filtered_item_size: true
```

The measurement is disabled when the field is omitted. It is a projected
logical item size, not an HTTP `Content-Length`, compressed size, or
authoritative billing value.

## GET formats

| Value | Description |
| --- | --- |
| `event_fields` | Query-string `key=value` pairs are treated as fields of a single event and processed through field mapping rules. Used for browser beacon (`t.js`) fallback. |
| `disabled` | GET requests return `405 Method Not Allowed`. |

## Response formats

| Value | Body shape | Description |
| ------- | ------------ | ------------- |
| `minimal` | `{}` | Empty JSON object. |
| `one_collector` | `{"acc":<n>,"rej":<n>,"errors":[...]}` | 1DS protocol response with accepted/rejected counts. |
| `azure_monitor` | `{"itemsReceived":<n>,"itemsAccepted":<n>,"errors":[...]}` | Azure Monitor / Application Insights protocol response. |

## Event name validation

| Value | Description |
| ------- | ------------- |
| `none` | No validation (default). |
| `printable_ascii` | Allows printable ASCII characters (U+0021-U+007E). |
| `otel_semconv` | OpenTelemetry semantic convention: `[a-z0-9._]`, must start with `[a-z]`, no consecutive delimiters. |

## Query-string header promotion

The `query_string_headers` map controls which query-string parameters are
promoted to virtual HTTP headers.  Promoted parameters are consumed during
parsing and are **not** included in GET event data.  They become available to
`header_mappings` and to built-in protocol checks (`Content-Type`,
`Content-Encoding`, `NoResponseBody`).

```yaml
query_string_headers:
  content-type: "Content-Type"
  content-encoding: "Content-Encoding"
  noresponsebody: "NoResponseBody"
  apikey: "apikey"
  sdk-version: "SDK-Version"
  client-id: "Client-Id"
  upload-time: "upload-time"
```

Keys are lowercased QS parameter names; values are the canonical header names
used for logging/diagnostics (the lookup is always case-insensitive).  Both
keys and values are lowercased at compile time.

## Header mappings

Header mappings extract values from HTTP headers (or promoted query-string
parameters -- see above) and inject them into the log record.

```yaml
header_mappings:
  - header: "apikey"
    target:
      kind: resource
      key: "ms.iKey"

  - header: "SDK-Version"
    target:
      kind: attribute
      key: "ms.sdk.version"
```

Each entry specifies:

- `header` -- the header name (matched case-insensitively at runtime).
- `target` -- a `MappingTarget` describing where to place the value.

Only the `resource` and `attribute` target kinds are supported for header
mappings.

## Field mapping rules

Rules define how JSON body fields map to OTel log record fields.  They are
processed top-down in declaration order.

```yaml
rules:
  - path: "name"
    target:
      kind: event_name
    validate:
      expected_type: string
      max_length: 1024
      on_validation_failure: reject

  - path: "time"
    target:
      kind: timestamp
    validate:
      expected_type: string
      on_validation_failure: reject

  - path: "ext"
    children:
      - path: "dt"
        allowed_keys: [traceId, spanId, traceFlags]
        children:
          - path: "traceId"
            target: { kind: trace_id }
          - path: "spanId"
            target: { kind: span_id }
```

### FieldRule fields

| Field | Type | Default | Description |
| ------- | ------ | --------- | ------------- |
| `path` | `string` | *(required)* | Dot-separated JSON field path. Supports `*.*` wildcards. |
| `target` | `MappingTarget` | `null` | Where to place the value. Omit for container-only nodes that just hold `children`. |
| `validate` | `Validation` | `null` | Optional type checking and length constraints. |
| `allowed_keys` | `string[]` | `null` | Restrict accepted child keys; an unlisted key causes validation failure for the entire event. |
| `required` | `Required` | `null` | Mark the field as required; missing fields trigger the configured action. |
| `also` | `AlsoTarget[]` | `[]` | Write the same value to additional targets, optionally with transforms. |
| `children` | `FieldRule[]` | `null` | Nested child rules (flattened at compile time). |

### Mapping targets

The `target` object uses a `kind` discriminator:

| Kind | Extra fields | Description |
| ------ | ------------- | ------------- |
| `event_name` | `prefix`, `split_last`, `strip_segment` | Log record event name. |
| `timestamp` | -- | `time_unix_nano` (parsed from RFC 3339). |
| `observed_timestamp` | -- | RFC 3339 observed time. |
| `trace_id` | -- | 32 hex character trace identifier. |
| `span_id` | -- | 16 hex character span identifier. |
| `trace_flags` | -- | Integer trace flags. |
| `severity` | `map` | Maps scalar values to `severity_number` / `severity_text`; numeric and boolean values use their JSON string representation as lookup keys. |
| `attribute` | `key`, `prefix` | Log record attribute. If `key` is omitted, the JSON field name (optionally prefixed by `prefix`) is used. |
| `resource` | `key`, `prefix` | Resource attribute. If `key` is omitted, the JSON field name (optionally prefixed by `prefix`) is used. |
| `scope` | `field`, `split` | Instrumentation scope name or version. |
| `drop` | -- | Silently discard the field. |

Mapping targets, detailed `strip_segment` configurations, and transforms reject
unknown options. Misspellings are configuration errors rather than silently
selecting defaults.

### Validation

```yaml
validate:
  expected_type: string       # string | integer | float | boolean | string_coerce
  max_length: 1024            # max byte length (post-coercion)
  on_length_exceeded: truncate  # truncate (default) | reject
  on_validation_failure: skip   # skip (default) | reject
```

### Also targets

Fan out a single field value to multiple destinations with optional transforms:

```yaml
- path: "sdk.version"
  target:
    kind: attribute
    key: "telemetry.sdk.full_version"
  also:
    - target:
        kind: attribute
        key: "telemetry.sdk.name"
      transform: { kind: dash_semver_name }     # "1DS-Web-JS-4.4.1" -> "1DS-Web-JS"
    - target:
        kind: attribute
        key: "telemetry.sdk.version"
      transform: { kind: dash_semver_version }  # "1DS-Web-JS-4.4.1" -> "4.4.1"
```

Available transforms:

| Transform | Description |
| ----------- | ------------- |
| `split_take` | Split by `delimiter` and take the part at `index` (negative = from end). |
| `format` | Apply a template string with `{value}` placeholder. |
| `dash_semver_name` | Extract the name portion from a dash-separated semver string. |
| `dash_semver_version` | Extract the version portion from a dash-separated semver string. |

## Event overrides

Override or extend field mapping rules for specific event names.  Keys can be
exact names or glob patterns (trailing `*`).

Each override can specify an `action`:

| Action | Description |
| -------- | ------------- |
| `accept` | Process the event normally using the (possibly overridden) rules. This is the default. |
| `drop` | Silently discard the event (not counted as rejected). |
| `reject` | Reject the entire HTTP request payload containing this event. |

By default, an override inherits all endpoint-level `rules` and overlays its
own rules by path. Set `inherit_rules: false` to compile the override using
only the rules declared within that override:

```yaml
event_overrides:
  "Microsoft.ApplicationInsights.Metric":
    inherit_rules: false
    rules:
      - path: "data.baseData.metrics"
        target:
          kind: attribute
          key: "ai.baseData.metrics"
```

Exact matches take priority over glob patterns.  This allows a catch-all glob
to block unknown events while specific exact overrides accept known ones:

```yaml
event_overrides:
  # Accept specific known events (with optional rule overrides).
  "Ms.Web.PageView":
    action: accept
    rules:
      - path: "data.uri"
        target:
          kind: attribute
          key: "url.full"
        validate:
          expected_type: string
          max_length: 2048

  # Accept all events matching a prefix.
  "Ms.Web.*":
    action: accept
    rules:
      - path: "data.isLoggedIn"
        target:
          kind: attribute
          key: "ms.web.isLoggedIn"

  # Drop internal diagnostics events.
  "Microsoft.Internal.*":
    action: drop
```

## iKey filtering

Each endpoint can optionally filter events based on their instrumentation key
(iKey).  The filter is checked before field mapping -- rejected events never
reach the pipeline.

Rule compilation, evaluation, and external-file loading are implemented in the
iKey filter module. The receiver remains a skeleton: it does not load endpoint
files or invoke the filter during request handling yet. HTTP responses, metrics,
and applying remapped keys to emitted telemetry describe the integration
contract for the receiver runtime, not behavior emitted by this phase.

### Configuration

The `ikey_filter` section supports two forms:

**Inline rules** -- defined directly in the endpoint YAML:

```yaml
ikey_filter:
  keys:
    "o:bad-key-guid":
      action: reject
    "o:limited-key":
      action: accept
      allowed_events:
        - "Ms.Web.PageView"
        - "Ms.Device.*"
    "o:browser-key":
      action: accept
      require_origin: true
      allowed_origins:
        - "https://myapp.example.com"
        - "https://staging.example.com"
    "o:legacy-key":
      action: accept
      remap_ikey: "o:new-key-guid"
    "*":
      action: accept
```

**External file reference** -- rules loaded from a separate YAML file:

```yaml
ikey_filter:
  filter_file: "ikey-rules.yaml"
```

The external file uses the same `keys:` schema as inline rules.  Relative
paths are resolved against the pipeline config directory.

When both `filter_file` and `keys` are present, `filter_file` takes
precedence. A missing or invalid external file is an error; inline rules
are not used as a fallback.

The external file contains the `keys` map directly, without an `ikey_filter`
wrapper:

```yaml
keys:
  "o:allowed-key":
    action: accept
  "*":
    action: reject
```

### Configuration fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `filter_file` | `string` | unset | External YAML file, taking precedence over inline `keys`. |
| `keys` | ordered map | `{}` | iKey patterns mapped to rules in declaration order. |

Every rule supports the following fields. Unknown configuration fields and
unknown actions are rejected.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `action` | `accept`, `reject`, or `ignore` | required | Verdict for a matching key. |
| `allowed_events` | `string[]` | `[]` | Event allow-list for `accept` rules; empty means unrestricted. |
| `require_origin` | `bool` | `false` | Require an origin matching `allowed_origins` for `accept` rules. |
| `allowed_origins` | `string[]` | `["*"]` | Case-insensitive exact origins or `"*"`, checked only when required. |
| `remap_ikey` | `string` | unset | Replacement key returned after an `accept` rule passes all checks. |

### Compilation and file loading

`compile_source` returns an optional validator and the external file path,
if one was used. No file and no inline rules returns no validator. An external
file with an empty `keys` map creates a validator that accepts everything.

When a pipeline configuration directory is supplied, external paths must be
relative, must not contain `..`, and must resolve within that directory after
symlinks or junctions are resolved. File resolution, read, and YAML parse errors
are returned to the caller with path context. When no configuration directory
is supplied, the loader uses the path as supplied, with relative paths resolved
against the process working directory. The runtime must supply its pipeline
configuration directory to enforce confinement.

### Matching rules

- Key patterns are matched in **declaration order**; the **first match wins**.
- `"*"` matches any key (catch-all).
- All other patterns are compared as exact, case-sensitive strings.
- A missing iKey matches only the catch-all, not an exact key pattern.
- When `allowed_events` is set on an `accept` rule, the event name must match
  at least one pattern.  Patterns support a trailing `*` glob
  (e.g. `"Ms.Device.*"` matches `"Ms.Device.Info"`).
- An event that matches an `accept` rule but fails the `allowed_events` check
  is **rejected**.
- Event matching is case-sensitive. Only a trailing `*` is a glob; stars in
  other positions are literal characters.
- If the resolved event name is not yet known, the event-name check is skipped.
- An empty `allowed_events` list allows every event name.
- If no rule matches, the event is **accepted** (open policy).

### Origin validation

When `require_origin: true` is set on a rule, incoming requests must include
an `Origin` (or `Referer`) header that matches one of the `allowed_origins`
patterns.  This is useful for browser-facing iKeys where you want to restrict
which web origins can submit telemetry.

The validator accepts a caller-resolved origin string; extracting the `Origin`
header or a `Referer` fallback belongs to the future HTTP handler. These checks
are not authentication: non-browser clients can supply arbitrary headers.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `require_origin` | `bool` | `false` | When `true`, the request must carry a matching origin. |
| `allowed_origins` | `string[]` | `["*"]` | Origins permitted for this key. `"*"` allows any origin. Matching is case-insensitive. |

- If `require_origin` is `false` (the default), origin checking is skipped
  entirely -- the request is accepted regardless of the `Origin` header.
- If `require_origin` is `true` and the request has no `Origin` header (and no
  `Referer` fallback), the event is rejected as **unauthorized**.
- If `require_origin` is `true` and the origin does not match any entry in
  `allowed_origins`, the event is rejected as **unauthorized**.
- `"*"` allows any supplied origin but does not satisfy a missing origin.
  An explicitly empty `allowed_origins` list allows no origins.
- Origin validation runs before the event-name check. A failed origin check
  returns `RejectUnauthorized`, representing HTTP 401 for runtime integration.
- Unauthorized events are counted in the `ikey.unauthorized` metric.

### iKey remapping

The `remap_ikey` field allows an accepted event's `iKey` to be rewritten
before it enters the pipeline.  This is useful for migrating instrumentation
keys without requiring client-side changes.

```yaml
"o:old-key":
  action: accept
  remap_ikey: "o:new-key"
```

- Remapping occurs **after** origin and `allowed_events` checks pass.
- The remapped key is **not** re-evaluated against the filter rules.
- Only applies to `accept` rules; `reject` and `ignore` rules ignore
  `remap_ikey`.
- Event name `strip_segment` matching (e.g. removing the iKey segment from
  Application Insights envelope names like
  `Microsoft.ApplicationInsights.{iKey}.{Type}`) uses the **original**
  iKey, not the remapped value.  The remapped value is written to the
  resource attribute (e.g. `ai.iKey`).

With `strip_segment: { index: 2, match_field: "iKey" }`, the candidate
segment is compared with the original event field. Canonical hyphenated GUIDs
and compact 32-hex GUIDs compare ignoring hyphens and hex case. This matches
the browser SDK, which embeds a dashless key in the envelope name while
retaining the original GUID in `iKey`. Other identifiers compare exactly and
case-sensitively; a missing field never matches. This normalization applies
only to event-name stripping, not to the iKey filter's exact-key matching.
The comparison helper is available for the future mapper; the receiver still
returns `NotImplemented`.

The browser mapping uses the SDK name `Microsoft.ApplicationInsights.Pageview`.
Page-view `name`, `url`, `duration`, and `id` are optional and validated only
when supplied.

### Actions

| Action | Description |
| -------- | ------------- |
| `accept` | Allow the event (subject to `allowed_events` and origin checks). Optionally remap the iKey via `remap_ikey`. |
| `reject` | Reject the event -- the sender receives an error response. Counted in `ikey.rejected`. |
| `ignore` | Silently drop the event -- no error is returned. Counted in `ikey.ignored`. |

### Metrics

| Metric | Description |
| -------- | ------------- |
| `ikey.rejected` | Events rejected by the iKey filter. |
| `ikey.ignored` | Events silently dropped by the iKey filter. |
| `ikey.unauthorized` | Events rejected due to missing or disallowed origin. |

To reject everything else, use a catch-all rule:

```yaml
  "*":
    action: reject
```

Override rules are merged with (and can override) the base `rules` at compile
time.  The `action` is checked before any mapping work -- `drop` and `reject`
overrides short-circuit immediately.

## Environment variable mapping

The `env_resource_attrs` map lets you inject process environment variables
as OTel resource attributes.  This is useful in containerised deployments
(e.g. Azure Container Apps, Kubernetes) where the platform injects
metadata into the environment.

```yaml
env_resource_attrs:
  CONTAINER_APP_NAME: "ms.container_app.name"
  CONTAINER_APP_REVISION: "ms.container_app.revision"
  CONTAINER_APP_REPLICA_NAME: "ms.container_app.replica_name"
  CONTAINER_APP_ENV_DNS_SUFFIX: "ms.container_app.env_dns_suffix"
  CONTAINER_APP_HOSTNAME: "ms.container_app.hostname"
  CONTAINER_APP_PORT: "ms.container_app.port"
  HOSTNAME: "host.name"
  POD_NAME: "k8s.pod.name"
```

- Keys are environment variable names; values are the resource attribute
  keys they map to.
- Values are resolved **once** at endpoint compile time.
- Environment variables that are **unset** or **empty** are silently
  skipped -- no attribute is emitted.
- JSON body values always win: if the event JSON already contains a
  resource attribute with the same key, the env-based value is not applied.

To generate a unique per-process instance ID, use the receiver-level
`instance_id_attr` setting instead (see the receiver README).

## CORS

CORS headers are applied in two layers:

1. **Receiver-level** (`allowed_origins` in the main config) -- controls which
   origins receive `Access-Control-Allow-Origin`.

2. **Endpoint-level** -- each endpoint configures CORS via the `cors` object:
   - `allow_credentials` -> `Access-Control-Allow-Credentials`
   - `max_age_seconds` -> `Access-Control-Max-Age`
   - `cache_control` -> `Cache-Control`
   - `allow_headers` -> `Access-Control-Allow-Headers`
   - `expose_headers` -> `Access-Control-Expose-Headers`
   - `extra_response_headers` -> arbitrary additional headers on every response

`allowed_methods` is reflected in `Access-Control-Allow-Methods` for OPTIONS
preflight responses.

### CorsConfig

The `cors` object controls all CORS response headers included on every
response (including OPTIONS preflight):

| Field | Type | Default | Description |
| ------- | ------ | --------- | ------------- |
| `allow_credentials` | `bool` | `true` | Whether to include `Access-Control-Allow-Credentials: true`. |
| `max_age_seconds` | `integer` | `3600` | Value for `Access-Control-Max-Age` (seconds). |
| `cache_control` | `string` | `"no-cache, no-store, max-age=<max-age>"` | Value for the `Cache-Control` header. The `<max-age>` token is replaced with the resolved `max_age_seconds` value. |
| `allow_headers` | `string` | `null` | Value for `Access-Control-Allow-Headers` (comma-separated). |
| `expose_headers` | `string` | `null` | Value for `Access-Control-Expose-Headers`. |
| `extra_response_headers` | `map<string, string>` | `{}` | Additional headers added to every response (including OPTIONS). |

```yaml
# All fields shown with their defaults:
cors:
  allow_credentials: true          # default: true
  max_age_seconds: 3600            # default: 3600
  # cache_control default: "no-cache, no-store, max-age=<max-age>"
  # The <max-age> token is replaced with the max_age_seconds value.
  # Override to set a custom value:
  # cache_control: "max-age=<max-age>, public"
  allow_headers: null              # default: null (omitted from response)
  expose_headers: null             # default: null (omitted from response)
  extra_response_headers: {}       # default: {} (none)
```

The `<max-age>` token in `cache_control` is replaced with the resolved
`max_age_seconds` value at compile time.  For example, with the default
`max_age_seconds: 3600`, the resolved header becomes
`"no-cache, no-store, max-age=3600"`.  You can also use the token in
custom values, e.g. `"max-age=<max-age>, public"`.

## Complete examples

Examples now live alongside this document:

- [onecollector-1ds-endpoint.yaml](onecollector-1ds-endpoint.yaml)
  - OneCollector / 1DS Common Schema 4.0 endpoint with full field mapping rules.
- [azure-monitor-browser-events.yaml](azure-monitor-browser-events.yaml)
  - Expanded browser-event attributes, individual field validation, and event overrides.
- [azure-monitor-envelope.yaml](azure-monitor-envelope.yaml)
  - Envelope-preserving mapping for JSON arrays, single objects, and NDJSON.
- [azure-monitor-exporter.yaml](azure-monitor-exporter.yaml)
  - Complete receiver-to-Azure pipeline with identity authentication,
    both ingestion routes, exporter mappings, and connection wiring.
- [ikey-rules.yaml](ikey-rules.yaml)
  - External instrumentation-key rules.
- [pipeline-local-console.yaml](pipeline-local-console.yaml)
  - Minimal local pipeline; the receiver still returns `NotImplemented`.

### Choosing an Application Insights / Azure Monitor mapping

The two endpoint files accept the same family of inbound envelopes but serve
different downstream goals. Their names do not select different HTTP routes
or force a particular exporter.

| Concern | Preserving: `azure-monitor-envelope.yaml` | Expanded: `azure-monitor-browser-events.yaml` |
| --- | --- | --- |
| Goal | Re-emit the original envelope fields through the paired exporter. | Expose telemetry fields directly to OTel processing and other exporters. |
| Nested data | Keep `tags`, `data`, and `ext` together in dedicated JSON-string attributes. | Extract tags, base data, properties, and measurements into individual attributes. |
| OTel fields | Populate event name and timestamp while retaining their original values separately. | Also derive trace/span IDs, severity, resource attributes, and instrumentation scope. |
| Transformations | Avoid flattening, coercion, iKey stripping, and diagnostic-event drops. | Apply field validation, coercion, event-name transforms, and per-event overrides. |
| Output | Existing column mappings emit envelope attributes; containers remain JSON strings. | Original nesting and values cannot generally be recovered from normalized fields. |

For example, the preserving mapping stores the entire `data` object under
`azure_monitor.envelope.data`, including `baseType`, `baseData`, properties,
measurements, and nested arrays. The expanded mapping instead makes fields
such as `data.baseData.url` available as `ai.baseData.url`,
`data.baseData.properties.region` as `ai.properties.region`, and
`data.baseData.measurements.duration` as `ai.measurements.duration`. It also
maps `tags.ai.operation.id` to the OTel trace ID and
`tags.ai.internal.sdkVersion` to instrumentation scope name/version.

The preserving mapping is deliberately simpler, not a reduced semantic
mapping that still extracts every nested field. The Azure Monitor exporter is
a schema-driven **Logs Ingestion API** exporter: it maps configured OTel fields
and attributes to columns, but does not understand how to invert all of the
expanded mapping's transforms or rebuild Application Insights envelopes.
Keeping each container intact avoids needing that inverse transformation.
The paired schema explicitly maps the envelope attributes back to their
original column names. It emits JSON-string containers unchanged, not as nested
objects; no exporter changes or JSON-decoding settings are required.

This is a choice for preservation, not a requirement imposed by the exporter.
The Azure Monitor exporter can also export expanded attributes using a
different column schema when queryable individual columns are preferred over
envelope reconstruction. Conversely, downstream OTel processors cannot
directly match a property buried inside a JSON-string container without first
parsing it. Choose the expanded mapping for field-level filtering, enrichment,
and cross-backend analysis; choose the simpler mapping to retain container
contents without expansion, accepting string-valued container columns.

### Browser-event extraction for Azure Data Explorer

The [browser-event mapping](azure-monitor-browser-events.yaml) extracts
custom properties, measurements, and known browser telemetry fields so they
can be validated and queried individually rather than left inside an opaque
JSON-string envelope. This is useful for browser-event analytics in
[Azure Data Explorer (ADX)](../../exporters/azure_data_explorer_exporter/README.md),
not just for general OTel processing.

The ADX exporter writes log attributes into the `LogsAttributes` dynamic
property-bag column and resource attributes into `ResourceAttributes`.
Extracted fields become separate keys in those bags, not automatically
separate physical table columns. Custom property names can vary between
browser events without requiring a table-schema change for every new key.
Measurements retain numeric values for aggregation, while field rules can
normalize other values to their configured types.

For example, with `LogsAttributes` configured as a dynamic column, the extracted
`ai.properties.region` and `ai.measurements.duration` attributes can be queried
directly using bracket notation because their keys contain dots:

```kusto
OpenTelemetryLogs
| summarize AverageDuration = avg(todouble(LogsAttributes["ai.measurements.duration"]))
    by Region = tostring(LogsAttributes["ai.properties.region"])
```

Replace `OpenTelemetryLogs` with the configured ADX log table name. In contrast,
the preserving mapping keeps `data` as one JSON-string attribute; querying a
property inside it requires parsing that string first.

Extraction also enables **individual field validation at the receiver level**.
Rules specify expected types, string-length limits, required fields, and
event-specific constraints. For example, `data.baseData.url` has a string-type
check and a length limit, measurements require numeric values, and event
overrides declare required fields for particular envelope types. Configured
failure policies determine whether an invalid value is rejected or skipped,
and length policies determine whether an overlong string is truncated or
rejected. This catches or normalizes invalid fields before downstream
processing and ADX ingestion instead of deferring all validation to the sink.

These are the intended mapping semantics once request handling is implemented.
The current receiver skeleton validates receiver configuration only; it does
not yet apply field rules or validate incoming browser events.

The browser mapping derives native OTel severity only from
`data.baseData.severityLevel`: 0 (Verbose) maps to TRACE (1), 1 to INFO (9),
2 to WARN (13), 3 to ERROR (17), and 4 (Critical) to FATAL (21).
The original integer is also retained as `ai.baseData.severityLevel`.
Missing, unmapped, or invalid levels leave severity UNSPECIFIED; `baseType`
is retained as `ai.baseType` and does not supply a competing severity fallback.
Exception and message overrides inherit this common severity rule.

Both expanded endpoint examples use `unmapped.mode: body` and omit a
`data.baseData.*` catch-all. Unhandled children, including `exceptions` arrays,
retain their original nesting in the body without string coercion or a
field-level truncation limit. Request and event size limits still apply.
The OneCollector example therefore preserves unknown fields rather than
reporting only their paths in `unmapped.fields`.

### Azure Monitor envelope preservation

Use the envelope-preserving endpoint instead of the semantic Application
Insights example when preserving input values matters. The semantic example
flattens fields, strips embedded iKeys, coerces values, and drops diagnostics;
those changes are intentionally absent from the envelope-preserving mapping.

The preserving mapping keeps `name`, `time`, `iKey`, `ver`, `sampleRate`, `seq`,
`flags`, `tags`, `data`, and `ext` in dedicated log attributes. `name` and `time`
also populate the OTel event name and timestamp. Nested containers are retained
as JSON strings rather than flattened fields. The existing exporter emits
those attributes as strings. It does not parse them into nested JSON values.
Envelope fields use explicit log-attribute mappings. Request and deployment
metadata are exported separately using `resource_mapping`; they are not
inserted into the serialized envelope containers.

The exporter sends these rows to the **Logs Ingestion API**, not the
Application Insights `/v2/track` API. Configure the DCR stream/table to accept
the envelope columns (`tags`, `data`, and `ext` as string columns).

The [complete Azure Monitor pipeline](azure-monitor-exporter.yaml) includes the
receiver, both `/v2/track` and `/v2.1/track` routes, an `azure_identity_auth`
extension, the exporter capability binding, and connections. Replace its API
placeholders, configure browser origins, and authorize the selected identity
to write to the DCR. It uses development credentials by default; its comments
describe switching to managed identity. Enable `microsoft-json`,
`azure-monitor`, `azure-identity-auth`, and a crypto provider feature.

#### Resource metadata in the Azure Monitor pipeline

The pipeline explicitly keeps user-agent and HTTP enrichment on resources and
enables request-origin and generated receiver-instance attributes. The existing
exporter's `log_record_mapping.attributes` covers log attributes only:
`resource_mapping` is required to include resource attributes in output rows.

| Resource attribute | DCR column | Source |
| --- | --- | --- |
| `user_agent.original` | `UserAgent` | Request user-agent enrichment. |
| `http.request.method` | `HttpMethod` | Request HTTP enrichment. |
| `http.origin` | `RequestOrigin` | Receiver `origin_attr` setting; omitted without an origin/referrer. |
| `receiver.instance.id` | `ReceiverInstanceId` | Receiver `instance_id_attr` setting; generated at startup. |
| `service.name` | `ServiceName` | Optional `OTEL_SERVICE_NAME` environment variable. |
| `service.version` | `ServiceVersion` | Optional `SERVICE_VERSION` environment variable. |
| `service.instance.id` | `ServiceInstanceId` | Optional `SERVICE_INSTANCE_ID` environment variable. |

The envelope endpoint's `env_resource_attrs` supplies the optional service
metadata; unset or empty environment variables are omitted. These values
identify the receiving deployment, not the originating browser application.
Create the listed DCR columns as strings alongside the envelope columns and
`TimeGenerated`. Destination names are distinct across resource and log
mappings, as required by the existing exporter.

Resource mappings do not create source attributes. If you change enrichment
targets to `log_attribute`, update the corresponding exporter mappings as
well. Remove unwanted resource mappings and their DCR columns together.
These columns intentionally add metadata beyond the original envelope.

This is not structural or byte-for-byte JSON equality: `tags`, `data`, and
`ext` become string-valued columns containing serialized JSON, rather than
nested objects. Whitespace, object-key order, and numeric spelling may change.
Recovering nested values requires separate downstream parsing; the current
exporter cannot reconstruct the original envelope through configuration alone.
Azure adds `TimeGenerated` from event time. Unknown top-level fields
remain in the log body but are not re-emitted by this explicit exporter schema;
add paired rules and columns for extensions. Do not enable iKey remapping or
lossy validation when preserving the original envelope.

The receiver runtime and Arrow builder have not been ported. Tests validate
the example schemas against the existing configuration types; they do not
demonstrate live HTTP-to-Azure round-trip behavior.
