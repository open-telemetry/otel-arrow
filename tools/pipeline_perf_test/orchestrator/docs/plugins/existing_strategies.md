# `existing_strategies`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
| ----------- | -------- | ------- | -------------- | ---------------------- |
| `docker` | `lib.impl.strategies.existing.docker` | `ExistingDocker` | `ExistingDockerConfig` | Resolve a running Docker container for observation without owning it |
| `kubernetes` | `lib.impl.strategies.existing.kubernetes` | `ExistingKubernetes` | `ExistingKubernetesConfig` | Resolve ready Kubernetes pods without changing cluster resources |

---

## `docker`

**Class**: `lib.impl.strategies.existing.docker.ExistingDocker`

**Config Class**: `lib.impl.strategies.existing.docker.ExistingDockerConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Resolve a running Docker container for observation without owning it.
"""
```

**Example YAML:**

```yaml
components:
  existing-service:
    existing:
      docker:
        container: my-service
    monitoring:
      docker_component:
        interval: 1
```

## `kubernetes`

**Class**: `lib.impl.strategies.existing.kubernetes.ExistingKubernetes`

**Config Class**: `lib.impl.strategies.existing.kubernetes.ExistingKubernetesConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Resolve ready Kubernetes pods without changing cluster resources.
"""
```

**Example YAML:**

```yaml
components:
  existing-service:
    existing:
      kubernetes:
        namespace: telemetry
        label_selector: app=my-service
    monitoring:
      prometheus:
        endpoint: http://127.0.0.1:8888/metrics
```
