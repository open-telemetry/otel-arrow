# Syslog and CEF Encoding Behavior

Native OTAP logs can be encoded as syslog or CEF. Unsupported selected values
fail explicitly, without truncation or partial output. See
[parsing behavior](syslog-parsing-behavior.md) for accepted input.

## Output profiles

Set `codecs.bindings.syslog-batch.options.profile`:

| Profile | Output |
| --- | --- |
| `rfc3164` | RFC 3164 syslog |
| `rfc5424` | RFC 5424 syslog; default |
| `cef` | Bare CEF |
| `cef_rfc3164` | CEF inside RFC 3164 |
| `cef_rfc5424` | CEF inside RFC 5424 |
| `all` | Test-oriented rotation through these five profiles |

Received CEF remains CEF when selecting a syslog profile; the profile chooses
its wrapper. An explicit profile forces re-encoding; otherwise, matching
encoded batches may pass through unchanged.

Output is a `syslog-batch` envelope, not transport framing. No exporter is
included. See the [codec guide](../../../../../docs/codecs.md) and
[registry examples](registry/README.md).

## Message and header selection

```text
Nonempty string body -> message
Otherwise:
  RFC 3164 -> syslog.content, then syslog.message
  RFC 5424 -> syslog.message, then syslog.content
  CEF      -> msg
```

The first present fallback wins regardless of attribute order. Only selected
values are validated; invalid unused defaults do not cause failure. Non-string
bodies are rejected, except empty AnyValues permit fallback. Syslog messages
must be strings; CEF `msg` also accepts integers.

| Syslog field | Selection, highest priority first |
| --- | --- |
| Hostname | `syslog.host_name`, resource `host.name`, `-` |
| Application | `syslog.app_name`, resource `service.name`, `-` |
| Process ID | `syslog.process_id_str`, `syslog.process_id`, resource `process.pid`, `-` |
| Message ID | `syslog.msg_id`, `-`; RFC 5424 only |
| Facility | `syslog.facility`, otherwise `1` |
| Severity | `syslog.severity`, otherwise mapped from OTel severity |

Header strings must be nonempty printable non-space ASCII. Maximum lengths
are 255 bytes for hostname, 48 for application, 128 for process ID, and 32 for
message ID. Numeric process IDs must be nonnegative. RFC 3164 limits application
names to 32 bytes, disallows `:` in hostnames and `:[]` in application names,
and requires digit-only string process IDs or `-`.

Facility and syslog severity must be integers in `0..=23` and `0..=7`,
respectively. OTel severity maps as:
unset/`0`/`9..=12` -> `6`, `1..=8` -> `7`, `13..=16` -> `4`,
`17..=20` -> `3`, `21..=24` -> `2`; other values fail.
Bare CEF ignores syslog headers and OTel severity.

## Preservation and CEF fields

RFC 3164 preserves the original `syslog.tag` when application and process ID
still match. RFC 5424 preserves `syslog.version` (`0..=255`),
`syslog.structured_data`, and message text, including empty values, leading
brackets, and literal BOMs. Other profiles reject non-NIL structured data.
Unclosed structured data cannot be combined with a message.

CEF requires `cef.device_vendor`, `cef.device_product`, `cef.device_version`,
`cef.device_event_class_id`, `cef.name`, and `cef.severity`. Generated headers
must be nonempty strings or integers without CR/LF. Severity must be `0..=10`
or `Unknown`, `Low`, `Medium`, `High`, or `Very-High`; optional `cef.version`
is `0` (default) or `1`. Received CEF also preserves empty headers and other
accepted severity strings.

Extensions use bare keys such as `src`, `msg`, `cs1`, and `vendorZone` for
both generated and received records. Keys must match `[A-Za-z0-9_-]+`;
values must be strings or integers. Repeated keys retain their values and
same-key order. A nonempty body replaces all `msg` occurrences.

Escaping preserves field values, not original wire spelling or ordering
between different keys. A trailing-space value must be last; multiple such
values or conflicting same-key order cause failure rather than whitespace loss.
Invalid extension keys and unsupported `cef.*` attributes fail.
Unrelated namespaced generated attributes, scope metadata, and trace context
are omitted; encoding is not a lossless OTLP conversion.

## Timestamps and limits

RFC 5424 uses UTC with microsecond precision, or `-` for missing or zero timestamps.
RFC 3164 requires a nonzero timestamp, uses local time with second precision,
and rejects ambiguous local times. It requires a discoverable IANA timezone
or named `TZ` override; recreate the codec after a timezone change.
Bare CEF emits no timestamp.

Batches contain `1..=8192` logs. Each complete message, including headers and
escaping, is limited to 16 KiB; the framed batch is limited to 2 MiB or the
caller's smaller size budget. Exceeding a limit fails the entire operation.
