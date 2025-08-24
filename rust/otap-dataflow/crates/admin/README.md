# Admin Interface

## HTTP Endpoints

- `/` (TBD): list available endpoints (OpenAPI spec)

### Health Check (TBD)

- `/health`: simple health check

### Pipelines (TBD)

- `/pipeline-groups` - list active pipeline groups
- `/pipeline-groups/:id` - get details of a specific pipeline group
- `/pipeline-groups/:id/pipelines` - list active pipelines and their status
- `/pipeline-groups/:id/pipelines/:id` - get details of a specific pipeline

### Telemetry

- `/telemetry/live-schema`: current semantic conventions registry
- `/telemetry/metrics`: current aggregated metrics in JSON, line protocol, or Prometheus text format
- `/telemetry/metrics/aggregate`: aggregated metrics grouped by metric set name and optional attributes
