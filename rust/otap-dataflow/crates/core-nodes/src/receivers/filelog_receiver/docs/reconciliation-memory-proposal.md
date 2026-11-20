# Temporary proposal: discovery memory budget

**This is a temporary decision proposal for [Epic #2844, Task 36][tracker]:
agreeing the discovery memory allowance before receiver integration.** It is
not a running work log or an accepted design. The conformance budget and
configuration are unchanged.

## Why a change is needed

Discovery keeps a file's main path and up to two alternate names so it can
recheck them safely. These paths can exceed the current allowance of 1,024 bytes
per tracked file. Duplicate path copies in lookup indexes have already been
removed.

For 10,000 tracked files and a 4,096-byte path limit:

- The accepted tracked-state allowance is **10.24 MB**.
- The proposed tracked-state allowance is **128 MB**.
- A test filling the path buffers measured **126.75 MB** of component heap.

These are decimal MB. The measurement is a long-path fixture, not typical usage
or total receiver memory. It was measured on Linux x86_64 with Rust 1.98.1 using
the memory test linked below.

## Proposed allowance

Let `L` be the path-byte limit and `F` the fingerprint-byte limit:

```text
tracked_state = max_tracked_files * (3 * L + 512)
candidate_state = (pending + offers) * (2 * (L + F) + 512)
fixed_and_temporary = 4096 + L + F
standalone_total = tracked_state + candidate_state + fixed_and_temporary
```

`pending` and `offers` are the candidate and per-pass offer limits. The formula
covers retained paths, bookkeeping and temporary buffer replacement. It is an
upper allowance, not memory allocated immediately. `L` is currently an internal
limit, not an operator setting. Construction rejects totals above 1 GiB.

The [memory test] checks measured heap against the estimate and allowance.
Recheck these bounds when the layout, toolchain or dependencies change.

## Decision needed before integration

Task 36 needs to settle three things before integration:

1. Agree the discovery memory allowance.
2. Account for scanner, identity, checkpoint and reader memory without gaps or
   double counting.
3. Update the conformance budget and configuration admission together.

The proposed allowance covers the standalone discovery component. A
whole-receiver memory limit still needs that integrated accounting and approval.

Once Task 36 is resolved, record the accepted formula in the conformance and
resource-budget documentation, and update configuration admission. Remove this
proposal or replace it with a short link to that decision. Git history and
validation reports retain the supporting evidence.

See [reconciliation.md] for implementation and ownership details.

[tracker]: https://github.com/open-telemetry/otel-arrow/issues/2844#issuecomment-5610254481
[memory test]: ../../../../tests/filelog_reconcile_memory.rs
[reconciliation.md]: reconciliation.md
