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

Each adopted pod becomes a component target with pod identity attributes.
`endpoints` optionally publishes named HTTP(S) endpoints on every target.
Requires pod discovery access and `get pods/proxy`; reuses the configured
kubeconfig/context without modifying workloads or launching `kubectl`.

```text
Prometheus -> named endpoint.fetch(timeout)
                      |
          adoption-owned authenticated API client
                      |
              API server pod proxy -> pod IP:port/path
```

Adoption owns one reusable API client for discovery, validation, and all endpoint
fetches. It closes on component destruction or suite exit, after scrapers stop;
failed startup also closes the client. Re-adoption releases old access before
discovering new targets.

Endpoint `path` defaults to `/metrics`; `scheme` defaults to `http` and can be
`https` for an HTTPS listener in the pod. API-server authentication and TLS still
come from kubeconfig. Scrapes use Prometheus monitoring's `request_timeout`;
HTTP status, authorization, and TLS errors fail immediately.

The listener must bind a pod-reachable interface, such as `0.0.0.0`, not only
loopback. There is no automatic forwarding fallback. For a loopback-only listener,
manage a port-forward externally and use Prometheus's explicit `endpoint` URL.
