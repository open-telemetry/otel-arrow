# Vendored OpenTelemetry semantic conventions

This directory holds a copy of the `model/` directory from the
[OpenTelemetry semantic conventions](https://github.com/open-telemetry/semantic-conventions)
repository, at the tag recorded in [VERSION](VERSION). The upstream license is
in [LICENSE](LICENSE).

The copy is unmodified except for a small set of representative
`traffic-generator` annotations, one per supported annotation kind, in these
files:

- `model/azure/registry.yaml`
- `model/db/registry.yaml`
- `model/db/spans.yaml`
- `model/feature-flags/events.yaml`
- `model/feature-flags/registry.yaml`
- `model/jvm/metrics.yaml`
- `model/jvm/registry.yaml`

The `traffic_generator` receiver reads these annotations to choose value
ranges and distributions (see the
[receiver README](../../crates/dev-nodes/src/receivers/traffic_generator/README.md#registry)).
A `dev-nodes` test fails if any representative annotation goes missing.

The `dev-nodes` annotation tests load this copy from disk. Other users opt in
by setting the `traffic_generator` receiver's `registry_path` to this folder
(`data_source: semantic_conventions`); the default `registry_path` remains the
upstream git repository.

Don't edit these files by hand, other than the `traffic-generator`
annotations.
