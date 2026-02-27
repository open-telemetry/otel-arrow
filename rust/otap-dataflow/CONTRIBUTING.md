# Contributing to the OTAP Pipeline project

The OTAP Pipeline project is a part of the [OTEL Arrow
Project](https://github.com/open-telemetry/otel-arrow). See the
project-level [CONTRIBUTING][] document.

[CONTRIBUTING]: ../../CONTRIBUTING.md

## OTAP-Dataflow Development Process

Run `cargo xtask check` to check the structure of the project.

## Telemetry and logging

All internal logging MUST use the `otel_*` macros from `otap_df_telemetry`
(not `tracing::info!` or `println!`). See the
[Events Guide](docs/telemetry/events-guide.md) details.

TODO: Add metrics information

## Building a Docker image

Run `docker build  --build-context otel-arrow=../../ -f Dockerfile -t df_engine .`
