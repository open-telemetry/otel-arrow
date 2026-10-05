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

From the repo root:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -r tools/comparison_dashboard/requirements.txt
# Only needed if you want to use `dashboard.py run` (which spawns the orchestrator):
pip install -r tools/pipeline_perf_test/orchestrator/requirements.txt
```

The dashboard's own deps (`pyyaml`, `jinja2`) are sufficient for `validate`,
`build`, and `serve`. The orchestrator deps are only needed for `run` because
that subcommand invokes the orchestrator as a subprocess in the same env.

Run all `dashboard.py` commands from `tools/comparison_dashboard/` with that
environment active.

## Commands

```bash
cd tools/comparison_dashboard

# Check the manifest (slug uniqueness, comparison cross-refs, suite meta)
python dashboard.py validate

# Build the static site from the manifest + any published suite results
python dashboard.py build

# Serve the built site locally; visit http://localhost:3000/compare/
python dashboard.py serve
python dashboard.py serve --port 8080

# Run one or more suites (matches positional args against manifest entries)
python dashboard.py run "suites/dfe/*.yaml"
python dashboard.py run "suites/**/*.yaml" --generate-only
```

`build` and `validate` share the exact same validation code path, so any
manifest issue surfaces with identical wording in either verb.

`build` writes (paths relative to `<site_root>` from the manifest, or
overridden via `--data-dir` / `--site-dir`):

- `<data_dir>/<slug>/data.js` and `suite.yaml` for each suite
- `<site_dir>/index.html` (landing page) and `<site_dir>/shared/{app.js,styles.css}`
- `<site_dir>/<comparison_slug>/index.html` (per-comparison detail page)

`run` stages run artifacts in `.data/<slug>/<timestamp>/` and publishes
results to `<publish_dir>/<slug>/` (default `<site_root>/comparison_data`,
override with `--publish-dir`).

The build assumes `<site_dir>` and `<data_dir>` deploy as siblings, so the
emitted HTML references suite data at `../comparison_data/<slug>/data.js`
(landing page) and `../../comparison_data/<slug>/data.js` (comparison
stubs). The default subdirectory names (`compare/`, `comparison_data/`)
match the deployed layout under `docs/` on the `benchmarks` branch.

## Syslog Kafka receiver-only benchmark

This suite measures **Kafka receiver -> local Perf**: Syslog decoding, Arrow
materialization, internal handoff and counting on pipeline core 1. It deploys
exactly three components (Python generator, Kafka broker, DFE consumer), not a
batch processor, network exporter, or remote backend. It does not measure
isolated parser speed or backend ingestion. The recorded results, image IDs and
limitations are in the
[benchmark summary](../../KAFKA_SYSLOG_RECEIVER_ONLY_BENCHMARK_SUMMARY.md).

Run from Linux or a native WSL2 checkout with Docker's Linux engine/integration,
not Windows-native Docker orchestration or a `/mnt/c` checkout. Install Python
3.11+, `curl`, coreutils, Docker with BuildKit named-context support, and the
Python dependencies in [Setup](#setup), including the orchestrator requirements.
Use a host with at least two logical CPUs (core index 1 must exist) and spare
CPU/RAM for Kafka and the generator. From a checkout containing this suite:

```bash
# Repository root; initialize submodules before building the engine.
git submodule update --init --recursive
docker pull apache/kafka:latest
(
  cd rust/otap-dataflow
  docker build --build-arg FEATURES=kafka \
    --build-context otel-arrow=../../ -f Dockerfile -t df_engine:latest .
)
docker build -t load_generator:kafka-syslog \
  tools/pipeline_perf_test/load_generator
docker run --rm df_engine:latest -h | grep 'urn:otel:receiver:kafka'

cd tools/comparison_dashboard
python dashboard.py validate
python dashboard.py run suites/dfe/dfe-logs-kafka-syslog-receiver-only.yaml \
  --generate-only
