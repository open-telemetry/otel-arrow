# Internal logging handlers

This module contains a simple encoder and formatter for use with Tokio
tracing subscribers to enable a lightweight bridge into the
OTAP-Dataflow engine.

## OTLP bytes first

This module currently implements encoding support for OTLP bytes, in
two forms:

- Partial: The `LogRecord` type encodes the dynamic arguments from the event
  along with a timestamp, yielding a representation that can be passed into
  an internal pipeline because it is already encoded as bytes. This representation
  allows sorting and filtering records before encoding full OTLP messages.
- Full: The `DirectLogRecordEncoder` type supports appending the OTLP bytes
  representation for the complete LogRecord (without Scope and Resource wrappers).

## Raw logging handler

This package supports a `RawLoggingLayer` tracing subscriber that
prints colorized or uncolorized messages on the console. The
formatting code path in this module is safe for logging in critical
regions and to be used as a fallback inside other logging handlers.
It uses no synchronization and depends only on the console.

Presently, the raw logging code path allocates memory, however this is
not the desired state. [See the issue with potential
improvements.](https://github.com/open-telemetry/otel-arrow/issues/1746)
