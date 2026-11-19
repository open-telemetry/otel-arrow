# Observe existing components

`observe-existing.yaml` is a scenario template: adopt the caller's components,
start monitoring, observe, stop monitoring, then run a caller-supplied SQL report.
It neither deploys nor removes components.

The flow is metric- and platform-agnostic. It has no built-in CPU, memory,
Kubernetes, or engine report. Callers choose components, monitoring strategies,
metrics, queries, validity checks, and result tables.

```yaml
name: Observe a caller-owned service
components:
  service:
    monitoring:
      prometheus:
        endpoint: http://127.0.0.1:8888/metrics
        include: [queue_depth]
tests:
  - from_template:
      path: templates/observe-existing.yaml
      variables:
        adopt: false
        duration_seconds: 300
        report_config:
          queries:
            - name: Queue depth
              sql: |
                CREATE TABLE queues AS
                SELECT AVG(value) AS mean_depth FROM metrics
                WHERE metric_name = 'queue_depth';
          result_tables:
            - name: queues
              finite_columns: [mean_depth]
```

Run from the orchestrator directory, or use absolute template paths when running
in a separate output directory. Reports write to the current working directory.

The template requires `report_config`. Optional variables are `name` (default
`Observe existing components`), `adopt` (true), `duration_seconds` (60),
`report_name` (`Observation results`), and `output_prefix` (`summary`). Adoption
requires an `existing` strategy on each component; set `adopt: false` for
endpoint-only monitoring. It prints the report and writes `<output_prefix>.md`
and `<output_prefix>.json`. Observation boundaries are recorded as
`observation_start` and `observation_stop`; each report decides how to use them.

SQL report definitions also support the existing `from_template` convention,
whether inline or loaded with `report_config_file`:

```yaml
report_config:
  from_template:
    path: /path/to/caller-owned-report.yaml
    variables:
      custom_parameter: custom_value
```

Top-level fields outside `from_template` override the rendered report fields,
using the same shallow merge as suite/scenario/step templates. Paths retain their
existing working-directory semantics.

The [Kubernetes example](../examples/existing-kubernetes.yaml) selects CPU/memory
in its own [example report](../examples/kubernetes-resources-report.yaml). These
are example choices, not requirements of the observation template.
