# Internal logging handlers

This module contains a simple encoder and formatter for use with Tokio
tracing subscribers to enable a lightweight bridge into the
OTAP-Dataflow engine.

## OTLP bytes first

This module currently implements encoding support for OTLP bytes, in
two forms:

- Partial: The `LogRecord` type encodes the dynamic arguments from the event
  along with a timestamp, yielding a representation that can be passed into
  an internal pipeline because it is already encoded nas bytes. This representation
  allows sorting and filtering records before encoding full OTLP messages.
- Full: The `DirectLogRecordEncoder` type supports appending the OTLP bytes
  representation for the complete LogRecrd (without Scope and Resource wrappers).

## Internal logging configuration

Internal logging is the default configuration. In this configuration,
messages are written to the console. Note this can impact performance
due to contention over the console itself, however this configuration
does not use any `Sync + Send` synchronization.

```yaml
service:
  telemetry:
    logs:
      level: "debug"
      internal:
        enabled: true
```

The default configuration is subject to change.  In the future, the `internal` 
configuration block will be extended to route internal logs through dedicated
internal OTAP dataflow pipelines.
