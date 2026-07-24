# Multitenancy Overview

**Status:** Draft

## Background

Two other open-source systems influence this design:

- [Kubernetes multitenant concepts](https://kubernetes.io/docs/concepts/security/multitenancy/)
- [Envoy rate limit configuration](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/other_features/global_rate_limiting.html)

Both of these systems will be familiar to many users and we aim to
keep our concepts close to theirs.

## Definitions

As the dataflow engine can be deployed to perform in a wide range of
application scenarios, there is no single definition of or data model
for a tenant. Multitenancy describes a set of features for managing
tenancy requirements, not a specific aspect of the dataflow engine.

Tenancy requirements depend on the use-case, covering what resources
are being shared, what needs to be isolated, and the acceptable level
of operational complexity. Tenancy use-cases are often divided into
categories based on the types of relationship between the principal
and the tenant(s):

- **Multiple teams** that share an administrator boundary (e.g., divisions
  in a company). These are usually small in number, tenants are
  cooperative and share administrative control.
- **Multiple customers** of a SaaS sharing a service endpoint have a
  contractual relationship, compete for shared service resources, and
  may be large in number.
- The **self-observability** pipeline is treated as a special tenant.
- **Multiple producers** of telemetry in different namespaces are
  processed separately.

Sometimes there will be more than one concept of tenancy in use at a
time (e.g., SaaS customer account and signed-in user). Sometimes
multitenancy is applied at multiple levels (e.g., both thread-local
and global rate limits).

## CPU limit policies

CPU limits are provided through built-in integration with the
operating system through such mechanisms as Linux Container Groups
(a.k.a. `cgroups`) and Windows Job Objects. The dataflow engine is
required to support both absolute maximum and relative CPU limits,
under `resources.cpu_limiter` configuration.

CPU limits are given as an example of a multitenant mechanism because
they are relatively simple, and directly supported by operating
systems. Here we establish a convention for naming the aspects of a
limit, either in absolute or in relative terms.

- The `cpu_limiter.cpu_limit` policy field indicates an absolute
  limit in milli-CPUs. Example "100m" indicates 10% of one
  CPU.
- The `cpu_limiter.cpu_weight` policy field indicates a relative
  limit, taken as a ratio compared with the sum across all siblings
  in the configuration.

As an example, the following dataflow engine configuration allows the
dataflow engine to consume 10% of one CPU per CPU.

```
policies:
  resources:
    cpu_limiter:
      # Engine limited to maximum 100 mCPU or 10% CPU usage
      cpu_limit: 100m

engine:
  observability:
    policies:
      resources:
        cpu_limiter:
          # Self observability at maximum 1% CPU usage
          cpu_limit: 10m
    pipeline: { ... }

groups:
  main-group:
    pipelines:
      first-pipe:
        policies:
          resources:
            cpu_limiter:
              # First pipeline at 80% relative
              cpu_weight: 80
        # ...
      second-pipe:
        policies:
          resources:
            cpu_limiter:
              # Second pipeline at 20% relative
              cpu_weight: 20
        # ...

```

## Detailed design documents

### Memory management

Process-wide memory limits and a system of regulation for real memory
allocations is [detailed in a separate
document](./multitenancy-memory.md).

### Tenant identification

A mechanism for extracting tenant-specific details from request
context is defined. A corresponding mechanism for conditional behavior
based on propagating tenant "tokens" is [detailed in a separate
document](./multitenancy-tenant.md).

### Limiter extensions

A general-purpose mechanism for limiter policies and limiter
implementations is developed, covering rate limits and resource limits
in pre-defined units, for example to limit the number of requests per
second or network bytes per second and concurrent quantities such as
requests in flight. Limiter units name the quantity being measured
such as:

- `request_bytes` measures the in-memory size of the request
- `network_bytes` measures the on-wire size of the request
- `request_count` measures one unit per request
- `request_items` measures one unit per item of telemetry data
- `storage_bytes` measures one unit per byte of storage
- `storage_ops` measures one unit per storage read or write

Rate and Resource limiter extensions are [detailed in a separate
document](./multitenancy-limiters.md)

