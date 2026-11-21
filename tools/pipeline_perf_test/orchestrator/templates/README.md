# Scenario templates

Reusable scenarios loaded with `from_template`. Define components and monitoring
in the calling suite. Paths resolve from the working directory; run from the
orchestrator directory or use absolute paths.

## `observe-existing.yaml`

Optionally adopt components, start monitoring, observe, stop monitoring, then run
caller-selected SQL. It never deploys or removes components.

```yaml
tests:
  - from_template:
      path: templates/observe-existing.yaml
      variables:
        adopt: false # Endpoint-only monitoring; no resource identity to discover.
        duration_seconds: 300
        report_config:
          from_template:
            path: /path/to/caller-owned-report.yaml
```

`report_config` is required and accepts inline SQL report configuration or
`from_template`. SQL reports loaded through `report_config_file` also support
template references; outer fields shallowly override rendered fields.

| Optional variable | Default |
| ----------------- | ------- |
| `name` | `Observe existing components` |
| `adopt` | `true`; requires an `existing` strategy on each component |
| `duration_seconds` | `60` |
| `report_name` | `Observation results` |
| `output_prefix` | `summary` |

Adoption discovers Docker/Kubernetes resource identity without taking ownership.
Set `adopt: false` when components only supply reachable metrics endpoints.
Reports print to the console and write `<output_prefix>.md` and
`<output_prefix>.json`. SQL can use the `observation_start` and `observation_stop`
events to select its measurement interval.
