# Admin Interface

`otap-df-admin` provides:

- admin, health, status, and telemetry HTTP endpoints;
- an embedded single-page UI served from the same process and origin.

For architecture and runtime behavior details, see
[`docs/admin/architecture.md`](../../docs/admin/architecture.md).
For the admin docs landing page, see
[`docs/admin/README.md`](../../docs/admin/README.md).

## Main routes

### Embedded UI

- `GET /`: UI entry point.
- `GET /dashboard`: alias to `/`.
- `GET /static/*`: embedded UI assets (HTML/CSS/JS/vendor files).

### Telemetry

- `GET /telemetry/live-schema`
- `GET /telemetry/metrics`
- `GET /telemetry/metrics/aggregate`
- `GET /metrics` (alias for `/telemetry/metrics`)

### Health and status

- `GET /status`
- `GET /livez`
- `GET /readyz`
- `GET /pipeline-groups/status`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/status`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/livez`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/readyz`
- `POST /pipeline-groups/shutdown`

## Embedded UI layout (crate-relative)

- `ui/index.html`: UI shell and page structure.
- `ui/css/ui.css`, `ui/styles.css`: styles.
- `ui/js/main.js`: orchestration, polling, rendering, interactions.
- `ui/js/metrics-api.js`: endpoint candidate strategy and fetch fallback.
- `ui/js/pipeline-utils.js`: attribute normalization and pipeline/core selectors.
- `ui/js/engine-metrics.js`: engine card aggregation logic.
- `ui/js/control-utils.js`, `ui/js/inter-pipeline-topology.js`: shared UI helpers.
- `ui/js/dom-safety.js`: escaping helpers for HTML, attributes, and selector values.
- `ui/vendor/*`: vendored browser runtime dependencies.

Assets are embedded in the binary via `include_dir` and served by
`src/dashboard.rs`.

## Runtime notes

- UI polling cadence is 2 seconds.
- UI metrics polling uses same-origin `/telemetry/metrics` then `/metrics`.
- Supported optional query param: `keep_all_zeroes=true|false`.

For operational semantics, metric-name contracts, graph rules, and testing
guidance, see [`docs/admin/architecture.md`](../../docs/admin/architecture.md).
