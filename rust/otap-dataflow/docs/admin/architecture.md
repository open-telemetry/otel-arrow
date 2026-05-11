# Admin UI Architecture

## Scope

This document describes the embedded single-page admin UI served by the
`otap-df-admin` crate.

The UI continuously polls metrics snapshots and renders:

- engine cards,
- pipeline cards,
- Tokio runtime cards,
- time-series charts,
- channel DAG topology (data and optional control),
- selection details for nodes and channels.

Polling cadence is controlled by `POLL_INTERVAL_MS` (currently 2000 ms).

## Runtime overview

The UI and telemetry endpoints are served from one origin by the same admin
HTTP server.

Request flow:

1. `GET /` or `GET /dashboard` serves the embedded UI shell.
2. Browser loads static assets from `/static/*`.
3. UI polls `/api/v1/telemetry/metrics` (or `/api/v1/metrics` alias) with:
   - `format=json`
   - `reset=false`
   - optional `keep_all_zeroes=true|false`
4. Client logic derives deltas/rates/summaries and updates cards, charts, and
   topology.

When calling endpoints directly outside browser-relative paths, use:

- `http://<admin-host>:<admin-port>/api/v1/telemetry/metrics`
- `http://<admin-host>:<admin-port>/api/v1/metrics`

## Main design principles

- Keep browser interactions single-origin.
- Prefer deterministic transformations over implicit heuristics.
- Handle sparse/missing metrics without UI flicker.
- Keep topology readable first, then dense.
- Keep the UI diagnostic-focused and near-real-time.
- Favor modular JS boundaries over a growing monolith.

## General development guidelines

- Reuse existing controls/styles before introducing new UI patterns.
- When adding metrics, wire extraction/aggregation/rendering together.
- Preserve freeze/scrub and hover synchronization behavior.
- Preserve default selector semantics (`ALL` core where applicable).
- Avoid changing metric-name contracts silently.

## Data source and acquisition

### Endpoint strategy

`metrics-api.js` builds same-origin candidates only:

1. `/api/v1/telemetry/metrics?...`
2. `/api/v1/metrics?...`

`fetchMetricsFromCandidates()` probes candidates in order and caches the first
successful URL for the next cycle.

### Snapshot contract

Expected payload:

- `timestamp`
- `metric_sets[]`

Main consumed metric-set families:

- `engine`
- `pipeline`
- `tokio.runtime`
- `channel.sender`
- `channel.receiver`
- `topic.exporter`
- `topic.receiver`
- node metric sets keyed by `node.id`

## State model

UI state lives in module-level variables in `main.js`.

### Acquisition state

- `lastSampleTs`, `lastSampleSeconds`
- `lastMetricSets`
- `resolvedMetricsUrl`
- connection/error display state

### Filter and selection state

- `selectedPipelineKey`, `selectedCoreId`
- `selectedEdgeId`, `selectedNodeId`
- `hideZeroActivity`, `dagSearchQuery`, `metricMode`
- `showControlChannels`, `showPipelineCharts`

Filter changes reset dependent visualization state through
`resetVisualizationStateForFilterChange()`.

### Time and hover state

- `windowMinutes`
- freeze/scrub: `freezeActive`, `freezeAnchorMs`, `freezeTimeMs`
- synchronized hover: `globalHoverTs`, `pipelineHoverTs`, `nodeHoverTs`

### Derived and chart state

- previous counters for rate derivation: `pipelinePrev`, `tokioPrev`
- hold-last engine values: `lastEngineCpuUtilPercent`,
  `lastEngineMemoryRssMiB`, `lastEngineUptimeSeconds`
- bounded in-memory series: `pipelineSeries`, `channelSeries`, `nodeSeries`
- live chart instances: `pipelineCharts`, `channelChart`, `nodeCharts`

Series are trimmed to `MAX_WINDOW_MS`.

## Selector and attribute handling

`pipeline-utils.js` centralizes:

- attribute normalization (`normalizeAttributes`)
- pipeline/group extraction
- stable selection-key construction
- core selector fallback (`resolveSelectedCoreId`, default `__all__`)

