# Admin Interface

`otap-df-admin` exposes:

- Admin and telemetry HTTP endpoints.
- An embedded single-page UI served by the same process and origin.

For browser runtime details, see [`docs/admin-ui-architecture.md`](../../docs/admin-ui-architecture.md).

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
- `ui/vendor/*`: vendored browser runtime dependencies.

Assets are embedded in the binary via `include_dir` and served by
`src/dashboard.rs`.

## UI runtime notes

- Poll interval is 2 seconds (`POLL_INTERVAL_MS`).
- Optional query param:
  - `keep_all_zeroes` (default `true`)
- Endpoint candidate order is same-origin only:
  - `/telemetry/metrics?...`
  - `/metrics?...`
- Current polling query: `format=json&reset=true&keep_all_zeroes=<...>`.
- When accessed directly without browser relative paths, use:
  - `http://<admin-host>:<admin-port>/telemetry/metrics`
  - `http://<admin-host>:<admin-port>/metrics`

### Operational notes

- `reset=true` is intentional: rate-oriented UI views are derived from successive
  snapshots and avoid long-lived counter accumulation in the browser view.
- `keep_all_zeroes=true` keeps zero-valued metric sets in snapshots, which can
  improve topology/selector stability in sparse or bursty traffic windows.
- Engine cards can hold prior CPU/RSS values when current snapshots omit/zero
  those fields, reducing visible flicker.

## Project principles

- Engine-embedded debugging UI first.
- In-memory sliding window only (no persistence layer).
- Signal-driven source of truth.
- Runtime topology/state inferred from telemetry, not static config files.
- Keep visual/system behavior deterministic and easy to reason about.
- Keep dependencies and complexity proportional to debugging value.

## Metric-name dependencies

These names are referenced explicitly by the UI and should be treated as
UI-facing contracts.

- `engine.metrics`: `cpu.utilization`, `memory.rss`
- `pipeline.metrics`: `cpu.utilization`, `memory.usage`, `uptime`, `cpu.time`,
  `memory.allocated.delta`, `memory.freed.delta`,
  `context.switches.voluntary`, `context.switches.involuntary`,
  `page.faults.minor`, `page.faults.major`
- `tokio.runtime`: `worker.count`, `task.active.count`,
  `global.task.queue.size`, `worker.busy.time`, `worker.park.count`,
  `worker.park.unpark.count`
- `channel.sender`: `send.count`, `send.error_full`, `send.error_closed`,
  `capacity`
- `channel.receiver`: `recv.count`, `recv.error_empty`, `recv.error_closed`,
  `capacity`, `queue.depth`
- `topic.exporter.metrics`, `topic.receiver.metrics` are used for
  inter-pipeline topology derivation.

UI-derived series keys (not emitted by engine metric sets directly):

- `engine.cpu.utilization`
- `engine.memory.rss`
- `cpu.time.rate`
- `memory.alloc.rate`
- `memory.free.rate`
- `memory.net.rate`

## Development guardrails

- Keep single-origin behavior (`/`, `/static/*`, `/telemetry/*`) intact.
- Prefer small JS modules over growing one monolithic file.
- Preserve deterministic selector/filter behavior.
- Keep topology rendering readable before adding density/features.
- When adding metrics, wire extraction, rendering, and validation end-to-end.
- Avoid changing metric-name contracts without updating docs and UI parsing.

## How to test

Minimal checks:

```bash
cargo check -p otap-df-admin
```

Optional JS syntax checks:

```bash
node --check crates/admin/ui/js/main.js \
  crates/admin/ui/js/metrics-api.js \
  crates/admin/ui/js/pipeline-utils.js \
  crates/admin/ui/js/engine-metrics.js \
  crates/admin/ui/js/inter-pipeline-topology.js \
  crates/admin/ui/js/control-utils.js
```
