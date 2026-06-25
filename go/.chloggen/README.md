# Go changelog entries

This directory holds in-flight changelog entries for the Go components
(`go/`, `collector/`). Each pull request that changes user-facing Go behavior
must add at least one YAML file here. At release time,
[`chloggen`](https://github.com/open-telemetry/opentelemetry-go-build-tools/tree/main/chloggen)
collapses these entries into [`../CHANGELOG.md`](../CHANGELOG.md) and the
YAML files are deleted.

For Rust changes use
[`rust/otap-dataflow/.chloggen/`](../../rust/otap-dataflow/.chloggen/)
instead. A PR that touches both trees adds one entry in each.

## Adding an entry

Copy `TEMPLATE.yaml` in this directory to a new `.yaml` file
(e.g. `arrow-encoder-fix-null-handling.yaml`) and fill in the fields:

```yaml
change_type: enhancement       # breaking | deprecation | new_component | enhancement | bug_fix
component: pkg/otel/arrow      # must be listed in this directory's config.yaml
note: Add support for X.
issues: [1234]                 # issue or PR number(s)
subtext:                       # optional multi-line details
```

Validate locally:

```bash
make chlog-validate
make chlog-preview             # renders entries without modifying CHANGELOGs
```

## When no entry is needed

Skip the entry when the change is not user-facing. The PR-validation
workflow honors any of:

- `chore` (case-insensitive) anywhere in the PR title.
- The `chore` label. (for maintainers).
- The `skipchangelog` label (for maintainers).
- The `documentation` label (auto-applied if all changes are in `docs/` or `rust/otap-dataflow/docs/`).
- The `dependencies` label (auto-applied by Renovate).
- Bot authors (`dependabot[bot]`, `renovate[bot]`, `otelbot`).

## Configuration

`config.yaml` in this directory defines the allowed `components` list and the
output changelog. See the comments in that file for details.
