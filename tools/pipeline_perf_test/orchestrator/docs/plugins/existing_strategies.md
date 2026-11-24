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
pod name, UID and node name. Named endpoints use adoption-owned port-forwards:

```text
runner -> localhost tunnel -> authenticated Kubernetes API -> pod localhost:port/path
```

Port-forwarding is the default: it reaches localhost-only listeners; pod-proxy
uses the pod IP. Proxy support is not implemented; add it only as an explicit
opt-in if needed.

Each endpoint requires `port`; `path` defaults to `/metrics`, and `scheme` defaults
to `http` (`https` uses normal certificate verification). Local requests bypass
HTTP proxy settings. The runner needs
`kubectl`, `list`/`get` on `pods`, and permission to open `pods/portforward`
(typically `create`, or `get` for WebSocket clients) in the selected namespace.
Tunnels bind only to runner localhost and start within 10 seconds.

Listeners can stay on `127.0.0.1`, avoiding exposure to other pods. Forwarding
authorizes the whole port, including any admin APIs, not just metrics.

Pod and running-container identities are checked before each scrape round.
Replacements or restarts require stopping monitoring and adopting again.
Discovery and validation share one API client. Endpoints on the same pod/port
share a tunnel. Scrapers stop before tunnels and client access are released on
destroy, re-adoption or suite cleanup; failed forwarding is surfaced, not restarted.
