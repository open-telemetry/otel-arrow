# Pipeline data

The pipeline data object is an enriched library of code based on
underlying objects used in transport.

For OTLP, the underlying object is a protobuf message defined in the
[opentelemetry-proto](https://github.com/open-telemetry/opentelemetry-proto)
repository.

- **[OTLP pipeline data interface](./otlp/README.md)**

For OTAP data frames, the underlying object is a set of Arrow arrays
corresponding with OpenTelemetry concepts such as Resource, Scope,
LogRecord, Span, etc.

For OTAP streams, the associated protobuf messages are [defined in
this repository](../../../../proto/README.md).

- **[OTAP pipeline data interface](./otap/README.md)**
