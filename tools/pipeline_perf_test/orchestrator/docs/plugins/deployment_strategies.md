# `deployment_strategies`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
|-----------|--------|-------|--------------|----------------------|
| `docker` | `lib.impl.strategies.deployment.docker` | `DockerDeployment` | `DockerDeploymentConfig` | Deployment strategy to manage the lifecycle of components using Docker containers |

---

## `docker`

**Class**: `lib.impl.strategies.deployment.docker.DockerDeployment`

**Config Class**: `lib.impl.strategies.deployment.docker.DockerDeploymentConfig`

**Supported Contexts:**

- StepContext

**Installs Default Hooks:**

- CreateDockerNetwork
- TidyExistingContainer
- WaitForDockerStatus
- DeleteDockerNetwork

**Description:**

```python
"""
Deployment strategy to manage the lifecycle of components using Docker containers.

This class handles starting and stopping Docker containers based on the given
deployment configuration. It also registers default lifecycle hooks for Docker
operations such as network creation, container cleanup, and status checks.

Methods:
    start(component: Component, ctx: StepContext):
        Starts a Docker container for the specified component using the deployment
        configuration. Handles setting up the container with networking, volumes,
        ports, environment variables, and commands.

    stop(component: Component, ctx: StepContext):
        Stops and removes the Docker container associated with the component, using
        container ID stored in the component runtime. Raises errors if container
        cannot be found or stopped.
"""
```

**Example YAML:**

```yaml
components:
  otel-collector:
    deployment:
      docker:
        image: otel/opentelemetry-collector:latest
        network: testbed
        volumes:
          - ./system_under_test/otel-collector/collector-config-with-batch-processor.yaml:/etc/otel/collector-config.yaml:ro
        command: ["--config", "/etc/otel/collector-config.yaml"]
        ports:
          - "8888:8888"
```
