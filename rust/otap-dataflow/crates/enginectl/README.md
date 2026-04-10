# df_enginectl

`df_enginectl` is a control CLI for the OTAP Dataflow Engine admin API.

It is built on top of the public `otap-df-admin-api` SDK and is intended to
work well both for humans at a terminal and for automation in shell scripts or
agent workflows.

## Highlights

- supports the current public admin SDK surface
- works against local or remote engines
- exposes human-readable output by default
- supports `json`, `yaml`, and `ndjson` machine-oriented output modes
- supports long-running `watch` commands for logs, metrics, rollouts, and
  shutdown tracking

## Quick Start

Local engine using the default admin address:

```bash
df_enginectl engine status
df_enginectl telemetry logs watch
```

Remote engine behind a gateway:

```bash
df_enginectl --url https://admin.example.com/engine-a engine readyz
df_enginectl --url https://admin.example.com/engine-a \
  telemetry metrics get --shape full --output json
```

Pipeline reconfigure from a YAML file:

```bash
df_enginectl pipelines reconfigure tenant-a ingest --file pipeline.yaml --wait
```

Pipeline reconfigure from stdin:

```bash
cat pipeline.yaml | \
  df_enginectl pipelines reconfigure tenant-a ingest --file - --output json
```

## Connection Configuration

By default, `df_enginectl` targets `http://127.0.0.1:8085`.

You can override connection settings with:

- CLI flags such as `--url`, `--ca-file`, and `--client-cert-file`
- environment variables using the `DF_ENGINECTL_` prefix
- an explicit YAML profile passed with `--profile-file`

Precedence is:

1. CLI flags
2. environment variables
3. profile file
4. default local URL

Example profile:

```yaml
url: https://admin.example.com/engine-a
connect_timeout: 3s
request_timeout: 10s
ca_file: /etc/otap/admin-ca.pem
include_system_ca_certs: true
```

## Output Modes

One-shot commands support:

- `--output human`
- `--output json`
- `--output yaml`

Long-running `watch` commands support:

- `--output human`
- `--output ndjson`

Mutation commands support `human`, `json`, `yaml`, and `ndjson`. Use
`--output ndjson` together with `--watch`.
