# Testing Guide

This guide describes the current testing strategy for `rust/otap-dataflow`.
The project intentionally uses multiple complementary test layers rather than
one universal test style.

The core rule is: use the smallest test layer that can prove the property you
care about. No single layer covers functional correctness, liveness, graceful
shutdown ordering, and end-to-end protocol wiring equally well.

## Property Taxonomy

These terms are used throughout the guide:

- `functional correctness`: pure logic, state transitions, parsing, and local
  transformations produce the expected result
- `integration / protocol correctness`: real node combinations, OTLP/OTAP
  wiring, and controller/runtime setup behave correctly together
- `Ack/Nack correctness`: completion subscriptions, unwinding, retry/refusal
  behavior, and terminal outcomes are routed as intended
- `liveness / eventual progress`: admitted work eventually moves forward or
  reaches an explicit terminal outcome instead of getting stuck
- `graceful shutdown / drain correctness`: receiver-first drain, shutdown
  ordering, deadlines, and forced completion behave as designed
- `stress / performance`: high-volume or long-running scenarios expose
  throughput, memory, or backlog behavior under pressure

## Testing Layers

| Layer | Typical location / harness | Best for | Main properties covered | What it does not prove |
| --- | --- | --- | --- | --- |
| Standard unit tests | crate-local `#[test]` modules | Pure local logic | Functional correctness, local state transitions, config parsing, data structures | Real async wiring, runtime fairness, end-to-end protocol behavior |
| Node-level harness tests | `crates/engine/src/testing/{receiver,processor,exporter}.rs` | Single-node behavior with real engine-style wiring | Functional correctness, Ack/Nack behavior at one node, selected liveness or shutdown paths | Multi-node interactions, real protocol interoperability, controller behavior |
| Small pipeline liveness / integration tests | `crates/otap/tests/core_node_liveness_tests.rs` and similar | Representative real-node combinations through the runtime | Liveness, shutdown/drain behavior, retry/batch/exporter interaction, real runtime wiring | Exhaustive topology coverage, protocol-semantic equivalence, full controller-driven validation |
| Validation framework | `crates/validation` | End-to-end SUV validation with traffic generation and capture | Integration/protocol correctness, semantic equivalence, signal drop, attribute or batch-shape checks | Fine-grained concurrency ordering, engine-internal liveness proofs, exhaustive shutdown analysis |
| Deterministic simulation testing (DST) | `crates/engine/src/testing/dst` plus receiver DST | Concurrency-sensitive engine behavior under controlled interleavings | Liveness, graceful shutdown ordering, Ack/Nack correctness, bounded progress under mixed load | Ordinary business logic, transport interoperability, every cross-pipeline or fatal-failure case |
| Continuous and nightly benchmark suites | `../../tools/pipeline_perf_test/test_suites/integration/{continuous,nightly}` via CI on a dedicated server | Throughput, backpressure, idle-state, and comparative performance tracking | Stress/performance behavior, sustained-load regressions, benchmark trending across representative pipeline scenarios | Fast local debugging, detailed concurrency proofs, broad functional correctness by themselves |

## When To Use What

Use the smallest layer that gives a trustworthy answer:

- Pure local logic bug or parsing/state bug: start with a standard unit test.
- Node behavior involving effect handlers, control messages, or node-local
  timers: use a node-level harness test.
- Engine fairness, Ack/Nack unwinding, shutdown ordering, or mixed control and
  `pdata` pressure: use DST.
- Representative real-node progress through a runtime pipeline: use a small
  pipeline liveness / integration test.
- End-to-end semantic equivalence, signal-drop checks, protocol wiring, or
  attribute/batch validation: use `otap-df-validation`.
- High-volume, backpressure, saturation, or idle-state behavior where the main
  question is performance or regression tracking: use the continuous or nightly
  benchmark suites.

Common anti-patterns:

- Do not use the validation framework for fast local logic checks.
- Do not use DST for ordinary business logic that has no concurrency risk.
- Do not use node-level harness tests to claim real OTLP/OTAP interoperability.
- Do not treat small pipeline liveness tests as exhaustive topology proof.

## Current Test Surfaces

The current repo already exposes several reusable testing surfaces:

- engine component harnesses in `crates/engine/src/testing`
- shared liveness helpers in `crates/engine/src/testing/liveness.rs`
- deterministic simulation testing in `crates/engine/src/testing/dst`
- small end-to-end liveness tests in `crates/otap/tests`
- end-to-end validation scenarios in `crates/validation`
- continuous and nightly benchmark suites in
  `../../tools/pipeline_perf_test/test_suites/integration`

In practice, these layers complement each other:

- unit tests keep local logic cheap to verify
- node harnesses make single-node control/data behavior easy to isolate
- DST targets concurrency-sensitive engine logic that is hard to cover with
  conventional tests
- small pipeline liveness tests confirm real node combinations keep making
  progress once wired through the runtime
- validation scenarios confirm end-to-end traffic semantics with real
  controller/admin/runtime setup
- benchmark suites provide sustained-load and performance-regression coverage
  through CI-driven runs on a dedicated benchmark server

## Current Limits and Next Directions

The current layers have intentional boundaries:

- `otap-df-validation` is currently output/capture-oriented. It is the main
  end-to-end correctness layer, but it is not yet the main liveness oracle.
- DST is intentionally scoped. It does not yet cover every topic-based,
  cross-pipeline, or fatal-process-failure scenario.
- Small pipeline liveness tests are representative runtime checks, not
  exhaustive coverage of every node combination or stress shape.

Likely next directions:

- extend validation scenarios with liveness-oriented checks based on runtime
  metrics plus `/status` or event polling
- continue growing node-local and DST coverage for the most concurrency-
  sensitive processors, receivers, and exporters

## Example Commands

Representative commands:

```bash
# Target a fast local unit or harness test in one crate.
cargo test -p otap-df-engine local::processor::tests::

# Replay one deterministic simulation run.
DST_SEED=17 cargo test -p otap-df-engine dst -- --nocapture

# Run a larger DST sweep.
DST_SEEDS=25 cargo test -p otap-df-engine dst

# Run the small runtime-wired liveness scenarios.
cargo test -p otap-df-otap core_node_liveness_tests -- --nocapture

# Run validation-framework tests or a specific scenario test.
cargo test -p otap-df-validation -- --nocapture

# Final repo-wide verification before merging.
cargo xtask check
```
