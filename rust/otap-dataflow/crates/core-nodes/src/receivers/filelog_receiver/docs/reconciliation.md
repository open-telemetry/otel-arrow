# Discovery reconciliation

`discovery::reconcile::Reconciler` consumes bounded scanner observations and
selects candidates for the identity owner. It owns no filesystem handles,
workers, checkpoint writes, source offsets or finalization decisions.

## Ownership and call order

Use one owner and one active pass:

1. Seed `TrackedFile` views. `TrackedId` must remain unique for the reconciler's
   lifetime, even after removal; it is never derived from a locator or path.
2. Call `begin` with a newer generation, monotonic time and overflow-selection
   seed. The implementation also mixes the generation into the seed.
3. Feed every scanner observation to `observe` in callback order. Apply emitted
   evidence synchronously, before consuming the next observation.
4. After traversal, create one `Scanner::binding_batch`. Drive
   `next_binding_check`, borrow `binding_request(id)`, call `batch.check`, then
   apply its scanner-owned result with `apply_binding_check`. Repeat until the
   cursor returns None. No path copies or repeated full-table searches are
   needed. The preallocated worklist admits each tracked ID at most once per
   pass; each ID needs at most an earlier-alias check and a binding check.
   If `batch.check` rejects the request before returning evidence, call
   `skip_binding_checks` for that ID or abort the pass. Skipping advances work
   conservatively: no negative conclusion is allowed from that failed pass.
5. Pass the exact scanner `ScanCompletion` to `finish`. Targeted checks do not
   contribute to its observation count. Protocol errors poison completeness;
   `abort` can end a failed or cancelled consumer pass.
6. Process at most `limits.offers` candidates through the identity owner.
   Prefer `admit`: shared preflight validation runs before the identity callback,
   and a private insertion path avoids repeating protocol validation afterward.
   The callback supplies the actual admission time or defers without mutation.
7. Drain actionable `rebounds` before complete-pass binding publication.
   `rebind` holds the exclusive reconciler borrow while its commit callback runs;
   false defers publication without changing the old binding.
8. Schedule the next pass from completion plus `reconcile_delay`. Workers and
   timers belong to the caller. Configuration owns interval/jitter bounds and
   must exclude 100% jitter when a positive delay is required; the receiver's
   policy caps jitter at 25%. Neither slow scans nor early requests allow overlap.

Callbacks must not panic and then resume the owner, or return a deferred result
following an ambiguous durable write. These APIs do not implement checkpoint
recovery. A transport adapter must preserve synchronous ordering and bounded
backpressure. Permanent-EOF decisions must serialize against already-delivered
observations and failures; an empty-channel check is insufficient.

## Evidence is not a lifecycle decision

`Present` blocks discovery-derived absence conclusions but does not reconnect a
file or cancel finalization. `Validated` supplies fresh eligible evidence for
identity and resident-handle validation. Locator equality alone proves neither.

`CoverageLost`, `RootChanged(path)` and `ParentChanged(id)` arrive before pass
completion. Both traversal and targeted checks identify changed roots. Parent
changes remain latched, including for inactive retained records, until the
expected environment returns or the receiver is recreated.

Only an effectively complete pass emits `Disappeared`. This starts the lifecycle
owner's pending-disappearance work; it does not establish permanent EOF.
Retention, runtime vetoes, terminal framing and finalization remain with their
owners. Evidence can repeat across passes and must be consumed idempotently.
Rejected candidates may be reconsidered after later fresh observations.

`Excluded` is distinct from disappearance. A positively ineligible binding can
support prompt exclusion during an incomplete pass. An excluded alias alone
cannot revoke a reader with a currently validated eligible binding or alias.
One earlier eligible alias is retained as a hint and revalidated before using it
to prevent exclusion. A definitive miss or a fresh different locator expires
that hint; busy or inaccessible checks retain it. Unknown binding checks require
the complete-pass fallback. Required alias-check coverage failures block prompt
exclusion for the rest of that pass, even if a later binding check is locally
conclusive. Exclusion never changes offsets or durable records.

Targeted checks share traversal's ancestor subtree policy, root validation,
pin-bound source probes and transient descriptor budget. They recheck parents
and close handles before returning evidence. A missing parent with valid roots
is binding-local uncertainty; it permits the complete-pass fallback. Root,
environment and required-policy failures still invalidate coverage. Traversal
stack depth is a resource bound, not an exclusion rule.

