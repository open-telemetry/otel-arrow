# Syslog Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:syslog` (`urn:otel:exporter:syslog`)
- Feature gate: `syslog`
- Stability: Experimental

## Overview

The exporter forwards encoded `syslog-batch` log payloads over TCP or UDP. It
validates the complete batch with the syslog codec, extracts each original
syslog or CEF message, and adds only transport framing. The internal batch
envelope is never sent. Native OTAP/OTLP logs and other encodings receive a
permanent NACK until a syslog codec encoder is available.

## Configuration

TCP with newline framing (the default):

```yaml
type: exporter:syslog
config:
  protocol:
    tcp:
      endpoint: "127.0.0.1:514"
      framing: newline
```

Set `framing: octet_counting` for RFC 6587 length-prefixed TCP messages. For
UDP, use:

```yaml
type: exporter:syslog
config:
  protocol:
    udp:
      endpoint: "127.0.0.1:514"
```

Exactly one transport and a numeric IP address and port are required. TLS is
not part of this phase.

## Send semantics

The exporter sends messages sequentially with one pipeline-local connection
and no extra queue, so slow sends backpressure the upstream channel. Network
operations time out after five seconds, and a whole batch after 30 seconds.
TCP connects on the first batch and
reconnects after a failed write. UDP sends one message per datagram.

The exporter rejects empty messages. Newline framing rejects messages that
contain LF or exceed the receiver's 16 KiB frame limit including the delimiter.
UDP rejects messages that reach the receiver's 16 KiB receive buffer limit.
Octet-counted TCP accepts messages up to 16 KiB. These checks happen for the
entire batch before the first send.

Successful writes count as local transport acceptance, not receiver processing.
The exporter ACKs a batch only after all its messages are written. On a send
failure, it NACKs the batch; messages written earlier in that batch may be
duplicated if an upstream retry resends it. TCP has no per-message delivery
confirmation, and UDP may drop datagrams after a successful local send.

`exporter.syslog.sent_messages` and `sent_bytes` count locally accepted original
messages and their payload bytes. `failed_messages` and `failed_bytes` count
the failed message and the unsent remainder of a batch after a transport error.
`rejected_batches` counts unsupported, malformed, or incompatible input batches.
