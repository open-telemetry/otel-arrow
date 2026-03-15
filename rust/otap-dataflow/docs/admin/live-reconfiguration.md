# Live Pipeline Reconfiguration

This document describes the live reconfiguration flow exposed by the admin API.

The feature lets a running OTel Dataflow Engine mutate one logical pipeline at
a time without restarting the process or reloading the full startup file.

## Goals

- Reconfigure one pipeline in a running engine instance.
- Keep the mutation scoped to a single `(pipeline_group_id, pipeline_id)`.
- Preserve service continuity for topology/config changes with a blue/green
  rollout on the affected cores.
- Support pure resource policy changes, including scale up and scale down,
  without restarting unchanged cores.
- Make progress observable through admin endpoints instead of hidden internal
  controller state.

## What v1 Supports

- Create a new pipeline inside an existing pipeline group.
- Replace an existing pipeline with a new topology or node configuration.
- Resize an existing pipeline when the only effective runtime change is
  `policies.resources.core_allocation`.
- Track rollout progress with a rollout id.
- Shutdown a logical pipeline and track shutdown progress with a shutdown id.

## Boundaries and Current Limits

- Updates are in-memory only. The startup YAML file is not rewritten.
- The target pipeline group must already exist.
- Runtime topic broker mutation is rejected. In practice this means:
  - no new or removed declared topics;
  - no change to the selected topic mode;
  - no change to topic backend or topic policies.
- Group-level and engine-level policy mutation is out of scope.
- There is no dedicated scale endpoint. Scale-only changes use the same `PUT`
  endpoint as topology changes.
- There is not yet a no-op fast path for identical updates. If the submitted
  config is effectively identical to the committed one, the controller still
  plans a replace rollout today.

## How It Works

1. The client submits a candidate pipeline config to
   `PUT /pipeline-groups/{group}/pipelines/{id}`.
1. The controller patches exactly that pipeline into its live in-memory
   `OtelDataflowSpec`.
1. The candidate config is validated as a full engine snapshot:
   - pipeline structure and canonicalization;
   - component config validation;
   - whole-config validation, including topic cycle checks;
   - topic runtime profile compatibility.
1. The controller classifies the update:
   - `create`: the logical pipeline does not exist yet;
   - `replace`: the runtime graph or runtime-significant node config changed;
   - `resize`: only the effective core allocation changed.
1. The controller executes the plan:
   - `create`: start all target instances in parallel and commit only if they
     all become healthy.
   - `replace`: do a serial blue/green rollout per common core. Start the new
     generation on one core, wait for admission and readiness, then drain the
     old generation on that core.
   - `resize`: start only newly added cores and drain only removed cores.
     Common cores stay up and keep serving the current generation.
1. The controller records rollout progress and mirrors a summary into
   `GET /pipeline-groups/{group}/pipelines/{id}/status`.

### Success Gate

For `replace` and `create`, a new instance must reach both `Admitted` and
`Ready` before the controller commits the new serving state for that step.

The request body carries two timeouts:

- `stepTimeoutSecs`: how long to wait for the new instance to admit and become
  ready. Default: `60`.
- `drainTimeoutSecs`: how long to wait for graceful drain of the old instance.
  Default: `60`.

The query string also supports an overall client wait timeout:

- `timeout_secs` on the `PUT` request when `wait=true`.

### Failure Handling

- `create`: if any target instance fails to admit or become ready, the
  controller shuts down the instances that were already launched and leaves the
  committed config unchanged.
- `replace`: if a core fails during the rollout, the controller stops and
  automatically rolls back already switched cores to the previous generation.
- `resize`: if added or removed cores fail during the operation, the controller
  rolls the resize back by draining newly added cores and relaunching retired
  cores when possible.
- If rollback cannot restore a healthy serving set, the rollout ends in
  `rollback_failed` and the mixed state remains visible through status
  endpoints.

## API Surface

### Read current pipeline config

`GET /pipeline-groups/{group}/pipelines/{id}`

Returns:

- `pipelineGroupId`
- `pipelineId`
- `activeGeneration`
- `pipeline`
- optional `rollout` summary

### Create, replace, or resize a pipeline

`PUT /pipeline-groups/{group}/pipelines/{id}?wait=true|false&timeout_secs=<overall>`

Request body:

```json
{
  "pipeline": {
    "...": "PipelineConfig"
  },
  "stepTimeoutSecs": 60,
  "drainTimeoutSecs": 60
}
```

Behavior:

- If `(group, id)` does not exist, the action is `create`.
- If only the effective core allocation changed, the action is `resize`.
- Otherwise the action is `replace`.

Response body is a `PipelineRolloutStatus` with:

- `rolloutId`
- `action` (`create`, `replace`, `resize`)
- `state` (`pending`, `running`, `succeeded`, `failed`, `rolling_back`,
  `rollback_failed`)
- `targetGeneration`
- `previousGeneration`
- `startedAt`
- `updatedAt`
- optional `failureReason`
- `cores`

Status codes:

- `202 Accepted`: request accepted and `wait=false`
- `200 OK`: `wait=true` and the rollout finished successfully
- `404 Not Found`: pipeline group does not exist
- `409 Conflict`: another rollout or shutdown is already active for the same
  logical pipeline, or a waited rollout finished in failure
- `422 Unprocessable Entity`: validation failure or unsupported runtime
  mutation
