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

## Rust toolchain

Both Rust workspaces pin an exact toolchain version in `rust-toolchain.toml`
rather than tracking the `stable` channel, so local checks and CI build with
the same compiler and a new Rust release never breaks unrelated pull requests.
The pin is a directory override, so run `cargo` from inside the workspace; it
does not apply to commands invoked from elsewhere via `--manifest-path`.

To move to a new toolchain, open a dedicated pull request that bumps `channel`
in both `rust/otap-dataflow/rust-toolchain.toml` and
`rust/contrib/data_engine/rust-toolchain.toml`, and include any lint or
compile fixes the new version requires. `rustup` installs the pinned toolchain
automatically on the next `cargo` invocation.

## Test documentation

Document every test immediately above its declaration with Rust doc comments:

```rust
/// Scenario: <the behavior or condition under test>
/// Guarantees: <the observable invariant protected by the test>
```

Make both statements specific enough for a reviewer to understand the test's
intent and the behavior that must not regress without reading its implementation.

## Reliable tests

A test must protect an observable contract and pass independently and alongside
other tests, regardless of machine speed or permitted scheduling order. Treat a
pass on retry as evidence of an unresolved problem, not evidence of correctness.
The recurring failures tracked in [#2720][flaky-test-report] motivate these rules;
that report identifies intermittent failures, not their root causes.

### Synchronize on state, not elapsed time

- Do not use fixed sleeps, repeated `yield_now()`, or short timeouts to assume a
  worker started, a queue drained, or a subscription became active. Use an
  explicit readiness/completion signal, an acknowledgement, a joined task, or a
  directly observable state transition. Register the waiter before triggering
  the action so the signal cannot be missed.
- Use channels or other coordination primitives appropriate to the runtime.
  Test coordination is not a reason to change the engine's thread-per-core,
  share-nothing model or introduce unjustified `Send` bounds.
- Bound waits with a deadline that allows for loaded CI machines, and report the
  expected transition and last observed state on failure. If real external state
  must be polled, poll that specific condition with a bounded deadline; do not
  retry the whole scenario or swallow unexpected errors.
- A timeout is a failure guard, not proof of progress or absence of an event.
  Prove negative cases at a controlled boundary, such as a pending future before
  a virtual deadline or a drained stream after shutdown. Do not require a real
  operation to finish within a tight wall-clock interval.

### Control the clock used by the code

- Test retries, expiry, periodic flushes, and shutdown deadlines with a controlled
  clock rather than real waiting. For Tokio timer-only tests, prefer
  `#[tokio::test(start_paused = true)]` on the current-thread runtime with
  `test-util` enabled. Use `tokio::time::Instant` consistently; pausing Tokio time
  does not pause `std::time::Instant`, `SystemTime`, or clocks in other threads,
  processes, or native libraries.
- Establish that the worker has armed its timer before advancing time. Await the
  resulting acknowledgement or state transition before asserting.
  `tokio::time::advance()` makes a time jump; it does not await the affected
  tasks, and timers crossed by the jump may be polled in any order. Prefer
  paused-time auto-advance when sequential timer progression is the scenario.
- Paused time auto-advances when the runtime has no work. Do not mix it blindly
  with sockets, filesystem watchers, brokers, or other real I/O: a virtual timeout
  can expire before the external operation completes. Separate timer logic behind
  an injected clock or fake dependency from real-I/O integration tests.
- For unavoidable real-time tests, explain why virtual time is unsuitable and
  assert eventual observable behavior within a generous bounded deadline. Keep
  throughput and latency thresholds in benchmarks, not ordinary correctness
  tests.

### Isolate fixtures and shared state

- Cargo's test harness runs tests concurrently within a process; nextest runs
  each test in its own process. Tests must work with both. Nextest isolation does
  not remove background threads within a test or conflicts over external
  resources shared by test processes.
- Use per-test temporary directories, unique broker topics or resource names,
  and loopback listeners with OS-assigned ports. Keep the bound listener rather
  than probing a free port, dropping it, and later rebinding. Await service
  readiness before sending requests.
- Prefer explicit configuration and per-test telemetry registries over mutating
  environment variables, the current directory, global subscribers, or singleton
  state. When testing environment-based configuration, pass the environment to a
  child process with `Command::env`; a test mutex does not make environment
  mutation safe against unrelated library threads on Unix.
- If a global resource genuinely cannot be removed, document and enforce its
  isolation for every accessor and supported runner. Serializing one test or
  adding `--test-threads=1` is not a general fix, and process-local locks do not
  coordinate separate nextest processes.
- Signal shutdown, then await tasks and join threads; check their results so
  background panics and failures fail the test. Ensure failure paths also clean
  up child processes, services, files, and other external resources, using
  scoped guards where appropriate.

### Measure only the intended operation

- Do not infer an operation's allocations, leaks, or cleanup from an unscoped
  process-wide heap delta, RSS, or global registry count. Harness work, other
  tests, logging, lazy initialization, and background workers can affect those
  measurements. A before/after subtraction or a one-test binary alone does not
  establish attribution.
- Prefer operation-owned counters, retained-capacity checks, ownership/drop
  evidence, or instrumentation scoped to the relevant thread/operation. A
  thread-local counter is only suitable when the measured work stays on that
  thread and unrelated tasks cannot contribute during the measurement.
- If an allocation contract requires an allocator-level test, document the
  measurement boundary and isolate all contributing work, including the
  harness. Put setup, logging, and assertions outside the measured region unless
  they are explicitly part of the contract. Warm up only initialization excluded
  by the contract; do not warm away construction costs a test claims to cover.
- Distinguish allocation events, total allocated bytes, and retained live bytes.
  An unchanged live-byte count does not prove no allocation occurred. Allocator
  observations are toolchain/profile-specific evidence, not a Rust language
  guarantee that an allocation happens; optimizers may eliminate allocations.
  Use benchmarks/profiling for measurements that cannot be reliably attributed.

### Control nondeterministic inputs without hiding bugs

- Use explicit fixtures or reproducible seeds for random input, and preserve
  failing inputs/seeds as regression cases. For property tests, retain the
  minimized counterexample. Do not require a finite random sample to distribute
  evenly as an ordinary correctness assertion.
- Compare unordered outputs as sets or multisets, or normalize them before
  comparison; preserve duplicate counts when they matter. Assert ordering only
  when it is part of the API contract. Do not sort away an ordering regression.
- Exercise concurrency with explicit handoffs at meaningful race boundaries,
  without serializing away the race being tested. Repeated stress runs supplement
  this reasoning, not replace it.
- Use local controlled services/fakes for failure scenarios, not public network
  availability, host load, or platform-specific filesystem event timing. Keep
  real-service integration coverage where needed, with explicit readiness,
  isolated resources, bounded waits, and cleanup.

### Validate reliability before submitting

Run the affected test repeatedly without retries, then with neighboring tests
under normal parallel execution. For timing, allocation, or shared-state changes,
check both Cargo's shared-process behavior and nextest's process-per-test
behavior when nextest is available. From `rust/otap-dataflow`, for example:

```bash
cargo test -p <crate> <test-filter> -- --nocapture
cargo test -p <crate>
cargo nextest run -p <crate> <test-filter> --retries 0
```

Confirm the filter actually selects the intended tests. Repeat the targeted
commands and exercise the relevant CI platform/features when the failure is
platform- or feature-dependent; record exact commands and results in the PR.
A successful repeat run is supporting evidence, not a proof of determinism.
These checks supplement, rather than replace, `cargo xtask check`.

Do not "fix" flakiness by adding retries, arbitrary sleeps, blanket serialization,
or weaker assertions. A larger timeout is justified only when the test already
waits for the right condition and evidence shows the failure guard is too tight.
If temporary quarantine is necessary, link a tracking issue and explain the
missing coverage and restoration condition.

### References

The Rust and Tokio rules above follow these primary sources:

- [Rust Book: parallel tests and shared state][rust-test-runner]
- [Tokio: testing with paused time][tokio-testing],
  [pause and auto-advance][tokio-pause],
  [advance caveats][tokio-advance], and
  [yield_now non-guarantees][tokio-yield]
- [Rust: environment mutation safety][rust-env]
- [Rust: global allocation observations and optimization][rust-alloc]
- [Nextest: process-per-test isolation][nextest-isolation] and
  [retries and flaky tests][nextest-retries]

[flaky-test-report]: https://github.com/open-telemetry/otel-arrow/issues/2720
[rust-test-runner]: https://doc.rust-lang.org/book/ch11-02-running-tests.html
[tokio-testing]: https://tokio.rs/tokio/topics/testing
[tokio-pause]: https://docs.rs/tokio/latest/tokio/time/fn.pause.html
[tokio-advance]: https://docs.rs/tokio/latest/tokio/time/fn.advance.html
[tokio-yield]: https://docs.rs/tokio/latest/tokio/task/fn.yield_now.html
[rust-env]: https://doc.rust-lang.org/std/env/fn.set_var.html
[rust-alloc]: https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html
[nextest-isolation]: https://nexte.st/docs/design/why-process-per-test/
[nextest-retries]: https://nexte.st/docs/features/retries/

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

All internal logging MUST use the `otel_*` macros from `otel_arrow_dfe_telemetry`
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
