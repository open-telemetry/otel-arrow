# Plugin Documentation

This directory contains auto-generated documentation for all plugin registries.

## Per-Registry Doc Files

- [deployment_strategies](./deployment_strategies.md)
- [monitoring_strategies](./monitoring_strategies.md)
- [configuration_strategies](./configuration_strategies.md)
- [execution_strategies](./execution_strategies.md)
- [hook_strategies](./hook_strategies.md)
- [step_actions](./step_actions.md)
- [report_formatters](./report_formatters.md)
- [report_writers](./report_writers.md)

## Plugin Summary: `deployment_strategies`

| Type | Plugin Name | Module | Class | Config Class | Description |
|------|-------------|--------|-------|--------------|-------------|
| `docker` | `docker` | `lib.impl.strategies.deployment.docker` | `DockerDeployment` | `DockerDeploymentConfig` | Deployment strategy to manage the lifecycle of components using Docker containers |

## Plugin Summary: `monitoring_strategies`

| Type | Plugin Name | Module | Class | Config Class | Description |
|------|-------------|--------|-------|--------------|-------------|
| `docker_component` | `docker_component` | `lib.impl.strategies.monitoring.docker_component` | `DockerComponentMonitoringStrategy` | `DockerComponentMonitoringConfig` | Strategy for monitoring a docker container resource mentrics via the python docker API client |
| `prometheus` | `prometheus` | `lib.impl.strategies.monitoring.prometheus` | `PrometheusMonitoringStrategy` | `PrometheusMonitoringConfig` | Strategy for monitoring a prometheus endpoint for a component |

## Plugin Summary: `execution_strategies`

| Type | Plugin Name | Module | Class | Config Class | Description |
|------|-------------|--------|-------|--------------|-------------|
| `pipeline_perf_loadgen` | `pipeline_perf_loadgen` | `lib.impl.strategies.execution.pipeline_perf_loadgen` | `PipelinePerfLoadgenExecution` | `PipelinePerfLoadgenConfig` | Execution strategy implementation for controlling the pipeline performance load generator |

## Plugin Summary: `hook_strategies`

| Type | Plugin Name | Module | Class | Config Class | Description |
|------|-------------|--------|-------|--------------|-------------|
| `build_docker_images` | `build_docker_images` | `lib.impl.strategies.hooks.docker.build_docker_image` | `BuildDockerImages` | `BuildDockerImagesConfig` | Hook strategy to build Docker images for multiple components locally |
| `build_docker_image` | `build_docker_image` | `lib.impl.strategies.hooks.docker.build_docker_image` | `BuildDockerImage` | `BuildDockerImageConfig` | Hook strategy implementation for building a single Docker image |
| `get_docker_logs` | `get_docker_logs` | `lib.impl.strategies.hooks.docker.logs` | `GetDockerLogs` | `GetDockerLogsConfig` | Hook strategy to retrieve logs and store them in the component's process_runtime |
| `create_docker_network` | `create_docker_network` | `lib.impl.strategies.hooks.docker.network` | `CreateDockerNetwork` | `CreateDockerNetworkConfig` | Hook strategy to create a Docker network for a component if it does not already exist |
| `delete_docker_network` | `delete_docker_network` | `lib.impl.strategies.hooks.docker.network` | `DeleteDockerNetwork` | `DeleteDockerNetworkConfig` | Hook strategy to delete a Docker network associated with a component |
| `tidy_existing_container` | `tidy_existing_container` | `lib.impl.strategies.hooks.docker.tidy_existing_container` | `TidyExistingContainer` | `TidyExistingContainerConfig` | Hook strategy to remove an existing Docker container with the same name as the current component |
| `wait_for_status` | `wait_for_status` | `lib.impl.strategies.hooks.docker.wait_for_status` | `WaitForDockerStatus` | `WaitForDockerStatusConfig` | Hook strategy to wait for a Docker container to reach a specific status |
| `raise_exception` | `raise_exception` | `lib.impl.strategies.hooks.raise_exception` | `RaiseExceptionHook` | `RaiseExceptionConfig` | Hook strategy that raises an exception |
| `record_event` | `record_event` | `lib.impl.strategies.hooks.record_event` | `RecordEventHook` | `RecordEventConfig` | Hook strategy that records an event to the context's current span |
| `run_command` | `run_command` | `lib.impl.strategies.hooks.run_command` | `RunCommandHook` | `RunCommandConfig` | Hook strategy that runs a specified shell command |

## Plugin Summary: `step_actions`

| Type | Plugin Name | Module | Class | Config Class | Description |
|------|-------------|--------|-------|--------------|-------------|
| `component_action` | `component_action` | `lib.impl.actions.component_action` | `ComponentAction` | `ComponentActionConfig` | Step action implementation for executing a lifecycle phase on a named component |
| `multi_component_action` | `multi_component_action` | `lib.impl.actions.multi_component_action` | `MultiComponentAction` | `MultiComponentActionConfig` | Step action that executes a specified lifecycle phase on one or more components |
| `wait` | `wait` | `lib.impl.actions.wait_action` | `WaitAction` | `WaitActionConfig` | Step action that introduces a delay during test execution |
| `update_component_strategy` | `update_component_strategy` | `lib.impl.actions.update_component_strategy` | `UpdateComponentStrategyAction` | `UpdateComponentStrategyConfig` | Step action that applies updates to a strategy configuration of a managed component |

## Plugin Summary: `report_formatters`

| Type | Plugin Name | Module | Class | Config Class | Description |
|------|-------------|--------|-------|--------------|-------------|
| `noop` | `noop` | `lib.impl.strategies.common.report` | `NoopFormatter` | `NoopFormatterConfig` | A report formatter that performs no formatting and returns an empty string |
| `json` | `json` | `lib.impl.strategies.common.report` | `JsonFormatter` | `JsonFormatterConfig` | Formats a report as a JSON string using the specified configuration |
| `template` | `template` | `lib.impl.strategies.common.report` | `TemplateFormatter` | `TemplateFormatterConfig` | Formats a report using a Jinja2 template specified either by file path or inline string |

## Plugin Summary: `report_writers`

| Type | Plugin Name | Module | Class | Config Class | Description |
|------|-------------|--------|-------|--------------|-------------|
| `noop` | `noop` | `lib.impl.strategies.common.report` | `NoopDestination` | `NoopDestinationConfig` | A destination writer that performs no action |
| `console` | `console` | `lib.impl.strategies.common.report` | `ConsoleDestination` | `ConsoleDestinationConfig` | Writes report data to the console (stdout) or via a logger |