python -u dashboard.py run suites/dfe/dfe-logs-kafka-syslog-receiver-only.yaml \
  --tests 100k,200k,300k,400k,600k,800k,1000k --observation-interval 20
python dashboard.py build
python dashboard.py serve --port 3000
```

Run targets sequentially, without concurrent benchmarks or image builds. The
suite refuses to overwrite existing `load-generator`, `kafka-broker`,
`kafka-consumer`, or `backend-service` containers. It binds host loopback ports
18085 (generator HTTP), 19094 (Kafka), and 18088 (consumer admin); it does **not**
deploy a backend or bind 18087. The isolated Docker network is
`kafka-syslog-benchmark`. Pin image references in `manifest.yaml` for repeat runs.
Fresh builds reproduce the workflow, not necessarily the historical numbers:
the retained measured engine's exact build commit is unknown.

Each Kafka value is one uncompressed 1024-byte RFC 5424 log, one topic/partition,
one producer thread, scheduling batches of 100. A 10s warmup precedes the 20s
observation. Producer stop/flush and a bounded 10s drain precede final snapshots
**while the consumer admin is alive**, then its 15s shutdown. Both root
`policies.telemetry.runtime_metrics: normal` and Perf's `item_counts: true` are
required for the retained benchmark image. Missing successful log **item**
counters fail verification/reporting; message counts and zero are not fallbacks.

Visit `http://localhost:3000/compare/kafka_receiver_syslog_receiver_only/`.
A fresh checkout has no published results; `build` does not run benchmarks or
create measured zeros. The chart shows guidance until results are published.
Successful runs retain evidence in
`.data/dfe_logs_kafka_syslog_receiver_only/<timestamp>/tests/<rate>/` and publish
reports/time series under
`.site/data/suite/dfe_logs_kafka_syslog_receiver_only/`.
`build` generates the page under `.site/compare/`. Keep these local artifacts;
do not use `--clean` when preserving earlier runs. The failed initial historical
run was not published.

**Received Log Rate** selects successful local Perf logs/s; **Offered Load Rate**
is broker-confirmed input/s, not the configured target. CPU/RAM describe only the
consumer container; 100% CPU is approximately one allocated core, not a cgroup
limit. Final `verified-delivery.json` deficits mean not observed by the bounded
cutoff, not proven loss; `group_lag=0` is not drain evidence. Missing decode-error
series are unavailable, not zero. No network-output bytes or dropped-loss
estimates are emitted. The default seven high-rate cases allow backlog; only an
explicit `rates: [1000]` suite override activates full count-equality checking.

## Directory Structure

```text
tools/comparison_dashboard/
  dashboard.py        CLI: validate | build | run | serve
  manifest.yaml       Inventory + framework config (variables, meta, etc.)
  requirements.txt    Dashboard's own Python deps
  shared/             Static JS/CSS bundled with the dashboard (input to build)
  suites/             Per-binary suite definitions
  comparisons/        Comparison definitions
  .site/              Generated by `build` -- gitignored
    compare/          Static dashboard site (index.html, shared/, <slug>/index.html)
    comparison_data/  Published per-suite data (suite.yaml, data.js, <test>/...)
  .data/              Run staging area (gitignored)
```

## Hosting

The deployed site lives on the repo's `benchmarks` branch under `docs/`,
alongside the existing `docs/benchmarks/` viewer. Two flows feed it:

- **Manual data PRs**: humans (or future automation) PR new
  `docs/comparison_data/<slug>/...` into the `benchmarks` branch.
- **Automatic site rebuild**: `.github/workflows/comparison-dashboard.yml`
  watches `docs/comparison_data/**` on `benchmarks`. On any change it
  checks out main (for `dashboard.py`) plus `benchmarks` (for the data),
  runs `dashboard.py build --data-dir <target>/docs/comparison_data
  --site-dir <target>/docs/compare`, and opens a PR back to `benchmarks`
  with the rebuilt `docs/compare/`. Once that PR merges, Pages serves the
  updated site at `<owner>.github.io/<repo>/compare/`.
