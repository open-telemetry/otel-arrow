# dfctl

`dfctl` is the OTAP Dataflow Engine command-line client built on top of
the public Rust admin SDK, `otap-df-admin-api`.

It is intended to be:

- easy to use for humans from a terminal
- explicit and deterministic for shell scripts
- machine-friendly for agent-style automation

## Design Principles

`dfctl` is designed as the operator-facing control surface for the admin API.
These principles guide command design, output contracts, and TUI behavior:

- The public `otap-df-admin-api` SDK is the source of truth. The CLI should
  expose SDK capabilities directly and avoid duplicating protocol logic.
- Local and remote engines should have the same user experience. Target URL,
  TLS, profile, and environment resolution should be consistent for every
  command.
- Automation and interactive use are both first-class. Scripts and agents need
  stable machine-readable output, while humans need readable tables, color,
  diagnostics, and a useful TUI.
- Output contracts must be explicit. `human`, `json`, `yaml`, `agent-json`,
  and `ndjson` exist for different consumers and should not require scraping
  terminal help text.
- Long-running operations need live feedback. Rollouts, shutdowns, log streams,
  metric streams, and TUI refreshes should show ongoing progress clearly.
- Operational clarity is more important than raw API dumps. Status, readiness,
  diagnosis, vitals, logs, metrics, and object details should be shaped around
  what an operator needs to understand quickly.
- Mutations should be safe by default. Shutdown, scale, reconfigure, and
  redeploy flows should support confirmation, dry-run or preflight checks,
  wait or watch modes, and clear failure reporting where possible.
- The TUI should complement the CLI, not replace it. Interactive screens should
  help users inspect objects, run actions, and learn the equivalent
  non-interactive command.
- Agent readiness is part of the CLI contract. Command catalog metadata,
  schemas, stable errors, examples, and safety metadata should make commands
  discoverable without human-only parsing.
- Extensibility and maintainability matter. Commands, renderers, TUI panes,
  recipes, and tests should stay modular enough to expose new admin SDK
  features without large rewrites.
- Troubleshooting should gather evidence before guessing. Composite views,
  diagnoses, events, logs, metrics, and bundles should help operators correlate
  failures across admin endpoints.
- User-interface polish is valuable when it preserves function. Color, tables,
  mouse support, command palettes, detail panes, and activity indicators should
  improve usability without weakening scriptability or layout stability.
- Tests should document behavior. CLI contracts and TUI rendering tests should
  describe the scenario and the guarantee they protect.

## Security and Privacy

`dfctl` is an operational tool and can expose sensitive engine, pipeline, and
telemetry data. The current implementation has several guardrails, but support
bundles and telemetry outputs are not redacted by default.

### What Is Already In Place

- Connection setup supports HTTPS, custom CA files, system CA selection, mTLS
  client certificates and keys, and the explicit `--insecure-skip-verify`
  escape hatch.
- TLS-enabled builds require one configured Rustls crypto provider before the
  admin SDK client is used.
- `config view` reports whether a client private key is configured, but does
  not print the private key path.
- The TUI equivalent-command overlay redacts the client private key path as
  `<client-key-file>` and labels the generated command as redacted.
- Support bundles written with `--file` are created with owner-only `0600`
  permissions on Unix.
- Client-side diagnostics use stderr, preserving stdout for command output and
  reducing accidental mixing between machine-readable data and diagnostics.
- Machine-readable outputs do not include ANSI escape sequences, even when
  color is forced for human output.
- Human renderers and TUI row builders neutralize terminal control sequences in
  engine-provided text before writing them to a terminal.
- Mutating commands expose dry-run or preflight paths, and the command catalog
  marks high-impact commands plus dry-run, wait, watch, stdin, and idempotency
  metadata for automation clients.

### How This Is Tested

The current test suite covers the security and privacy guardrails above through
the following scenarios:

- `config_view_json_reports_resolved_connection` verifies local config
  resolution and client-key path redaction.
- `build_command_context_redacts_client_key_path` verifies TUI equivalent
  command redaction for private key paths.
- `terminal_safe_strips_escape_sequences` verifies terminal control sequence
  neutralization.
- `metrics_json_output_ignores_color_setting` verifies machine-readable output
  stays ANSI-free even when color is forced.
