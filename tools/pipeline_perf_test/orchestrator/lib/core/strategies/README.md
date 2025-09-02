# `core.strategies` - Pluggable Strategy Interfaces

The `core.strategies` package defines a modular and extensible strategy pattern
for key behaviors in a load generator testbed and test orchestration
environment. It separates concerns like configuration, deployment, execution,
monitoring, hooks, and reporting into independently pluggable components.

These strategies provide clear, interchangeable abstractions for customizing
the behavior of testbed components without modifying their core logic.

---

## Overview of Modules

### `base.py` - **Strategy Foundation**

Defines common base classes for all strategies.

**Classes:**

- `BaseStrategyConfig`: Pydantic model that holds shared configuration parameters.
- `BaseStrategy`: Abstract base class for all strategies.

> All strategy types inherit from these foundational classes.

---

### `configuration_strategy.py` - **Component Configuration Strategies**

Defines how a component is configured before execution.

**Use Cases:**

- Writing local config files.
- Generating Kubernetes manifests.
- Fetching remote configuration.

**Classes:**

- `ConfigurationStrategyConfig`: Base config model for configuration strategies.
- `ConfigStrategy`: Abstract base with a required `start()` method.

> Enables flexible, backend-agnostic configuration of components.

---

### `deployment_strategy.py` - **Component Deployment Strategies**

Defines the interface for deploying and managing component lifecycles.

**Use Cases:**

- Running containers with Docker.
- Deploying to Kubernetes.
- Launching local processes.

**Classes:**

- `DeploymentStrategyConfig`: Config model for deployment strategies.
- `DeploymentStrategy`: Abstract base for deployment behavior.

> Enables swappable deployment backends across environments.

---

### `execution_strategy.py` - **Component Execution Strategies**

Encapsulates runtime behavior for a component during a test.

**Use Cases:**

- Generating load (e.g., OTLP exporters).
- Receiving load (e.g., receivers, sinks).
- Coordinating workloads in a scenario.

**Classes:**

- `ExecutionStrategyConfig`: Config model for execution strategies.
- `ExecutionStrategy`: Abstract base for starting/stopping workload execution.

> Decouples *what* a component does from *how* and *where* it runs.

---

### `hook_strategy.py` - **Lifecycle Hook Strategies**

Defines injectable hooks that run during specific phases of a test lifecycle.

**Use Cases:**

- Setup/teardown routines.
- Validation steps.
- External service integrations.

**Classes:**

- `HookStrategyConfig`: Config passed into hook strategies.
- `HookStrategy`: Abstract base class to execute hook logic.

> Hooks provide runtime extension points without modifying core logic.

---

### `monitoring_strategy.py` - **Component Monitoring Strategies**

Provides interfaces for monitoring test components.

**Use Cases:**

- Gathering metrics or logs.
- Watching system health or performance.
- Integrating with observability platforms.

**Classes:**

- `MonitoringStrategyConfig`: Config model for monitoring strategies.
- `MonitoringStrategy`: Abstract base for monitoring integration.

> Keeps observability concerns modular and reusable.

---

### `reporting_hook_strategy.py` - **Test Reporting Pipelines**

Implements pluggable pipelines for formatting and exporting test results.

**Components:**

- `ReportFormatter`: Formats raw reports into desired output (e.g., JSON, HTML).
- `DestinationWriter`: Writes formatted reports to a destination (e.g., file, database).
- `ReportOutputPipeline`: Composes formatter and writer into a processing chain.
- `ReportingHookStrategy`: Specialized `HookStrategy` for injecting reporting behavior.

> Supports extensible, format-agnostic reporting workflows.

---

## Strategy Pattern Summary

Each strategy type encapsulates one domain of behavior:

```plaintext
+--------------------+------------------------------+
| Strategy Type      | Purpose                      |
+--------------------+------------------------------+
| ConfigStrategy     | Setup component configuration|
| DeploymentStrategy | Deploy component to runtime  |
| ExecutionStrategy  | Run workload inside testbed  |
| HookStrategy       | Inject logic into lifecycle  |
| MonitoringStrategy | Collect observability data   |
| ReportingStrategy  | Format & output test reports |
+--------------------+------------------------------+
```

This design promotes:

- **Separation of concerns** between subsystems.
- **Plug-and-play behavior** using abstract interfaces.
- **Customizability** for different environments and needs.

---

## Summary

The `core.strategies` package enables a robust, modular architecture for
building test orchestration systems that are adaptable, extensible, and
environment-agnostic. By leveraging strategy patterns across major
operational domains, it supports sophisticated test workflows while
maintaining clean separation and composability.
