# Bounded discovery scanner

`discovery::Scanner` finds and checks files on Linux. It walks configured
include directories, applies exclusions, and delivers small batches of
observations. Each call to `scan` is one pass and returns a `ScanCompletion`.

[Reconciliation](reconciliation.md) owns bounded evidence, candidate selection and
binding proposals. Identity and lifecycle owners decide admission and finalization.
The scanner only supplies observations. Linux file access is
separate from the observation and batching contracts for future platform reuse.

## Complete and incomplete passes

- **Complete:** the required traversal and checks succeeded. Reconciliation can
  use the results together with its rotation and retention rules. This is still
  a polling result, not a snapshot proving that a file is gone.
- **Incomplete:** some coverage or required check was unavailable. Candidates
  that passed their own checks remain usable, but the pass cannot support
  conclusions that unseen files have disappeared or can age out of retention.

Incomplete passes are expected, for example when a directory changes during a
scan. They do not by themselves stop existing readers. The cause matters:
exclusion failures can prevent admission, and traversal failures can leave files
undiscovered. The worker can retry on a later pass; persistent causes need
attention. Later comparisons of tracked files' parent directories can also
invalidate absence conclusions from an otherwise complete pass.

## Using the scanner

- Supply absolute directory prefixes, precompiled matchers, symlink policy, and
  limits. Matchers check discovered names, verified targets, and full exclusion
  patterns through directory aliases; they must not retain every scanned name.
- Open the checkpoint store first and obtain its namespace identity through
  `CheckpointStore::namespace_locator()`. Exclude that namespace, not the whole
  engine state directory.
- Run `scan` on a blocking worker. Its callback borrows at most `limits.batch`
  observations. No directory or source handles remain open during delivery.
  Keep any copies bounded and check cancellation while waiting for downstream
  capacity. Returning `false` stops the scan. A blocked kernel call cannot be
  interrupted by the cancellation callback.
- Share one `ProbeBudget` with reader reopens. It reserves both temporary source
  descriptor slots together. A busy probe yields and retries once; if still busy,
  the scan is incomplete. Worker scheduling owns backoff and fair access. Under
  sustained reader load, bounded permit holds must still let readers progress
  and stable scans within the work budget complete, with responsive cancellation.
  The single retry does not establish that fairness.
- Consume every batch before acting on the completion result. Overlapping roots
  or aliases can produce duplicates; reconciliation deduplicates its bounded
  tracked and pending populations.

## Understanding results

| Result | Meaning |
| --- | --- |
| `reason == None` | Both probes and policy checks passed. |
| `PresenceOnly` | Known locator and policy checked; no content read. |
| `Directory` | Identity for retained binding-parent comparisons. |
| Other reasons | Rejection or failure; errno and presence where available. |

Full candidates include handle-derived size, modification time, and fingerprint
bytes. Presence alone does not authorize admission or reconnection.

`scan_with_requests` lets the caller use a bounded lookup to request presence-only
checks for known files. If the locator differs, the scanner performs full probing.
Admission, reconnection, and validated reappearance require a full request;
matching device/inode alone does not establish continuity.

Failed or unstable probes do not publish candidate size or modification time.
A failed second pin clears the unconfirmed locator. A successful pin may still
provide presence evidence when policy or content access fails.

## Checking files and exclusions

`PinnedSource` holds the file open without opening its content for reading.
The scanner checks policy while that pin remains alive, then converts it to a
read handle through trusted procfs. Full validation uses two sequential probes;
the first closes before the second starts. Locators and resolved paths must
match, size must not shrink, and the second fingerprint must extend the first.

Target verification reads the bounded `/proc/self/fd` path and uses
`openat2(RESOLVE_NO_SYMLINKS)` to check it against the pin. This rejects symlinks
in any path component. The syscall was introduced in Linux 5.6 and must be
permitted by the host or container security settings. The first scan checks
actual availability. If unavailable or blocked, it reports
`VerificationUnavailable` and admits no candidates; there is no fallback.
Permanent capability failures are cached until scanner recreation, while
transient resource failures can retry on a later scan.

Ancestor identities identify directory aliases and the checkpoint namespace.
Exclusions still use the full pattern. A second target verification checks for
substitution during inspection. Concurrent hostile changes and uninspected
hardlinks retain the design's limits: these checks are not an atomic snapshot.
Zero links and names ending in `" (deleted)"` do not bypass policy.

