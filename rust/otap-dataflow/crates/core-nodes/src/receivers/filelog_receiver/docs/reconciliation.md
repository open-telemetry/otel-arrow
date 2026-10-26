# Discovery reconciliation

`discovery::reconcile::Reconciler` turns scanner observations into bounded
in-memory state and candidate offers. It does not read log contents, own file
handles, write checkpoints or decide finalization.

## Where it fits

```text
Scan files -> reconcile observations -> resolve identity -> schedule reading
           -> frame records -> build batches -> deliver -> checkpoint progress
```

Scanning and reconciliation run on each discovery pass, not for every line or
batch. Already-admitted readers have separate read turns and EOF reprobes.

The planned Phase 1 runtime has three parts:

- A discovery OS thread runs scanning and reconciliation.
- A read/checkpoint OS thread owns identity, reading, framing and checkpoint I/O.
- An async task on the pipeline core sends batches and handles completion.

Bounded channels connect them; there is no thread per file. Worker creation,
timers and receiver registration are separate integration work. See the
[receiver architecture][architecture].

## Terms used in this guide

| Term | Meaning |
| --- | --- |
| Locator | The filesystem identifier used to match observations to files. A matching locator alone does not prove that a file is safe to resume. |
| Tracked ID | A record's runtime ID. It must remain unique for the reconciler's lifetime, even after the record is removed. |
| Binding | The main path remembered for a tracked file. |
| Alias | Another path observed for the same locator. |
| Rebound | A binding now points to a different locator. The identity owner must resolve this replacement evidence. |
| Offer | A candidate selected for the identity owner to check. An offer does not authorize a new identity or transfer reading progress. |
| Publication | Committing a new identity or changing a record's binding after the required checks. |
| Superseded | A record whose Live ownership has been replaced. It may still reconnect in its existing role. |

### Example: a rotated log

Suppose `app.log` is renamed to `app.log.1`, and a new file is created at
`app.log`.

1. The scanner observes the old file at `app.log.1` and the new file at
   `app.log`.
2. The reconciler matches the old locator to its tracked record and records
   `app.log.1` as an alias. Its remembered binding, `app.log`, now points to a
   different locator, so the reconciler reports a rebound once that target is
   qualified.
3. The identity owner checks the offered files and resolves the rebound. The
   new file never inherits the old file's reading progress.
4. Publishing a different binding for the old record requires a complete pass
   and resolved rebound feedback. Reconnecting the old file can happen earlier
   when fresh evidence permits it.

The sections below define the checks behind those steps, including what happens
when a scan is incomplete or a directory changes.

## Drive one pass

Use one owner and never overlap passes. Each pass has three stages.

### 1. Collect observations

1. Register `TrackedFile` views. Give each record a unique `TrackedId`. Do not
   derive it from a path or locator, or reuse it after removal.
2. Call `begin` with an increasing generation, monotonic time and a selection
   seed. The reconciler mixes the generation into the seed.
3. Feed every scanner observation to `observe` in order. Process the resulting
   evidence synchronously before sending the next observation.

### 2. Check remembered paths and finish traversal

After traversal, create one `Scanner::binding_batch`. Do this even when no
binding checks are queued: refreshing the roots can reveal a failure that
applies to the whole receiver.

1. If root refresh fails, call `apply_binding_batch_error` or abort the pass.
   Never discard the error because the queue is empty.
2. For a successful batch, get the next ID with `next_binding_check`, then obtain
   its path and expected parent with `binding_request`.
3. Call `batch.check` and pass the result to `apply_binding_check`. Results borrow
   scanner storage, so no path copy is needed.
4. Repeat until the worklist is exhausted. If `check` rejects a request, call
   `skip_binding_checks` or abort. Skipping advances the cursor and makes the
   pass incomplete.
5. Call `finish` with the exact scanner completion marker. Targeted checks do
   not count as traversal observations. Protocol errors invalidate completeness.
   Use `abort` to end a failed consumer pass.

