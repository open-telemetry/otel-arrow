# Observed State

Status: **Work-In-Progress**

## System Overview

![Engine Observability](assets/engine-observability.svg)

ToDo: add metrics, event exporters

## Hierarchical Observed State

![Observed State](assets/hierarchical-obv-state.svg)

## Pipeline Runtime State Machine

![Pipeline Runtime State Machine](assets/pipeline-runtime-state-machine.svg)

### Conditions

Every pipeline core emits Kubernetes-style conditions alongside its phase:

- **Accepted**: `True` once admission/config validation succeeds,
  `False/Unknown` otherwise.
- **Ready**: `True` when the core is healthy enough to process telemetry (
  policy-driven), `False/Unknown` otherwise.

Each condition carries:

- `status`: `True`, `False`, or `Unknown`.
- `reason`: a strongly-typed `ConditionReason` enum (e.g. `ConfigValid`,
  `Pending`, `QuorumNotMet`, `ForceDeleting`, `Unknown(String)`).
- `message`: human-readable context.
- `last_update`: timestamp of the most recent transition.

Pipeline-level conditions are synthesized from the per-core conditions (
respecting quorum policies) so API consumers no longer have to infer health from
legacy "phase" strings.

### Phases

- Pending: Exists but not admitted; awaiting a decision.
- Starting: Admitted; provisioning-initialization in progress.
- Running: Serving traffic normally.
- Updating: Applying a new spec-version under control.
- RollingBack: Reverting after update failure.
- Draining: Quiescing; no new work; finishing in-flight.
- Stopped: Cleanly stopped; can be restarted with re-admission.
- Rejected(AdmissionError|ConfigRejected): Input was invalid or disallowed; fix
  inputs.
- Failed(RuntimeError|DrainError|RollbackFailed|DeleteError): Unrecoverable
  runtime/teardown failure.
- Deleting(Graceful|Forced): Teardown in progress (forced may drop in-flight
  work).
- Deleted: All resources removed; terminal.

## Liveness & Readiness Probes

First-class Kubernetes probes (/livez, /readyz):

- `/livez`: fails only when a non-benign `Accepted=False/Unknown` condition is
  observed (e.g. `AdmissionError`, `ConfigRejected`, `RuntimeError`). Pipelines
  with no observed runtimes are ignored.
- `/readyz`: fails when any tracked pipeline reports `Ready=False/Unknown` and
  the configured ready quorum is not met (reason surfaced via the aggregate
  condition).

### Per-core probe mapping (policy-driven)

- ProbePolicy:
  - live_if: phases considered alive (default: all except `Deleted`).
  - ready_if: phases considered ready (default: `Running` and optionally
    `Updating`).

### Aggregate probe policy (quorums)

- Quorum: All | AtLeast(n) | Percent(p) (of non-Deleted cores).
- AggregationPolicy:
  - core_probe: ProbePolicy
  - live_quorum (default AtLeast(1))
  - ready_quorum (default All; popular alternative: Percent(80))
