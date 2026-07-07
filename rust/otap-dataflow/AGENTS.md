# AGENTS.md - OTAP-Dataflow

Read and follow [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines,
coding conventions, and telemetry/logging rules.

All commands below assume the working directory is the repository root
(the directory containing the top-level `Makefile` and `go/`, `rust/`, `proto/`
directories).

## Markdown rules

- After editing Markdown files, run markdownlint on the changed files (requires
  Node.js/npx to be installed):

```bash
npx markdownlint-cli2 <file1.md> <file2.md>
```

## Sanity checks

After modifying any Markdown, YAML, or Python file, run:

```bash
python3 tools/sanitycheck.py
```

Fix any errors before committing.

## Component naming conventions

When adding a new component, keep public names consistent across the module,
URN, and primary telemetry metric set. Prefer snake_case component names for
consistency. Component IDs in URNs may use lowercase letters, digits,
underscores (`_`), hyphens (`-`), and dots (`.`).

Component URNs should use:

```text
urn:otel:<component_kind>:<component_name>
```

Examples:

```text
urn:otel:receiver:journald
urn:otel:processor:transform
urn:otel:exporter:topic
```

Primary metric set names for new components should generally use:

```text
<component_kind>.<component_name>
```

Examples:

```text
receiver.journald
receiver.host_metrics
processor.transform
processor.filter.pdata
exporter.topic
exporter.azure_monitor
```

Use established component-specific prefixes or suffixes only when they already
exist for that component family, such as `.pdata` for pdata-specific metrics or
existing `otap.*` component families. Do not introduce reversed or redundant
names such as `journald.receiver.metrics`.

## After every Rust code change

After modifying any Rust file, run a quick compile check on the affected crate:

```bash
cd rust/otap-dataflow && cargo check -p <crate_name>
```

For broader iterative validation while you are still working, you can also run:

```bash
cd rust/otap-dataflow && cargo xtask quick-check
```

This is an iterative subset only:

- it runs structure checks, `cargo fmt --all`, and clippy on
  `--workspace --lib --bins --tests`
- it only compiles test targets with `cargo test --workspace --lib --bins
  --tests --no-run`
- it does not replace the final full `cargo xtask check`

If you touched bench targets or bench-only code, you can validate them with:

```bash
cd rust/otap-dataflow && cargo xtask check-benches
```

If you need to troubleshoot slow checks or identify compile and test hotspots,
you can run:

```bash
cd rust/otap-dataflow && cargo xtask check --diagnostics
```

Interpretation guidance for this output is documented in
[`docs/xtask-diagnostics.md`](docs/xtask-diagnostics.md).

## Changelog entries

If your Rust change is user-facing, add a changelog entry by **copying
[`.chloggen/TEMPLATE.yaml`](.chloggen/TEMPLATE.yaml)** to a new `.yaml` file
in [`.chloggen/`](.chloggen/) (e.g. `otlp-exporter-fix-data-loss.yaml`) and
filling in the fields.

Required fields: `change_type` (one of `breaking`, `deprecation`,
`new_component`, `enhancement`, `bug_fix`), `component` (must be listed in
[`.chloggen/config.yaml`](.chloggen/config.yaml)), `note`, and `issues`.

Skip the entry only when the change is not user-facing. In that case include
`chore` in the PR title.

Doc-only PRs are also excluded from the changelog requirement.

See [`.chloggen/README.md`](.chloggen/README.md) for full details.

## Before finalizing changes

When all changes are ready, run the full check suite:

```bash
cd rust/otap-dataflow && cargo xtask check
```

This is the required full validation path. It runs:

- structure checks
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Fix any warnings or errors before committing.
