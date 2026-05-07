# Comparison Dashboard

Static site for comparing telemetry engines and protocols across various
scenarios.

This dashboard is fully static. It does not query a backend or fetch benchmark
results from a remote service. Displayed benchmark data comes from local suite
runs published into `site/data/suite/<slug>/` typically via `dashboard.py run`.

Benchmark execution via is delegated to the
[orchestrator](../pipeline_perf_test/orchestrator) -- `dashboard.py run` is just
a wrapper that knows how to render orchestrator templates, store the results in
a temporary directory (typically `.data`) and then copy the results to the
appropriate `site/data` directory.

Any data in those directories is then crunched along with various site manifests
by `dashboard.py build` in order to generate the appropriate page stubs and
`data.js` files.

The dashboard can then be hosted on some static site server like github pages,
but can also be served locally using `dashboard.py serve`.

## Concepts

**Suites** define an orchestrator test to run. Each suite is scoped to a single
binary and runs some set of tests that typically vary over a single dimension
like loadgen rate. Suite files live in `suites/` and reference an orchestrator
template that has the test definitions hardcoded.

**Comparisons** define which suites are comparable. A comparison references
multiple suites and charts them side by side, grouping by test name. Comparison
files live in `comparisons/`.

**Manifest** (`manifest.yaml` at the dashboard root) is the single source of
truth. It declares:

- the path to every suite and comparison file
- the site root directory
- `variables`: top-level Jinja variables passed straight through to template
  rendering. The script treats these as opaque pass-through values -- e.g. set
  image refs here rather than via CLI flags.
- `meta`: the closed schema of allowed keys and values for every suite's `meta`
  block. Validation rejects undeclared keys and disallowed values.

`dashboard.py` is the single entry point. All subcommands read the manifest.

## Setup

The dashboard reuses the orchestrator's Python environment. Set it up once from
the repo root:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -r tools/pipeline_perf_test/orchestrator/requirements.txt
```

Run all `dashboard.py` commands from `tools/comparison_dashboard/` with that
environment active.

## Commands

```bash
cd tools/comparison_dashboard

# Check the manifest (slug uniqueness, comparison cross-refs, suite meta)
python dashboard.py validate

# Build the static site from the manifest + any published suite results
python dashboard.py build

# Serve the built site locally
python dashboard.py serve              # http://localhost:3000
python dashboard.py serve --port 8080

# Run one or more suites (matches positional args against manifest entries)
python dashboard.py run "suites/dfe/*.yaml"
python dashboard.py run "suites/**/*.yaml" --generate-only
```

`build` and `validate` share the exact same validation code path, so any
manifest issue surfaces with identical wording in either verb.

`build` writes:

- `site/data/suite/<slug>/data.js` for each suite with published data
- `site/index.html` (landing page with comparison sections)
- `site/compare/<slug>/index.html` (per-comparison detail page)

`run` stages run artifacts in `.data/<slug>/<timestamp>/` and publishes results
to `site/data/suite/<slug>/` on success.

## Directory Structure

```text
tools/comparison_dashboard/
  dashboard.py        CLI: validate | build | run | serve
  manifest.yaml       Inventory + framework config (variables, meta, etc.)
  suites/             Per-binary suite definitions
  comparisons/        Comparison definitions
  site/               Static dashboard site
    shared/           Shared JS/CSS assets
    index.html        Generated landing page
    compare/          Generated per-comparison pages
    data/suite/       Published per-suite data
  .data/              Run staging area (gitignored)
```
