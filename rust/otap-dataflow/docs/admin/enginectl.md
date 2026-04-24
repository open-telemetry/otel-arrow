# dfctl

`dfctl` is the OTAP Dataflow Engine command-line client built on top of
the public Rust admin SDK, `otap-df-admin-api`.

It is intended to be:

- easy to use for humans from a terminal
- explicit and deterministic for shell scripts
- machine-friendly for agent-style automation

## Command Overview

```text
dfctl completions <shell>
dfctl completions install <shell>
dfctl commands
dfctl config view
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

Inspect the resolved client configuration without contacting the engine:

```bash
dfctl config view
dfctl --profile-file ./dfctl-profile.yaml config view --output json
```

## Output Modes

One-shot commands support:

- `--output human`
- `--output json`
- `--output yaml`
- `--output agent-json`

Long-running `watch` commands support:

- `--output human`
- `--output ndjson`

Mutation commands support `--output human`, `--output json`, `--output yaml`,
and `--output agent-json` for single responses. Use `--output ndjson` with
`--watch`.

Bundle commands support `--output json`, `--output yaml`, and
`--output agent-json`.

## Command Catalog

`dfctl commands` emits a human-readable table of runnable commands. It is local
only and does not connect to the admin API:

```bash
dfctl commands
```

Use JSON for scripts and agents that need to discover command paths,
arguments, aliases, output modes, examples, and execution hints:

```bash
dfctl commands --output json
```

The catalog is generated from the clap command tree and uses
`schemaVersion: dfctl-command-catalog/v1`. Top-level `globalArguments` apply to
all commands. Each command entry includes local `arguments`, `outputModes`,
`examples`, `requiresAdminClient`, `interactive`, `longRunning`, and `mutation`.

## Shell Completions

Generate a completion script on stdout:

```bash
dfctl completions bash
dfctl completions zsh
dfctl completions fish
```

Install a completion script into the default user-local directory for the
selected shell:

```bash
dfctl completions install bash
dfctl completions install zsh
dfctl completions install fish
```

Use `--dir` for custom shell completion directories:

```bash
dfctl completions install zsh --dir ~/.zsh/completions
```

## Color Policy

Human-readable output supports:

- `--color auto`
- `--color always`
- `--color never`

`--color auto` is the default. It enables ANSI styling only when stdout is a
terminal and `NO_COLOR` is not set.

`json`, `yaml`, and `ndjson` outputs stay unstyled regardless of the color
setting.

## Diagnostics and Errors

Use `-v` or `-vv` to emit client-side diagnostics on stderr without changing
stdout:

```bash
dfctl -v engine status --output json
dfctl -vv config view
```

Runtime errors support text and structured stderr formats:

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
