# Contributing to the OTAP Pipeline project

The OTAP Pipeline project is a part of the [OTEL Arrow
Project](https://github.com/open-telemetry/otel-arrow). See the
project-level [CONTRIBUTING][] document.

[CONTRIBUTING]: ../../CONTRIBUTING.md

## OTAP-Dataflow Development Process

Use the xtask commands below depending on the stage of development:

- Run `cargo xtask quick-check` for faster local iteration while working on
  Rust changes. This runs a narrower subset of the full checks and only
  compiles test targets instead of running the full workspace test suite.
- Run `cargo xtask check-benches` when bench targets or bench-only code
  changes.
- Run `cargo xtask check --diagnostics` when you need a timing-oriented summary
  of slow check phases, compile hotspots, or test binaries. See
  [docs/xtask-diagnostics.md](docs/xtask-diagnostics.md).
- Run `cargo xtask check` before sending changes. This is the required full
  validation suite: structure checks, formatting, clippy on `--all-targets`,
  and `cargo test --workspace`.

For this workspace, we keep `cargo test --workspace` as the default full test
runner instead of `nextest`. In local measurements, `nextest` was slower for
the full check path, likely because many of the longest tests are concentrated
in a few large integration-style binaries, so the extra runner orchestration did
not offset the limited parallelism gains.

## Telemetry and logging

All internal logging MUST use the `otel_*` macros from `otap_df_telemetry`
(not `tracing::info!` or `println!`). See the
[Events Guide](docs/telemetry/events-guide.md) details.

TODO: Add metrics information

## Building a Docker image

Run

```bash
docker build \
  --build-context otel-arrow=../../ \
  -f Dockerfile \
  -t df_engine \
  .
