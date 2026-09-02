# Copilot Instructions for otel-arrow

See [AGENTS.md](../AGENTS.md) for build, test, and contribution conventions,
including changelog entries, ASCII-only Rust source, and test documentation.

## Reviewing pull requests

When reviewing changes under `rust/`, follow the project's review guide,
[AI-Assisted Pull Request Review][review-guide]. It defines what matters in an
OTAP Dataflow review: architectural invariants, runtime and performance
characteristics, bounded resources, backpressure, correctness, semantic
fidelity, test adequacy, and security.

The highest-signal rules are restated as path-specific instructions in
[.github/instructions](instructions/), which apply automatically to matching
files.

Report only risks supported by the diff, nearby code, or project guidance, and
avoid stylistic nitpicks unless they affect correctness, readability, or
maintainability.

[review-guide]: ../rust/otap-dataflow/docs/ai/ai-assisted-pr-review.md
