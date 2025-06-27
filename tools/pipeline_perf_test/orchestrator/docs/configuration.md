# Test Framework Orchestrator Configuration Guide

This document describes the configuration schema for defining test
suites in the orchestrator using a YAML-based format.

---

## Table of Contents

1. [Test Suite Overview](#test-suite-overview)
2. [Components](#components)
   - [Hooks](#hooks)
   - [Configuration Strategy](#configuration-strategy)
   - [Deployment Strategy](#deployment-strategy)
   - [Execution Strategy](#execution-strategy)
   - [Monitoring Strategy](#monitoring-strategy)
   - [Strategy Reconfiguration](#strategy-reconfiguration)
3. [Tests](#tests)
   - [Test Steps](#test-steps)
4. [Test Suite Hooks](#test-suite-hooks)
   - [Reports](#reports)
5. [Error Handling](#error-handling)

---

## Test Suite Overview

A **test suite** is the top-level configuration unit in the orchestrator. It
defines everything needed to execute one or more test scenarios, including the
system components involved, how those components are deployed and monitored, the
test logic to execute, and any reporting or automation hooks to run before or
after key phases.

The suite is described in YAML format and contains four primary sections:

- **`name`**: A human-readable label for identifying the suite.
- **`components`**: The services or systems under test (e.g., load generators,
   collectors, backends).  Each component defines its deployment, execution,
   and monitoring strategies.
- **`tests`**: A list of ordered test scenarios, each made up of steps like deploying
   components, starting monitoring, executing load, and generating reports.
- **`hooks`**: Lifecycle hooks that run before or after all test scenarios in the
   suite-typically used for reporting, cleanup, or global setup.

### Basic Structure

```yaml
name: Default Test Suite

components:
  load-generator:
    deployment: { ... }
    execution: { ... }
    monitoring: { ... }

tests:
  - name: Example Test
    steps: [ ... ]

hooks:
  run:
    post:
      - pipeline_perf_report: { ... }
```

This structure provides a modular way to define reusable and automated test
environments, with clear separation between setup, execution, and analysis phases.

Each of the fields above is described in greater detail in the following sections.

## Components

**Components** represent the individual systems, services, or tools that are
involved in your test environment. This can include things like load generators,
telemetry collectors, backend services, or anything else that participates
in the test lifecycle.

Each component is defined under the `components` section by name (e.g.,
`load-generator`) and includes configuration for how it should be:

- **Configured** (optional settings or initialization logic)
- **Deployed** (e.g., using Docker, Kubernetes, processes)
- **Executed** (how and when it performs its role in the test)
- **Monitored** (how the orchestrator collects metrics or health signals)
- **Hooked** into test lifecycle phases (for pre/post actions during `deploy`,
   `start`, `stop`, etc.)

Components are reusable across multiple test scenarios and form the backbone of
the testbed.

### Example Structure

```yaml
components:
  load-generator:
    hooks: {}
    configuration: {}
    deployment:
      docker:
        image: load_generator:latest
        network: testbed
        command: ["--serve"]
        environment:
          - OTLP_ENDPOINT=otel-collector:4317
        ports:
          - "5001:5001"
    execution:
      pipeline_perf_loadgen:
        threads: 1
        batch_size: 5000
    monitoring:
      docker_component:
        interval: 1
      prometheus:
        endpoint: http://localhost:5001/metrics
```

### Key Sections Within a Component

- **`hooks`**: Optional. Define custom behavior to run before or after specific
   phases (`deploy`, `start`, `stop`, etc.).
- **`configuration`**: Optional. Placeholder for configuration logic, if needed
   by certain plugins or strategies.
- **`deployment`**: Optional. Specifies how the component is brought up -
   typically using a strategy like `docker`, `process`, or `k8s`.
- **`execution`**: Optional. Defines what the component *does* when it's told
   to `start` or `stop`, e.g., sending traffic, consuming messages, etc.
- **`monitoring`**: Optional. Lists one or more strategies (like `prometheus`,
   `docker_component`) for observing component behavior during the test.

Each of these areas is explored in depth in the next sections.

## Hooks

**Hooks** are a powerful mechanism that allow you to attach custom logic before
or after key lifecycle phases in a test suite, test scenario, step, or
component. They act as automation extension points - useful for setup,
teardown, validation, logging, triggering external actions, or recording
important test events.

Hooks can be defined at multiple levels:

- **Suite-level**: Run before or after all scenarios.
- **Scenario-level**: Run before or after a specific scenario.
- **Step-level**: Run before or after an individual step.
- **Component-level**: Run before or after component phases like `deploy`,
   `start`, `stop`, etc.

Hooks are always associated with a **phase** and can be configured to:

- Run **before** (`pre`) or **after** (`post`) the phase
- Be additive (default behavior) or override existing hooks via `*_strategy: replace`

---

### Hook Structure Overview

```yaml
hooks:
  run:
    pre:
      - some_hook_type:
          key: value
    post:
      - another_hook:
          config: yes
```

- **`run`**: The common execution phase for hooks. Can be replaced with specific
   phase names on components (e.g., `deploy`, `start`, etc.).
- **`pre` / `post`**: Define when the hook should run relative to the phase.
- **`pre_strategy` / `post_strategy`**: Optional. Use `replace` to overwrite
   default hooks for that phase.

---

### Example: Suite-Level Reporting Hooks

```yaml
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReport - Max Rate
          output:
            - format:
                template: {}
              destination:
                console: {}
          between_events:
            start:
              name: test_framework.test_start
              attributes:
                test.name: Test Max Rate Logs
            end:
              name: test_framework.test_end
              attributes:
                test.name: Test Max Rate Logs

      - process_report:
          name: Process - Max Rate
          components:
            - load-generator
            - otel-collector
            - backend-service
          between_events:
            start:
              name: observation_start
              attributes:
                test.name: Test Max Rate Logs
            end:
              name: observation_stop
              attributes:
                test.name: Test Max Rate Logs
```

These hooks run **after all scenarios complete**, generating performance and
process-level reports using events recorded during test execution.

---

### Example: Scenario-Level Hook with Error Handling

```yaml
hooks:
  run:
    pre:
      - raise_exception:
          message: "This is an exceptional exception."
          on_error:
            continue: true
    pre_strategy: replace
```

- This hook intentionally raises an exception before the scenario starts.
- The `on_error.continue: true` flag ensures the test proceeds even if the hook fails.
- The `pre_strategy: replace` line ensures this hook replaces any defaults for
   the `pre` phase.

---

### Common Hook Types

See [hook_strategies](plugins/hook_strategies.md) for a current list of hooks.

Hooks are a core part of test orchestration. Use them to modularize test
setup/teardown, capture key events, inject test failures, and more.

## Configuration Strategy

The **configuration strategy** defines how a component prepares any required
configuration files or settings before being deployed or started.
This section is currently a **placeholder** in most setups - but it's designed
to support dynamic configuration generation and external resource loading in
the future.

---

### What It's For

The configuration strategy allows components to:

- Render templated config files before deployment
- Download configuration from a remote service or repo
- Validate settings prior to startup
- Mount or inject dynamic runtime parameters

It provides a structured way to automate component preparation in complex or
dynamic environments.

---

### Current Status

Right now, `configuration:` is typically left empty:

```yaml
components:
  load-generator:
    configuration: {}
```

However, the intention is to support plugin-based strategies similar to
deployment, execution, and monitoring. You might eventually see something like:

```yaml
configuration:
  jinja_template:
    template_path: ./configs/loadgen.j2
    output_path: /tmp/rendered-config.yaml
    variables:
      otlp_endpoint: otel-collector:4317
```

Or:

```yaml
configuration:
  remote_fetch:
    url: https://config-service.myorg.com/loadgen/config.yaml
    destination: /etc/service/config.yaml
```

---

### Potential Lifecycle Phase

Configuration strategies would run in the `configure` phase.

```yaml
- name: Configure All
  action:
    multi_component_action:
      phase: configure
```

---

### Configuration Strategy - Summary

- Future plugins may support templating, config downloading, or injecting
   environment-specific values.
- Tied to the `configure` phase.
- Follows the same strategy pattern as other component systems.

This section will evolve as plugins and use cases are added. For now, you can
safely omit or leave it empty unless you're extending the orchestrator with
custom logic.

## Deployment Strategy

The **deployment strategy** defines *how* a component is launched into the test
environment -
whether that means starting a Docker container, running a local process,
launching a Kubernetes pod, or using a custom strategy.

This is configured under the `deployment` key of each component and is required
for most components
to function properly during test execution.

See [deployment_strategies](plugins/deployment_strategies.md) for an up to date
list of strategies.

---

### Why Deployment Strategy Matters

Deployment strategies allow your test framework to manage the full lifecycle of
each component. This includes:

- Starting services before the test (`deploy`)
- Cleaning them up afterward (`destroy`)
- Running isolated test environments in CI/CD pipelines
- Simulating realistic infrastructure setups (e.g. networked microservices)

---

### Example: Docker Deployment

```yaml
components:
  load-generator:
    deployment:
      docker:
        image: load_generator:latest
        network: testbed
        command: ["--serve"]
        environment:
          - OTLP_ENDPOINT=otel-collector:4317
        ports:
          - "5001:5001"
```

- **`docker`**: The name of the deployment strategy plugin (in this case, Docker).
- **`image`**: The container image to run.
- **`network`**: The virtual network to connect to.
- **`command`**: The entrypoint command override.
- **`environment`**: Environment variables injected into the container.
- **`ports`**: Ports to map from the container to the host system.

When a test step triggers the `deploy` phase for this component, the orchestrator
uses this config to bring up the container.

---

### Test Step Example

```yaml
- name: Deploy All
  action:
    multi_component_action:
      targets:
        - load-generator
        - otel-collector
      phase: deploy
```

This step invokes the deployment strategy of each listed component (e.g. Docker).

---

### Lifecycle Phases

| Phase     | Description                                 |
|-----------|---------------------------------------------|
| `deploy`    | Starts the component                      |
| `destroy`   | Cleans up the component                   |

These phases are accessible via test steps and can have hooks for additional automation.

---

### Deployment Strategy -Summary

- The deployment strategy tells the orchestrator how to start and stop a component.
- It is required for most testable components.
- Strategy is plugin-based (`docker`, `process`, `k8s`, etc.).
- Works in conjunction with test steps that invoke the `deploy` or `destroy` phases.

## Execution Strategy

The **execution strategy** defines how a component performs its role during a
test - typically in response to the `start` and `stop` phases.
Execution strategies are plugin-driven and determine what happens when a
component is "started" or "stopped" within a test step.

Execution strategies are component-specific and are configured under the
`execution` key of a component definition.

See [execution_strategies](plugins/execution_strategies.md) for an up to date
list of available strategies.

---

### Why Execution Matters

The execution strategy is what turns a passive service (e.g. a deployed container)
into an *active participant* in a test. For example:

- A **load generator** might send traffic during the `start` phase.
- A **telemetry forwarder** might simulate logs or metrics.
- A **custom agent** might run scripted behavior in response to test triggers.

---

### Example: Load Generator Execution

```yaml
components:
  load-generator:
    execution:
      pipeline_perf_loadgen:
        threads: 1
        batch_size: 5000
```

- **`pipeline_perf_loadgen`**: The plugin used to execute load.
- **`threads`**: Number of concurrent execution threads.
- **`batch_size`**: Size of each emitted data batch.

When this component is triggered in a test step like:

```yaml
- name: Start Load Generator
  action:
    component_action:
      phase: start
      target: load-generator
```

The orchestrator invokes the `start` logic of `pipeline_perf_loadgen`
with the defined parameters.

---

### Plugin-Based Strategy

Execution strategies are modular. You can plug in different behaviors
for different components by changing the strategy key:

```yaml
execution:
  my_custom_plugin:
    param_a: value1
    param_b: value2
```

Only one execution strategy should be defined per component.

---

### Execution Strategy Phases

| Phase | Description                           |
|-------|---------------------------------------|
| `start` | Begins the execution behavior (e.g. send load) |
| `stop`  | Ends the execution behavior          |

These phases are tied to `component_action` test steps that reference the component.

---

### Execution Strategy - Summary

- Defined under each component's `execution` block.
- Specifies what the component *does* during a test.
- Plugin-based: behavior varies depending on strategy.
- Tied directly to `start` and `stop` phases in test steps.

Next, we'll explore **Deployment Strategy**, which governs how components
are brought online and made available during testing.

## Monitoring Strategy

The **monitoring strategy** defines how the orchestrator observes a component
during test execution - capturing metrics, health signals, resource usage, or
custom telemetry. It enables visibility into how components behave under load
and is essential for reporting and performance validation.

Monitoring is defined per component under the `monitoring` key. Each component
can use one or more monitoring strategies simultaneously.

---

### Why Monitoring Strategy Matters

Monitoring allows the test framework to:

- Collect runtime metrics (CPU, memory, logs, custom telemetry)
- Determine component readiness and health
- Feed data into reports (like `process_report` or `pipeline_perf_report`)
- Support steady-state validation

Without monitoring, the test suite is blind to how systems perform under test load.

---

### Example: Multiple Monitoring Strategies

```yaml
components:
  load-generator:
    monitoring:
      docker_component:
        interval: 1
      prometheus:
        endpoint: http://localhost:5001/metrics
```

- **`docker_component`**: Polls Docker stats (CPU, memory, etc.) at 1-second intervals.
- **`prometheus`**: Scrapes metrics from a Prometheus-compatible HTTP endpoint.

This configuration allows the orchestrator to track both low-level resource usage
and high-level application telemetry.

---

### Monitoring Phases

Monitoring is controlled via test steps that use these phases:

| Phase            | Description                          |
|------------------|--------------------------------------|
| `start_monitoring` | Begins collecting metrics           |
| `stop_monitoring`  | Stops metric collection             |

These phases are usually invoked through `multi_component_action` steps in a scenario:

```yaml
- name: Monitor All
  action:
    multi_component_action:
      phase: start_monitoring

- name: Stop Monitoring All
  action:
    multi_component_action:
      phase: stop_monitoring
```

Monitoring should typically begin just before load starts and stop after all
components have finished executing.
Event windows can be used to further isolate reporting to relevant windows
(e.g. after steady-state is reached).

---

### Supported Monitoring Strategies

See supported [monitoring_strategies](plugins/monitoring_strategies.md) for an
up to date list of plugins.

---

### Event-Based Time Windows

Monitoring data is often used in reports that define analysis windows using
recorded events:

```yaml
between_events:
  start:
    name: observation_start
    attributes:
      test.name: Test Max Rate Logs
  end:
    name: observation_stop
    attributes:
      test.name: Test Max Rate Logs
```

These event names are typically emitted via `record_event` hooks in test steps.

---

### Monitoring Phases - Summary

- Monitoring strategies define how the orchestrator collects runtime metrics.
- Each component can use multiple strategies simultaneously.
- Start/stop phases control data collection during test scenarios.
- Data is fed into post-test reports for performance and behavior analysis.

With monitoring in place, your test suite can observe, validate, and report
on system behavior under simulated or real test loads.

## Tests

The **`tests`** section defines one or more **scenarios** to be executed in
sequence as part of the test suite. Each scenario represents a self-contained
test flow, consisting of:

- A **name** to identify the scenario
- Optional **hooks** to run before/after the scenario
- An ordered list of **steps**, which execute actions across components
or the test system

Tests allow you to model and automate complex workflows like:

- Deploying and tearing down environments
- Starting and stopping load generators
- Monitoring system behavior
- Validating steady state before collecting metrics
- Generating and recording test-specific events

Each scenario runs independently, but all tests within a suite can share
the same components.

---

### Tests - Example Structure

```yaml
tests:
  - name: Test Max Rate Logs
    hooks: { ... }
    steps:
      - name: Deploy All
        action: { ... }
      - name: Start Load Generator
        action: { ... }
      ...
```

- **`name`**: Unique name for the test scenario
- **`hooks`**: Optional hooks that run before/after the scenario
- **`steps`**: A list of ordered operations that define the test flow

---

### Test-Level Hooks

Just like suite-level hooks, each scenario can define `hooks` to execute
logic before or after the scenario starts.

```yaml
hooks:
  run:
    pre:
      - raise_exception:
          message: "This is an exceptional exception."
          on_error:
            continue: true
    pre_strategy: replace
```

This hook will run before the scenario starts and simulate a failure.
`on_error.continue` ensures the test continues.

---

### Test Steps

Steps are the atomic units of execution within a test. Each step performs an action
(deploying, waiting, starting component(s), etc.) and can optionally have
its own `hooks`.

#### Example Step Types

| Action Plugin             | Purpose                                      |
|---------------------------|----------------------------------------------|
| `multi_component_action`  | Run a phase (e.g., deploy) on multiple components |
| `component_action`        | Run a phase on a single component            |
| `wait`                    | Delay execution for a specified time         |

See the full list of [supprted step action types](plugins/step_actions.md).

---

#### Test Step Hooks

### Scenario-Level Hooks

Just like suite and test-level hooks, each step can define `hooks` to execute
logic before or after the step starts.

```yaml
hooks:
  run:
    pre:
      - raise_exception:
          message: "This is an exceptional exception."
          on_error:
            continue: true
    pre_strategy: replace
```

This hook will run before the step starts and simulate a failure.
`on_error.continue` ensures the test continues.

### Example Steps Breakdown

#### 1. Deploy All Components

```yaml
- name: Deploy All
  action:
    multi_component_action:
      targets:
        - load-generator
        - otel-collector
        - backend-service
      phase: deploy
```

- Runs the `deploy` phase on the listed components.

#### 2. Start Monitoring

```yaml
- name: Monitor All
  action:
    multi_component_action:
      phase: start_monitoring
```

- Starts monitoring on all components that support it.

#### 3. Wait for System to Settle

```yaml
- name: Wait For Otel
  action:
    wait:
      delay_seconds: 3
```

- Simple delay to give components time to initialize.

#### 4. Start Load Generation

```yaml
- name: Start Load Generator
  action:
    component_action:
      phase: start
      target: load-generator
```

- Triggers the component's execution strategy for the `start` phase.

#### 5. Observe System Behavior

```yaml
- name: Wait For Test
  action:
    wait:
      delay_seconds: 5
  hooks:
    run:
      pre:
        - record_event:
            name: resource_observation_start
      post:
        - record_event:
            name: resource_observation_stop
```

- During this wait period, the system is under load.
- Events are recorded before and after to mark observation intervals.

#### 6. Stop Load Generator

```yaml
- name: Stop Load Generator
  action:
    component_action:
      phase: stop
      target: load-generator
```

#### 7. Graceful Drain Time

```yaml
- name: Wait For Drain
  action:
    wait:
      delay_seconds: 3
```

- Allows metrics/logs to finish flushing before teardown.

#### 8. Stop Monitoring

```yaml
- name: Stop Monitoring All
  action:
    multi_component_action:
      phase: stop_monitoring
```

#### 9. Destroy Environment

```yaml
- name: Destroy All
  action:
    multi_component_action:
      phase: destroy
```

- Tears down all components.

---

### Tests - Summary

The `tests` section defines repeatable, modular test scenarios. Each test:

- Can include its own setup, execution, and teardown flow
- Interacts with defined components
- Uses steps and hooks to control timing, validation, and automation

## Strategy Reconfiguration

In addition to defining static strategies for each component (execution,
deployment, etc.), the orchestrator supports **runtime reconfiguration**
of a component's strategies during a test scenario.

This is useful when you want to:

- Test multiple configurations of the same component in one test
- Switch a component's behavior mid-test (e.g., change its config or execution mode)
- Inject configuration variants (OTLP vs OTAP, local vs remote, etc.)

---

### Strategy Reconfiguration - How It Works

Use the `update_component_strategy` step action to override one or more
strategy blocks for a component. This applies the new configuration
**immediately**, and any future phases (e.g., `deploy`, `start`, `execution`)
will use the updated config. No immediate impact on the running component
will be observed (e.g. the load generator doesn't change the parameters
of the load it's sending unless you re-run the "start" phase on it).

---

### Example: Swap Config on a Collector

```yaml
- name: Reconfigure Sender Collector
  action:
    update_component_strategy:
      target: otel-collecto
      deployment:
        docker:
          volumes:
            - ../configs/test_otlp_vs_otap/component_configs/sender-side-collector-otap.yaml:/etc/otel/collector-config.yaml:ro
```

- **`target`**: The name of the component to reconfigure.
- **`deployment`**: In this case, overrides the Docker deployment
   strategy to mount a new config file.
- You can also override `execution`, `monitoring`, or `configuration`
   in the same step.

This allows for powerful test flows where a component is torn down,
reconfigured, and restarted (either in the same Scenario or in a different Scenario
in the same Suite)

```yaml
- name: Stop Sender Collector
  action:
    component_action:
      target: sender-side-collector
      phase: stop

- name: Reconfigure Sender Collector
  action:
    update_component_strategy:
      target: sender-side-collector
      deployment:
        docker:
          volumes:
            - new-config.yaml:/etc/config.yaml:ro

- name: Redeploy Sender Collector
  action:
    component_action:
      target: sender-side-collector
      phase: deploy

- name: Restart Collector
  action:
    component_action:
      target: sender-side-collector
      phase: start
```

---

### Strategy Update Notes

- Only the provided keys are replaced - other parts of the component remain unchanged.
- Reconfiguration is **in-memory**: it's not persisted across test scenarios.
- Typically used between `stop` and `deploy` phases for a clean transition.

---

### Strategy Update Summary

- `update_component_strategy` lets you modify component strategies at runtime.
- Useful for A/B testing, config swaps, or adaptive test flows.
- Can modify any strategy type: `deployment`, `execution`, `monitoring`, or `configuration`.

This gives you fine-grained control over component behavior across
different test phases.

## Test Suite Hooks

**Test suite hooks** define automation logic that runs once before or
after all test scenarios in the suite. These are useful for:

- Generating test reports
- Performing global cleanup
- Emitting start/end events
- Integrating with external systems

They live under the top-level `hooks` key in the suite and follow the
standard `pre` / `post` hook structure.

---

### Test Suite Hooks - Basic Structure

```yaml
hooks:
  run:
    pre:  # Runs before any test scenarios
      - some_setup_hook: {}
    post:  # Runs after all test scenarios
      - some_teardown_or_report_hook: {}
```

- **`run`**: Indicates the global test suite execution phase.
- **`pre`** and **`post`**: Define actions before and after the suite runs.
- Common use cases for `pre` suite hooks include building containers / binaries,
   provisioning infrastructure, etc
- Common use cases for `post` hooks include reporting, metrics analysis,
   and result exporting.

---

## Reports

Reports are a common use of `post` suite-level hooks. They analyze the metrics
and events generated during the tests and output human-readable results to the
console, files, or other destinations.

There are two main report strategies available at the time of writing:

### 1. `pipeline_perf_report`

Generates performance metrics based on defined time windows using recorded
events (e.g., `test_framework.test_start` / `test_framework.test_end`).

```yaml
- pipeline_perf_report:
    name: PerfReport - Max Rate
    output:
      - format:
          template: {}  # Uses a Jinja2 template for formatting
        destination:
          console: {}   # Writes to console
    between_events:
      start:
        name: test_framework.test_start
        attributes:
          test.name: Test Max Rate Logs
      end:
        name: test_framework.test_end
        attributes:
          test.name: Test Max Rate Logs
```

- **`name`**: Identifier for the report.
- **`output`**: Defines format (template plugin) and destination
   (e.g., console, file).
- **`between_events`**: Determines the start and end window for analysis.

---

### 2. `process_report`

Focuses on low-level process metrics (CPU, memory, etc.) for selected
components, within a custom observation window (typically marked by `record_event`
hooks in test steps).

```yaml
- process_report:
    name: Process - Max Rate
    components:
      - load-generator
      - otel-collector
      - backend-service
    between_events:
      start:
        name: observation_start
        attributes:
          test.name: Test Max Rate Logs
      end:
        name: observation_stop
        attributes:
          test.name: Test Max Rate Logs
```

- **`components`**: The services being monitored.
- **`between_events`**: Same structure as above, but using custom event names.

---

### Combined Example

Here's how both reports might be defined together in the suite-level `post` hooks:

```yaml
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReport - Max Rate
          output:
            - format:
                template: {}
              destination:
                console: {}
          between_events:
            start:
              name: test_framework.test_start
              attributes:
                test.name: Test Max Rate Logs
            end:
              name: test_framework.test_end
              attributes:
                test.name: Test Max Rate Logs

      - process_report:
          name: Process - Max Rate
          components:
            - load-generator
            - otel-collector
            - backend-service
          between_events:
            start:
              name: observation_start
              attributes:
                test.name: Test Max Rate Logs
            end:
              name: observation_stop
              attributes:
                test.name: Test Max Rate Logs
```

---

### Suite-level hooks Summary

Suite-level hooks:

- Run once across the entire test suite
- Are often used for **reporting**, building containers,
   provisioning infrastructure, etc

With hooks and reporting covered, your test suite can now execute
full lifecycle automation from deployment to results output.

## Error Handling

The orchestrator includes a flexible error handling mechanism that can be applied
at **any phase** of test execution:

- Suite-level hooks
- Scenario-level hooks
- Step-level actions
- Component phases (`deploy`, `start`, `stop`, etc.)

This allows tests to fail fast, retry automatically, or continue execution in
the face of transient or expected errors - all configurable per context.

---

### Error Handling Schema

The error behavior is defined using the `on_error` config block:

```yaml
on_error:
  retries: 2
  retry_delay_seconds: 5
  continue: true
```

| Field                | Type      | Description                                                                 |
|----------------------|-----------|-----------------------------------------------------------------------------|
| `retries`            | `int`     | Number of times to retry the failed phase or hook (default: `0`)           |
| `retry_delay_seconds`| `int`     | Time (in seconds) to wait between retries (default: `10`)                  |
| `continue`           | `bool`    | If `true`, execution will continue even after the final failure (default: `false`) |

---

### Where It Can Be Used

You can apply `on_error` to all strategy types (hooks, deployment,
configuration, etc) and test framework elements

---

### Execution Behavior

The orchestrator interprets `on_error` as follows:

1. If an error occurs:
   - Retry the phase/action up to `retries` times
   - Wait `retry_delay_seconds` between attempts

2. If all retries fail:
   - If `continue: true`, execution proceeds to the next phase or step
   - If `continue: false`, the test halts and is marked as failed

This is especially useful for:

- Retrying flaky deployments or unstable environments
- Allowing test results to complete even if non-critical steps fail
- Ensuring cleanup still happens after a failed test step

---

### Error Handling - Summary

- The `on_error` block controls retry and continuation behavior on any test phase.
- Supports retries, delays, and graceful continuation.
- Useful at all levels: suite, scenario, step, and component.

Proper use of `on_error` gives your test suite resilience and flexibility,
especially in distributed or dynamic environments.