## Bindings and identity feedback

An eligible established binding wins over a smaller alias. New proposals use
minimum complete native path bytes; observations never overwrite bindings.

`Replacement::Possible(predecessor)` carries context to identity resolution,
including during incomplete scans and after the predecessor is retired or its
path changes again. This historical hint is not supersession authority. When
several predecessors contribute hints, the smallest runtime ID is retained as
a deterministic representative, not as the owner of the selected path.
A candidate's own existing durable state wins. Reconnection leaves the
predecessor Live and its rebound unresolved; a
competing live binding is rejected before the identity callback. Only admission
requiring a new identity can supersede the predecessor on an incomplete pass.
Only that branch can exempt a claim during binding preflight. The exemption
and supersession target come from the current Live owner of the selected
candidate path and must match fresh rebound evidence. Other hardlink predecessors
remain visible through `rebounds` for identity feedback. Historical context
alone never blocks admission or changes a predecessor. No predecessor progress
is transferred.

Both atomic admission and legacy `supersede`/`track` use the same successor-link
helper. It updates established ancestors and retains runtime ID plus locator,
so inode reuse cannot resolve an old rebound. Active Live ownership remains
valid without a resident reader. Rebinding clears the record's successor link.
A one-way hardlink to a still-valid binding is not a cycle; real swaps/cycles and
unstable observations still defer binding publication. The global rebound
barrier remains in force.

`TrackedFile::binding` holds a complete native path or is unavailable. The
bounded authoritative `AdvisoryPath` remains with checkpoint/identity state.
Its truncated suffix cannot be opened or used for native lexicographic ordering.
`associate_replacement` accepts narrowly associated owner-provided context; it
must not associate every candidate merely because some saved path is unavailable.

## Bounded selection and memory

Explicit limits bound tracked records, pending candidates, offers, path bytes
and fingerprints. Construction checks arithmetic and a standalone 1 GiB guard.
Integrated admission must additionally include scanner, checkpoint, reader and
transport storage. The proposed allowance requires separate approval: see
[the memory proposal](reconciliation-memory-proposal.md). The accepted conformance
formula is unchanged; this implementation must not be wired to its smaller
budget without resolving that design gate.

Pending candidates are offered oldest-first with a stable arrival-order tie.
Re-observation preserves age. Unseen pending entries survive incomplete scans,
but are not offered as fresh evidence; complete scans may discard them.
Overflow uses at most `limits.offers` sample slots with a generation-varying
locator rank. Mixed batches reserve overflow opportunities and backfill unused
slots; a one-slot budget alternates groups. Full tracked capacity still permits
identity matching and reconnection. Only identities proven to require new slots
are parked while capacity is full. No finite starvation bound is claimed.
Detected candidate instability is latched
for the entire pass, even when an overflow sample is later evicted. Token
validation uses generation, slot bounds, selection and acknowledgement state
without searching the offer list. A feedback cursor skips acknowledged prefixes
once, so draining offers does not repeatedly scan earlier acknowledgements.

`overflow_observations` counts sightings, not unique unretained files. Statistics
also report pending depth, oldest age and actual successful admission time.
Records live in a packed vector; locator/path/parent indexes store only IDs and
compare native keys against record-owned storage. Alias capacities are capped
at the path limit. Index removal precedes path mutation or slot removal, and
moving a packed record updates its primary ID entry. Full key comparisons remain
required after hash matches, including for shared historical bindings.

`retained_bytes` charges spare buffers and allocated tracked-table storage.
Candidate-index estimates and the admission proof are toolchain-specific and
must be revalidated on upgrades; this is not allocator-exact RSS.

## Validation scope

Deterministic regressions cover ordering, incomplete evidence, admission,
reconnection conflicts, repeated rotations, inode reuse, subtree policy,
diagnostics, alias expiry, fairness and resource bounds. Isolated allocator
tests measure warm overflow reconciliation and the full binding worklist,
scanner-check and result-application loop. Admission and rebind may allocate
bounded path/index storage; there is no blanket zero-allocation claim.

This is not a registered receiver. Durable recovery, worker/channel integration,
retention, finalization and whole-receiver performance remain separate work.
The accepted lifecycle contract is #4443 at
`d9781ee3f122647b050840f93869ea8bb068aa31`; its acceptance does not approve the
separate memory-model amendment.
