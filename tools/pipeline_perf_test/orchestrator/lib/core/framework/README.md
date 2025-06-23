# `core.framework` - Test Orchestration Framework

The `core.framework` package defines the core abstractions and execution model for a modular test orchestration system. It provides a flexible structure to define, organize, run, and report on tests composed of suites, scenarios, and steps, with full support for lifecycle hooks, extensibility, and reporting.

This package is central to the orchestrator's functionality and acts as the execution engine for all test logic.

---

## Overview of Modules

### `element.py` - **Framework Execution Backbone**

Defines the abstract base class `FrameworkElement`, which represents the foundational unit of test execution (e.g., test suite, scenario, or step).

**Key Concepts:**

- **Lifecycle Hooks**: Attach behaviors (setup, teardown, etc.) at any phase using `HookStrategy`.
- **Lifecycle Phases**: Modeled via the `TestLifecyclePhase` enum.
- **Extensibility**: Subclass and override `run()` to define specific execution logic.

> All testable entities in the framework derive from `FrameworkElement`.

---

### `suite.py` - **Test Suite Orchestration**

The `Suite` class manages execution of a collection of tests and the components they depend on.

- Initializes test environments and injects required components.
- Runs each test (typically a `Scenario`) in sequence.
- Provides isolated or shared context for consistent test behavior.

> Suites organize and encapsulate high-level test groupings.

---

### `scenario.py` - **Test Scenarios**

Encapsulates a specific test scenario within a test suite.

- Composed of an ordered list of `Step` instances.
- Supports execution hooks for customization and reporting.
- Represents the logical structure of a single test case.

> Scenarios provide mid-level orchestration: they define *what* to test within a suite.

---

### `step.py` - **Test Steps**

Defines the smallest executable unit in the test framework.

- A `Step` includes a name and an `action` (a callable).
- Executed sequentially within a `Scenario`.

> Steps are atomic actions-unit-level building blocks of scenarios.

---

### `report.py` - **Test Reporting**

Provides the `Report` class for capturing and serializing results from test executions.

**Features:**

- Structured results with metadata and timestamps.
- JSON serialization support for portability and analysis.
- Report aggregation (comparison, timeseries) for insight and trends.

> Reports capture outcomes, enabling traceability and observability across test executions.

---

## How It All Fits Together

```plaintext
Suite
|-- Scenario(s)
    |-- Step(s)
        |-- Action (callable)
```

- `FrameworkElement` is the base for all these layers, enabling common lifecycle hook support.
- Hooks can be applied to any `FrameworkElement` to extend behavior dynamically.
- `Report` collects results from any level for persistence and analysis.

---

## Summary

The `core.framework` module is the heart of the orchestrator's test system, combining flexibility, modularity, and structured execution. Whether you're running a high-level suite of systems tests or granular step-based validations, this framework provides the foundation for consistent and extensible test automation.
