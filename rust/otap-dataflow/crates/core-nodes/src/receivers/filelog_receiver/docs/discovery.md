# Bounded discovery scanner

`discovery::Scanner` implements synchronous Linux filesystem observation. It
accepts precompiled matching policy and fixed traversal prefixes, and returns
bounded, descriptor-free batches followed by a completion marker. It does not
register a receiver, assign durable identities, select pending candidates,
reconcile bindings, or finalize readers.

Outcome classification lives on `ScanReason`; `batch` owns reusable storage and
completion accounting without filesystem calls. Linux orchestration, root
resolution, traversal, and probing remain separate from those rules. Linux is
the only implemented backend; future backends can reuse the evidence and
batching contracts without adopting Linux directory traversal.

## Integration contract

- Configuration supplies absolute directory prefixes, matching policy, explicit
  limits, link policy, and the actual checkpoint namespace directory's locator.
  Obtain it from `CheckpointStore::namespace_locator()` after opening the store.
  Do not supply the engine's broader state-root identity in its place.
- Matchers evaluate lexical names, verified target paths, and full patterns
  through exclusion-root suffix mappings. They must not accumulate scanned names.
- Share one `ProbeBudget` with reader reopens. Acquisition reserves both transient
  descriptor slots together. A busy probe gets one scheduler yield and retry;
  continued contention makes coverage incomplete. Backoff and fairness belong
  to the worker; this allowance is not an OS reservation.
- Run `scan` on a blocking worker. The callback borrows at most `limits.batch`
  observations, with no filesystem handles retained. It must bound copies and
  check cancellation while waiting for downstream capacity. Returning false
  cancels the pass. Kernel calls already in progress cannot be interrupted.
- `reason == None` identifies a policy-checked, stable candidate. Its raw prefix,
  size, and modification time are handle-derived observations, not proof of
  durable identity. `Directory` observations let reconciliation compare bounded
  retained binding-parent identities. Other outcomes preserve errno and, where
  available, locator presence independently of candidate eligibility.
- `scan_with_requests` accepts a caller-owned bounded lookup. A presence-only
  request still pins the source and checks policy, but does not open/read content.
  It returns `PresenceOnly`, not a validated candidate. A different locator
  triggers full probing. Admission, reconnection, and validated reappearance
  require a full request; locator equality alone never grants those permissions.
- Failed or unstable probes do not publish candidate size or modification time.
  A failed second pin clears the unconfirmed locator; a single successful pin
  may still supply presence on policy/content failures.
- Overlapping roots and aliases can emit multiple observations. Reconciliation
  deduplicates its bounded tracked/pending populations; the scanner never builds
  an unbounded match or seen-inode set.
- Consume all batches before using the returned completion marker. Complete
  coverage is an operational polling result, not a namespace snapshot. A later
  retained-parent comparison may invalidate negative evidence too.
- `retained_bytes()` accounts for scanner-owned capacity between scans. Matcher
  memory and consumer copies have separate owners. Frame buffers are pooled up
  to the depth bound; retained accounting includes them after traversal. Root
  resolution, entry buffers, exclusion mappings, and fingerprints remain bounded.

## Source and target verification

`PinnedSource` separates the existing path-only validation from read conversion.
Discovery keeps the pin alive while evaluating policy and converting it through
trusted procfs. Each candidate has two sequential probes, with the first closed
before the second. Both locators and resolved paths must agree, size must not
shrink, and the second raw fingerprint must extend the first.

Target evidence uses bounded `/proc/self/fd` link text and a locator check through
`openat2(RESOLVE_NO_SYMLINKS)`. This checks intermediate components as well as the
last one. A capability check on the first pass reports `VerificationUnavailable`
if the required operation is unavailable, rather than reporting every root as
missing. Permanent capability failures stay cached until scanner recreation;
transient resource failures can retry on a later pass. There is no unrestricted
verification fallback. Successful verification therefore requires Linux 5.6 or
newer and a security profile permitting this syscall. The trusted procfs read-open
itself is unchanged. Neither zero links nor a `" (deleted)"` suffix bypasses policy.

