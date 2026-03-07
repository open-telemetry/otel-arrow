# Admin Docs

This section documents the admin surface of the OTAP Dataflow Engine:

- runtime endpoints used for health, status, and telemetry;
- embedded browser UI behavior and architecture.

## Document map

- [Admin UI Architecture](architecture.md)
- [Crate README (admin endpoints and crate layout)](../../crates/admin/README.md)

## Quick start

Assuming the engine is running with admin HTTP enabled:

- Open UI: `http://<admin-host>:<admin-port>/`
- Metrics JSON: `http://<admin-host>:<admin-port>/telemetry/metrics?format=json`
- Prometheus output: `http://<admin-host>:<admin-port>/metrics`

For architecture details (state model, derivation rules, graph rules, testing),
start with [Admin UI Architecture](architecture.md).
