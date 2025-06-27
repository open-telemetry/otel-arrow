# `step_actions`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
|-----------|--------|-------|--------------|----------------------|
| `component_action` | `lib.impl.actions.component_action` | `ComponentAction` | `ComponentActionConfig` | Step action implementation for executing a lifecycle phase on a named component |
| `multi_component_action` | `lib.impl.actions.multi_component_action` | `MultiComponentAction` | `MultiComponentActionConfig` | Step action that executes a specified lifecycle phase on one or more components |
| `wait` | `lib.impl.actions.wait_action` | `WaitAction` | `WaitActionConfig` | Step action that introduces a delay during test execution |
| `update_component_strategy` | `lib.impl.actions.update_component_strategy` | `UpdateComponentStrategyAction` | `UpdateComponentStrategyConfig` | Step action that applies updates to a strategy configuration of a managed component |

---

## `component_action`

**Class**: `lib.impl.actions.component_action.ComponentAction`

**Config Class**: `lib.impl.actions.component_action.ComponentActionConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Step action implementation for executing a lifecycle phase on a named component.

This class is executed at runtime during a test step, and it:

- Resolves the component by name from the test context.
- Verifies the component supports the requested lifecycle phase.
- Executes the corresponding method on the component, passing in the current context.
"""
```

**Example YAML:**

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
    - name: Start Load Generator
      action:
        component_action:
          phase: start
          target: load-generator
```

## `multi_component_action`

**Class**: `lib.impl.actions.multi_component_action.MultiComponentAction`

**Config Class**: `lib.impl.actions.multi_component_action.MultiComponentActionConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Step action that executes a specified lifecycle phase on one or more components.

This action retrieves the target components (either all in the context or a specified subset)
and invokes the method corresponding to the configured lifecycle phase on each.

Attributes:
    config (MultiComponentActionConfig): Configuration object specifying the phase and targets.
"""
```

**Example YAML:**

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Deploy All
        action:
          multi_component_action:
            phase: deploy
            targets:
              - load-generator
              - otel-collector
              - backend-service
```

## `wait`

**Class**: `lib.impl.actions.wait_action.WaitAction`

**Config Class**: `lib.impl.actions.wait_action.WaitActionConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Step action that introduces a delay during test execution.

This action simply waits for the duration specified in the configuration before proceeding.
Useful for introducing timing gaps or waiting for external processes to settle.

Attributes:
    config (WaitActionConfig): Configuration containing the delay duration in seconds.
"""
```

**Example YAML:**

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
    - name: Wait For Test
      action:
        wait:
          delay_seconds: 3
```

## `update_component_strategy`

**Class**: `lib.impl.actions.update_component_strategy.UpdateComponentStrategyAction`

**Config Class**: `lib.impl.actions.update_component_strategy.UpdateComponentStrategyConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Step action that applies updates to a strategy configuration of a managed component.

This action merges partial updates into the existing component configuration and
rebuilds relevant strategies such as deployment, monitoring, execution, or configuration.

Attributes:
    config (UpdateComponentStrategyConfig): The configuration specifying the target
        component and the partial updates to apply.
"""
```

**Example YAML:**

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Reconfigure Otel Collector Docker Volume
        action:
          update_component_strategy:
            target: otel-collector
            deployment:
              docker:
                volumes:
                  - ./configs/test_batch_sizes/component_configs/collector-config-batch-10k.yaml:/etc/otel/collector-config.yaml:ro
```
