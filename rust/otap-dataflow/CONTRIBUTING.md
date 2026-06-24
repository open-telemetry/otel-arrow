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

## Test and example server bind addresses

When a test or example starts a listening server, bind it to the loopback
interface (`127.0.0.1`, or `[::1]` for IPv6), not an all-interfaces address
(`0.0.0.0`, or `[::]`/`[::]:0` for IPv6). Prefer an ephemeral port
(`127.0.0.1:0`) and read the assigned port back from the listener.

Windows Defender Firewall exempts loopback binds from its allow/deny prompt, so
a loopback bind avoids the repeated firewall prompts that an all-interfaces bind
triggers on every `cargo test` / `cargo xtask check` rebuild (test binaries are
named by content hash, so a granted exception does not persist across rebuilds).
Production defaults that intentionally serve external traffic may still bind
`0.0.0.0` (or `[::]`).

## Changelog entries

User-facing Rust changes are recorded in
[`CHANGELOG.md`](./CHANGELOG.md). Changelog entries are added per PR as YAML
files under [`.chloggen/`](./.chloggen/) in this directory and collapsed into
the CHANGELOG at release time.

Copy `TEMPLATE.yaml` in the `.chloggen/` directory to a new `.yaml`
file (e.g. `otlp-exporter-fix-data-loss.yaml`) and fill in the fields.

See [`.chloggen/README.md`](./.chloggen/README.md) for the full guide,
including allowed `component:` values and skip conditions.

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
