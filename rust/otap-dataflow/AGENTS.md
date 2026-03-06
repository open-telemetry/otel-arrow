# AGENTS.md - OTAP-Dataflow

Read and follow [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines,
coding conventions, and telemetry/logging rules.

All commands below assume the working directory is the repository root
(the directory containing the top-level `Makefile` and `go/`, `rust/`, `proto/`
directories).

## Markdown rules

- Use only ASCII characters in Markdown files. No smart quotes, em-dashes, or
  Unicode symbols.

## Sanity checks

After modifying any Markdown, YAML, or Python file, run:

```bash
python3 tools/sanitycheck.py
```

This checks for non-ASCII characters, trailing whitespace, consistent line
endings (LF only), and correct indentation. Fix any errors before presenting
the result.

## After every Rust code change

After modifying any Rust file, **always** run:

```bash
cd rust/otap-dataflow && cargo xtask check
```

This runs formatting, clippy, structure checks, and tests.
Fix any warnings or errors before presenting the result.
