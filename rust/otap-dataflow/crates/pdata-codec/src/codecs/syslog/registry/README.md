# Shared syslog and CEF event registry

An offline Weaver registry of structured events, independent of wire format:

```text
events.yaml -> trafficgen -> structured logs -> output codec -> syslog / CEF
```

| ID | Data shape |
| --- | --- |
| `syslog.common` | Shared syslog and required CEF headers |
| `syslog.application` | Application messages |
| `syslog.firewall` | Network addresses, ports, and actions |
| `syslog.vendor_firewall` | Firewall fields plus synthetic vendor extensions |

From `rust/otap-dataflow`, set
`registry_path: ./crates/pdata-codec/src/codecs/syslog/registry`.
Trafficgen cycles the three events per batch; use at least three logs to include
each. Message examples use parser-style attributes: `syslog.content` (RFC 3164),
`syslog.message` (RFC 5424), and `msg` (CEF). Custom CEF fields use
`cef.extension.<wire-key>`; the encoder strips the prefix and escapes values.

## Encoding

Set `codecs.bindings.syslog-batch.options.profile` to `rfc3164`, `rfc5424`
(default), `cef`, `cef_rfc3164`, or `cef_rfc5424`. Test-oriented `all` rotates
through that order per record, continuing across successful batches. An explicit
profile forces re-encoding instead of forwarding existing syslog bytes.

A nonempty string body overrides the message attributes above. CEF requires its
six `cef.*` header attributes. Unmapped fields are omitted, non-NIL structured
data is unsupported, and escaped CEF headers are not byte-preserving.

The [console sample](../../../../../../configs/trafficgen-syslog-semconv-console.yaml)
uses `all` to rotate the five formats per record across batches **when encoding
is requested**. Console only displays structured logs; no syslog exporter is
included. A consuming transport must remove the internal batch framing.
Registry files require a local path and are not embedded in the binary.

## Encoder benchmark

`cargo bench -p benchmarks --bench codec_encoders` uses this registry through
trafficgen's Weaver loader and log generator. Registry resolution and generation
run once before timing; fixed timestamps and deterministic example selection
give every codec the same inputs. Changing the registry changes the benchmark
workload, so earlier fixture results are not directly comparable.
