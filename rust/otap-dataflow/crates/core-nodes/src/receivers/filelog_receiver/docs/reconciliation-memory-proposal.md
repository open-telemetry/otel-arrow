# Proposed compact discovery memory allowance

Status: **for maintainer review after measured storage compaction**. This is not
an accepted conformance change. PR #4443 does not approve this separate budget.
The accepted conformance formula and configuration remain unchanged.

## Layout and bound

The implementation now uses:

- A packed record vector and a small ID-to-slot table. Removal moves the last
  record and updates its ID entry; indexes never retain a slot as identity.
- ID-only locator, binding and parent indexes. Native keys are compared against
  record-owned paths, including full equality after hash matches. Historical
  records can share keys. Rebind removes old entries before changing a path.
- One exact-length binding and two reusable alias buffers. Alias buffers grow
  lazily, but their capacity never exceeds the native-path limit L. No full-path
  copies or per-key ID vectors are owned by the indexes.

The conservative standalone charge is:

```text
tracked_state = max_tracked_files * (3 * L + 512)
candidate_state = (pending + offers) * (2 * (L + fingerprint_bytes) + 512)
fixed_and_temporary = 4096 + L + fingerprint_bytes
standalone_total = tracked_state + candidate_state + fixed_and_temporary
```

The constructor checks arithmetic and rejects totals exceeding 1 GiB. L is an
internal scanner limit, not a currently exposed operator configuration field.
This change adds no shared byte-pool exhaustion or new runtime failure mode.

The 512-byte metadata term is a structural upper bound, not an average derived
from one fixture. It covers the record, three ID worklists and four ID/slot hash
tables reserved at twice their maximum live population. At the pinned hashbrown
load factor, power-of-two rounding requires fewer than 32N/7 buckets per table.
The record/worklist sizes plus rounded table payloads fit within 512 bytes per
record; a regression checks this when types change. Fixed owner/table padding
is charged separately. One replacement can transiently retain an old path or
fingerprint buffer while allocating its replacement, hence the global L + F.

Tracked tables report their allocated storage through hashbrown's public API.
Candidate-index accounting still uses pinned standard-library assumptions.
Revalidate bounds and heap measurements on toolchain/dependency upgrades. The
charge covers owned component storage, not allocator-exact RSS or the receiver.

## Measurements

All measurements ran on testvm, Linux x86_64, Rust 1.98.1. Values below are bytes.
The same long-path and short-path fixtures were used before and after compaction.

| Fixture (10,000 records) | Previous live heap | Compact live heap | Compact estimate | Compact charge |
| --- | ---: | ---: | ---: | ---: |
| Near-limit paths and alias buffers | 293,623,320 | 126,370,264 | 126,371,144 | 128,025,680 |
| Short-path metadata replacement churn | 10,620,392 | 3,500,264 | 3,501,144 | 5,215,215 |
| Populated short binding/parent indexes | Not previously measured | 3,620,264 | 3,621,144 | 6,085,360 |

Long-path live heap fell by about 57%; short-path metadata by about 67%.
The long-path peak was 126,374,358 bytes, within the charge including temporary
replacement storage. Dropping each owner releases all measured live allocations.
Capacity-boundary regressions also populate and churn all indexes without growth.
Warm reconciliation and the complete binding-check loop remain allocation-free.

The isolated bookkeeping benchmark alternates old/new pass order, uses three
warm-up pairs and 100 measured pairs, and repeats the experiment three times.
Median pass times across those runs were:

| Workload | Previous, microseconds | Compact, microseconds | Interpretation |
| --- | ---: | ---: | --- |
| 10,000 resident short-path observations | 3508.122 | 2989.086 | About 15% lower |
| 30,000 resident/alias/parent observations | 5784.922 | 5768.656 | Effectively unchanged |
| 10,000 observations with pending overflow | 1095.768 | 1070.586 | About 2% lower in this fixture |

These timings exclude filesystem scanning, readers and downstream processing;
they establish neither end-to-end receiver performance nor superiority to Fluent
Bit. Per-run results and the preserved before/after harness accompany validation.

## Decision for later integration

At L=4096 and 10,000 tracked files, the compact tracked-state allowance is
128,000,000 bytes, versus the accepted discovery term's 10,240,000 bytes. The
previous implementation proposed 348,164,096 bytes including its fixed overhead.
Compaction materially reduces the increase, but does not eliminate the need for
an explicit decision about complete-path storage and integrated admission.

Maintainers should evaluate replacing the discovery term with the compact bound,
account for fixed/temporary storage, and reconcile candidate/scanner/checkpoint
ownership without double counting. One independent maximum-length binding alone
exceeds the accepted 1024-byte discovery allowance. Integration must not silently
use that smaller allowance for this representation, or treat this proposal as
already approved. No maintainer outreach or accepted-budget edit is performed
by this implementation.

A fixed metadata charge plus a configured shared path-byte pool is a fallback,
not the recommendation here. It could reduce typical memory use, but pool
exhaustion would need explicit admission deferral and incomplete-pass behavior
when evidence cannot be retained. That expands the lifecycle contract beyond
this storage-only change.
