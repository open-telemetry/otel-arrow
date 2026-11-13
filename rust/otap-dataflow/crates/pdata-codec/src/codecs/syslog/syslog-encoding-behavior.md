# Syslog and CEF Encoding Behavior

This reference describes how native OTAP logs become syslog or CEF messages.
For accepted input formats and decoding behavior, see
[Syslog Parsing Behavior](syslog-parsing-behavior.md). Encoding is deliberately
more restrictive than parsing: it rejects unsupported selected values rather
than guessing, truncating, or emitting a partially representable message.

## Output profiles

Set `codecs.bindings.syslog-batch.options.profile`:

| Profile | Output |
| --- | --- |
| `rfc3164` | RFC 3164 syslog |
| `rfc5424` | RFC 5424 syslog; default when no profile is configured |
| `cef` | CEF without a syslog wrapper |
| `cef_rfc3164` | CEF inside RFC 3164 syslog |
| `cef_rfc5424` | CEF inside RFC 5424 syslog |
| `all` | Test-oriented rotation through the five profiles above, per record |

An explicit profile forces re-encoding of already encoded syslog batches.
Without an explicit profile, matching encoded batches may pass through
unchanged. Rotation continues across successful batches and does not advance
after a failed batch.

Output uses the internal `syslog-batch` envelope, not raw TCP or UDP framing.
No syslog exporter is included. A consuming transport must extract the messages
and apply its transport framing. See the [codec user guide](../../../../../docs/codecs.md)
and [registry examples](registry/README.md).

## Message selection

Selection does not depend on attribute order:

```text
Nonempty string body?
  yes -> use body
  no  -> choose the first present attribute for the output profile:
           RFC 3164: syslog.content, then syslog.message
           RFC 5424: syslog.message, then syslog.content
           CEF:      msg (including both syslog-wrapped profiles)
```

An absent body, an empty AnyValue, or an empty string body permits fallback.
Other body types are rejected; integers are not automatically converted to
message text. If no fallback is present, an empty string body stays empty and
an absent body stays absent.

Only the selected message fallback is validated. Syslog message attributes must
be strings; CEF `msg` may be a string or integer. Invalid unused fallbacks do
not reject an otherwise valid message. A selected invalid fallback is an
error, not a reason to try the next attribute.

For example, after a processor summarizes a message:

```text
body:           "Request failed; details removed"
syslog.message: <20 KiB of original text>

RFC 5424 message text: "Request failed; details removed"
```

The oversized attribute is not emitted or validated. If the body is removed
or becomes empty, that same selected attribute causes encoding to fail.
Unrelated headers and emitted CEF extensions still undergo validation.

## Syslog headers and defaults

These rules apply to RFC 3164, RFC 5424, and their CEF-wrapped profiles.
Bare CEF ignores syslog header attributes, resource header defaults, and OTel
`severity_number`. Its severity comes from `cef.severity` instead.

| Output field | Selection, highest priority first |
| --- | --- |
| Hostname | Log `syslog.host_name`, resource `host.name`, `-` |
| Application | Log `syslog.app_name`, resource `service.name`, `-` |
| Process ID | Log `syslog.process_id_str`, log `syslog.process_id`, resource `process.pid`, `-` |
| Message ID | Log `syslog.msg_id`, `-`; emitted only by RFC 5424 |
| Facility | Log `syslog.facility`, otherwise `1` |
| Severity | Log `syslog.severity`, otherwise mapped from `severity_number` |

Resource defaults and process ID candidates are validated only if selected.
For example, a valid log hostname can override an invalid resource hostname.
String process IDs take precedence over numeric IDs regardless of attribute
order. RFC 5424 preserves opaque strings such as `00123` and `-42`.
Numeric process IDs must be nonnegative integers.

Selected hostname, application, string process ID, and message ID values must
be nonempty printable non-space ASCII, with maximum lengths of 255, 48, 128,
and 32 bytes respectively. RFC 3164 additionally limits the application to
32 bytes, forbids `:` in the hostname and `:`, `[` or `]` in the application,
and permits only digit strings or `-` as string process IDs.

Facility must be an integer in `0..=23`; explicit syslog severity must be an
integer in `0..=7`. Otherwise, OTel severity is mapped as follows:

| OTel `severity_number` | Syslog severity |
| --- | --- |
| Unset, `0`, or `9..=12` | `6` (informational) |
| `1..=8` | `7` (debug) |
| `13..=16` | `4` (warning) |
| `17..=20` | `3` (error) |
| `21..=24` | `2` (critical) |

Other OTel severity numbers are rejected when this mapping is needed.
`syslog.structured_data` must be absent or `-`; non-NIL structured data is
unsupported. Validating message precedence does not suppress these errors.

## CEF fields and escaping

All CEF profiles require `cef.device_vendor`, `cef.device_product`,
`cef.device_version`, `cef.device_event_class_id`, `cef.name`, and
`cef.severity`. Header values must be strings or integers, nonempty when
rendered, and contain no CR or LF. Severity must be `0..=10` or one of
`Unknown`, `Low`, `Medium`, `High`, or `Very-High`. Optional `cef.version`
must be integer `0` or `1`; the default is `0`.

Supported standard extensions are `src`, `dst`, `spt`, `dpt`, `proto`, `suser`,
`deviceExternalId`, `act`, and `msg`. Custom attributes use
`cef.extension.<wire-key>`; keys must match `[A-Za-z][A-Za-z0-9_]*` and must
not shadow a supported standard extension. Values must be strings or integers.
Custom extension order is unspecified.

CEF headers escape backslashes and pipes. Extension values escape backslashes,
equals signs, CR, and LF. Escaping can increase message size. Re-encoding
parsed CEF headers is not guaranteed to preserve the original bytes.

## Timestamps, limits, and omitted data

RFC 5424 emits UTC event timestamps with microsecond precision, or `-` when the
event timestamp is absent or zero. RFC 3164 requires a nonzero event timestamp,
uses the local timezone with second precision, and rejects ambiguous local
times. Its year is retained in the internal envelope because the wire format
does not carry a year. Bare CEF does not emit the event timestamp.

Each record's observed timestamp is retained in the envelope; missing observed
time becomes zero. Observed timestamps must fit a signed 64-bit nanosecond
value.

Batches must contain `1..=8192` logs. Each complete encoded message, including
headers and escaping, is limited to 16 KiB. The framed batch is limited to
2 MiB or the caller's smaller encoded-size budget. Violations fail the encode
operation without returning partial output.

Unmapped attributes, scope metadata, and trace context are not serialized.
Encoding is not a lossless OTLP conversion; successful parsing does not imply
that every parsed log can be re-encoded.
