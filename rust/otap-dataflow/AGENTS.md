# AGENTS.md — OTAP-Dataflow

Read and follow [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines,
coding conventions, and telemetry/logging rules.

## After every code change

After modifying any Rust file, **always** run this command before considering
the task complete:

```bash
cargo xtask check
```

This runs formatting, clippy, structure checks, and tests.
Fix any warnings or errors before presenting the result.
