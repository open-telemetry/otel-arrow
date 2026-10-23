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
4. After traversal, create one `Scanner::binding_batch`, even when no binding
   requests are queued. On root-refresh error, call `apply_binding_batch_error`
   or abort the pass; do not discard the error because the worklist is empty.
   On success, drive
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
completion. Candidate shrink, changed fingerprint prefixes, contradictory
tracked bindings, and conflicts against retained candidate paths emit
`CoverageLost(Unstable)` when detected, before the observation call returns.
Consumers must process that evidence even if the pass never finishes. Both
traversal and targeted checks identify changed roots. Parent
changes remain latched, including for inactive retained records, until the
expected environment returns or the receiver is recreated. Explicit `Unstable`
outcomes also invalidate matching retained candidate, binding and selected-alias
work, even when the failed probe supplies no locator. A retained rebound connects
the failure to its candidate if that candidate has since selected another alias;
unrelated stable candidates remain usable during the incomplete pass. Candidate-local
policy and verification failures likewise invalidate their dependent retained
evidence without blocking unrelated positive exclusion.

Only an effectively complete pass emits `Disappeared`. This starts the lifecycle
owner's pending-disappearance work; it does not establish permanent EOF.
Retention, runtime vetoes, terminal framing and finalization remain with their
owners. Evidence can repeat within or across passes and must be consumed
idempotently. In particular, `CoverageLost(Unstable)` can be emitted for several
conflicting observations/records and again at completion; it is not an
exactly-once notification.
Rejected candidates may be reconsidered after later fresh observations.

`Excluded` is distinct from disappearance. A positively ineligible binding can
support prompt exclusion during an incomplete pass. An excluded alias alone
cannot revoke a reader with a currently validated eligible binding or alias.
One earlier eligible alias is retained as a hint and revalidated before using it
to prevent exclusion. A definitive miss or a fresh different locator expires
that hint; busy or inaccessible checks retain it. Unknown binding checks require
the complete-pass fallback. Required alias-check coverage failures block prompt
exclusion for the rest of that pass, even if a later binding check is locally
conclusive. Detected root and exclusion-root failures block missing-path
conclusions receiver-wide for the rest of the pass, including after a successful
check retry. A parent-change latch carried from an earlier pass also blocks those
conclusions. Missing-path evidence retained before the failure cannot authorize
a later exclusion. Fresh positive policy or replacement evidence for an unaffected
binding remains usable; unrelated candidate-local policy failures do not revoke
that authority. A changed binding parent blocks that binding's prompt exclusion
and replacement publication until its expected identity returns. The final
complete-pass check still requires every retained parent latch to be clear.
Ordinary unrelated traversal incompleteness alone does not prevent a contextual
binding miss from authorizing exclusion.
Exclusion never changes offsets or durable records.

Targeted checks share traversal's ancestor subtree policy, root validation,
pin-bound source probes and transient descriptor budget. They recheck parents
and close handles before returning evidence. A missing parent with valid roots
is binding-local uncertainty; it permits the complete-pass fallback. Root,
environment and required-policy failures still invalidate coverage. Traversal
stack depth is a resource bound, not an exclusion rule.

## Bindings and identity feedback

An eligible established binding wins over a smaller alias. New proposals use
minimum complete native path bytes; observations never overwrite bindings.
Publishing an alias removes it from the alias index before clearing the hint,
so the next check uses the distinguished binding's parent context. Successful
publication marks the binding eligible; another proposal or rebind in the same
generation returns that published path. Invalid or empty proposals are rejected
before calling the durable owner. New bindings
reuse an established parent baseline from the existing parent index, including
the old binding when rebinding within one directory. A latched expected identity
wins over an unlatched neighbour, with runtime ID as the deterministic tie-breaker.
Unknown neighbours share that expected baseline rather than adopting a replacement
directory independently. A first targeted validation seeds unknown neighbours
together to avoid repeatedly scanning the group. A directory with no retained
baseline establishes its identity on first validation; discovery does not retain
an unbounded history of previously visited directories.

`Replacement::Possible(predecessor)` carries context to identity resolution,
including during incomplete scans and after the predecessor is retired or its
path changes again. This historical hint is not supersession authority. When
several predecessors contribute hints, the smallest runtime ID is retained as
a deterministic representative, not as the owner of the selected path. This rule
also applies to owner-supplied associations, independent of their order.
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

Rebound feedback is per-pass work. If a candidate is admitted at a smaller alias,
another path's predecessor can remain Live without a successor link. Its rebound
is reported again on each later pass that observes the replacement at that path,
even when the replacement is resident and produces no offer. The owner must
handle identity feedback and call `resolve_rebound` for the current generation
each time; `supersede` cannot be used after the replacement already has a Live
owner. This can repeat throughout incomplete passes, until a qualifying complete
pass publishes a new binding or the lifecycle owner deactivates/retires the
predecessor. Disappearance evidence alone does not retire the record.

Both atomic admission and legacy `supersede`/`track` use the same successor-link
helper. It updates established ancestors and retains runtime ID plus locator,
so inode reuse cannot resolve an old rebound. Active Live ownership remains
valid without a resident reader. Rebinding clears the record's successor link.
A one-way hardlink to a still-valid binding is not a cycle. New uncontested
rebound edges update per-pass disjoint sets held in the tracked records. Only
an edge joining already-connected endpoints needs a directed chain check;
this avoids repeatedly walking growing acyclic chains. Closing a swap/cycle
reports coverage loss even if the pass never finishes. Repeated unchanged
rebounds do not repeat this check. Once failure has been delivered and the pass
is incomplete, further incremental checks stop. Completion still sweeps the
graph to defer
dependent binding work. A confirmed-cycle walk can visit all tracked records;
rotation latency remains a qualification concern. The global rebound barrier
remains in force.

`TrackedFile::binding` holds a complete native path or is unavailable. The
bounded authoritative `AdvisoryPath` remains with checkpoint/identity state.
Its truncated suffix cannot be opened or used for native lexicographic ordering.
`associate_replacement` accepts narrowly associated owner-provided context; it
must not associate every candidate merely because some saved path is unavailable.
Its evidence callback reports contradictory associations immediately too.

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
for the entire pass, even when an overflow sample is later evicted. A bounded
slot-only path index checks fresh observations against retained candidates'
selected native paths without copying path keys. It includes pending and
sampled candidates, updates before path mutation or slot reuse, and resets at
`begin`; historical paths from earlier passes are not conflict evidence. It is
not an unbounded history of all observed aliases. A second ID-only index covers
the current-pass minimum alias already retained for each tracked record, so
resident-first and resident-versus-resident conflicts use the same evidence
rules. Both indexes reset at `begin`; the previous-pass alias hint is not fresh
path-conflict evidence. Conflicting candidates are
withheld, including a newly sampled candidate that replaces the conflicting
slot. Token
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
