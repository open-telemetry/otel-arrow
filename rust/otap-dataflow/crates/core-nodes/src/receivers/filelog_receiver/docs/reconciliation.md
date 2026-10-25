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

## Drive one pass

Use one owner and never overlap passes.

1. Register `TrackedFile` views. Each `TrackedId` is unique for this reconciler's
   lifetime, even after removal; do not derive it from a path or locator.
2. Call `begin` with an increasing generation, monotonic time and a selection
   seed. The reconciler mixes the generation into the seed.
3. Feed every scanner observation to `observe` in order. Process its evidence
   synchronously before the next observation.
4. After traversal, create one `Scanner::binding_batch`, even with no queued
   checks. Apply a root-refresh error with `apply_binding_batch_error` or abort;
   never discard it because the queue is empty. For a successful batch, repeat
   `next_binding_check` -> `binding_request` -> `batch.check` ->
   `apply_binding_check`. Results borrow scanner storage, so no path copy is
   needed. If `check` rejects a request, call `skip_binding_checks` or abort.
   Skipping advances the cursor but makes the pass incomplete.
5. Call `finish` with the exact scanner completion marker. Targeted checks do
   not count as traversal observations. Protocol errors invalidate completeness;
   use `abort` to end a failed consumer pass.
6. Process up to `limits.offers` candidates. Prefer `admit`: it checks the
   transition before the identity owner's durable callback. The callback returns
   the actual admission time, or defers without mutation.
7. Resolve current-generation `rebounds` before publishing bindings with
   `rebind`. Returning false from its callback defers publication.
8. Schedule from completion plus `reconcile_delay`. Configuration validates
   interval/jitter bounds; its jitter policy caps at 25%. The helper can return
   zero at 100% jitter, so callers requiring a positive delay must exclude that.

The binding worklist holds each ID once per pass, with at most an earlier-alias
check and a binding check. Slow consumers apply bounded backpressure.

Callbacks must not defer after an ambiguous durable write, or panic and then
resume the owner. These APIs do not implement checkpoint recovery. Transport
adapters must preserve ordering: permanent-EOF decisions must account for
already-delivered observations and failures, not just an empty channel.

## Interpret evidence

| Evidence | Meaning |
| --- | --- |
| `Present` | Prevent absence conclusions for this locator. It does not reconnect a file or cancel finalization. |
| `Validated` | Fresh eligible evidence for identity and handle validation. Locator equality alone is not continuity. |
| `CoverageLost`, `RootChanged`, `ParentChanged` | Process the failure immediately, even if the pass stalls and never finishes. |
| `Excluded` | The reader can be revoked under the exclusion rules below. This is not disappearance and changes no offsets or durable records. |
| `Disappeared` | A complete pass found no presence. This starts pending-disappearance handling; it does not establish permanent EOF. |

Evidence may repeat within and across passes; consume it idempotently. The
lifecycle owner retains responsibility for retention, runtime vetoes, terminal
framing and finalization.

Instability, contradictory bindings and selected-path conflicts report failure
before the observation call returns. They invalidate dependent candidate and
binding work, including locator-less failures and retained rebound associations.
Policy/verification failures likewise invalidate dependent evidence. Unrelated
stable work can remain usable during an incomplete pass.

### Exclusion and environment changes

- A currently validated eligible binding or alias keeps the reader eligible.
  An excluded alias alone cannot revoke it.
- An earlier alias is only a hint: recheck it before relying on it. A definite
  miss or different locator expires the hint; busy/inaccessible checks retain it.
  An alias-check coverage failure blocks exclusion for the rest of the pass.
- Positive exclusion or replacement evidence can authorize prompt exclusion.
  A missing name needs an available, unchanged parent and valid environment
  context. Unknown or unstable bindings require complete-pass fallback.
- Root/exclusion-root failures and retained parent-change latches block
  missing-path conclusions receiver-wide, including misses recorded before the
  failure. A successful retry does not clear that pass's failure. Unrelated
  incomplete traversal alone does not invalidate a contextual miss.
- A changed parent blocks the affected bindings' exclusion and publication.
  Traversal and targeted checks use one handler for all retained siblings,
  including inactive records. Expected identities are preserved. Restoration
  clears persistent latches, but cannot revive evidence collected under the
  replacement directory in that pass; recovery needs fresh evidence. Persistent
  blocks last until expected identities return or the receiver is recreated.
  Receiver recreation does not reset checkpoints.