Pipeline selector UI keeps hierarchy readability using optgroups by
`pipeline.group.id`.

## Metric derivation and card updates

### Engine cards

`engine-metrics.js` provides:

- `extractEngineSummary()`
- `deriveEngineCardValues()`

Rules:

- optional skip of all-zero engine snapshots (`skipAllZeroSnapshots`)
- optional hold-last behavior for sparse snapshots (`holdLastValues`)
- derive group/pipeline/core counts from observed attributes/sets
- derive engine uptime from `pipeline.uptime` (max observed)

### Pipeline and Tokio cards

`main.js` derives:

- direct averages/sums for gauge-like fields
- rates from delta counters using current sample interval
- rates from cumulative counters using previous snapshot state

Derived values are also recorded into `pipelineSeries` for chart rendering and
freeze/scrub playback.

## Graph construction and topology rules

### Graph construction (`buildGraph`)

Input: filtered metric sets + allowed channel kinds (`pdata` or `control`).

Process:

- collect sender/receiver endpoint records by `channel.id`
- merge node attributes from node and endpoint metric sets
- materialize edges as sender x receiver pairs
- assign stable edge ids: `<channelId>::<source>::<target>::<port>`
- keep node port lists for rendering

### Build-time cleanup rules

- ignore channel sets without `channel.id`
- ignore channels with missing sender or receiver
- filter channels by kind when requested
- remove nodes not participating in any channel
- fallback missing `node.port` to `default`
- sort node and port lists for deterministic behavior

### Layout and readability rules (`layoutGraph`)

- classify nodes into receiver/processor/exporter lanes using `node.type`
- compute processor columns from processor->processor topological levels
- rank nodes by traffic for ordering
- size nodes from port count and label width
- reduce overlap/intersection through iterative placement refinement
- emit lane metadata for visual lane backgrounds and labels

### Render-time cleanliness rules (`renderGraph`)

- optional zero-activity filter removes inactive edges/nodes
- optional search filter keeps relevant subgraph context
- invalid/stale selections are cleared automatically
- node ports are re-sorted by current traffic intensity
- control channels are overlaid only when explicitly enabled

## How to test

UI JS module tests (from repo root):

```bash
./scripts/run-ui-js-tests.sh
```

Single UI test (or glob):

```bash
./scripts/run-ui-js-tests.sh crates/admin/ui/js/tests/graph-renderer.test.mjs
```

Recommended JS syntax check:

```bash
node --check crates/admin/ui/js/main.js \
  crates/admin/ui/js/charts-controller.js \
  crates/admin/ui/js/dag-interaction-controller.js \
  crates/admin/ui/js/graph-renderer.js \
  crates/admin/ui/js/pipeline-charts-controller.js \
  crates/admin/ui/js/selection-details-controller.js \
  crates/admin/ui/js/metrics-api.js \
  crates/admin/ui/js/pipeline-utils.js \
  crates/admin/ui/js/engine-metrics.js \
  crates/admin/ui/js/inter-pipeline-topology.js \
  crates/admin/ui/js/control-utils.js \
  crates/admin/ui/js/dom-safety.js
```

## Future CI integration

UI module tests should be integrated as a **separate GitHub Actions job**
focused only on JavaScript UI validation.

Recommended UI-only CI job:

1. Trigger only when UI JS paths change:
   - `rust/otap-dataflow/crates/admin/ui/js/**`
   - `rust/otap-dataflow/scripts/run-ui-js-tests.sh`
2. Run only:
   - `./rust/otap-dataflow/scripts/run-ui-js-tests.sh`
3. Mark this job as its own required PR status check
   (independent from Rust checks).

Practical CI notes:

- Pin a Node.js version in CI to avoid drift.
- Keep the JS tests browserless and deterministic
  (no network, no timers by default).
- Keep this job independent from `rust-required-status-check` so UI failures
  are isolated.

## Current limits

- This UI is not a persistence/analytics frontend.
- Snapshot polling is periodic, not event-streamed.
- Some card values are derived from emitted telemetry; fidelity depends on
  metric availability and quality.
