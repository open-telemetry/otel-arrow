# Syslog Parsing Behavior

The syslog/CEF codec accepts RFC 3164, RFC 5424, bare CEF, and syslog-wrapped
CEF. See the [receiver guide](../../../../core-nodes/src/receivers/syslog_cef_receiver/README.md)
for configuration and [encoding behavior](syslog-encoding-behavior.md)
for output constraints.

## Detection and fallback

```text
Input -> CEF -> RFC 5424 -> RFC 3164 fallback
```

Empty input is rejected. Any nonempty input is accepted, falling back to
RFC 3164 when CEF or RFC 5424 is malformed. Missing or invalid priority
produces a partial parse and retains the original input in the log body.
Fully parsed messages are stored as attributes rather than the body.

Valid priority ranges from `<0>` to `<191>`; leading zeros are invalid.
RFC 5424 additionally requires `<PRI>VERSION` followed by a space.

## Fields and edge cases

RFC 5424 accepts `-` for absent header fields and preserves structured data.
Multiple structured-data elements are supported; unclosed data consumes the
remaining input without rejecting the message. One leading UTF-8 BOM is
removed from message text.

RFC 3164 accepts missing timestamps and lenient tags. Numeric bracketed
process IDs are extracted; other bracket contents remain in the tag.
Without recognizable header fields, the remaining text becomes content.

CEF versions `0` and `1` are accepted. Headers may be empty, and extensions
may be absent. CEF headers unescape `\|` and `\\`; extension values unescape
`\\`, `\=`, `\n`, and `\r`. Unknown escapes and trailing backslashes remain
literal. Extension attributes use bare wire keys and preserve repeated
occurrences.

## Syslog severity mapping

| Syslog severity | OTel number | OTel text |
| --- | --- | --- |
| 0 (Emergency) | 21 | `FATAL` |
| 1 (Alert) | 19 | `ERROR3` |
| 2 (Critical) | 18 | `ERROR2` |
| 3 (Error) | 17 | `ERROR` |
| 4 (Warning) | 13 | `WARN` |
| 5 (Notice) | 10 | `INFO2` |
| 6 (Informational) | 9 | `INFO` |
| 7 (Debug) | 5 | `DEBUG` |