All retained bindings in a directory share its expected parent and persistent
change state. New bindings copy one member's state, even when the parent is
unknown; admission does not scan the sibling group. First validation establishes
an unknown baseline for the whole family. Healthy targeted checks and repeated
changes already propagated this pass avoid another directory-family walk.

Targeted checks share traversal's ancestor policy, root verification, pin-bound
probes and descriptor budget. Handles close before evidence is returned. A
missing parent with valid roots remains local uncertainty, not proof of binding
ineligibility. Traversal depth is a resource bound, not an exclusion rule.

## Reconnection and binding publication

A binding is the record's distinguished path; a rebound means that path now
names another locator. Offers are opportunities for identity checks, not
permission to publish a new identity or transfer progress.

| Operation | Rule |
| --- | --- |
| Same-ID reconnection | Keep the locator and binding unchanged. Parent blocks alone do not suppress a known-locator offer, including at its retained path or a path shared with a historical record. Actual evidence conflicts still withhold it. |
| Unverified recovery | `admit` may promote to Live with fresh eligible binding evidence and no binding block. |
| Superseded record | Reconnect in the same role. Only `rebind` restores Live, after a complete pass and all rebound feedback; it also clears the successor memo. |
| New identity | Reject publication at any active blocked binding, regardless of role. An uncontested new identity may supersede the current Live owner during an incomplete pass, with fresh matching rebound evidence. |
| New/reselected binding | Keep an eligible established binding; otherwise choose the minimum complete native path. Require a complete pass and resolved rebounds. |

A complete pass requires all persistent parent latches and per-pass binding
blocks to be clear. Unchanged-role reconnection and independent positive reading
remain available during these blocks. Admission still validates ownership:
locator reuse cannot bypass the new-identity guard.

`rebind` rejects invalid/empty paths before its callback. Repeating it in the
same generation returns the published path. Alias publication removes the index
entry before clearing its buffer, so later checks use binding-parent context.

`Replacement::Possible` is historical start-policy context, not supersession
authority. Keep it across deferral or predecessor retirement; combine hints by
smallest runtime ID, including owner associations. The candidate's own durable
state wins, and predecessor progress is never transferred. Admission chooses
supersession authority from the current Live owner of the selected path.

Rebound feedback repeats each generation, even if the replacement is resident
and produces no offer. For example, admission at a smaller alias can leave
another path's predecessor Live. Call `resolve_rebound` each pass; `supersede`
cannot replace it once the replacement already has a Live owner. This can
continue until complete-pass reselection or owner retirement. Disappearance
alone does not retire a record.

For a current, unacknowledged token, blocked new-identity admission may return
`Feedback` before its callback. The token remains usable for a `Deferred`
acknowledgement. Stale/already-acknowledged tokens are protocol errors, not
retryable offers. Deferred work may consume an offer slot again in later passes.

`TrackedFile::binding` must be a complete native path or absent. Checkpoint-owned
`AdvisoryPath` may be truncated; its suffix cannot be opened or used for native
path ordering. `associate_replacement` accepts only narrowly justified owner
context, never a blanket association for unavailable saved paths.

## Selection, storage and maintenance

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
- Conflict detection indexes selected candidate paths and retained current-pass
  minimum aliases, including resident records. It does not retain every alias
  ever seen. Indexes reset at `begin`; previous-pass hints are not fresh conflict
  evidence. A detected conflict stays latched even if its sample is evicted.
- Packed records and indexes store IDs/slots, not duplicate path keys. Remove
  entries before changing paths or slots; compare full native keys after hashes.
  Shared historical keys are allowed. Alias buffers are reused and capped at L.
- Successor links use runtime ID plus locator, so inode reuse cannot resolve old
  rebound work. Live ownership survives reader closure. A valid one-way hardlink
  is not a cycle: incremental connectivity filters potential cycles, a directed
  walk confirms them, and completion sweeps dependent binding work. Failures
  arrive immediately; incremental checks stop once the pass is incomplete.

Tokens are validated directly by generation, slot, selection and acknowledgement
state. A cursor skips acknowledged prefixes once. Statistics report pending
count, oldest age and actual admission time; overflow counts sightings, not
unique files. Graph walks and successor updates can visit all tracked records,
so rotation/admission latency still needs integrated qualification.

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
Admission/rebind may allocate bounded paths: there is no blanket zero-allocation
claim. See the implementation's [binding rules][bindings] and [storage][storage]
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
