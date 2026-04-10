# dfctl

`dfctl` is the OTAP Dataflow Engine command-line client built on top of
the public Rust admin SDK, `otap-df-admin-api`.

It is intended to be:

- easy to use for humans from a terminal
- explicit and deterministic for shell scripts
- machine-friendly for agent-style automation

## Command Overview

```text
dfctl engine status|livez|readyz
dfctl groups describe|status|shutdown
dfctl groups events get|watch
dfctl groups diagnose shutdown
dfctl groups bundle
dfctl pipelines get|describe|status|livez|readyz|reconfigure|shutdown
dfctl pipelines events get|watch
dfctl pipelines diagnose rollout|shutdown
dfctl pipelines bundle
dfctl pipelines rollouts get|watch
dfctl pipelines shutdowns get|watch
dfctl telemetry logs get|watch
dfctl telemetry metrics get|watch
```

## Connection Resolution

`dfctl` resolves connection settings in this order:

1. CLI flags
1. environment variables with the `DFCTL_` prefix
1. an explicit YAML profile passed with `--profile-file`
1. the default local URL: `http://127.0.0.1:8085`

Common flags include:

- `--url`
- `--connect-timeout`
- `--request-timeout`
- `--no-request-timeout`
- `--ca-file`
- `--client-cert-file`
- `--client-key-file`
- `--include-system-ca-certs`
- `--insecure-skip-verify`

Example:

```bash
dfctl --url https://admin.example.com/engine-a engine readyz
```

## Output Modes

One-shot commands support:

- `--output human`
- `--output json`
- `--output yaml`

Long-running `watch` commands support:

- `--output human`
- `--output ndjson`

Mutation commands also support `--output ndjson` when used together with
`--watch`.

Bundle commands support `--output json` and `--output yaml`.

## Color Policy

Human-readable output supports:

- `--color auto`
- `--color always`
- `--color never`

`--color auto` is the default. It enables ANSI styling only when stdout is a
terminal and `NO_COLOR` is not set.

`json`, `yaml`, and `ndjson` outputs stay unstyled regardless of the color
setting.

## Examples

Read engine status from the default local target:

```bash
dfctl engine status
```

Get a pipeline status snapshot as JSON:

```bash
dfctl pipelines status tenant-a ingest --output json
```

Submit a pipeline update and wait for completion:

```bash
dfctl pipelines reconfigure tenant-a ingest \
  --file pipeline.yaml \
  --wait
```

Submit a pipeline update and keep watching rollout progress:

```bash
dfctl pipelines reconfigure tenant-a ingest \
  --file pipeline.yaml \
  --watch \
  --output ndjson
```

Watch retained logs:

```bash
dfctl telemetry logs watch --tail 50
```

Watch retained logs for one pipeline only:

```bash
dfctl telemetry logs watch \
  --tail 100 \
  --group tenant-a \
  --pipeline ingest \
  --node receiver
```

Force terminal styling for a human watch session:

```bash
dfctl --color always telemetry logs watch --tail 50
```

Watch compact metrics snapshots:

```bash
dfctl telemetry metrics watch --shape compact --output ndjson
```

Describe one pipeline with status, probes, and recent events:

```bash
dfctl pipelines describe tenant-a ingest
```

Watch recent pipeline events:

```bash
dfctl pipelines events watch tenant-a ingest --kind error --tail 20
```

Diagnose a stuck shutdown:

```bash
dfctl pipelines diagnose shutdown tenant-a ingest --shutdown-id shutdown-3
```

Export a support bundle:

```bash
dfctl pipelines bundle tenant-a ingest --file pipeline-bundle.json
```

Watch coordinated group shutdown progress:

```bash
dfctl groups shutdown --watch
```

`groups shutdown --watch` is currently a CLI-side heuristic built from repeated
`groups status` polling. A future improvement should replace this with a
first-class server-side group shutdown operation resource and matching SDK
support.
