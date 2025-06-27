# `hook_strategies`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
|-----------|--------|-------|--------------|----------------------|
| `build_docker_images` | `lib.impl.strategies.hooks.docker.build_docker_image` | `BuildDockerImages` | `BuildDockerImagesConfig` | Hook strategy to build Docker images for multiple components locally |
| `build_docker_image` | `lib.impl.strategies.hooks.docker.build_docker_image` | `BuildDockerImage` | `BuildDockerImageConfig` | Hook strategy implementation for building a single Docker image |
| `get_docker_logs` | `lib.impl.strategies.hooks.docker.logs` | `GetDockerLogs` | `GetDockerLogsConfig` | Hook strategy to retrieve logs and store them in the component's process_runtime |
| `create_docker_network` | `lib.impl.strategies.hooks.docker.network` | `CreateDockerNetwork` | `CreateDockerNetworkConfig` | Hook strategy to create a Docker network for a component if it does not already exist |
| `delete_docker_network` | `lib.impl.strategies.hooks.docker.network` | `DeleteDockerNetwork` | `DeleteDockerNetworkConfig` | Hook strategy to delete a Docker network associated with a component |
| `tidy_existing_container` | `lib.impl.strategies.hooks.docker.tidy_existing_container` | `TidyExistingContainer` | `TidyExistingContainerConfig` | Hook strategy to remove an existing Docker container with the same name as the current component |
| `wait_for_status` | `lib.impl.strategies.hooks.docker.wait_for_status` | `WaitForDockerStatus` | `WaitForDockerStatusConfig` | Hook strategy to wait for a Docker container to reach a specific status |
| `raise_exception` | `lib.impl.strategies.hooks.raise_exception` | `RaiseExceptionHook` | `RaiseExceptionConfig` | Hook strategy that raises an exception |
| `record_event` | `lib.impl.strategies.hooks.record_event` | `RecordEventHook` | `RecordEventConfig` | Hook strategy that records an event to the context's current span |
| `run_command` | `lib.impl.strategies.hooks.run_command` | `RunCommandHook` | `RunCommandConfig` | Hook strategy that runs a specified shell command |

---

## `build_docker_images`

**Class**: `lib.impl.strategies.hooks.docker.build_docker_image.BuildDockerImages`

**Config Class**: `lib.impl.strategies.hooks.docker.build_docker_image.BuildDockerImagesConfig`

**Supported Contexts:**

- ComponentHookContext
- FrameworkElementHookContext

**CLI Flags:**

**Docker Options:**

| Flag | Description | Default | Required |
|------|-------------|---------|----------|
| `--docker.no-build` | Skip build of Docker containers. | - | - |

**Description:**

```python
"""
Hook strategy to build Docker images for multiple components locally.

This strategy builds Docker images for specified components or,
if no components are specified, for all applicable managed components
with Docker deployment configurations that include a build section.

Attributes:
    config (BuildDockerImagesConfig): Configuration for this build strategy.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    pre:
      - build_docker_images:
          # Omit components to build any docker component with a build section.
          components:
            - load-generator
            - backend-service
          log_build: false
```

## `build_docker_image`

**Class**: `lib.impl.strategies.hooks.docker.build_docker_image.BuildDockerImage`

**Config Class**: `lib.impl.strategies.hooks.docker.build_docker_image.BuildDockerImageConfig`

**Supported Contexts:**

- ComponentHookContext
- FrameworkElementHookContext

**Description:**

```python
"""
Hook strategy implementation for building a single Docker image.

This class is responsible for executing the Docker build process for a specified
component. It can be used in test steps or hook phases to ensure that a Docker image
is built prior to component execution.

Behavior:

- Uses the component name from the configuration if specified.
- Falls back to resolving the component from the execution context if not explicitly provided.
- Skips execution if the component is missing, not a ManagedComponent, or lacks a Docker build config.

Args:
    config (BuildDockerImageConfig): The configuration object containing build options.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    pre:
      - build_docker_image:
          # Omit components to build any docker component with a build section.
          component: load-generator
          log_build: false
```

## `get_docker_logs`

**Class**: `lib.impl.strategies.hooks.docker.logs.GetDockerLogs`

**Config Class**: `lib.impl.strategies.hooks.docker.logs.GetDockerLogsConfig`

**Supported Contexts:**

- ComponentHookContext

**Description:**

