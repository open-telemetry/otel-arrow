---
applyTo: "rust/**/*.rs"
---

# Rust review rules (OTAP Dataflow)

These restate the highest-signal rules from the project's review guide,
[AI-Assisted Pull Request Review][review-guide]. Consult that guide for the
full review criteria; it remains the source of truth.

## 1. Prefer `!Send` futures

The engine is share-nothing and thread-per-core, so work stays local to a core
by default. Prefer `!Send` futures and `?Send`-friendly traits.

Flag a newly introduced `Send` bound, or a change that forces one on an
existing trait or future, unless the pull request explains why it is required.

## 2. Justify shared-state and cross-thread primitives

`Arc<Mutex<_>>`, `RwLock`, atomics, and `tokio::spawn` are not banned, but each
use must be justified in a code comment or the pull request description.
Prefer pipeline-local `spawn_local` / `LocalSet` work instead.

Flag any new use that arrives without a justification, and say what is missing
rather than simply objecting to the primitive. Hidden cross-core
synchronization breaks the runtime model even when the code compiles and the
tests pass.

## 3. Document every test

Every test carries doc comments immediately above its declaration:

```rust
/// Scenario: <the behavior or condition under test>
/// Guarantees: <the observable invariant protected by the test>
```

Both statements must be specific enough for a reviewer to understand the test's
intent, and what must not regress, without reading its implementation. Flag new
tests that omit them or that restate the test name.

## Out of scope for review comments

Do not comment on non-ASCII characters in Rust source, or on missing changelog
entries. Continuous integration already enforces both and fails the build, so
review comments about them add noise without adding signal.

[review-guide]: ../../rust/otap-dataflow/docs/ai/ai-assisted-pr-review.md
