# Contributing to the OTAP Pipeline project

The OTAP Pipeline project is a part of the [OTEL Arrow
Project](https://github.com/open-telemetry/otel-arrow). See the
project-level [CONTRIBUTING][] document.

[CONTRIBUTING]: ../../CONTRIBUTING.md

## OTAP-Dataflow Development Process

Use the xtask commands below depending on the stage of development:

- Run `cargo xtask quick-check` for faster local iteration while working on
  Rust changes.
- Run `cargo xtask check-benches` when bench targets or bench-only code
  changes.
- Run `cargo xtask check` before sending changes. This is the full validation
  suite and remains the required final check.

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