```python
"""
Hook strategy to retrieve logs and store them in the component's process_runtime.

This hook is intended to be used as part of a pipeline or test framework where
container logs are collected for debugging, auditing, or analysis. When executed,
it locates the Docker container associated with the current component and stores
the logs in the component's `ProcessRuntime`, making them accessible for later inspection.

Typical usage:
    - Collect logs before or after component execution (e.g., in pre/post hooks)
    - Enhance observability in test pipelines using Docker-based deployments

Attributes:
    config (GetDockerLogsConfig): Configuration for the log retrieval behavior.
"""
```

**Example YAML:**

```yaml
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        destroy:
            pre:
            - get_docker_logs: {}
```

## `create_docker_network`

**Class**: `lib.impl.strategies.hooks.docker.network.CreateDockerNetwork`

**Config Class**: `lib.impl.strategies.hooks.docker.network.CreateDockerNetworkConfig`

**Supported Contexts:**

- ComponentHookContext
- FrameworkElementHookContext

**Description:**

```python
"""
Hook strategy to create a Docker network for a component if it does not already exist.

This hook ensures that a Docker network is available for the component prior to
deployment. It first checks whether the specified or inferred network already exists;
if not, it creates the network and marks it in the component's runtime so it can be
cleaned up later if needed.

Network name resolution:
    - If explicitly provided in the config, that name is used.
    - If not provided, the network name is inferred from the component's deployment config.
    - If no network name can be determined, the hook is skipped.

Hook status:
    - SKIPPED if no network is needed or it already exists.
    - FAILURE if Docker API errors occur during creation.

Raises:
    docker.errors.APIError: If the Docker daemon encounters an error during network creation.

Typical usage:
    - Automatically creating isolated Docker networks in test environments.
    - Supporting component-based Docker orchestration with isolated networking.
"""
```

**Notes:**

This hook is automatically installed by the docker deployment strategy and
doesn't need to be added explicitly

**Example YAML:**

```yaml
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        deploy:
            pre:
            - create_docker_network:
                network: foo-network
```

## `delete_docker_network`

**Class**: `lib.impl.strategies.hooks.docker.network.DeleteDockerNetwork`

**Config Class**: `lib.impl.strategies.hooks.docker.network.DeleteDockerNetworkConfig`

**Supported Contexts:**

- ComponentHookContext
- FrameworkElementHookContext

**Description:**

```python
"""
Hook strategy to delete a Docker network associated with a component.

This hook is typically run during the teardown (post-destroy) phase of a component's
lifecycle. It attempts to remove the specified Docker network, but only if the network
was created during execution (i.e., not a pre-existing or default network).

Network name resolution:
    - If explicitly specified in the config, that name is used.
    - If not provided, it is inferred from the component's deployment configuration.
    - If no network name can be resolved, the hook is skipped.

Runtime checks:
    - The network is only removed if `network_created` is set in the component's
      Docker runtime. This prevents accidental deletion of shared or external networks.

Hook status:
    - SKIPPED if no valid network is found or deletion is unnecessary.
    - FAILURE if the network is expected but cannot be found.
    - Raises if the Docker daemon returns an API error.

Raises:
    docker.errors.APIError: If there is a problem communicating with the Docker daemon.
    docker.errors.NotFound: If the specified Docker network does not exist.

Typical usage:
    - Cleaning up isolated Docker networks created during automated tests or temporary deployments.
    - Ensuring that dynamically created networks do not persist beyond their intended lifecycle.
"""
```

**Notes:**

This hook is automatically installed by the docker deployment strategy and
doesn't need to be added explicitly

**Example YAML:**

```yaml
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        destroy:
            post:
            - delete_docker_network:
                network: foo-network
```

## `tidy_existing_container`

**Class**: `lib.impl.strategies.hooks.docker.tidy_existing_container.TidyExistingContainer`

**Config Class**: `lib.impl.strategies.hooks.docker.tidy_existing_container.TidyExistingContainerConfig`

**Supported Contexts:**

- ComponentHookContext
- FrameworkElementHookContext

**Description:**

```python
"""
Hook strategy to remove an existing Docker container with the same name as the current component.

This hook is useful during the pre-deployment phase of a component lifecycle to ensure that
any previously running container (possibly left over from an earlier run) is cleaned up before
starting a new one. This avoids container name conflicts and ensures consistent environment setup.

Behavior:
    - If a container is found with the same name as the component, it is stopped and removed.
    - If no container is found (404), the hook is skipped silently.
    - If no component is available in the context, the hook fails.

Hook status:
    - FAILURE if no component is found in context.
    - SKIPPED if the container is not found or no action is needed.

Raises:
    docker.errors.APIError: Only if an unexpected Docker API issue occurs (currently suppressed for 404s).

Typical usage:
    - Pre-deployment cleanup to avoid naming conflicts.
    - Resetting environment between pipeline/test runs where container reuse is not desired.
"""
```

