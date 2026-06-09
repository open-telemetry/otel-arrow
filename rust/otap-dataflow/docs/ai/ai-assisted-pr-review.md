# AI-Assisted Pull Request Review

This document describes what matters when reviewing OTAP Dataflow Engine pull
requests with AI assistance. It is intended for human reviewers and agent
reviewers.

The intended reader is an engineer already familiar with the OTAP Dataflow
Engine, Rust, and OpenTelemetry. AI can accelerate diff inspection and risk
analysis, but the reviewer remains responsible for correctness, maintainability,
security, and architectural judgment.

Always inspect the full diff before evaluating a pull request. For GitHub PRs,
use `gh pr diff <number>` or equivalent repository tooling; do not rely only on
summaries, titles, or changed-file lists.

This guideline is expected to evolve. Update it when recurring PR comments,
issues, incidents, or maintainer feedback reveal review gaps that should become
shared guidance.

## Agent Reviewer Quick Checklist

If an agent only has time for one focused pass, it should:

- inspect the full diff and relevant nearby code before forming conclusions
- run targeted searches for runtime-risk patterns when Rust async or runtime
  code changes, such as `tokio::spawn`, `spawn_blocking`, `Arc<Mutex`,
  `RwLock`, `unbounded_channel`, `block_on`, `std::fs`,
  `std::thread::sleep`, `unwrap(`, `expect(`, and `unreachable!`
- use `rust/otap-dataflow/scripts/check-async-blocking.sh` from the repository
  root as a review aid when async/runtime paths are touched
- report only risks supported by the diff, nearby code, or project guidance
- state residual risk when no material findings are found

## Review Posture

Prioritize architectural integrity, performance characteristics, resource
safety, and long-term maintainability. Avoid stylistic nitpicks unless they
affect correctness, readability, or maintainability.

For every important finding:

- reference the file and specific change
- explain the architectural, performance, correctness, or security impact
- distinguish required fixes from improvement opportunities
- propose a concrete alternative when rejecting an approach

## Core Architectural Invariants

Reviewers must protect the OTAP runtime model.

Check that changes preserve the thread-per-core, share-nothing design:

- no shared mutable state across cores unless explicitly justified
- no hidden cross-core synchronization through `Arc<Mutex<_>>`, `RwLock`,
  atomics, shared async runtimes, or cross-thread wakeups
- no coordination pattern that centralizes work or breaks locality
- prefer `!Send` futures unless `Send` is strictly required
- prefer pipeline-local `spawn_local` / `LocalSet` work; require justification
  for `tokio::spawn`, `Send` bounds, `Arc`, atomics, or cross-thread task
  movement

Check that single-threaded async runtime responsiveness is preserved. Each
pipeline instance runs on a single-threaded async runtime, so blocking or
monopolizing that thread can stall data processing, backpressure, shutdown,
live reconfiguration, telemetry, and ack/nack progress.

Flag runtime-path work that can block or monopolize the core:

- synchronous filesystem, I/O, networking, sleep, process execution, DNS,
  compression, serialization, crypto, or native library calls
- blocking waits on channels, mutexes, condition variables, or external
  processes
- long CPU sections over records, batches, encoders, decoders, aggregations,
  sorting, or flushing without bounded work units or cooperative await points

When runtime-path code calls an external crate or native library, the PR should
record important assumptions that are not visible at the call site. Reviewers
should look for whether the library can block, start threads, use shared or
global state, allocate or buffer substantially, retry internally, or hide
backpressure. Evidence can be a library source or documentation link, a code
comment, a component development note, or a focused test or benchmark. Dependency
upgrades that affect such calls should re-check these assumptions.

`spawn_blocking` is not automatically acceptable. Blocking offload must be
bounded, cancel-aware, backpressure-integrated, observable, and justified
because it can break locality or create hidden queues.

Check that resource usage is bounded:

- no unbounded channels, buffers, queues, maps, retry state, task spawning, or
  fan-out
- explicit capacities, limits, load shedding, or backpressure for structures
  that can grow under load
- no hidden buffering that delays failure while memory grows

Check that backpressure propagates end to end:

- slow downstream systems must apply pressure upstream
- components must not silently absorb sustained overload
- overload behavior must be explicit and observable

## Runtime and Performance

Identify whether the diff affects hot paths such as ingest, per-record
processing, dispatch, encoding/decoding, or tight-loop allocation.

On hot paths, call out unnecessary:

- heap allocations, cloning, or temporary buffers
- branching, indirection, dynamic dispatch, or logging
- locks, cross-thread wakeups, task scheduling, or system calls
- materialization that could remain passthrough or zero-copy
- long non-yielding loops or oversized batches that delay other local tasks

