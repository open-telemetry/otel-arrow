# Publishing the data-engine crates

This workspace is released independently from `rust/otap-dataflow`. Its
`otel-arrow-contrib-data-engine-*` names reflect current repository
stewardship, not a dependency on OTAP or OpenTelemetry. See the workspace
[README](README.md#scope-and-repository-placement).

## Crates and order

Publish these crates in order:

1. `otel-arrow-contrib-data-engine-expressions`
2. `otel-arrow-contrib-data-engine-parser-abstractions`
3. `otel-arrow-contrib-data-engine-kql-parser`
4. `otel-arrow-contrib-data-engine-recordset`

`otel-arrow-contrib-data-engine-columnar` remains unpublished.

Internal dependencies use exact versions such as `=0.1.0` because these
experimental crates may make breaking changes in patch releases. Update the
workspace version and all exact pins together for later releases.

## Publish

Authenticate with `cargo login` or `CARGO_REGISTRY_TOKEN`, then run:

```bash
cd rust/contrib/data_engine

for crate in expressions parser-abstractions kql-parser engine-recordset; do
  (cd "$crate" && cargo package --list)
done

(cd expressions && cargo publish)
# Wait for expressions to appear on crates.io.
(cd parser-abstractions && cargo publish)
# Wait for parser-abstractions to appear on crates.io.
(cd kql-parser && cargo publish)
(cd engine-recordset && cargo publish)
```

Publish from the intended commit. Published versions cannot be replaced.

## Owners

After the first publish, add the maintainer team and named recovery owners:

```bash
for crate in \
  otel-arrow-contrib-data-engine-expressions \
  otel-arrow-contrib-data-engine-parser-abstractions \
  otel-arrow-contrib-data-engine-kql-parser \
  otel-arrow-contrib-data-engine-recordset
do
  cargo owner --add github:open-telemetry:arrow-maintainers "$crate"
  cargo owner --add drewrelmas "$crate"
  cargo owner --add lquerel "$crate"
  cargo owner --add jmacd "$crate"
  cargo owner --list "$crate"
done
```

Each named owner must have logged into crates.io before being added.
