# Controller Live Control

`live_control` owns the in-process runtime model used by the admin control
plane to reconfigure and shut down logical pipelines while the engine is
running. It is deliberately internal to the controller: public admin API
shapes live in `otap-df-admin` and `otap-df-admin-types`, while this module
tracks the mutable controller state required to execute those API requests.

## Goals

- Accept per-pipeline rollout and shutdown requests without restarting the
  whole engine.
- Keep controller state consistent across asynchronous pipeline-thread exits,
  rollout workers, shutdown workers, and observed-state updates.
- Preserve useful recent operation snapshots while bounding in-memory
  retention.
- Keep old runtime instances visible only while active controller work still
  needs generation-specific status.

## Architecture

The module is split by responsibility:

- `mod.rs` is the facade. It defines `ControllerRuntime`, the control-plane
  adapter, startup registration, shared pruning helpers, and the
  `ControlPlane` implementation.
- `state.rs` defines the in-memory state model: rollout/shutdown records,
  runtime-instance records, candidate plans, panic/error reports, and retention
  constants.
- `planning.rs` validates requests, classifies rollout actions, prepares
  candidate rollout/shutdown plans, records accepted operations, updates status
  snapshots, and spawns background workers.
- `execution.rs` runs rollout and shutdown workers. It handles create, resize,
  replace, rollback, panic cleanup, and per-core progress updates.
- `runtime.rs` launches pipeline threads, registers instances, records exits,
  sends shutdown requests, waits for readiness/exit, and exposes global runtime
  shutdown/error helpers.

`ControllerRuntime` is the shared owner. It is held behind an `Arc` by the
admin control-plane adapter and by detached rollout/shutdown workers. Pipeline
threads receive a `Weak<ControllerRuntime<_>>` so they can report exits without
extending the controller lifetime.

## Lifecycle Model

Live control separates three related concepts:

- A logical pipeline is identified by `(pipeline_group_id, pipeline_id)` and
  points at the committed resolved pipeline plus its active generation.
- A pipeline group is the config hierarchy that contains related pipelines,
  group-local topics, and group-level policies. Current live-control operations
  target one logical pipeline inside that group.
- A deployed runtime instance is identified by `(pipeline_group_id,
  pipeline_id, core_id, deployment_generation)` and tracks whether that thread
  is still active or has exited.
- A controller operation is a rollout or shutdown record with public progress
  state and per-core details.

Rollouts are classified before execution:

- `create` launches a logical pipeline that did not exist.
- `noop` commits an identical request without launching a worker.
- `resize` changes core placement without changing the runtime shape.
- `replace` launches a new generation, waits for readiness, then drains the
  previous generation.

Shutdown targets the currently active deployed instances for one logical
pipeline. Global shutdown bypasses operation history and broadcasts shutdown to
all active instances.

## Design Decisions

- The controller is the authority for when old generations can be retired.
  Observed-state compaction is invoked only after active rollout/shutdown work
  no longer needs generation-specific entries.
- The current consistency scope is one logical pipeline. Planning validates a
  candidate against a cloned full config snapshot, but commit patches only that
  pipeline into the latest live config. This intentionally does not provide
  whole-config serializability across concurrent operations on different
  logical pipelines.
- Terminal rollout and shutdown records are retained in memory with both a
  per-logical-pipeline cap and a TTL. This keeps recent admin lookups useful
  without unbounded history growth.
- Runtime exit reporting is race-tolerant. A pipeline thread can exit before
  `register_launched_instance()` publishes it as active; such exits are parked
  in `pending_instance_exits` and reconciled during registration.
- Worker panic handling is unwind-safe. Panic cleanup records terminal failure,
  clears active-operation conflict state, and reports concise public failure
  reasons plus detailed internal diagnostics.
- Topic broker runtime shape is not mutable through live reconfiguration.
  Rollout planning rejects requests that would require changing declared topic
  backend, policy, or selected implementation mode.

## Current Limits

- Rollout and shutdown workers are detached OS threads. They are supervised by
  panic cleanup, but there is no bounded worker pool or join-handle supervisor.
- Topic declaration changes are intentionally rejected. Supporting them would
  require a separate broker migration model.
- Operation history is in-memory only. It is bounded and useful for recent
  lookups, but it is not durable across controller restarts.
- Full group shutdown is orchestrated above this module by issuing
  per-pipeline/global control-plane calls; this module tracks per-pipeline
  live-control state.
- Future group-level reconfiguration can widen the active-operation conflict
  scope from logical pipeline to pipeline group without changing the existing
  per-pipeline endpoint shape.
- Rollbacks are best effort. If rollback itself fails, the operation records
  `rollback_failed` and preserves diagnostics for operators.
