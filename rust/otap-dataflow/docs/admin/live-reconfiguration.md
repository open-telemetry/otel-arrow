# Live Pipeline Reconfiguration

This document describes the live reconfiguration flow exposed by the admin API.

The feature lets a running OTel Dataflow Engine mutate one logical pipeline at
a time without restarting the process or reloading the full startup file.

## Goals

- Reconfigure one pipeline in a running engine instance.
- Keep the mutation scoped to a single `(pipeline_group_id, pipeline_id)`.
- Preserve service continuity for topology/config changes with a serial rolling
  cutover that overlaps old and new instances only on the affected cores.
- Support pure resource policy changes, including scale up and scale down,
  without restarting unchanged cores.
- Make progress observable through admin endpoints instead of hidden internal
  controller state.

## Supported Operations

- Create a new pipeline inside an existing pipeline group.
- Replace an existing pipeline with a new topology or node configuration.
- Resize an existing pipeline when the only effective runtime change is
  `policies.resources.core_allocation`.
- Accept an effectively identical update as a `noop`.
- Track rollout progress with a rollout id.
- Shutdown a logical pipeline and track shutdown progress with a shutdown id.

## Terminology

Live reconfiguration uses a few controller-specific terms. They are important
because the admin API exposes both committed pipeline state and in-progress
runtime state.

- Logical pipeline: the named pipeline addressed by `(pipeline_group_id,
  pipeline_id)`. A logical pipeline can have several runtime instances over
  time as it is rolled, resized, or shut down.
- Runtime instance: one concrete execution of a logical pipeline on one core.
  Runtime instances are identified by `(pipeline_group_id, pipeline_id, core_id,
  deployment_generation)`.
- Deployment generation: a monotonically assigned version for runtime
  instances of one logical pipeline. `create` and `replace` rollouts start a new
  generation. `resize` keeps the same generation and only changes the active
  core set.
- Active generation: the generation currently committed by the controller as
  the logical pipeline's desired serving generation.
- Serving generation: the generation currently selected for a specific core in
  observed state. During a rolling cutover, different cores may temporarily
  serve different generations.
- Candidate pipeline config: the pipeline config submitted by the client and
  validated by the controller before it is committed into the live in-memory
  engine config.
- Candidate generation: the target generation for a `create` or `replace`
  rollout while it is still being tested and has not yet been committed as the
  active generation.
- Candidate instance: a runtime instance launched from the candidate generation.
  Candidate instances must become admitted and ready before the controller uses
  them for serving. If the rollout fails before commit, the controller
  best-effort shuts them down.
- Rollout worker: the background controller thread that executes an accepted
  rollout plan after the admin request has been accepted. The API can return
  before this worker finishes when `wait=false`.
- Rollout worker panic: an unexpected Rust panic in the rollout worker itself,
  not a normal pipeline runtime error. The controller catches this panic, marks
  the rollout failed, reports diagnostics, clears the active-operation conflict,
  and cleans up uncommitted candidate instances when needed.
- Drain: a graceful shutdown step. The runtime stops accepting new ingress,
  lets already admitted work finish as far as the node contracts allow, and
  exits before the drain timeout.

This document uses `serial rolling cutover with overlap` for topology-changing
replacement.

During `replace`, the controller overlaps old and new instances only on the
core currently being switched:

- start the new instance for one core;
- wait for `Admitted` and `Ready`;
- drain the old instance on that same core;
- move to the next core.

This does not start a second complete serving fleet and perform one atomic
traffic flip across the whole pipeline.

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

## Consistency Model

The current API serializes live operations per logical pipeline, identified by
`(pipeline_group_id, pipeline_id)`. A rollout or shutdown conflicts with another
active operation for the same logical pipeline, while operations for different
logical pipelines may run concurrently.

Rollout planning validates a candidate by patching one pipeline into the
controller's current in-memory `OtelDataflowSpec` snapshot and running full
engine validation on that candidate snapshot. That validation does not make the
operation a whole-config transaction: another logical pipeline can commit before
this rollout commits, and commit applies only the accepted pipeline back into
the latest live config.

The API intentionally leaves room to widen the consistency scope later. If
group-level invariants become mutable, the controller can serialize
config-mutating operations per pipeline group and return `409 Conflict` for
concurrent operations in that group without changing the existing pipeline
endpoint or response schema. Engine-level reconfiguration can be added as a
separate operation surface if full-engine transactions become necessary.

## How It Works

1. The client submits a candidate pipeline config to
   `PUT /groups/{group}/pipelines/{id}`.
1. The controller patches exactly that pipeline into its live in-memory
   `OtelDataflowSpec`.
