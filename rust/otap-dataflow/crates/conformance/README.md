# OTAP Dataflow conformance driver

From the Rust workspace, run `cargo xtask conformance` to check a candidate
engine, or add `--component FULL_URN` to exercise a single linked component.
Xtask builds this binary, evaluates the resulting observations and produces
the report. Both use the [conformance model](model/README.md) to read the
contract catalog and exchange deployment observations.

To build the driver on its own with the local suite's component registrations,
run `cargo build -p otel-arrow-dfe-conformance --bin conformance --features offline-fixtures`.

The [execution plan](src/plan.rs) selects scenarios and
[live component fixtures](fixtures/fixtures.yaml). The
[release fixtures](fixtures/releases/) preserve historical configurations for
validation with the candidate engine. The driver uses the validation library's
[engine session](../validation/src/deployment/mod.rs) to deploy the
engine and observe it through its admin API. The [conformance
guide](../../docs/conformance/README.md#how-the-checker-works) traces an
operation from its catalog entry to the report.