- `verbose_config_view_writes_diagnostics_to_stderr` verifies diagnostics stay
  on stderr while stdout remains command output.
- `reconfigure_dry_run_emits_preflight_outcome` and
  `shutdown_dry_run_emits_preflight_outcome` verify mutation preflight output.
- `catalog_marks_mutation_safety_and_preflight_metadata` verifies command
  catalog safety, dry-run, wait, watch, stdin, and idempotency metadata.

### Known Security and Privacy Gaps

- Support bundles, logs, metrics, diagnosis evidence, and pipeline
  configuration are not redacted today and may contain sensitive telemetry or
  operational data.
- There is no `--redact`, `--include-sensitive`, or field-level redaction
  policy for bundle or telemetry output.
- The support bundle schema does not include sensitivity classification or
  redaction metadata.
- Unix private bundle file permissions are implemented, but there is no direct
  regression test for the `0600` file mode and no equivalent Windows ACL
  hardening guarantee.
- dfctl tests cover TLS option resolution, but not end-to-end TLS or mTLS
  handshake behavior.
- Errors and diagnostics may include server-provided messages; there is no
  centralized secret scanner or redactor for arbitrary remote error text.
- `--insecure-skip-verify` is explicit, but the CLI does not add an additional
  human warning beyond the flag name and resolved config output.

## Command Overview

```text
dfctl completions <shell>
dfctl completions install <shell>
dfctl commands
dfctl schemas [schema-name]
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

Long-running `watch` commands support:

- `--output human`
- `--output ndjson`

Mutation commands support `--output human`, `--output json`, `--output yaml`,
and `--output agent-json` for single responses. Use `--output ndjson` with
`--watch`.

Bundle commands support `--output json`, `--output yaml`, and
`--output agent-json`.

Use `--agent` or `DFCTL_AGENT_MODE=true` when running in automation-heavy
contexts. Agent mode changes defaults only:

- one-shot read and mutation commands default to `--output agent-json`
- watch commands default to `--output ndjson`
- runtime errors default to `--error-format agent-json`
- human color defaults to `--color never`

Explicit `--output`, `--error-format`, and `--color` flags always win.

## Support Bundles

A `dfctl` support bundle is a point-in-time troubleshooting package, not a live
engine data bundle. It is intended for incident handoff between operators,
automation, and agents when a single command should capture the evidence needed
to investigate a problem.

Group bundles collect fleet-wide status, diagnosis, retained logs, and metrics.
Pipeline bundles collect the same kind of evidence for one pipeline and may also
include rollout or shutdown status when requested. Use bundles when a diagnosis
needs to be shared, archived, or inspected offline; use `describe`, `diagnose`,
`events`, `logs`, or `metrics` directly when you only need one narrow view.

Bundle output supports `json`, `yaml`, and `agent-json`. Bundles are not
redacted today and may contain sensitive telemetry, endpoint names, pipeline
configuration, diagnosis evidence, and operational metadata. Files written with
`--file` are created with owner-only permissions on Unix.

## Command Catalog

`dfctl commands` emits a human-readable table of runnable commands. It is local
only and does not connect to the admin API:

```bash
dfctl commands
```

Use JSON for scripts and agents that need to discover command paths,
arguments, aliases, output modes, examples, schemas, safety level, stdin
support, dry-run support, and execution hints:

```bash
dfctl commands --output json
```

The catalog is generated from the clap command tree and uses
`schemaVersion: dfctl-command-catalog/v1`. Top-level `globalArguments` apply to
all commands.

Discover output schemas without connecting to the admin API:

```bash
dfctl schemas
dfctl schemas --output json
dfctl schemas dfctl.error.v1 --output json
```

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

Validate a local reconfigure request without creating a rollout:

```bash
dfctl pipelines reconfigure tenant-a ingest \
  --file pipeline.yaml \
  --dry-run \
  --output json
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
dfctl groups shutdown --dry-run --output json
dfctl groups shutdown --watch
```

`groups shutdown --watch` is currently a CLI-side heuristic built from repeated
`groups status` polling. A future improvement should replace this with a
first-class server-side group shutdown operation resource and matching SDK
support.
