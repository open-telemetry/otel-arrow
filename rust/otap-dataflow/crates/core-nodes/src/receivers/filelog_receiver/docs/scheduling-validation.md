# Scheduler validation record

Validation uses x86_64 Linux `testvm`, native ext4 source/state storage, the pinned
Rust toolchain and proto submodules. Build output uses a task-owned persistent
target directory. Tests use normal parallel execution and no test retries.

## Review follow-up

The follow-up to `9d31eaf5f` merges upstream Task 12 (`65541beab`) and addresses:

- descriptor selection surviving a turn exhausted by eviction;
- two-way probe handoff and complete discovery passes under contention;
- waiter-specific wakeup and immediate request cleanup on Stop/cancel/failure;
- empty slot release on descriptor/probe/open-backoff waits;
- lazy remainder/history allocation and bounded history growth;
- explicit pending-replay budget coverage and current discovery documentation;
- committed ReaderSeed, lease, continuation and applied-evidence integration.

## Commands

From `rust/otap-dataflow` on `testvm`:

```sh
cargo check --locked -p otel-arrow-dfe-core-nodes
cargo test --locked -p otel-arrow-dfe-core-nodes receivers::filelog_receiver --lib
cargo test --locked -p otel-arrow-dfe-core-nodes
cargo xtask check
```

From the repository root:

```sh
python3 tools/sanitycheck.py
npx markdownlint-cli2 \
  rust/otap-dataflow/crates/core-nodes/src/receivers/filelog_receiver/docs/discovery.md \
  rust/otap-dataflow/crates/core-nodes/src/receivers/filelog_receiver/docs/scheduling.md \
  rust/otap-dataflow/crates/core-nodes/src/receivers/filelog_receiver/docs/scheduling-validation.md
```

## Results

Final follow-up validation on `testvm`:

- Affected-crate compile check: passed.
- Filelog tests: 504 passed, 5 existing ignored; all 42 scheduler tests passed.
- Complete affected-crate tests, including integration/allocation and doc tests:
  passed under normal parallel execution.
- Markdown lint, repository sanity and diff checks: passed.
- Full workspace: structure, component inventory, formatting, all-target Clippy
  and no-std builds passed. The test stage failed in the unchanged HTTP exporter
  test `test_tls_mtls_failure_wrong_client_cert` at `mod.rs:4152` (expected-failure
  boolean assertion). Full-workspace validation is not green.

An earlier broad run also failed in unchanged checkpoint tests
`interrupted_creation_write` (`Locked`) and `cleanup_sync_failure_is_terminal`.
The original standalone head had unrelated exporter port-binding/empty-result
failures as well. These observations are retained; successful subsequent runs
are not evidence that those unrelated reliability problems have been fixed.
No retries, test serialization or unrelated source fixes were added to hide them.
