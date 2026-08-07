<!-- markdownlint-disable MD013 -->

# ETW Receiver

## Metadata

- Type: `receiver:etw` (`urn:otel:receiver:etw`)
- Feature gate: `etw-receiver`
- Stability: experimental
- Platform: Windows only (compiled under `#[cfg(target_os = "windows")]`)

## Overview

This receiver subscribes to Windows
[Event Tracing for Windows (ETW)](https://learn.microsoft.com/windows/win32/etw/about-event-tracing)
providers, decodes the emitted events, and converts them into OTAP logs for
downstream processing.

It is compiled only on Windows; on other platforms the node type is not
registered.

### Multi-core fan-out

Windows allows only one real-time ETW session per session name, while the engine
may instantiate the receiver once per allocated core. To reconcile these two
models, a single OS trace session is created per `session_name` with one
consumer channel per core. The `ProcessTrace` callback round-robins events
across the channels so each core receives an even share of the event stream.

## Getting Started

Subscribe to at least one ETW provider (identified by `name` or `guid`):

```yaml
type: receiver:etw
config:
  providers:
    - name: "Microsoft-Windows-Kernel-Process"
      level: information
  session_name: "OtelArrowETW"
  batching:
    max_size: 512
    max_duration: 100ms
```

## Configuration

- `providers` (required): one or more ETW providers to trace. Each provider
  accepts:
  - `name` or `guid` (required): the ETW provider name (e.g.
    `"Microsoft-Windows-Kernel-Process"`) or GUID. Exactly one of the two must
    be set - specifying both, or neither, is rejected.
  - `level` (optional): trace level filter, one of `critical`, `error`,
    `warning`, `information`, or `verbose`. Defaults to `information`.
  - `keywords` (optional): a keyword bitmask that further filters events. When
    omitted, all keywords are matched.
- `session_name` (optional): name of the ETW trace session. Defaults to
  `"OtelArrowETW"`. Because Windows permits only one real-time session per
  name, distinct receivers that must run concurrently need distinct names.
- `batching` (optional): OTAP log batching limits.
  - `max_size` (optional): maximum number of log records per emitted batch.
    Must be greater than zero. Defaults to `512`.
  - `max_duration` (optional): maximum time to hold a non-empty batch before
    flushing it downstream. Defaults to `100ms`.

At least one provider must be configured, and `batching.max_duration`, when
set, must be greater than zero.

## Related Docs

- Catalog: [`contrib-nodes` README](../../../README.md)
- Node stability levels: [`docs/node-stability.md`](../../../../../docs/node-stability.md)
