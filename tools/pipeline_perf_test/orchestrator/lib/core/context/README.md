# `contexts/` Folder

This folder contains context classes and related enums used to manage state, data flow, and hook execution throughout the lifecycle of a test suite in the orchestration framework. Contexts provide structured access to runtime information for different stages of test execution, supporting fine-grained control and extensibility through hooks.

## Modules Overview

### `base.py`

Provides a foundational context class:

- **`BaseContext`**: A base class shared across all context types. It defines common fields and utilities needed by more specialized context classes throughout the framework.

---

### `component_hook_context.py`

Defines context structures and enums for **component-level lifecycle hooks**.

- **`HookableComponentPhase`** *(enum)*: Enumerates all stages in a component's lifecycle that support hook execution (e.g., deploy, configure, start, stop, monitor).
- **`ComponentHookContext(BaseContext)`**: A context object passed to component hooks, giving access to the relevant component and test step. Enables strategies to introspect and extend behavior at precise lifecycle points.

**Use Cases:**
- Inject validation or mocking before deployment.
- Collect diagnostics after shutdown.
- Set up monitoring during startup.

---

### `framework_element_contexts.py`

Contains core context classes for managing **execution state** at different levels:

- **`SuiteContext`**: Global context for an entire test suite. Manages shared metadata and registered components.
- **`ScenarioContext`**: Execution context for a single test or scenario. Provides access to suite-level context and stores per-test state.
- **`StepContext`**: Represents a single step in a test. Offers access to test and suite contexts and tracks step-specific metadata.

These contexts ensure consistent component access and metadata tracking throughout a test run.

---

### `framework_element_hook_context.py`

Supports **hook execution for test framework elements** such as suites, scenarios, or steps.

- **`HookableTestPhase`** *(enum)*: Enumerates hookable lifecycle stages for framework elements (e.g., before test, after test).
- **`FrameworkElementHookContext(BaseContext)`**: Context passed to test framework element hooks, offering structured access to lifecycle phase info and the associated test object.

**Use Cases:**
- Run setup logic before a test starts.
- Clean up or validate results after test completion.
- Customize behavior dynamically based on test metadata or phase.

---

## Summary

The `contexts` folder is the backbone of context management in the test orchestration framework. It encapsulates:

- Unified access patterns to components and test metadata.
- Extensibility through hook contexts tied to lifecycle events.
- A structured approach to managing and passing runtime information between different test layers (suite, scenario, step, component).

This organization promotes modularity, introspection, and control across complex, multi-phase test executions.
