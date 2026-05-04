# Performance Dashboard

Benchmarking of the DF Engine and the OTel Collector using the
upstream orchestrator framework, with decoupled suites and comparisons.

## Concepts

**Suites** define what to run. Each suite is scoped to a single binary (DFE or
OTC) and runs standardized tests at 100k, 200k, 300k, and 400k signals/second.
Suite files live in `suites/` and reference an orchestrator template that has
the test definitions hardcoded.

**Comparisons** define what to show on the dashboard. A comparison references
multiple suites and charts them side by side, grouping by test name. Comparison
files live in `comparisons/`.

## Setup

From the repo root:

```bash
bash tools/pipeline_perf_test/dashboard/setup.sh
source <venv-path>/bin/activate   # printed by setup.sh
```

## Running Benchmarks

All commands run from `tools/pipeline_perf_test/`.

```bash
cd tools/pipeline_perf_test

# Run a single suite
python dashboard/scripts/run.py dashboard/suites/dfe-passthrough-otap.yaml

# Run all suites
python dashboard/scripts/run.py "dashboard/suites/*.yaml"

# Run only DFE suites
python dashboard/scripts/run.py "dashboard/suites/dfe-*.yaml"

# Generate orchestrator configs without running
python dashboard/scripts/run.py "dashboard/suites/*.yaml" --generate-only

# Custom images
python dashboard/scripts/run.py "dashboard/suites/*.yaml" \
    --df-engine-image my-registry/df-engine:dev \
    --otelcol-image otel/opentelemetry-collector-contrib:0.120.0
```

Results are published to `site/data/suite/<slug>/`.

## Building the Dashboard

After running suites, build the static site:

```bash
python dashboard/scripts/build.py
```

This generates:
- `site/data/suite/<slug>/data.js` for each suite with data
- `site/index.html` landing page with comparison sections
- `site/compare/<slug>/index.html` detail pages per comparison

## Viewing the Dashboard

```bash
python dashboard/scripts/run-site.py
# Open http://localhost:3000
```

## Directory Structure

```
dashboard/
├── suites/              Suite definitions (one per binary+config combination)
├── comparisons/         Comparison definitions (what to chart together)
├── templates/
│   ├── orchestrator/    Orchestrator templates (DFE and OTC variants)
│   ├── steps/           Step templates for df-engine and otelcol
│   ├── engine/          DF Engine pipeline configs
│   ├── otelcol/         OTel Collector configs
│   ├── loadgen/         Load generator configs (Jinja2)
│   ├── backend/         Backend service configs (Jinja2)
│   └── reports/         Report output templates
├── reports/             SQL report configs
├── scripts/
│   ├── run.py           Suite runner
│   ├── build.py         Static site builder
│   └── run-site.py      Dashboard site server
├── site/                Dashboard web UI (generated + shared assets)
│   ├── shared/          Shared JS and CSS
│   ├── data/suite/      Published benchmark data
│   └── compare/         Comparison detail pages
├── setup.sh             One-time environment setup
└── .data/               Staging area for run artifacts (gitignored)
```

## Adding a Suite

Create a YAML file in `suites/`:

```yaml
name: DFE OTAP Passthrough (Logs)
slug: dfe_logs_otap_otap_none_passthrough
description: Dataflow Engine proxying OTAP logs with no processing
orchestrator_template: dashboard/templates/orchestrator/dfe-single-core-multi-rate.yaml

meta:
  binary: dfe
  protocols: [otap]
  signals: [logs]
  compression: none

variables:
  report: dashboard/reports/integration_report_logs.yaml
  engine_config: dashboard/templates/engine/otap-otap.yaml
  loadgen_config: dashboard/templates/loadgen/otap.yaml.j2
  backend_config: dashboard/templates/backend/otap.yaml.j2
  protocol: otap
  signal: logs
  observation_interval: 20
  max_batch_size: 1000
  compression_method: none
```

DFE suites set `meta.binary: dfe`, use `engine_config`, and the `dfe-single-core-multi-rate.yaml` template.
OTC suites set `meta.binary: otc`, use `collector_config`, and the `otc-single-core-multi-rate.yaml` template.

## Adding a Comparison

Create a YAML file in `comparisons/`:

```yaml
slug: my_comparison
name: My Comparison
description: Compare X to Y

suites:
  - name: Suite A
    slug: suite_a_slug
    short: A
  - name: Suite B
    slug: suite_b_slug
    short: B
```

The `short` field is used as the legend label in charts.