The worklist holds each ID once per pass. Each ID needs at most an earlier-alias
check and a binding check. Slow consumers apply bounded backpressure.

### 3. Process candidates and schedule the next pass

1. Process up to `limits.offers` candidates. Prefer `admit`: it checks the
   transition before calling the identity owner's durable callback. The callback
   returns the actual admission time, or defers without mutation.
2. Resolve current-generation `rebounds` before publishing bindings with
   `rebind`. Returning false from its callback defers publication.
3. Schedule the next pass from completion plus `reconcile_delay`. Configuration
   validates interval/jitter bounds and caps jitter at 25%. The helper itself
   can return zero at 100% jitter, so callers requiring a positive delay must
   exclude that setting.

Callbacks must not report deferral after an ambiguous durable write. If a
callback panics, do not resume the owner. These APIs do not implement checkpoint
recovery.

Transport adapters must preserve ordering. Before deciding permanent EOF, the
lifecycle owner must account for observations and failures already delivered to
it. An empty channel alone is insufficient.

## Interpret evidence

| Evidence | Meaning |
| --- | --- |
| `Present` | Prevent absence conclusions for this locator. It does not reconnect a file or cancel finalization. |
| `Validated` | Fresh eligible evidence for identity and handle validation. Locator equality alone is not continuity. |
| `CoverageLost`, `RootChanged`, `ParentChanged` | Process the failure immediately, even if the pass stalls and never finishes. |
| `Excluded` | The reader can be revoked under the exclusion rules below. This is not disappearance and changes no offsets or durable records. |
| `Disappeared` | A complete pass found no presence. This starts pending-disappearance handling; it does not establish permanent EOF. |

### What a name observation proves

Full observations support identity and publication checks. `PresenceOnly` and
`Content` observations provide weaker evidence: the name passed policy checks
and refers to the locator. They prevent exclusion, but cannot:

- produce an offer or a `Validated` event
- authorize reconnection, promotion or publication
- cancel disappearance handling

A content-access failure affects only that record. Other failures may also
carry a locator, but that does not prove the name passed policy checks. Preserve
the uncertainty without treating the result as either strength of name evidence.

Evidence may repeat within and across passes; consume it idempotently. The
lifecycle owner retains responsibility for retention, runtime vetoes, terminal
framing and finalization.

Instability, contradictory bindings and selected-path conflicts report failure
before the observation call returns. They invalidate dependent candidate and
binding work, including locator-less failures and retained rebound associations.
Policy/verification failures likewise invalidate dependent evidence. Unrelated
stable work can remain usable during an incomplete pass.

### When a reader can be excluded

A currently validated eligible binding or alias keeps the reader eligible.
Excluding one alias cannot revoke the reader while another eligible name
remains. Policy-qualified names also stop targeted checks whose only purpose
is exclusion. Batch roots must still be refreshed.

An alias remembered from an earlier pass is only a hint. Recheck it before
relying on it. A definite miss or a different locator expires the hint.
Busy or inaccessible checks retain it. If an alias check loses coverage,
exclusion is blocked for the rest of the pass.

The evidence needed to exclude a reader depends on what the check found:

| Result | Exclusion rule |
| --- | --- |
| Positive exclusion or replacement evidence | Can authorize prompt exclusion. |
| Missing name | Requires an available, unchanged parent and valid environment context. |
| Local lookup uncertainty | May use complete-pass fallback. |
| Resource or cancellation failure carrying a retained locator | Blocks that record's exclusion for the pass, without authorizing admission. This includes descriptor exhaustion. |

### When roots or parent directories change

Root failures, exclusion-root failures and retained parent-change latches block
missing-path conclusions across the receiver. This includes misses recorded
before the failure. A successful retry cannot clear the failure for that pass.
An unrelated incomplete traversal, however, does not invalidate a miss obtained
with valid context.

A changed parent also blocks exclusion and binding publication for its affected
records. Traversal and targeted checks apply the change to all retained siblings,
including inactive records, while preserving their expected parent identities.

