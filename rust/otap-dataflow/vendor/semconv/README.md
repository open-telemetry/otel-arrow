# Vendored OpenTelemetry semantic conventions

This directory holds an unmodified copy of the `model/` directory from the
[OpenTelemetry semantic conventions](https://github.com/open-telemetry/semantic-conventions)
repository, at the tag recorded in [VERSION](VERSION). The upstream license is
in [LICENSE](LICENSE).

Tests and benchmarks that generate semantic-convention traffic with the
`traffic_generator` receiver (`data_source: semantic_conventions`) load this
copy, so they don't need network access or a TLS crypto provider to clone
the upstream registry. The receiver's built-in default `registry_path` still
points at the upstream git repository. A unit test in the `dev-nodes` crate
checks that its pinned tag matches [VERSION](VERSION).

Don't edit these files by hand. To refresh or bump the vendored copy, run this
from `rust/otap-dataflow` (it needs `git` and network access):

```sh
# Re-sync the tag recorded in VERSION
cargo xtask update-semconv

# Vendor a different upstream tag
cargo xtask update-semconv --version v1.45.0
```

When you bump the version, also update `SEMCONV_REGISTRY_REFSPEC` in
`crates/dev-nodes/src/receivers/traffic_generator/config.rs`.