1. The candidate config is validated as a full engine snapshot:
   - pipeline structure and canonicalization;
   - component config validation;
   - whole-config validation, including topic cycle checks;
   - topic runtime profile compatibility.
1. The controller classifies the update:
   - `create`: the logical pipeline does not exist yet;
   - `noop`: the resolved pipeline and active serving footprint already match
     the request;
   - `replace`: the runtime graph or runtime-significant node config changed;
   - `resize`: only the effective core allocation changed.
1. The controller executes the plan:
   - `create`: start all target instances in parallel and commit only if they
     all become healthy.
   - `noop`: record an immediately successful rollout result without restarting
     any runtime instances.
   - `replace`: do a serial rolling cutover with overlap per common core.
     Start the new generation on one core, wait for admission and readiness,
     then drain the old generation on that core.
   - `resize`: start only newly added cores and drain only removed cores.
     Common cores stay up and keep serving the current generation.
1. The controller records rollout progress and mirrors a summary into
   `GET /groups/{group}/pipelines/{id}/status`.

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

### Controller Safety Behaviors

The controller treats live reconfiguration as a runtime lifecycle operation,
not just as an in-memory config edit. Several edge cases are handled explicitly
to avoid orphaned runtime instances, stale conflicts, or unbounded status
growth.

- Partial `create` launch failure: if one core fails to launch after earlier
  cores were already started, the controller best-effort shuts down the
  candidate instances that were launched by that same create operation before
  returning rollout failure.
- Readiness failure after candidate launch: if a candidate generation starts
  but does not reach `Admitted` and `Ready` before the step timeout, the
  controller shuts down the candidate instance before continuing with failure
  handling or rollback.
- Rollout worker panic: if the detached rollout worker panics, the controller
  records a terminal failed rollout, clears the active-operation conflict, and
  emits internal panic diagnostics. If the panic happened while an uncommitted
  target generation was active, the controller best-effort sends shutdown to
  those candidate instances first.
- Committed generation protection: panic cleanup does not shut down a target
  generation that is already committed as the active serving generation. This
  prevents a late bookkeeping panic from turning a successful rollout into an
  outage.
- Shutdown worker panic: if the detached shutdown worker panics, the controller
  records a terminal failed shutdown and clears the active-operation conflict,
  so later operations for the same logical pipeline are not blocked until
  restart.
- Runtime thread panic or error: runtime instance failures are reported back
  into observed state with a concise operator message and diagnostic source
  detail. The instance is marked exited so controller liveness accounting can
  progress.
- Launch and exit races: a runtime thread can exit before its launch
  registration is visible to the controller. The controller records early exits
  and reconciles them during registration, avoiding stale active-instance
  counts.
- Global shutdown dispatch: `POST /groups/shutdown` snapshots active instances
  and attempts shutdown delivery to all of them. One failed send does not
  prevent later instances from receiving shutdown. Dispatch is idempotent for
  instances that already accepted shutdown.
- Observed-state compaction: after active controller work no longer needs old
  generations, the controller compacts retained instance status to the selected
  serving view. During active rollout overlap, status still exposes both old and
  new generations so operators can debug cutover behavior.
- Bounded operation history: terminal rollout and shutdown records are retained
  only in a bounded in-memory window. Recent terminal ids remain useful for
  follow-up inspection, but old by-id history is intentionally evictable.

## API Surface

### Read current pipeline config

`GET /groups/{group}/pipelines/{id}`

Returns:

- `pipelineGroupId`
- `pipelineId`
- `activeGeneration`
- `pipeline`
- optional `rollout` summary

### Create, replace, or resize a pipeline

`PUT /groups/{group}/pipelines/{id}?wait=true|false&timeout_secs=<overall>`

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
- If the submitted config is already in effect, the action is `noop`.
- If only the effective core allocation changed, the action is `resize`.
- Otherwise the action is `replace`.

Response body is a `RolloutStatus` with:

- `rolloutId`
- `action` (`create`, `noop`, `replace`, `resize`)
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
- `409 Conflict`: another incompatible live operation is active in the
  controller's current consistency scope, or a waited rollout finished in
  failure. In the current version of the API, that scope is one logical
  pipeline.
- `422 Unprocessable Entity`: validation failure or unsupported runtime
  mutation
- `504 Gateway Timeout`: `wait=true` exceeded the overall wait timeout

### Read rollout progress

`GET /groups/{group}/pipelines/{id}/rollouts/{rolloutId}`

Returns the current `RolloutStatus` snapshot for that operation.
Terminal rollout ids are retained only within a bounded in-memory window, so
older ids may return `404 Not Found` after eviction.

### Read observed pipeline status