Ancestor identities supply full-pattern exclusion-root mappings and checkpoint
namespace exclusion. A second target verification detects substitutions during
inspection. This is point-in-time evidence, not an atomic ancestry snapshot.
Concurrent hostile namespace changes and uninspected hardlink aliases retain the
limitations in the design specification.

Exclusion mappings are refreshed every pass, independently of include-root
baselines. For an absent literal prefix, the scanner maps through its verified
existing ancestor and retains the missing suffix and original pattern index.
It still evaluates the full pattern. A dangling prefix alias, permission error,
or failure after opening a root is not treated as an absent literal prefix.
Lexically excluded names inspected during traversal may contribute metadata-only
presence; pruned subtrees are not searched.

## Traversal and resources

Traversal holds one directory handle. Before descent or delivery it closes that
handle and retains an opaque enumeration cookie, directory locator, and change
indicators. A changed directory locator prevents resumption. A content-change
indicator invalidates completeness, but positive traversal can continue when
resumption is possible. Change indicators are not proof of an immutable listing.

Cookie resumption is enabled for ext4, tmpfs, XFS, and btrfs. Other backends use
bounded replay from the beginning: the previously consumed prefix must match
its saved entry count and digest before scanning continues. Replay failures
stop that traversal unit. Directory observations retain their locators on all
backends. A pass needs no resumption qualification when each directory is read
without replay. Passes that use replay remain incomplete even when it works;
no generic overlayfs or network-filesystem coverage qualification is claimed.
Replay can be quadratic across many small batches and is charged to the work
limit. It does not silently skip the rest of every stable directory.

A missing root is incomplete; the first successful validation establishes its
baseline. A replacement never silently becomes the new baseline. Returning the
expected root or recreating the scanner is required for complete coverage.
Validated files in a changed include root may still be observed without adopting
that root as the expected baseline. Positive observations remain usable during
incomplete passes.

A persistent dangling or looping link under a followed recursive include can
hide a required subtree. It makes every affected pass incomplete, blocking
discovery-derived finalization and retention receiver-wide. Diagnostics identify
the link; positive observations and qualified resident unlink handling remain
available. A leaf-only broken link follows the candidate-local rules instead.

Limits cover roots, native path bytes, recursion depth, fingerprint bytes, batch
size, traversal/replay work, and the fixed 8 KiB directory buffer. The work limit
counts enumeration steps (including replay) and traversal-directory opens; it
also bounds acyclic alias fan-out. Root preparation is separately bounded by
configured roots and native path length. Work exhaustion reports `Limit` and
incomplete coverage; it is not a wall-clock or cross-root fairness guarantee.
Allocation failures remain process allocation failures; limits do not reserve
OS memory or descriptors. Root and filesystem operations preserve original
errors. Caller-owned retry must not turn descriptor exhaustion into disappearance.

## Validation and measurements

Run the Filelog unit tests and `filelog_scan_allocations` on Linux. Resumption
tests cover multi-batch and nested traversal and detected churn; run them on
each supported filesystem. Explicit privileged cases cover tmpfs, bind-mount
pattern mapping, overlayfs replay, and unreachable targets. Run these only in
an isolated mount namespace with `CAP_SYS_ADMIN`.

`cargo bench -p otel-arrow-dfe-core-nodes --bench filelog_scan` compares full and
presence-only probing on flat, six-level, followed-link, and churn workloads,
with exclusion mappings configured. Set `FILELOG_BENCH_ROOT` to select the
fixture filesystem. The benchmark reports coverage completion separately from
positive file observations. See the
[measurement guide](../../../../benches/filelog_scan/README.md) for commands and
qualification boundaries.

The allocator test checks stable retained heap and reuse of observation and
fingerprint buffers. These scanner measurements do not qualify integrated reader
latency, downstream backpressure, or resident-unlink finalization; those belong
to the corresponding receiver integration work.
