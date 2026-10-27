# PData codecs

Codecs let the Dataflow Engine carry encoded telemetry through a pipeline and
convert it to native OTAP Arrow records when a consumer needs them. A matching
output can forward encoded bytes without decoding them. A processor that needs
records, or an exporter that needs another encoding, requests conversion.

This preserves the original input for error reporting and avoids conversion
work on paths that do not need it. Codec-native batching can merge or split
encoded batches without converting to OTAP; it may allocate new output buffers.

For the architecture diagram, extension interfaces, and contribution workflow,
see the [codec developer guide](../crates/pdata-codec/README.md).

## Available encodings

Codec implementations are compiled into the engine, not dynamically loaded by
configuration. An encoding identity names a byte-compatible representation,
including incompatible format versions and intrinsic compression. Native OTAP
is the common processing representation, not a registered byte encoding.

| Encoding | Signals | Decode to OTAP | Encode from OTAP | Native batching |
| --- | --- | --- | --- | --- |
| `otlp-bytes` | Logs, metrics, traces | Yes | Yes | Encoded bytes |
| `syslog-batch` | Logs | Yes | No | Framed message items |

The syslog/CEF receiver creates independently decodable framed batches. This
internal representation is not raw syslog TCP/UDP framing. A syslog batch can
remain encoded through transparent nodes and native batching; an OTLP exporter
decodes it to OTAP before encoding OTLP. There is no syslog-batch exporter in
this series, and no Parquet codec or encoded Parquet pass-through.

## Optional configuration

No `codecs` section is required. The engine discovers linked implementations,
uses their default settings, and creates mutable instances only when needed.
An override is not an allowlist: other registered encodings remain available.
There is no global `defaults` section and no node-level codec configuration.

A **binding** selects one configuration per encoding per pipeline. An
**implementation** identifies registered code, independently of the bytes it
supports. Built-in IDs are `otel.otlp` and `otel.syslog_cef`. The latter is
available only when the syslog receiver is linked.

Configure just OTLP, leaving other encodings unchanged:

```yaml
codecs:
  bindings:
    otlp-bytes:
      decode:
        validation: strict
```

An inline binding may also specify `implementation` and implementation-owned
`options`. Omitting the implementation uses the distribution default for that
encoding. Built-in implementations currently accept no custom `options`.
A distribution with multiple implementations for an encoding must declare a
unique default or the user must select one explicitly. Link order never selects
an implementation. Duplicate implementation IDs and conflicting defaults fail.

### Named configurations

Names let pipelines reuse settings without sharing mutable state:

```yaml
codecs:
  configurations:
    checked_otlp:
      implementation: otel.otlp
      decode:
        validation: strict
  bindings:
    otlp-bytes:
      configuration: checked_otlp
```

Named definitions require an implementation ID. Declaring a name does not select
it, although its settings are validated during startup. A binding accepts either
a reference or inline settings, never both. Native instances remain lazy;
preparation can own immutable implementation resources. Different pipelines
using the same name have independent mutable codec state. Multiple selected
configurations for one encoding within a pipeline are not supported.

### Scope and inheritance

Bindings and named definitions inherit by key, from pipeline, group, then root.
A nearer entry replaces the entire entry; codec-specific options do not
recursively merge. Missing fields in a replacement use implementation defaults.
The observability pipeline inherits from `engine.observability.pipeline.codecs`
and root, without group inheritance.

`codecs: {}` inherits everything. By contrast, `bindings: {otlp-bytes: {}}`
explicitly replaces that binding with the distribution default and its default
settings. Unspecified encoding bindings are unchanged.

This complete example uses strict OTLP decoding by default, and resets it for
one pipeline:

```yaml
version: otel_dataflow/v1
codecs:
  bindings:
    otlp-bytes:
      decode:
        validation: strict
groups:
  ingest:
    pipelines:
      logs:
        codecs:
          bindings:
            otlp-bytes: {}
        nodes:
          input:
            type: urn:otel:receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
          output:
            type: urn:otel:exporter:console
            config: {}
        connections:
          - from: input
            to: output
```

Unknown names, implementations, encodings, options, unsupported decode settings,
and encoding/implementation mismatches fail startup. No
`policies.pdata.decode_validation` or `codecs.defaults` alias is provided.
Changes to selected settings redeploy the pipeline. Moving equivalent settings
between scopes or between inline and named forms does not; changing an unused
named definition does not change the runtime selection.

### Decode validation

| Value | Behavior when decoding |
| --- | --- |
| `best_effort` (built-in default) | Allows a faster parser that may miss malformed nested content and omit that content from the output. |
| `strict` | Requires validation of the complete encoded batch before returning OTAP records. |

For OTLP, best effort uses borrowed protobuf views and checks outer framing.
Strict mode uses Prost to decode the complete nested protobuf message before
Arrow conversion, adding CPU and allocation costs. Both modes can reject
malformed input. Strict validation concerns format validity, not every
semantically questionable telemetry value. The syslog codec validates framing
and uses the same syslog/CEF parser in both modes.

### When validation happens

Validation is performed only when decoding into OTAP is required:

- Admission and unchanged encoded forwarding do not decode or validate content.
- A directly borrowed encoded view is not validated by this setting.
- A view that falls back to native OTAP uses the selected decoder.
- Codec-native batching and item counting have their own format checks; this
  setting does not turn them into strict decoding.
- Already-native OTAP records do not run through a decoder.

Consequently, `strict` is not an ingress validation barrier. For example, a
matching OTLP-to-OTLP relay may never invoke the strict decoder. Consumers
remain responsible for handling malformed borrowed content and decode errors.
Recoverable materialization preserves the original input so nodes can issue
permanent Nacks where their delivery contract supports them.

## Future hosted codecs

This configuration selects registered implementations, not module files. WASM
loading, a guest interface, execution budgets and async offloading are not
implemented. The startup preparation interface can later own immutable compiled
components and create isolated pipeline-local guest instances. Future module
loading must be explicit; the engine does not scan directories or download code.

## What belongs elsewhere

Exporters select the output encoding. Receivers and exporters own transport
settings, including HTTP/gRPC compression, endpoints, and credentials. Batch
processors own target sizes, split budgets, and flush timing. The `codecs`
section does not select transports, register implementations, or change
encoding identities.

Codec state remains lazy and local to each pipeline runtime. Configuration is
resolved before processing; it does not add a configuration lookup per record.
Ordinary codec operations execute synchronously. Larger syslog inputs use
owned incremental decode jobs and yield between bounded work segments.
Configured factories support either path without holding codec-service access
across suspension. Encoding and Arrow finalization can still run synchronously;
async effect-handler methods do not imply codec offloading.
