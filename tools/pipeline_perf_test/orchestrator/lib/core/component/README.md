# Component Module

This module defines the abstract base class `Component`, used in a test
orchestrator framework to manage the lifecycle of testable components.
It offers a standardized structure and hook system to implement and trace
component behavior across various lifecycle phases.

## Overview

- **Component** is an abstract base class that:
  - Manages lifecycle phases: `configure`, `deploy`, `start`, `stop`,
    `destroy`, `start_monitoring`, `stop_monitoring`, and
    `collect_monitoring_data`.
  - Supports registration and execution of hooks around these phases.
  - Supports observability via OpenTelemetry tracing.
  - Provides runtime data management.

- **ComponentPhase** is an enum that defines the primary lifecycle phases:
  - `CONFIGURE`
  - `DEPLOY`
  - `START`
  - `STOP`
  - `DESTROY`
  - `START_MONITORING`
  - `STOP_MONITORING`

## Key Features

### Components, Targets, and Endpoints

A **component** owns a logical workload's lifecycle. Its **targets** identify
instances; each exposes named **endpoints**, such as `metrics`.

```text
component -> targets -> named endpoints -> monitor
              identity   URL or fetch(timeout) -> text
```

```python
from lib.core.component import ComponentTarget

component.set_targets(
    [ComponentTarget("instance-a", {"metrics": "http://localhost:8888/metrics"})],
    validate=validate_access,
)
```

Names must be nonempty and unique per component. Endpoint and identity-attribute
mappings are read-only snapshots. Endpoints are URLs or objects implementing
`fetch(timeout: float) -> str`; the provider owns discovery and access.

`get_targets(timeout)` runs optional provider validation, which must honor the
timeout and raise on invalid identity or access. Targets stay fixed during
monitoring; stop and re-adopt to rediscover.

```text
adopt -> publish -> monitor -> stop -> release access
```

For adopted components, `ExistingStrategy.release(component, ctx)` closes
provider-owned access while leaving caller-owned workloads running. Destroy,
re-adopt and suite cleanup stop monitoring first; failed shutdown prevents
release. Cleanup also covers partial adoption failure.

Existing Prometheus configurations use explicit endpoint URLs. Target publication
is currently exercised by backend-neutral tests; the Kubernetes follow-up adds
the first runtime provider.

### Lifecycle Management

Each component subclass must implement the following abstract methods:

- `_configure(ctx)`
- `_deploy(ctx)`
- `_start(ctx)`
- `_stop(ctx)`
- `_destroy(ctx)`
- `_start_monitoring(ctx)`
- `_stop_monitoring(ctx)`
- `_collect_monitoring_data(ctx)`

These are invoked by corresponding public methods that wrap them with tracing spans.

### Hooks System

Hooks can be registered for any lifecycle phase using:

```python
component.add_hook(HookableComponentPhase.PRE_DEPLOY, my_hook)
```

Hooks are executed with context tracking, error handling, and tracing.

### Tracing

Lifecycle methods and hooks can emit OpenTelemetry traces using the provided StepContext.

### Runtime Support

Components use a runtime object to store and manage state via:

- get_or_create_runtime(namespace, factory)
- set_runtime_data(namespace, data)

### Classes

- ComponentPhase: Enum representing lifecycle phases.
- Component: Abstract base class for defining and managing test components.

### Integration Notes

Integrates with opentelemetry.trace for observability.

Uses a hook strategy pattern from strategies.hook_strategy.

Depends on internal context and runtime modules (context.base,
context.test_contexts, etc.).

### Example Usage

```python
class MyComponent(Component):
    def _configure(self, ctx): ...
    def _deploy(self, ctx): ...
    def _start(self, ctx): ...
    def _stop(self, ctx): ...
    def _destroy(self, ctx): ...
    def _start_monitoring(self, ctx): ...
    def _stop_monitoring(self, ctx): ...
    def _collect_monitoring_data(self, ctx): ...
```

This file is the foundation for defining orchestrated test
components and provides extensibility and instrumentation
for complex testing scenarios.
