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
npx markdownlint-cli <file1.md> <file2.md>
```

## Sanity checks

After modifying any Markdown, YAML, or Python file, run:

```bash
python3 tools/sanitycheck.py
```

Fix any errors before committing.

## After every Rust code change

After modifying any Rust file, run a quick compile check on the affected crate:

```bash
cd rust/otap-dataflow && cargo check -p <crate_name>
```

For broader iterative validation while you are still working, you can also run:

```bash
cd rust/otap-dataflow && cargo xtask quick-check
```

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

## Before finalizing changes

When all changes are ready, run the full check suite:

```bash
cd rust/otap-dataflow && cargo xtask check
```

This runs formatting, clippy, structure checks, and tests.
Fix any warnings or errors before committing.
