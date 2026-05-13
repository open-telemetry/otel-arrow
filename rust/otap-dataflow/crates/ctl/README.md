# dfctl

`dfctl` is the command-line client for the OTAP Dataflow Engine admin API.

It is built on top of the public Rust SDK `otap-df-admin-api`, but end users
should think in terms of the installed `dfctl` command. The Rust package name
for this crate remains `otap-df-ctl`.

## Design Goals

`dfctl` is intended to be the practical control surface for local and remote
OTAP Dataflow Engine instances:

- expose the public admin SDK without inventing a parallel protocol
- keep local and remote engine workflows consistent
- provide stable machine-readable output for scripts, CI, and agents
- provide readable tables, color, diagnostics, and a TUI for humans
- make long-running operations observable with watch streams and progress
  feedback
- keep mutation flows safe through confirmation, dry-run or preflight checks,
  and clear failure reporting where possible
- keep commands, renderers, TUI panes, and tests modular enough to evolve with
  the admin API

For the full design principles, see
[docs/admin/dfctl.md](../../docs/admin/dfctl.md#design-principles).
For security and privacy behavior, see
[docs/admin/dfctl.md](../../docs/admin/dfctl.md#security-and-privacy).

## Before You Start

- The default admin target is `http://127.0.0.1:8085`.
- You can override the target with `--url` or `DFCTL_URL`.
- In this repo, build the CLI with:

```bash
cargo build -p otap-df-ctl --bin dfctl
```

Common local setup:

```bash
export CTL=./target/debug/dfctl
export DFCTL_URL=http://127.0.0.1:8085
```

## 5-Minute Quick Start

Assuming the engine is already running:

```bash
$CTL engine livez
$CTL engine readyz
$CTL engine status
$CTL config view

$CTL groups status

$CTL telemetry logs watch --tail 50
$CTL telemetry metrics get --shape compact
```

If you are pointing at a remote engine, replace `DFCTL_URL` or pass `--url`:

```bash
dfctl --url https://admin.example.com/engine-a engine readyz
```

## Common Tasks

### Check engine and groups

Use these first to confirm the engine is reachable and healthy:

```bash
dfctl config view
dfctl engine livez
dfctl engine readyz
dfctl engine status
dfctl groups status
```

### Inspect the resolved client configuration

`config view` resolves CLI flags, `DFCTL_` environment variables, and an
explicit profile file without connecting to the engine:

```bash
dfctl config view
dfctl --profile-file ./dfctl-profile.yaml config view --output json
```

The output reports the final target URL, timeout, TCP, and TLS settings. Client
key paths are redacted to a `configured` flag.

### Inspect one pipeline

```bash
export GROUP=tenant-a
export PIPE=ingest

dfctl pipelines describe "$GROUP" "$PIPE"
dfctl pipelines get "$GROUP" "$PIPE"
dfctl pipelines status "$GROUP" "$PIPE"
dfctl pipelines livez "$GROUP" "$PIPE"
dfctl pipelines readyz "$GROUP" "$PIPE"
```

Use `--output json` when you want a stable snapshot for tooling:

```bash
dfctl pipelines status "$GROUP" "$PIPE" --output json | jq .
```

### Watch logs and metrics

Human-friendly watch sessions:

```bash
dfctl telemetry logs watch --tail 50
dfctl telemetry metrics watch --shape compact
```

Scope the output when you are debugging one pipeline:

```bash
dfctl telemetry logs watch \
  --tail 100 \
  --group "$GROUP" \
  --pipeline "$PIPE" \
  --node receiver

dfctl telemetry metrics get \
  --shape compact \
  --group "$GROUP" \
  --pipeline "$PIPE" \
  --metric-name pending.sends
```

Machine-friendly watch sessions:

```bash
dfctl telemetry logs watch --tail 50 --output ndjson
dfctl telemetry metrics watch --shape compact --output ndjson
```

Agent-friendly one-shot snapshots can use an explicit dfctl envelope:

```bash
dfctl engine status --output agent-json
```

For automation-heavy sessions, `--agent` applies machine-safe defaults without
changing command behavior:

```bash
dfctl --agent engine status
DFCTL_AGENT_MODE=true dfctl telemetry logs watch --tail 20
```

Agent mode defaults one-shot commands to `agent-json`, watch commands to
`ndjson`, errors to `agent-json`, and color to `never`. Explicit flags still
win, so `dfctl --agent engine status --output json` emits plain JSON.

### Discover commands for automation and agents

Use `dfctl commands` to list runnable commands in a compact human table:

```bash
dfctl commands
```

Use JSON when a script or agent needs the command tree, arguments, aliases,
output modes, examples, and execution hints without scraping help text:

```bash
dfctl commands --output json
```

The JSON catalog uses `schemaVersion: dfctl-command-catalog/v1`. It includes
`globalArguments` once at the top level and per-command `arguments`,
`outputModes`, `examples`, `requiresAdminClient`, `interactive`, `longRunning`,
`mutation`, safety, schema, stdin, wait/watch, dry-run, and exit-code metadata.

Discover output schemas without connecting to an engine:

```bash
dfctl schemas
dfctl schemas --output json
dfctl schemas dfctl.error.v1 --output json
```

### Reconfigure a pipeline

Apply a pipeline config from a file and wait for completion:

```bash
dfctl pipelines reconfigure "$GROUP" "$PIPE" --file pipeline.yaml --wait
```

Apply from stdin instead:

```bash
cat pipeline.yaml | \
  dfctl pipelines reconfigure "$GROUP" "$PIPE" --file - --output json
```

Validate the local request shape without starting a rollout:

```bash
dfctl pipelines reconfigure "$GROUP" "$PIPE" \
  --file pipeline.yaml \
  --dry-run \
  --output json
```

If you want incremental progress instead of a single final result:

```bash
dfctl pipelines reconfigure "$GROUP" "$PIPE" \
  --file pipeline.yaml \
  --watch \
  --output ndjson
```

### Track rollout and shutdown progress

Watch a known rollout or shutdown operation:

```bash
dfctl pipelines rollouts get "$GROUP" "$PIPE" rollout-1
dfctl pipelines rollouts watch "$GROUP" "$PIPE" rollout-1

dfctl pipelines shutdown "$GROUP" "$PIPE" --dry-run --output json
dfctl pipelines shutdown "$GROUP" "$PIPE" --watch
dfctl pipelines shutdowns get "$GROUP" "$PIPE" shutdown-1
dfctl pipelines shutdowns watch "$GROUP" "$PIPE" shutdown-1
```

### Troubleshoot a stuck rollout or shutdown

Use the composite views first:

```bash
dfctl groups describe
dfctl pipelines describe "$GROUP" "$PIPE"
```

Inspect recent controller events:

```bash
dfctl groups events get --tail 20
dfctl groups events watch --kind error --tail 20

dfctl pipelines events get "$GROUP" "$PIPE" --tail 20
dfctl pipelines events watch "$GROUP" "$PIPE" --kind error --tail 20
```

Generate a diagnosis report from the current status, retained logs, and metrics:

```bash
dfctl groups diagnose shutdown
dfctl pipelines diagnose rollout "$GROUP" "$PIPE"
dfctl pipelines diagnose shutdown "$GROUP" "$PIPE" --shutdown-id shutdown-1
```

Export a support bundle for humans or other agents:

```bash
dfctl groups bundle --file shutdown-bundle.json
dfctl pipelines bundle "$GROUP" "$PIPE" --output yaml > pipeline-bundle.yaml
```

A `dfctl` support bundle is a point-in-time troubleshooting package. It is not
a live engine data bundle. It collects the evidence an operator or agent usually
needs to investigate an incident without running several commands manually:
description data, diagnosis output, retained logs, metrics, and optional
rollout or shutdown status.

Use a group bundle for fleet-wide shutdown or readiness questions. Use a
pipeline bundle when the investigation is scoped to one pipeline. Bundles are
not redacted today and may include telemetry content, endpoint names, pipeline
configuration, and other operational data. When written with `--file` on Unix,
`dfctl` creates the file with owner-only permissions.

Watch coordinated shutdown progress client-side:

```bash
dfctl groups shutdown --watch
```

`groups shutdown --watch` is a CLI-side heuristic built from repeated
`groups status` snapshots. It is useful for troubleshooting, but it is not yet
an authoritative server-side group shutdown resource.

### Connect to a remote engine with TLS

HTTPS with a custom CA:

```bash
dfctl \
  --url https://admin.example.com/engine-a \
  --ca-file /etc/otap/admin-ca.pem \
  --include-system-ca-certs true \
  engine status
```

HTTPS with mTLS:

```bash
dfctl \
  --url https://admin.example.com/engine-a \
  --ca-file /etc/otap/admin-ca.pem \
  --client-cert-file /etc/otap/admin-client.crt \
  --client-key-file /etc/otap/admin-client.key \
  engine status
```

### Use `dfctl` in scripts

Snapshot-style commands work well with `json` or `yaml`:

```bash
dfctl engine status --output json | jq .
dfctl pipelines get "$GROUP" "$PIPE" --output yaml
```

Long-running commands and watched mutations work best with `ndjson`:

```bash
dfctl telemetry metrics watch --shape compact --output ndjson

dfctl pipelines reconfigure "$GROUP" "$PIPE" \
  --file pipeline.yaml \
  --watch \
  --output ndjson
```

## Short Local Scenario

For a local repo-backed example, use
[`configs/engine-conf/topic_multitenant_isolation.yaml`](../../configs/engine-conf/topic_multitenant_isolation.yaml).
That sample binds the admin API to `127.0.0.1:8085` and includes retained log
tap support for `dfctl telemetry logs watch`.

Start the engine in one terminal:

```bash
cargo run --bin df_engine -- \
  --config configs/engine-conf/topic_multitenant_isolation.yaml
```

In another terminal:

```bash
export CTL=./target/debug/dfctl
export DFCTL_URL=http://127.0.0.1:8085
export GROUP=topic_multitenant_isolation
export PIPE=tenant_c_pipeline

$CTL pipelines status "$GROUP" "$PIPE"
$CTL telemetry logs watch --tail 20
```

Create a simple scale-up reconfigure request from the live config, apply it,
then verify the result:

```bash
$CTL pipelines get "$GROUP" "$PIPE" --output json \
  | jq '.pipeline | .policies.resources.coreAllocation.count = 2' \
  > /tmp/tenant_c_pipeline-scale-up.json

$CTL pipelines reconfigure "$GROUP" "$PIPE" \
  --file /tmp/tenant_c_pipeline-scale-up.json \
  --watch

$CTL pipelines status "$GROUP" "$PIPE" --output json \
  | jq '{runningCores, totalCores, activeGeneration, rollout}'
```

## Output and Color

Choose the output format based on the consumer:

- `human`: terminal-oriented text with optional color and tables. Use it for
  interactive use, not as a stable parsing contract.
- `json`: direct machine-readable command payload as one pretty-printed JSON
  document. Use it for scripts, `jq`, and CI snapshots.
- `yaml`: direct command payload as one YAML document. Use it for saved files,
  inspection, and hand editing.
- `agent-json`: direct payload wrapped in a `dfctl/v1` envelope with
  `schemaVersion`, `type`, `resource`, `generatedAt`, and `data`. Use it when
  an agent needs a uniform response shape and provenance.
- `ndjson`: incremental stream output with one compact JSON object per line.
  Use it for watch commands, shell pipelines, and long-running automation.

`agent-json` is not a different JSON encoding. It is JSON plus a stable
`dfctl` envelope around the same command data. `ndjson` is separate because a
long-running stream cannot be represented as one finite JSON document.

One-shot commands support:

- `--output human`
- `--output json`
- `--output yaml`
- `--output agent-json`

Bundle commands support:

- `--output json`
- `--output yaml`
- `--output agent-json`

Long-running `watch` commands support:

- `--output human`
- `--output ndjson`

Mutation commands support `human`, `json`, `yaml`, `agent-json`, and `ndjson`.
Use `--output ndjson` together with `--watch`.

`--agent` is a global shortcut for automation-safe defaults:

- one-shot read and mutation commands default to `--output agent-json`
- watch and stream commands default to `--output ndjson`
- runtime errors default to `--error-format agent-json`
- human color defaults to `--color never`

Set `DFCTL_AGENT_MODE=true` to apply the same defaults through the environment.
Pass explicit `--output`, `--error-format`, or `--color` flags to override them.

Human-readable output also supports:

- `--color auto`
- `--color always`
- `--color never`

`--color auto` is the default. It enables ANSI styling only when stdout is a
terminal and `NO_COLOR` is not set. Machine-readable outputs never include ANSI
escapes, even when `--color always` is selected.

Generate shell completion scripts with:

```bash
dfctl completions bash
dfctl completions zsh
dfctl completions fish
```

Install completion scripts into user-local shell completion directories with:

```bash
dfctl completions install bash
dfctl completions install zsh
dfctl completions install fish
```

Use `--dir` when your shell uses a custom completion directory:

```bash
dfctl completions install zsh --dir ~/.zsh/completions
```

## Diagnostics and Errors

Use `-v` or `-vv` to print client-side diagnostics to stderr while keeping
stdout reserved for command output:

```bash
dfctl -v engine status --output json
dfctl -vv config view
```

Runtime errors can be formatted for humans or automation:

```bash
dfctl --error-format text engine status
dfctl --error-format json engine status 2>error.json
dfctl --error-format agent-json engine status 2>error.json
```

Exit codes:

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 2 | Invalid CLI usage |
| 3 | Requested group, pipeline, rollout, or shutdown was not found |
| 4 | Admin API rejected the request as invalid or conflicting |
| 5 | A requested operation was accepted but failed or timed out |
| 6 | Configuration, I/O, transport, decode, or internal error |

## More Details

- CLI reference and command overview:
  [docs/admin/dfctl.md](../../docs/admin/dfctl.md)
- Live rollout and shutdown behavior:
  [docs/admin/live-reconfiguration.md](../../docs/admin/live-reconfiguration.md)
