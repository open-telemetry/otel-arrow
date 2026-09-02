# Signal Type Router Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:type_router` (`urn:otel:processor:type_router`)
- Feature gate: Default
- Stability: Experimental

## Overview

The signal type router routes OTAP payloads to output ports based on signal
type. It is an exclusive-routing processor: each inbound message selects at
most one downstream output route.

The router recognizes the well-known named output ports `logs`, `metrics`, and
`traces`. It is useful when one pipeline needs to split telemetry by signal
type before sending each signal to specialized downstream processing or
exporters.

## Getting Started

Declare the well-known output ports for the signal types you want to split:

```yaml
type: processor:type_router
outputs:
  - logs
  - metrics
  - traces
config: {}
```

## Configuration

```yaml
type: processor:type_router
outputs:
  - logs
  - metrics
  - traces
config:
  admission_policy:
    on_full: reject_immediately # or "backpressure"
```

Configuration fields:

- `admission_policy.on_full`: handling for a selected named or default route
  whose output channel is full. Defaults to `reject_immediately`.

The signal type router does not require explicit route mappings in its config.
It selects well-known output port names from the message signal type:

- logs use the `logs` output port
- metrics use the `metrics` output port
- traces use the `traces` output port

## Route Selection

For each message, the signal type router first tries the signal-specific named
output port:

- logs prefer `logs`
- metrics prefer `metrics`
- traces prefer `traces`

If the signal-specific named port is connected, it is selected. If that named
port is not connected, the router falls back to the node default output when
one exists.

Fallback only applies when the named port is not wired at all. A named route
that is full or closed remains the selected route and is handled by
selected-route admission.

If neither the signal-specific named port nor a default output exists, the
message is dropped with the historical routing-failure behavior.

## Selected-Route Admission

Once an output route has been selected, the router never awaits the downstream
send in the main router task. This avoids head-of-line blocking: a full or
closed selected route cannot freeze the router and prevent unrelated signal
types from making progress.

The selected route can produce three admission outcomes:

- `Accepted`: the message is forwarded to the selected output.
- `Full`: the output channel is temporarily unable to admit the message.
- `Closed`: the output route is unavailable.

`Closed` always produces an immediate route-local retryable NACK with
`cause = RouteClosed`.

`Full` follows `admission_policy.on_full`:

- `reject_immediately`: emit an immediate route-local retryable NACK with
  `cause = RouteFull`; unrelated healthy signal routes continue to flow.
- `backpressure`: park at most one message per blocked output port, keep
  admitting pdata while at least one selectable route is still making progress,
  close pdata admission only when every selectable route currently has a parked
  full message, and reopen admission once at least one parked route forwards.

Later messages for an already parked route are retryable-NACKed with
`cause = RouteFull`.

## Shutdown Behavior

If the signal type router owns locally parked messages when
`NodeControlMsg::Shutdown` starts:

- every parked message is retryable-NACKed locally
- those NACKs use `cause = NodeShutdown`
- the router clears its parked state instead of leaving messages in a local
  wait path

This applies only to work still owned by the router. Work already admitted to a
downstream channel is outside the router's scope.

## Ack/Nack Propagation

Router-generated NACKs are local to the processor unless an upstream component
has registered a caller subscription for NACK outcomes.

If no upstream frame has `Interests::NACKS`, the NACK is consumed locally for
router telemetry and completion accounting, but no upstream node receives a
`NodeControlMsg::Nack`.

Topic hops are one way to create or suppress such upstream visibility:
`ack_propagation.mode: auto` bridges Ack/Nack by adding the needed caller
subscription across the topic hop, while `disabled` does not.

## Observability

The signal type router exposes a single `signals.decision` measurement metric, broken down by `signal`, `outcome`, and `reason` attributes.

Selected-route NACKs include a machine-readable `NackCause`:

- `RouteFull`
- `RouteClosed`
- `NodeShutdown`

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.type_router`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.type_router.signals.decision` | `{message}` | Number of messages with a given routing outcome. |

The `signals.decision` metric includes `signal`, `outcome`, and `reason` attributes to describe the terminal routing result:

| Terminal routing result | `outcome` | `reason` |
| --- | --- | --- |
| Named signal output accepted | `success` | `named_route` |
| Default accepted because the named output is unwired | `success` | `default_route_unwired` |
| Selected output is full | `refused` | `route_full` |
| Selected output is closed | `refused` | `route_closed` |
| Neither the named nor default output exists | `failure` | `no_available_route` |
| Shutdown cancels parked work | `failure` | `node_shutdown` |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- Recognized named output ports are `logs`, `metrics`, and `traces`.
- If a signal-specific named port is not connected, the router can use the
  default output route.
- `backpressure` parks at most one message per blocked output port.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
