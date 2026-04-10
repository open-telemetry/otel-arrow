# dfctl

`dfctl` is the command-line client for the OTAP Dataflow Engine admin API.

It is built on top of the public Rust SDK `otap-df-admin-api`, but end users
should think in terms of the installed `dfctl` command. The Rust package name
for this crate remains `otap-df-enginectl`.

## Before You Start

- The default admin target is `http://127.0.0.1:8085`.
- You can override the target with `--url` or `DFCTL_URL`.
- In this repo, build the CLI with:

```bash
cargo build -p otap-df-enginectl --bin dfctl
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
dfctl engine livez
dfctl engine readyz
dfctl engine status
dfctl groups status
```

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

One-shot commands support:

- `--output human`
- `--output json`
- `--output yaml`

Bundle commands support:

- `--output json`
- `--output yaml`

Long-running `watch` commands support:

- `--output human`
- `--output ndjson`

Mutation commands support `human`, `json`, `yaml`, and `ndjson`. Use
`--output ndjson` together with `--watch`.

Human-readable output also supports:

- `--color auto`
- `--color always`
- `--color never`

`--color auto` is the default. It enables ANSI styling only when stdout is a
terminal and `NO_COLOR` is not set. Machine-readable outputs never include ANSI
escapes, even when `--color always` is selected.

## More Details

- CLI reference and command overview:
  [docs/admin/enginectl.md](../../docs/admin/enginectl.md)
- Live rollout and shutdown behavior:
  [docs/admin/live-reconfiguration.md](../../docs/admin/live-reconfiguration.md)
