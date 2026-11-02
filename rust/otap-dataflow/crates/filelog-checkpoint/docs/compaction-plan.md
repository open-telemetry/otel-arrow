# Task 13: checkpoint compaction and retention

Base: upstream/main `42b5b1ca6`. Epic: [#2844][epic]. Task 12 remains
unmerged in [#4566][identity]; this branch contains no Task 12 implementation.

## Existing behavior and scope

The Linux store already owns the namespace lock, validates replay, bounds WAL
appends, repairs proven torn tails, cleans recognized temporaries, and poisons
itself after any failed sync. It returns `CompactionRequired` at append limits.
Task 13 adds complete-file replacement using the existing codec, replay, and
filesystem boundary. The checkpoint format and Linux support boundary stay
unchanged. This standalone component has no user-facing receiver behavior;
use a `chore` PR title and no release-note entry.

## APIs and ownership

- `compact()` publishes current applied state with an empty WAL.
- `compact_with_retention(ids)` publishes one complete vetted removal set;
  empty retention-only work returns without publication.
- `recover_compaction()` resolves an interrupted attempt under the same lock,
  returning whether its replacement became authoritative. It never resumes
  after a failed synchronization.

All mutation needs exclusive `&mut CheckpointStore`. The dedicated checkpoint
worker serializes calls and retains unresolved batches. On
`CompactionRequired`, compact first, then retry the still-authorized append;
compaction itself authorizes no source progress.

## Publication and failure transitions

Validate generation increment, removals, and snapshot bounds before writing.
Create the fixed temporary exclusively, write and validate the complete
replacement, then sync it. Check and sync the old handle before closing it so
its deferred-write error history cannot be bypassed. Replace atomically and
sync the namespace directory. Reopen and validate before installing live state;
reset WAL accounting and sequence only with that installation.

Retain the old live table and attempt guard on failure. Before replacement,
old authority remains intact. Once replacement is attempted, assume neither
success nor rollback: block state use and mutations until guarded recovery
selects the complete old or proposed new state, completes the publication
barrier, and cleans recognized temporaries. A failed sync poisons the instance;
the supervisor must preserve that terminal condition across teardown.

## Retention and integration

Reject duplicate, unknown, or quarantined removal IDs as a whole before I/O.
No timestamp or age calculation grants eligibility here. Task 24 supplies
fresh continuous-absence evidence and checks discovery, logical readers,
descriptors, leases, candidates, open and retained batches, carry-over, and
rotation state. It keeps that evidence stable through publication and updates
runtime tracking only after durable success or resolved publication.

Task 12's current required-write path stops at `CompactionRequired`; its
IdentityOwner must expose serialized access to this protocol, retain
leases and pending admissions during uncertainty, invalidate association scans
when checkpoint state changes, and release tracking only after publication.
If #4566 merges during development, rebase and add that adapter; otherwise
report it as an outstanding integration dependency. Tasks 14, 16, and 18 stay
with Siddhartha. Task 24 owns maintenance deadlines and runtime veto wiring.

## Bounds and validation

Use configured record/snapshot admission, one bounded record scratch buffer,
checked generation/file arithmetic, and the existing streaming recovery.
Account explicitly for the old table, staged snapshot/table, bounded removal
IDs, and recovery table. No WAL-sized allocation or new storage framework.
Compaction is synchronous on the dedicated worker; kernel I/O can block.

Add deterministic Scenario/Guarantees tests for preserved identity/framing and
applied progress, append/restart, thresholds and overflow, removals larger than
one WAL transaction, invalid removal sets, interrupted temporary writes and
cleanup, both ambiguous replacement outcomes, reopen failures, namespace
ownership, and file/directory sync poisoning. Faults use the existing `Io`
seam, extended only for publication and cleanup boundaries.

Run Rust checks on testvm with native Linux storage:

```sh
cargo check -p otel-arrow-dfe-filelog-checkpoint
cargo test -p otel-arrow-dfe-filelog-checkpoint
cargo clippy -p otel-arrow-dfe-filelog-checkpoint --all-targets -- -D warnings
cargo fmt --all -- --check
cargo xtask check
```

Repeat focused tests without retries and alongside neighboring tests. Run
`npx markdownlint-cli2` on changed Markdown and `python3 tools/sanitycheck.py`
from the repository root. Separate focused results from full-workspace
results; reproduce failures on the same upstream base and testvm before
calling them pre-existing. No macOS/Docker substitution for testvm.

[epic]: https://github.com/open-telemetry/otel-arrow/issues/2844
[identity]: https://github.com/open-telemetry/otel-arrow/pull/4566