Exclusion mappings refresh every scan. An absent literal prefix can map through
its verified existing ancestor, retaining the missing suffix and pattern index.
A dangling alias, denied access, or failure after opening the prefix cannot use
that rule. Mapping failures report `ExclusionUnavailable`; candidate verification
failures report `Policy`. Both make coverage incomplete. While a required
mapping is unavailable, candidates that can still be pinned may report presence,
but none are admitted. Cancellation remains `Cancelled`.

Excluded names encountered during traversal can supply metadata-only presence.
Entirely excluded subtrees are not searched. Native path bytes are preserved
without truncation or lossy conversion, and repeated syscalls reuse bounded
C-string buffers, including for paths of 256 bytes or more.

## Directory traversal

Traversal holds one directory handle at a time. Before descent or batch delivery,
it closes that handle and saves the listing position, directory identity, and
change indicators. A different identity prevents resumption. Other detected
changes make coverage incomplete, but scanning can continue where resumption
is possible. Unchanged indicators do not prove the directory stayed unchanged.

| Filesystem | How a reopened listing continues |
| --- | --- |
| ext4, XFS, btrfs | Resume from the saved directory cookie. |
| Other backends | Replay the prefix; the scan stays incomplete. |

Replay applies to tmpfs, NFS, CIFS, overlayfs, and other unqualified backends.
It compares the saved entry count and digest before continuing. A match allows
progress but does not establish complete coverage.

A directory read in one opening needs no resumption support. Descending into a
subdirectory requires reopening the parent afterwards. On replay backends,
therefore, traversing subdirectories makes the scan incomplete even for a stable
tree. A recursive pattern alone does not cause this if no subdirectory is visited.
Tmpfs behavior varies by kernel; testing one kernel does not qualify other
versions or vendor backports for native resumption.

Replay reaches later entries in a stable directory. A mismatched prefix stops
that directory's traversal. Repeated changes can prevent reaching its tail, and
replaying across many batches can require quadratic work. Replay counts against
the work budget. These costs and the resulting cleanup delays need measurement
before relying on such filesystems for discovery-based finalization or retention.

A missing include root makes coverage incomplete. Its first successful validation
establishes the expected identity. A replacement root can supply positive
observations but does not become the new baseline. Complete coverage requires
the expected root to return or the scanner to be recreated.

A persistent dangling or looping link that might lead to a required subtree also
keeps scans incomplete. Diagnostics identify the link. This blocks discovery-based
finalization and retention across the receiver; positive observations and
qualified resident-unlink handling remain available. A leaf-only broken link is
handled as a candidate-local outcome instead.

## Resource limits

Limits cover root count, path length, depth, fingerprint size, batch size, and
traversal work. The directory buffer is fixed at 8 KiB. Frame buffers are pooled
up to the depth limit. `retained_bytes()` reports scanner-owned capacity between
scans, including pooled frames; matcher memory and consumer copies are accounted
for separately.

`ScanConfig.limits.work_units` defaults to 1,000,000. It counts directory opens
and enumeration steps, including replay, and bounds branching directory aliases.
Root preparation is separately bounded by root count and path length. Exhaustion
reports `Limit` and incomplete coverage. It does not bound time inside kernel
calls or guarantee fair progress across roots.

With a fixed traversal order, repeated exhaustion can leave the same files or
roots undiscovered until the budget or tree changes. Rotating roots alone cannot
fix this inside one large root. Integration must report persistent exhaustion as
an actionable health problem, without treating every temporary change as invalid
configuration.

These limits do not reserve OS memory or descriptors. Allocation failures keep
normal process behavior. Filesystem errors retain their original errno, and
retries must never treat descriptor exhaustion as evidence that files disappeared.

## Testing and measurements

Run the Filelog unit tests and `filelog_scan_allocations` on Linux. Test nested
and multi-batch traversal, resumption, and directory changes on each supported
filesystem. The privileged filesystem cases cover tmpfs, bind-mount exclusions,
overlayfs replay, and unreachable targets; use an isolated mount namespace with
`CAP_SYS_ADMIN` for mount-based tests.

The `filelog_scan` benchmark compares full and presence-only checks on flat,
six-level, followed-link, and changing-directory workloads, with exclusions.
`FILELOG_BENCH_ROOT` selects the fixture filesystem. Results report observed files
and complete passes separately. See the
[benchmark guide](../../../../benches/filelog_scan/README.md) for commands and
measurement limits.

The allocation test checks stable retained heap and buffer reuse. Reader latency,
downstream backpressure, and resident-unlink finalization still require receiver
integration tests.
