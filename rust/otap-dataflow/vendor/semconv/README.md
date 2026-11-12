# Vendored OpenTelemetry semantic conventions

This directory holds a copy of the `model/` directory from the
[OpenTelemetry semantic conventions](https://github.com/open-telemetry/semantic-conventions)
repository, at the tag recorded in [VERSION](VERSION). The upstream license is
in [LICENSE](LICENSE).

The copy is unmodified except for added `traffic-generator` annotations. Every
attribute definition (including deprecated ones) and every span, event, and
metric group with an `instrument` carries one, with values, ranges, and
distributions derived from the definition's brief, note, examples, enum
members, unit, and instrument. Attribute references (`ref:`), attribute
groups, and entity groups are not annotated.

A representative set, one per supported annotation kind, lives in these files
and is pinned by a `dev-nodes` test that fails if any of them goes missing:

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
Every annotation is validated when the registry is compiled, so an invalid
annotation fails the `dev-nodes` annotation tests.

The `dev-nodes` annotation tests load this copy from disk. Other users opt in
by setting the `traffic_generator` receiver's `registry_path` to this folder
(`data_source: semantic_conventions`); the default `registry_path` remains the
upstream git repository.

Don't edit these files by hand, other than the `traffic-generator`
annotations.
