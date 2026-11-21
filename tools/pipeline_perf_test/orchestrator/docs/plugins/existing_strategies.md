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
        label_selector: app=my-service,component=collector
        # Optional: use a specific kubeconfig and context.
        # kubeconfig: /path/to/kubeconfig
        # context: my-context
        endpoints:
          metrics:
            port: 8888
            path: /metrics
    monitoring:
      prometheus:
        target_endpoint: metrics
```

Kubernetes adoption publishes one target per ready pod, carrying its namespace,
pod name, UID and node name. Named endpoints use the API server's built-in pod
proxy:

```text
runner -> authenticated Kubernetes API -> pod IP:port/path
```

Each endpoint requires `port`; `path` defaults to `/metrics`, and `scheme` defaults
to `http` (`https` is also supported). The runner needs `list` on `pods` and `get`
on `pods/proxy` in the selected namespace. The API server must be able to reach
the listener through the pod IP, typically bound to `0.0.0.0`.

Pod and running-container identities are checked before each scrape round.
Replacements or restarts require stopping monitoring and adopting again.
Discovery, validation and fetching share one client, released after monitoring
stops on destroy, re-adoption or suite cleanup.
