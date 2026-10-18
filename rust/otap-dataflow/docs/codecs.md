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
| `syslog-cef-batch-v1` | Logs | Yes | No | Framed message items |

The syslog/CEF receiver creates independently decodable framed batches. This
internal representation is not raw syslog TCP/UDP framing. A syslog batch can
remain encoded through transparent nodes and native batching; an OTLP exporter
decodes it to OTAP before encoding OTLP. There is no syslog-batch exporter in
this series, and no Parquet codec or encoded Parquet pass-through.

## Configure decoding

Use a `codecs` section alongside `policies`, not inside it:

```yaml
codecs:
  defaults:
    decode:
      validation: best_effort
```

`validation` accepts these values:

| Value | Behavior when decoding |
| --- | --- |
| `best_effort` (default) | Allows a faster parser that may miss malformed nested content and omit that content from the output. |
| `strict` | Requires validation of the complete encoded batch before returning OTAP records. |

For OTLP, best effort uses borrowed protobuf views and checks outer framing.
Strict mode currently uses Prost to decode the complete nested protobuf message
before Arrow conversion, which adds CPU and allocation costs. Both modes can
reject malformed input. Strict validation describes format validity, not a
promise to reject every semantically questionable telemetry value.

A codec may use its strict decoder for both modes. The syslog codec validates
its framing and uses the same syslog/CEF parser in both modes.

### Scope and inheritance

The nearest explicit validation setting wins:

- Regular pipelines: pipeline, then group, then top-level `codecs`.
- Observability pipeline: `engine.observability.pipeline.codecs`, then
  top-level `codecs`.
- If no scope specifies validation, the value is `best_effort`.

For example, this configuration uses strict decoding globally and best effort
for one pipeline:

```yaml
version: otel_dataflow/v1
codecs:
  defaults:
    decode:
      validation: strict
groups:
  ingest:
    pipelines:
      logs:
        codecs:
          defaults:
            decode:
              validation: best_effort
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

Empty sections such as `codecs: {}` or `defaults: { decode: {} }` inherit their
parent; they do not reset it. Set `validation: best_effort` explicitly to
override inherited strict decoding. Defaults are not an enforced minimum.

Codec declarations are not supported at node scope. Unknown settings, modes,
and unsupported per-encoding overrides are rejected. No
`policies.pdata.decode_validation` alias is provided. Changing an effective
codec setting redeploys the affected pipeline; moving the same effective
setting between scopes does not.

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

## What belongs elsewhere

Exporters select the output encoding. Receivers and exporters own transport
settings, including HTTP/gRPC compression, endpoints, and credentials. Batch
processors own target sizes, split budgets, and flush timing. The `codecs`
section does not select transports, register implementations, or change
encoding identities.

Codec state remains lazy and local to each pipeline runtime. Configuration is
resolved before processing; it does not add a configuration lookup per record.
Codec execution is currently synchronous. Async effect-handler methods and
owned output for asynchronous transport do not imply codec offloading.