- `504 Gateway Timeout`: `wait=true` exceeded the overall wait timeout

### Read rollout progress

`GET /pipeline-groups/{group}/pipelines/{id}/rollouts/{rolloutId}`

Returns the current `PipelineRolloutStatus` snapshot for that operation.

### Read observed pipeline status

`GET /pipeline-groups/{group}/pipelines/{id}/status`

Returns the aggregated pipeline status. Useful fields during rollout:

- `conditions`
- `totalCores`
- `runningCores`
- `activeGeneration`
- `servingGenerations`
- `rollout`
- `instances`

Each `instances` entry is keyed by `(coreId, deploymentGeneration)`, so
overlapping old/new generations stay distinguishable during blue/green rollout.

### Related shutdown endpoints

- `POST /pipeline-groups/{group}/pipelines/{id}/shutdown`
- `GET /pipeline-groups/{group}/pipelines/{id}/shutdowns/{shutdownId}`
- `POST /pipeline-groups/shutdown`

These are separate from reconfiguration, but they use the same resident
controller and the same logical-pipeline locking rules.

## Manual Examples

The examples below use
[`configs/engine-conf/topic_multitenant_isolation.yaml`](../../configs/engine-conf/topic_multitenant_isolation.yaml).
That config binds admin HTTP to `127.0.0.1:8085` and defines the logical
pipeline `topic_multitenant_isolation/tenant_c_pipeline`.

### Start the sample engine

```bash
cd /Users/l.querel/oss/otel-arrow/rust/otap-dataflow
cargo run -- -c configs/engine-conf/topic_multitenant_isolation.yaml
```

In another terminal:

```bash
BASE=http://127.0.0.1:8085
GROUP=topic_multitenant_isolation
PIPE=tenant_c_pipeline
```

Inspect the current committed config and observed runtime state:

```bash
curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE" | jq .
curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE/status" | jq .
```

### Example: Topology change with blue/green replace

This example inserts a debug processor between the topic receiver and the retry
processor.

Build the request body from the live config:

```bash
curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE" \
  | jq '
      {
        pipeline: (
          .pipeline
          | .nodes += {
              tenant_c_debug: {
                type: "processor:debug",
                config: {
                  verbosity: "basic"
                }
              }
            }
          | .connections = [
              {from: "tenant_c_receiver", to: "tenant_c_debug"},
              {from: "tenant_c_debug", to: "tenant_c_retry"},
              {from: "tenant_c_retry", to: "tenant_c_sink"}
            ]
        ),
        stepTimeoutSecs: 60,
        drainTimeoutSecs: 60
      }
    ' \
  > /tmp/tenant_c_pipeline-debug.json
```

Submit the update and wait for completion:

```bash
curl -sS -X PUT \
  "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE?wait=true&timeout_secs=120" \
  -H 'content-type: application/json' \
  --data-binary @/tmp/tenant_c_pipeline-debug.json | jq .
```

Expected result:

- `action` is `replace`
- `state` ends as `succeeded`
- `targetGeneration` is greater than `previousGeneration`

Verify the committed config and rollout-aware status:

```bash
curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE" | jq .
curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE/status" \
  | jq '{conditions, totalCores, runningCores, activeGeneration, servingGenerations, rollout, instances}'
```

### Example: Async rollout tracking

Use `wait=false` to return immediately, then poll the rollout resource:

```bash
ROLLOUT_ID=$(
  curl -sS -X PUT \
    "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE?wait=false" \
    -H 'content-type: application/json' \
    --data-binary @/tmp/tenant_c_pipeline-debug.json \
  | jq -r '.rolloutId'
)

curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE/rollouts/$ROLLOUT_ID" | jq .
```

### Example: Pure resource-policy resize

This example changes only `coreAllocation.count` from `1` to `2`. The
controller detects that the runtime shape is otherwise unchanged and executes a
`resize` rollout instead of a full replace.

```bash
curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE" \
  | jq '
      {
        pipeline: .pipeline,
        stepTimeoutSecs: 60,
        drainTimeoutSecs: 60
      }
      | .pipeline.policies.resources.coreAllocation.count = 2
    ' \
  > /tmp/tenant_c_pipeline-scale-up.json
```

```bash
curl -sS -X PUT \
  "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE?wait=true&timeout_secs=120" \
  -H 'content-type: application/json' \
  --data-binary @/tmp/tenant_c_pipeline-scale-up.json | jq .
```

Expected result:

- `action` is `resize`
- `targetGeneration` stays equal to `previousGeneration`
- only the added core is started

Verify the pipeline footprint:

```bash
curl -s "$BASE/pipeline-groups/$GROUP/pipelines/$PIPE/status" \
  | jq '{totalCores, runningCores, activeGeneration, servingGenerations, rollout}'
```

Scale back down by setting `coreAllocation.count = 1` in the same request body
pattern.

## Operational Notes

- Different logical pipelines may roll concurrently.
- A single logical pipeline allows only one active rollout or shutdown at a
  time.
- `GET /pipeline-groups/{group}/pipelines/{id}` always returns the committed
  live config, not an uncommitted candidate.
- `GET /pipeline-groups/{group}/pipelines/{id}/status` is the best endpoint
  for watching serving generations and per-instance phase changes during a
  rollout.
