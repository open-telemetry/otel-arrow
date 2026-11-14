# otel-arrow-dfe-policy-engine

Telemetry policy evaluation engine for OTAP Dataflow.

This crate compiles telemetry policies (as defined by the
[Tero policy specification](https://github.com/usetero/policy/blob/master/spec.md))
into query-engine pipeline stages that can be executed against OTAP Arrow
record batches.
