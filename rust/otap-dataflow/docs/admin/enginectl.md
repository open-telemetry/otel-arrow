# df_enginectl

`df_enginectl` is the OTAP Dataflow Engine command-line client built on top of
the public Rust admin SDK, `otap-df-admin-api`.

It is intended to be:

- easy to use for humans from a terminal
- explicit and deterministic for shell scripts
- machine-friendly for agent-style automation

## Command Overview

```text
df_enginectl engine status|livez|readyz
df_enginectl groups status|shutdown
df_enginectl pipelines get|status|livez|readyz|reconfigure|shutdown
df_enginectl pipelines rollouts get|watch
df_enginectl pipelines shutdowns get|watch
df_enginectl telemetry logs get|watch
df_enginectl telemetry metrics get|watch
```

## Connection Resolution

`df_enginectl` resolves connection settings in this order:

1. CLI flags
1. environment variables with the `DF_ENGINECTL_` prefix
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
df_enginectl --url https://admin.example.com/engine-a engine readyz
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

## Examples

Read engine status from the default local target:

```bash
df_enginectl engine status
```

Get a pipeline status snapshot as JSON:

```bash
df_enginectl pipelines status tenant-a ingest --output json
```

Submit a pipeline update and wait for completion:

```bash
df_enginectl pipelines reconfigure tenant-a ingest \
  --file pipeline.yaml \
  --wait
```

Submit a pipeline update and keep watching rollout progress:

```bash
df_enginectl pipelines reconfigure tenant-a ingest \
  --file pipeline.yaml \
  --watch \
  --output ndjson
```

Watch retained logs:

```bash
df_enginectl telemetry logs watch --tail 50
```

Watch compact metrics snapshots:

```bash
df_enginectl telemetry metrics watch --shape compact --output ndjson
```
