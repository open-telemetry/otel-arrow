# Comparison Dashboard

Benchmarking of the DF Engine and the OTel Collector using the
upstream orchestrator framework, with decoupled suites and comparisons.

## Concepts

**Suites** define and orchestrator test to run. Each suite is scoped to a 
single binary and runs some set of tests that typically vary over a single
dimension like loadgen rate. Suite files live in `suites/` and reference an
orchestrator template that has the test definitions hardcoded.

**Comparisons** define what suites are comparable. A comparison references
multiple suites and charts them side by side, grouping by test name. Comparison
files live in `comparisons/`.

**Manifest** (`manifest.yaml` at the dashboard root) is the single source of
truth for the dashboard. It declares:
- the path to every suite and comparison file
- the site root directory
- `variables`: top-level Jinja variables passed straight through to template
  rendering (e.g. `df_engine_image`, `otelcol_image`). The scripts treat these
  as opaque pass-through values -- update images by editing the manifest.
- `meta`: the closed schema of allowed keys and values for every suite's
  `meta` block. The build validates each suite against it.

`dashboard.py` is the main script for doing dashboard operations like serving a
local site, validating configs, running a scenario, or building the site. It
works based on the manifest.

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
python dashboard/scripts/dashboard.py run dashboard/suites/dfe/dfe-logs-otap-none-baseline.yaml

# Run all suites
python dashboard/scripts/dashboard.py run "dashboard/suites/**/*.yaml"

# Run only DFE suites
python dashboard/scripts/dashboard.py run "dashboard/suites/dfe/*.yaml"

# Generate orchestrator configs without running
python dashboard/scripts/dashboard.py run "dashboard/suites/**/*.yaml" --generate-only
```

Results are published to `site/data/suite/<slug>/`.

## Building the Dashboard

After running suites, build the static site:

```bash
python dashboard/scripts/dashboard.py build
# or with an explicit manifest:
python dashboard/scripts/dashboard.py build --manifest dashboard/manifest.yaml
```

The build validates that:
- suite slugs are unique among suites
- comparison slugs are unique among comparisons
- every suite slug referenced by a comparison resolves to a manifest-listed suite
- every key in each suite's `meta` block is declared in `manifest.meta`, and
  every value (or every list element, for list-typed keys) is in the allowed set

The build aborts on any validation failure.

This generates:
- `site/data/suite/<slug>/data.js` for each suite with data
- `site/index.html` landing page with comparison sections
- `site/compare/<slug>/index.html` detail pages per comparison

## Viewing the Dashboard

```bash
python dashboard/scripts/dashboard.py serve
# Open http://localhost:3000
```

## Directory Structure

```
dashboard/
├── manifest.yaml        Inventory of suites + comparisons; defines site root
├── suites/              Suite definitions, grouped into per-binary subfolders
│   ├── dfe/             DFE suites
│   ├── otc/             OTel Collector suites
│   ├── fluentbit/       Fluent Bit suites
│   ├── rotel/           Rotel suites
│   └── vector/          Vector suites
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
│   └── dashboard.py     CLI with `run`, `build`, and `serve` subcommands
├── site/                Dashboard web UI (generated + shared assets)
│   ├── shared/          Shared JS and CSS
│   ├── data/suite/      Published benchmark data
│   └── compare/         Comparison detail pages
├── setup.sh             One-time environment setup
└── .data/               Staging area for run artifacts (gitignored)
```

## Adding a Suite

Create a YAML file in `suites/` and add its path to `manifest.yaml`:

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

Create a YAML file in `comparisons/` and add its path to `manifest.yaml`:

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
