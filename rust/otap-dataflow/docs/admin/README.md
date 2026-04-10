# Admin Docs

This section documents the admin surface of the OTAP Dataflow Engine:

- runtime endpoints used for health, status, and telemetry;
- live pipeline reconfiguration and shutdown operations;
- embedded browser UI behavior and architecture.
- the public Rust admin SDK.

## Document map

- [Admin UI Architecture](architecture.md)
- [df_enginectl CLI](enginectl.md)
- [Live Pipeline Reconfiguration](live-reconfiguration.md)
- [Crate README (admin endpoints and crate layout)](../../crates/admin/README.md)
- [Public Rust SDK README](../../crates/admin-api/README.md)

## Quick start

Assuming the engine is running with admin HTTP enabled:

- Open UI: `http://<admin-host>:<admin-port>/`
- Metrics JSON: `http://<admin-host>:<admin-port>/api/v1/telemetry/metrics?format=json`
- Prometheus output: `http://<admin-host>:<admin-port>/api/v1/metrics`

For Rust consumers, prefer the `otap-df-admin-api` crate rather than building
raw HTTP requests directly.

For architecture details (state model, derivation rules, graph rules, testing),
start with [Admin UI Architecture](architecture.md).

For the live mutation API used to create, replace, resize, and shut down
logical pipelines, see [Live Pipeline Reconfiguration](live-reconfiguration.md).

## UI module tests

Prerequisite:

- Node.js available on `PATH` (or set `NODE_BIN=/path/to/node`)

Start all UI JS module tests from repository root:

```bash
./scripts/run-ui-js-tests.sh
```

Run a single test file (or glob):

```bash
./scripts/run-ui-js-tests.sh crates/admin/ui/js/tests/graph-renderer.test.mjs
```

The script currently runs:

- `node --test --experimental-default-type=module`
- all `crates/admin/ui/js/tests/*.test.mjs` files by default