Prefer localized data structures, CPU locality, buffer reuse, preallocation,
event-driven non-blocking design, intentional work chunking, and predictable
latency under load. Avoid excessive `yield_now()` in tight loops; cooperative
yielding should be deliberate and tied to bounded progress.

Require evidence when a change claims, risks, or depends on performance impact.
Depending on the scope, this may be a benchmark, baseline comparison, complexity
analysis, or a clear explanation of why the path is not performance-sensitive.
Flag algorithmic complexity shifts, accidental loss of preallocation or
zero-copy behavior, and changes that make future regressions harder to measure.

## Design and Serviceability

Review whether the change keeps behavior easy to reason about under normal load
and overload.

Check for:

- explicit failure modes and graceful degradation
- no central coordination bottlenecks
- preserved component and plugin boundaries
- extensibility without unnecessary core-engine changes
- useful telemetry, diagnostics, and controllable debug features
- live reconfiguration and restart-free operation, when relevant
- compatibility with OTLP, OTAP, and Collector integration expectations
- clear operator-facing behavior for configuration, defaults, validation
  errors, unsupported platforms, and documentation examples
- stable telemetry contracts, including metric names, label cardinality,
  deterministic label order, and explicit collision handling
- precise telemetry semantics, including instrument kind, aggregation cadence,
  numerator/denominator consistency, monotonicity, units, dimensions, and
  scrape or reporting timing
- intentional shutdown, drain, flush, cancellation, and pending-message behavior
  for async tasks, streams, channels, and metric updates
- reuse of existing engine, Query Engine, pdata view, decoder, validation, or
  configuration abstractions before adding component-specific logic

Complex abstractions are acceptable only when they make behavior more reliable,
composable, or maintainable.

## Correctness, Security, and Portability

Review Rust correctness and safety through the lens of the OTAP runtime model:

- clear ownership, lifetimes, and cancellation behavior
- idiomatic Rust without sacrificing hot-path performance
- no implicit behavior that hides failure, retries, or data loss
- justified and reviewed `unsafe` blocks
- justified `expect()`, `unwrap()`, and `unreachable!()` in non-test code;
  document the invariant or use explicit error handling
- no locks, mutable borrows, permits, backpressure tokens, or large buffers held
  longer than necessary across `.await`
- long-running work observes cancellation and deadlines where shutdown or live
  reconfiguration can interrupt it
- prefer correctness by construction: use types, builders, validation, and
  fail-fast APIs to make invalid states impossible or explicit instead of
  relying on convention, comments, or debug-only checks

Review semantic fidelity where data models, protocols, or query behavior are
affected:

- preserve exact OTLP, OTAP, OpenTelemetry, Prometheus, KQL, DataFusion, and
  schema semantics where applicable
- preserve required fields, invalid-payload handling, ack/nack behavior, and
  backward compatibility unless a breaking change is explicit and justified
- check empty, partial, duplicate, unsupported, and malformed inputs
- make ordering, deduplication, type parity, and collision behavior
  deterministic and tested

Review test adequacy, not only test existence. Prefer direct tests at the
affected layer, plus negative, edge-case, and regression tests when behavior
changes. Round-trip or integration tests are useful evidence, but should not be
the only proof when a narrower semantic contract can be tested directly.

For unit tests, prefer a short function-level comment that states:

- `Scenario:` the behavior or state transition being exercised
- `Guarantees:` the invariant, contract, or regression protection the test
  provides

Security and privacy must be first-class review concerns:

- no sensitive telemetry leakage
- no validation gaps for untrusted input
- no denial-of-service or resource-exhaustion vector
- no accidental exposure of secrets, credentials, or private data

Check platform assumptions. New code should remain compatible with Linux and
Windows, with reasonable macOS support, and with x86-64 and ARM unless a
specific limitation is documented and justified. Platform-specific APIs,
dependencies, clocks, time domains, filesystem assumptions, and target-gated
code must be explicit.

Check dependency and feature hygiene:

- new dependencies should follow workspace conventions where applicable
- optional functionality should not expand the default build without
  justification
- target-specific dependencies should be feature- or platform-gated
- unreleased commit pins should be temporary, explained, and preferred only when
  a release tag cannot satisfy the requirement

## Agent Reviewer Instructions

An agent reviewer should:

- inspect the complete PR diff before forming conclusions
- use `rust/otap-dataflow/scripts/check-async-blocking.sh` from the repository
  root as a review aid when async/runtime paths are touched
- evaluate only risks supported by the diff or repository context
- focus findings on architecture, performance, bounded resources, backpressure,
  correctness, semantic fidelity, test adequacy, security, and maintainability
- avoid speculative or stylistic comments unless they affect system integrity
- include concise file-specific findings, ordered by severity
- state when no material findings are found and mention residual risk or tests
  not inspected

Do not approve a design solely because tests pass. Tests are evidence, not a
substitute for preserving OTAP architectural invariants.