**Notes:**

This hook is automatically installed by the docker deployment strategy and
doesn't need to be added explicitly

**Example YAML:**

```yaml
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        deploy:
            pre:
            - tidy_existing_container: {}
```

## `wait_for_status`

**Class**: `lib.impl.strategies.hooks.docker.wait_for_status.WaitForDockerStatus`

**Config Class**: `lib.impl.strategies.hooks.docker.wait_for_status.WaitForDockerStatusConfig`

**Supported Contexts:**

- ComponentHookContext

**Description:**

```python
"""
Hook strategy to wait for a Docker container to reach a specific status.

This hook polls the Docker daemon at a configurable interval until the container
associated with the current component reaches the desired lifecycle state (e.g., "running").
It is typically used in post-deploy hooks to ensure a container is healthy before proceeding.

Behavior:
    - Retrieves the container ID from the component's runtime.
    - Polls the Docker client for the current status.
    - If the container reaches the target status within the timeout, the hook succeeds.
    - If the timeout is exceeded, raises a TimeoutError and sets context status to TIMEOUT.
    - If no container ID is found, the hook fails immediately.

Hook status:
    - SUCCESS if desired container status is reached.
    - FAILURE if no container ID is available.
    - TIMEOUT if container does not reach the desired state in time.

Raises:
    TimeoutError: If the container fails to reach the desired status within the timeout.
    RuntimeError: If no container ID is available in the component's runtime.
    docker.errors.DockerException: If there are errors communicating with Docker.

Typical usage:
    - As a post-deploy hook to block until the container is `running`.
    - To coordinate pipeline steps based on container readiness.
"""
```

**Notes:**

This hook is automatically installed by the docker deployment strategy and
doesn't need to be added explicitly

**Example YAML:**

```yaml
components:
  otel-collector:
    deployment:
      docker: ...
    hooks:
        deploy:
            post:
            - wait_for_status:
                status: running
                timeout: 30
                interval: 1
```

## `raise_exception`

**Class**: `lib.impl.strategies.hooks.raise_exception.RaiseExceptionHook`

**Config Class**: `lib.impl.strategies.hooks.raise_exception.RaiseExceptionConfig`

**Supported Contexts:**

- FrameworkElementHookContext
- ComponentHookContext

**Description:**

```python
"""
Hook strategy that raises an exception.

This class is responsible for raising an exception when fired. Primarily for testing.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    pre:
      - raise_exception:
            message: This is a test exception.
```

## `record_event`

**Class**: `lib.impl.strategies.hooks.record_event.RecordEventHook`

**Config Class**: `lib.impl.strategies.hooks.record_event.RecordEventConfig`

**Supported Contexts:**

- FrameworkElementHookContext
- ComponentHookContext

**Description:**

```python
"""
Hook strategy that records an event to the context's current span.

This hook allows users to mark significant events or timestamps during test execution
or pipeline steps by recording them as events on the active trace/span. These events
are useful for correlating application behavior, measuring durations between steps,
or annotating traces for observability and debugging purposes.

Typical usage:
    - Marking the start and end of an observation window.
    - Logging test lifecycle events in distributed tracing systems.
    - Recording custom milestones in execution telemetry.
"""
```

**Example YAML:**

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Mark 10s Observation Window
        action:
          wait:
            delay_seconds: 10
        hooks:
          run:
            pre:
              - record_event:
                  name: observation_start
            post:
              - record_event:
                  name: observation_stop
```

## `run_command`

**Class**: `lib.impl.strategies.hooks.run_command.RunCommandHook`

**Config Class**: `lib.impl.strategies.hooks.run_command.RunCommandConfig`

**Supported Contexts:**

- FrameworkElementHookContext
- ComponentHookContext

**Description:**

```python
"""
Hook strategy that runs a specified shell command.

This class is responsible for executing a shell command defined in its configuration
when the hook is triggered.
"""
```

**Example YAML:**

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Run a command then wait step
        action:
          wait:
            delay_seconds: 10
        hooks:
          run:
            pre:
              - run_command:
                  command: python somefile.py
```
