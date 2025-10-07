# Observed State

Status: **Work-In-Progress**

## System Overview

![Engine Observability](assets/engine-observability.svg)

ToDo: add metrics, event exporters

## Hierarchical Observed State

![Observed State](assets/hierarchical-obv-state.svg)

## Pipeline Runtime State Machine

![Pipeline Runtime State Machine](assets/pipeline-runtime-state-machine.svg)

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

## Pipeline Aggregated Phase Decision Flow

```mermaid
flowchart TD
    START([Counts from all cores])
    START --> D0{total == deleted?}
    D0 -- yes --> H_Deleted[AggPhase=Deleted]
    D0 -- no --> D1{deleting > 0?}
    D1 -- yes --> H_Deleting[AggPhase=Deleting\nforced = any_forced, remaining = active]
    D1 -- no --> D2{failed > 0?}
    D2 -- yes --> H_Failed[AggPhase=Failed\nfailed, running, top_reason]
    D2 -- no --> D3{rejected > 0?}
    D3 -- yes --> H_Rejected[AggPhase=Rejected\nrejected, running, top_reason]
    D3 -- no --> D4{rolling_back > 0?}
    D4 -- yes --> H_Rollback[AggPhase=RollingBack\nrolling_back, running]
    D4 -- no --> D5{updating > 0?}
    D5 -- yes --> H_Updating[AggPhase=Updating\nupdating, running]
    D5 -- no --> D6{draining > 0?}
    D6 -- yes --> H_Draining[AggPhase=Draining\ndraining, running]
    D6 -- no --> D7{active > 0 && running == active?}
    D7 -- yes --> H_RunAll[AggPhase=RunningAll]
    D7 -- no --> D8{running > 0?}
    D8 -- yes --> H_RunDegraded[AggPhase=RunningDegraded\nrunning, total_active]
    D8 -- no --> D9{pending + starting > 0?}
    D9 -- yes --> H_Starting[AggPhase=Starting\npending, starting]
    D9 -- no --> D10{stopped == active?}
    D10 -- yes --> H_StopAll[AggPhase=StoppedAll\nstopped]
    D10 -- no --> H_StopPart[AggPhase=StoppedPartial\nstopped, total_active]
```

## Liveness & Readiness Probes

First-class Kubernetes probes (/livez, /readyz).

We expose `/livez` and `/readyz` at the logical pipeline level.

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