`GET /groups/{group}/pipelines/{id}/status`

Returns the aggregated pipeline status. Useful fields during rollout:

- `conditions`
- `totalCores`
- `runningCores`
- `activeGeneration`
- `servingGenerations`
- `rollout`
- `instances`

Each `instances` entry is keyed by `(coreId, deploymentGeneration)`, so
overlapping old/new generations stay distinguishable during a rolling cutover.

### Related shutdown endpoints

- `POST /groups/{group}/pipelines/{id}/shutdown`
- `GET /groups/{group}/pipelines/{id}/shutdowns/{shutdownId}`
- `POST /groups/shutdown`

These are separate from reconfiguration, but they use the same resident
controller and the same live-operation consistency scope.
Terminal shutdown ids are retained only within a bounded in-memory window, so
older ids may return `404 Not Found` after eviction.

## Manual Examples

The examples below use
[`configs/engine-conf/topic_multitenant_isolation.yaml`](../../configs/engine-conf/topic_multitenant_isolation.yaml).
That config binds admin HTTP to `127.0.0.1:8085` and defines the logical
pipeline `topic_multitenant_isolation/tenant_c_pipeline`.

### Start the sample engine

```bash
cargo run -- -c configs/engine-conf/topic_multitenant_isolation.yaml
```

In another terminal:

```bash
BASE=http://127.0.0.1:8085/api/v1
GROUP=topic_multitenant_isolation
PIPE=tenant_c_pipeline
```

Inspect the current committed config and observed runtime state:

```bash
curl -s "$BASE/groups/$GROUP/pipelines/$PIPE" | jq .
curl -s "$BASE/groups/$GROUP/pipelines/$PIPE/status" | jq .
```

### Example: Topology change with serial rolling cutover

This example inserts a debug processor between the topic receiver and the retry
processor.

Build the request body from the live config:

```bash
curl -s "$BASE/groups/$GROUP/pipelines/$PIPE" \
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
  "$BASE/groups/$GROUP/pipelines/$PIPE?wait=true&timeout_secs=120" \
  -H 'content-type: application/json' \
  --data-binary @/tmp/tenant_c_pipeline-debug.json | jq .
```

Expected result:

- `action` is `replace`
- `state` ends as `succeeded`
- `targetGeneration` is greater than `previousGeneration`

Verify the committed config and rollout-aware status:

```bash
curl -s "$BASE/groups/$GROUP/pipelines/$PIPE" | jq .
curl -s "$BASE/groups/$GROUP/pipelines/$PIPE/status" \
  | jq '{conditions, totalCores, runningCores, activeGeneration, servingGenerations, rollout, instances}'
```

### Example: Async rollout tracking

Use `wait=false` to return immediately, then poll the rollout resource:

```bash
ROLLOUT_ID=$(
  curl -sS -X PUT \
    "$BASE/groups/$GROUP/pipelines/$PIPE?wait=false" \
    -H 'content-type: application/json' \
    --data-binary @/tmp/tenant_c_pipeline-debug.json \
  | jq -r '.rolloutId'
)

curl -s "$BASE/groups/$GROUP/pipelines/$PIPE/rollouts/$ROLLOUT_ID" | jq .
```

### Example: Pure resource-policy resize

This example changes only `coreAllocation.count` from `1` to `2`. The
controller detects that the runtime shape is otherwise unchanged and executes a
`resize` rollout instead of a full replace.

```bash
curl -s "$BASE/groups/$GROUP/pipelines/$PIPE" \
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
  "$BASE/groups/$GROUP/pipelines/$PIPE?wait=true&timeout_secs=120" \
  -H 'content-type: application/json' \
  --data-binary @/tmp/tenant_c_pipeline-scale-up.json | jq .
```

Expected result:

- `action` is `resize`
- `targetGeneration` stays equal to `previousGeneration`
- only the added core is started

Verify the pipeline footprint:

```bash
curl -s "$BASE/groups/$GROUP/pipelines/$PIPE/status" \
  | jq '{totalCores, runningCores, activeGeneration, servingGenerations, rollout}'
```

Scale back down by setting `coreAllocation.count = 1` in the same request body
pattern.

## Operational Notes

- Different logical pipelines may roll concurrently in the current
  implementation.
- A single logical pipeline allows only one active rollout or shutdown at a
  time.
- Future group-level consistency can widen the conflict scope so concurrent
  operations in the same group return `409 Conflict`.
- `GET /groups/{group}/pipelines/{id}` always returns the committed
  live config, not an uncommitted candidate.
- `GET /groups/{group}/pipelines/{id}/status` is the best endpoint
  for watching serving generations and per-instance phase changes during a
  rollout.
