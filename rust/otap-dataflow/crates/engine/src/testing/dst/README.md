# Deterministic Simulation Testing

This directory contains the OTAP dataflow engine's deterministic simulation
testing (DST) scenarios.

## Test-only compilation

The reusable `dst` module is exported from
`crates/engine/src/testing/mod.rs` behind
`#[cfg(any(test, feature = "test-utils"))]`, but the scenario files in this
directory are wired from `mod.rs` with `#[cfg(test)]`. In practice, that means:

- the seeded scenario suites in this directory are compiled for engine test
  builds such as `cargo test`
- they are not part of normal production builds of `otap-df-engine`

## What DST is

Deterministic simulation testing is the layer we use for concurrency-sensitive
engine behavior that is hard to validate with ordinary unit tests.

Instead of writing a separate simulator, the DST harness runs the real engine
actors on the same kind of single-threaded runtime used by local engine
components, while replacing the non-deterministic pieces with deterministic
test control:

- `SimClock` gives tests explicit control over time, deadlines, and timer
  wakeups
- `DstRng` varies action ordering and pressure patterns while remaining fully
  replayable from a seed
- `dst_seeds` combines fixed regression seeds with optional environment-driven
  sweeps via `DST_SEED` and `DST_SEEDS`

## Our approach

The goal of these tests is to exercise real engine code under replayable
interleavings.

The DST suite intentionally uses production components such as:

- `ProcessorMessageChannel`
- `ExporterMessageChannel`
- `RuntimeCtrlMsgManager`
- `PipelineCompletionMsgDispatcher`

This keeps the tests close to real runtime behavior and lets us validate
properties such as bounded progress, Ack/Nack unwinding, and graceful shutdown
ordering without reimplementing those mechanisms in a separate model.

## Why this is useful

DST gives us three benefits that are difficult to get from ordinary async tests
alone:

- deterministic replay of failures from a printed seed
- explicit control over time-dependent behavior such as drain deadlines and
  delayed-data wakeups
- realistic validation of liveness-sensitive engine behavior with much lower
  flake than broad stress tests

This makes DST a good fit for engine fairness, shutdown ordering, control-plane
isolation, and other behaviors where "eventually makes progress" is part of the
contract.

## Existing scenario suites

The current seeded scenarios in this directory are:

- `message_channel.rs`
  Validates bounded-fair progress between control and `pdata`, processor
  shutdown draining after admission reopens, and exporter shutdown draining of
  buffered `pdata` even when normal admission stays closed.
- `control_plane.rs`
  Validates runtime-control and completion-path behavior under pressure,
  including timer and delayed-data progress, Ack/Nack unwind correctness,
  `RETURN_DATA` retention behavior, and receiver-first shutdown ordering.
- `heavy_ingress.rs`
  Exercises a realistic receiver -> processor -> exporter flow under sustained
  ingress, bounded channel pressure, processor admission gating, mixed
  Ack/Nack completions, runtime-control noise, and clean shutdown sequencing.
- `closed_admission.rs`
  Keeps the current known limitation explicit: if processor admission stays
  closed until the shutdown deadline, buffered `pdata` remains stranded and is
  abandoned when forced shutdown is returned.

Supporting helpers shared by those suites live in `common.rs`.

## Related DST coverage

Receiver-side deterministic shutdown and `wait_for_result` coverage also exists
outside this directory, notably in the OTLP receiver tests. Those tests use the
same `SimClock` and seed-driven approach, but stay next to the receiver code
because they validate receiver-specific terminal behaviors rather than generic
engine internals.