Recovery requires the expected identities to return or the receiver to be
recreated. When an expected parent returns, its persistent latch clears, but
evidence collected under the replacement directory in that pass remains invalid.
Fresh evidence is required. Recreating the receiver does not reset checkpoints.

Targeted checks use the same ancestor policy, root verification, pin-bound
probes and descriptor budget as traversal. Handles close before evidence is
returned. A missing parent with valid roots remains local uncertainty. It does
not prove that the binding is ineligible. Traversal depth limits resource use
and must not be treated as an exclusion rule.

## Reconnection and binding publication

Reconnection resumes a known identity. Publication commits a new identity or a
binding change. These operations have different requirements, particularly when
a parent directory has changed or a pass is incomplete.

### Requirements by operation

| Operation | Rule |
| --- | --- |
| Same-ID reconnection | Keep the locator and binding unchanged. Parent blocks alone do not suppress a known-locator offer, including at its retained path or a path shared with a historical record. Actual evidence conflicts still withhold it. |
| Unverified recovery | `admit` may promote to Live with fresh eligible binding evidence and no binding block. |
| Superseded record | Reconnect in the same role. Only `rebind` restores Live, after a complete pass and all rebound feedback; it also clears the successor memo. |
| New identity | Reject publication at any active blocked binding, regardless of role. An uncontested new identity may supersede the current Live owner during an incomplete pass, with fresh matching rebound evidence. |
| New/reselected binding | Preserve a Live binding while a policy-qualified name still validates it; defer publication if it lacks full evidence. Otherwise choose a fully validated binding or minimum alias. Require a complete pass and resolved rebounds. |

A complete pass requires all persistent parent latches and per-pass binding
blocks to be clear. Unchanged-role reconnection and independent positive reading
remain available during these blocks. Admission still validates ownership:
locator reuse cannot bypass the new-identity guard.

`rebind` rejects invalid or empty paths before its callback. Repeating it in the
same generation returns the published path.

### Replacement history and rebound feedback

`Replacement::Possible` records replacement history for the start policy. It
does not authorize supersession. Keep it when work is deferred or a predecessor
retires. Combine hints using the smallest runtime ID, including owner
associations.

The candidate's own durable state takes precedence. Never transfer progress from
its predecessor. Admission uses the selected path's current Live owner to decide
which record it may supersede.

Rebound feedback repeats each generation, even if the replacement is resident
and produces no offer. For example, admission at a smaller alias can leave
another path's predecessor Live. Call `resolve_rebound` each pass; `supersede`
cannot replace it once the replacement already has a Live owner. This can
continue until complete-pass reselection or owner retirement. Disappearance
alone does not retire a record.

### Deferred offers and saved paths

For a current, unacknowledged token, blocked new-identity admission may return
`Feedback` before its callback. The token remains usable for a `Deferred`
acknowledgement. Stale/already-acknowledged tokens are protocol errors, not
retryable offers. Deferred work may consume an offer slot again in later passes.

`TrackedFile::binding` must be a complete native path or absent. Checkpoint-owned
`AdvisoryPath` may be truncated; its suffix cannot be opened or used for native
path ordering. `associate_replacement` accepts only narrowly justified owner
context, never a blanket association for unavailable saved paths.

## Candidate selection

- Offer retained candidates oldest-first, with arrival order breaking ties.
  Preserve age across observations.
  Incomplete passes retain unseen entries but never offer them as fresh.
  Complete passes may discard unseen entries.
- Overflow uses at most `limits.offers` sample slots with a generation-varying
  rank. Retained and overflow groups share offers and backfill spare slots; a
  one-slot budget alternates groups. There is no finite starvation guarantee.
- Full tracked capacity still allows identity matching/reconnection. Park only
  candidates proven to need a new slot. Rejected candidates can return on a
  later fresh observation.

## Implementation reference

These details describe how the reconciler preserves the rules above while
bounding its storage and lookup work.

### Parent state shared by siblings

