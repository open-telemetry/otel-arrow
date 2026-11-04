# Vendored OpenTelemetry semantic conventions

This directory holds a copy of the `model/` directory from the
[OpenTelemetry semantic conventions](https://github.com/open-telemetry/semantic-conventions)
repository, at the tag recorded in [VERSION](VERSION). The upstream license is
in [LICENSE](LICENSE).

The copy is unmodified except for `traffic-generator` annotations added to
attributes, metrics, spans, and events. The `traffic_generator` receiver reads
these annotations to choose value ranges and distributions (see the
[receiver README](../../crates/dev-nodes/src/receivers/traffic_generator/README.md#annotations)).

This copy is the receiver's default registry when `registry_path` is not set
(`data_source: semantic_conventions`). Tests and benchmarks that generate
semantic-convention traffic load it from disk, so they don't need network
access or a TLS crypto provider.

Don't edit these files by hand, other than the `traffic-generator`
annotations.
