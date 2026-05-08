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

Prefer localized data structures, CPU locality, buffer reuse, preallocation,
event-driven non-blocking design, and predictable latency under load.

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

Complex abstractions are acceptable only when they make behavior more reliable,
composable, or maintainable.

## Correctness, Security, and Portability

Review Rust correctness and safety through the lens of the OTAP runtime model:

- clear ownership, lifetimes, and cancellation behavior
- idiomatic Rust without sacrificing hot-path performance
- no implicit behavior that hides failure, retries, or data loss
- justified and reviewed `unsafe` blocks

Security and privacy must be first-class review concerns:

- no sensitive telemetry leakage
- no validation gaps for untrusted input
- no denial-of-service or resource-exhaustion vector
- no accidental exposure of secrets, credentials, or private data

Check platform assumptions. New code should remain compatible with Linux and
Windows, with reasonable macOS support, and with x86-64 and ARM unless a
specific limitation is documented and justified.

## Agent Reviewer Instructions

An agent reviewer should:

- inspect the complete PR diff before forming conclusions
- evaluate only risks supported by the diff or repository context
- focus findings on architecture, performance, bounded resources, backpressure,
  correctness, security, and maintainability
- avoid speculative or stylistic comments unless they affect system integrity
- include concise file-specific findings, ordered by severity
- state when no material findings are found and mention residual risk or tests
  not inspected

Do not approve a design solely because tests pass. Tests are evidence, not a
substitute for preserving OTAP architectural invariants.