All retained bindings in a directory share its expected parent and persistent
change state. New bindings copy one member's state, even when the parent is
unknown. Admission does not scan the sibling group.

The first validation establishes the baseline for the whole family when it is
unknown. Healthy targeted checks and repeated changes already propagated this
pass avoid another family walk.

### Weak observations at replaced bindings

A weak observation can show a different locator at a retained binding without
qualifying that target for replacement handling. Remember such observations and
their owner associations for contradiction checks, regardless of arrival order.
They authorize neither actionable rebound feedback nor supersession.

Wait for the target to be qualified before reselecting that binding. This keeps
the replacement context at the old path. Unrelated binding proposals remain
available.

### Alias buffers and indexes

- The minimum-alias buffer holds one alias and its evidence strength. Full aliases
  replace policy-only hints, even if they sort later. Equal-path upgrades change
  only strength; weaker observations never downgrade full evidence. An earlier
  hint carries only a name into the next pass, not current authority. Publication
  cannot select a policy-only minimum.
- Conflict detection indexes selected candidate paths and retained current-pass
  minimum aliases, including policy-only aliases and resident records. It does
  not retain every alias ever seen. Indexes reset at `begin`. Previous-pass hints
  are not fresh conflict evidence. A detected conflict stays latched even if its
  sample is evicted.
- Binding targets share one locator slot with observed, pending and resolved
  states. Only qualified targets enter rebound feedback. Aggregate name strength
  is derived from binding and alias evidence, rather than stored again.
- Packed records and indexes store IDs/slots, not duplicate path keys. Remove
  entries before changing paths or slots; compare full native keys after hashes.
  Shared historical keys are allowed. Alias buffers are reused and capped at L.

Alias publication removes the index entry before clearing its buffer, so later
checks use binding-parent context.

### Tokens, statistics and graph work

Successor links use runtime ID plus locator, so inode reuse cannot resolve old
rebound work. Live ownership survives reader closure. A valid one-way hardlink
is not a cycle: incremental connectivity filters potential cycles, a directed
walk confirms them, and completion sweeps dependent binding work. Failures
arrive immediately. Incremental checks stop once the pass is incomplete.

Tokens are validated directly by generation, slot, selection and acknowledgement
state. A cursor skips acknowledged prefixes once. Statistics report pending
count, oldest age and actual admission time; overflow counts sightings, not
unique files. Graph walks and successor updates can visit all tracked records,
so rotation/admission latency still needs integrated qualification.

### Memory accounting

Explicit limits bound records, candidates, offers, paths and fingerprints.
Construction checks arithmetic and a standalone 1 GiB guard. `retained_bytes`
includes spare capacity; some table accounting depends on the pinned toolchain
and must be rechecked on upgrades. It is not process RSS. The accepted integrated
budget is unchanged; resolve [Task 36's memory proposal][memory] before wiring
this representation into that budget.

## Validation and integration limits

Deterministic tests cover ordering, conflicts, recovery, role transitions,
parent changes, rotation, alias policy, fairness and capacity bounds. Allocator
tests cover warm overflow reconciliation and the complete binding-check loop.
Warm allocation tests include weak/full alias rekeying, equal-path upgrades and
cross-pass hint reuse. Admission/rebind may allocate bounded paths: there is no
blanket zero-allocation claim. See the implementation's [binding
rules][bindings] and [storage][storage]
for detailed invariants.

This is not a registered receiver. Durable recovery, worker/channel wiring,
retention, finalization and whole-receiver performance remain integration work.
The lifecycle contract is #4443 at
`d9781ee3f122647b050840f93869ea8bb068aa31`; it does not approve the separate memory
allowance. Integrated accounting must include scanner, checkpoint, reader and
transport storage without double counting.

[architecture]: ../../../../../../docs/filelog-receiver.md#phase-1-architecture
[memory]: reconciliation-memory-proposal.md
[bindings]: ../discovery/reconcile/bindings.rs
[storage]: ../discovery/reconcile/storage.rs
