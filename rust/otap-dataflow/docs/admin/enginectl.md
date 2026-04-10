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
dfctl groups status|shutdown
dfctl pipelines get|status|livez|readyz|reconfigure|shutdown
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

Force terminal styling for a human watch session:

```bash
dfctl --color always telemetry logs watch --tail 50
```

Watch compact metrics snapshots:

```bash
dfctl telemetry metrics watch --shape compact --output ndjson
```
