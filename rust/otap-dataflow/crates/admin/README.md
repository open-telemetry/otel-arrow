# Admin Interface

`otap-df-admin` provides:

- admin, health, status, and telemetry HTTP endpoints;
- live pipeline mutation endpoints for create, replace, resize, rollout
  tracking, and shutdown tracking;
- an embedded single-page UI served from the same process and origin.

For architecture and runtime behavior details, see
[`docs/admin/architecture.md`](../../docs/admin/architecture.md).
For the admin docs landing page, see
[`docs/admin/README.md`](../../docs/admin/README.md).
For the operator guide to live pipeline mutation, see
[`docs/admin/live-reconfiguration.md`](../../docs/admin/live-reconfiguration.md).

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
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/status`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/rollouts/{rollout_id}`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdowns/{shutdown_id}`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/livez`
- `GET /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/readyz`

### Pipeline lifecycle

- `PUT /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}`
- `POST /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdown`
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

## Security

### Security controls currently in place

- Embedded UI assets are compiled into the binary via `include_dir` and served
  from in-memory embedded files (no runtime static file directory exposure).
- Default admin bind address is loopback (`127.0.0.1:8080`) unless explicitly
  overridden in config.
- UI/static responses include hardened browser headers:
  - `Content-Security-Policy` (self-only scripts/connect/object/base/frame
    restrictions)
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: no-referrer`
  - `Cache-Control: no-store, no-cache, must-revalidate`
- UI dependencies are vendored and served locally from `/static/vendor/*`
  (no CDN dependency at runtime).
- UI code uses explicit escaping helpers (`ui/js/dom-safety.js`) for dynamic
  HTML, attributes, and selector values.

### Security improvement TODO

- [ ] Add authentication and authorization for admin endpoints (or require it
  through an enforced integration layer).
- [ ] Add TLS support in-process or enforce TLS at a mandatory front proxy
  boundary.
- [ ] Protect mutating endpoints such as
  `PUT /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}`,
  `POST /pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdown`,
  and `POST /pipeline-groups/shutdown` with stricter access controls than
  read-only endpoints.
- [ ] Apply the same hardened response headers to API endpoints
  (`/status`, `/livez`, `/readyz`, `/telemetry/*`, `/metrics`), not only UI/static.
- [ ] Harden CSP further by removing `style-src 'unsafe-inline'` (move toward
  nonce/hash-based style policies).
- [ ] Add rate limiting / throttling to protect admin and telemetry endpoints.
- [ ] Add CSRF protection strategy for mutating endpoints when deployed behind
  cookie-based auth proxies.

### Deployment recommendations

- Keep admin bound to loopback whenever possible.
- If remote access is needed, place admin behind a reverse proxy with:
  - TLS termination
  - strong authentication/authorization
  - network ACLs / source allow-listing
  - route-level restrictions for mutating endpoints such as
    `/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}`,
    `/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/shutdown`,
    and `/pipeline-groups/shutdown`
