# File identity design rationale

The [receiver specification][spec] and [conformance matrix][matrix] define the
recovery contract. This document explains its source-read error handling,
Task 12's association interpretation and the relevant test obligations. These
rules do not change the checkpoint wire format.

## 1. A read error does not prove that a file changed

Before resuming a file, Filelog checks its fingerprint prefix and a small byte
window near the saved offset. This **frontier window** helps check whether the
saved progress still belongs to the file being opened.

An inability to read evidence is different from evidence that the file changed.
The specification makes this distinction:

| What happened | Behavior |
| --- | --- |
| The bytes do not match the saved digest | Apply the identity-mismatch policy. |
| The saved offset is beyond the file size, or EOF prevents reading the full window | Apply the identity-mismatch policy. |
| Reading fails with an I/O error | Keep the saved identity and progress. Return the original error for worker retry or failure handling. |
| A read is interrupted | Retry within a bounded budget, checking cancellation between attempts. If the budget runs out, return a source error. |

For example, suppose the saved offset is 1,000 and the disk returns an error
while Filelog checks the window. That error tells us the read failed. It does
not tell us that the file was replaced. Resetting or quarantining the identity
on that evidence alone could discard valid recovery state.

On an I/O error, preserve the record, file ID, offset, epoch, prefix and frontier.
Do not replace the identity or quarantine it solely because of the error.
Repeated errors do not turn into proof of replacement.

The cost is availability: a persistently unreadable file retains its tracked
slot and cannot resume until continuity validates. Errors retain their original
classification: retry or termination depends on the operation and failure, not
on an assumption that every error is temporary. Environmental retries use
bounded backoff and rate-limited health reporting, including for persistent
unreadability. Repeated failure does not become mismatch evidence. Failed
checkpoint syncs remain terminal.

Tests cover EIO, WouldBlock, successful retries after Interrupted, retry-budget
exhaustion, cancellation between retries, partial reads that complete, and early
EOF. Repeated source-read failures preserve state under every mismatch policy
until a later attempt validates continuity. Tests inject failures through a
private helper around a real `SourceFile`;
there is no new public source-backend framework.

## 2. How a file at a different locator can be associated

A locator identifies a file on the local filesystem. When a candidate has a
different locator, Filelog may still find evidence that it is related to a
saved Active record, for example after a copy.

This is **association**, not proof that the files are the same stream. It chooses
which starting policy to use. It never transfers the old file ID or progress.

The baseline conformance cases include both short and full matching fingerprints.
They favor rereading copied bytes over accidentally skipping unread bytes. Task
12 keeps that preference rather than using the earlier path-only proposal.

Task 12's comparison rule is:

- Consider retained Active records only.
- A full native path matches when its kind, complete length and digest match.
- For fingerprints, start at the same `ignored_header_bytes` offset and compare
  as many bytes as the shorter fingerprint contains. All those bytes must match,
  and there must be at least one byte. Short files and prefixes that grew can
  match; empty fingerprints cannot.
- Keep the existing precedence: the candidate's own exact-locator record and
  validated replacement context come before association.

Association uses `on_recovery_mismatch` instead of `start_at` or
`ignore_older_than`. That has a tradeoff: unrelated files can share headers or
prefixes, especially when the stored fingerprint is short. Under `fail`, a false
association can quarantine an unrelated file. The configuration guide must make
these effects clear.

A larger fingerprint threshold does not remove all ambiguity. Requiring the old
locator to disappear is not sufficient either: copytruncate leaves the original
file in place, and an incomplete scan cannot prove that a file disappeared.

This rule clarifies the comparison. It does not claim that fingerprints uniquely
identify files or guarantee lossless copytruncate capture.

### Why not require the candidate to contain the whole saved fingerprint?

That alternative would reduce some false matches, but could skip surviving data:

1. A file is copied while it is short.
2. The original grows, and Filelog saves a longer fingerprint for it.
3. The original is deleted.
4. After restart, the shorter copy contains the only surviving unread bytes.

The copy cannot match the entire longer fingerprint, so that alternative rejects
its association. This can happen even when the original was only ever appended
to. A regression test covers this order of events.

Keep common-prefix matching for the MVP. Narrowing it requires either accepting
this capture gap or collecting additional evidence. We cannot assume that every
copy was made after the latest fingerprint was saved.

## 3. Behaviors to include in conformance tests

These cases make the admission and checkpoint contracts explicit:

- **Progress during admission:** ordinary progress on another file does not
  invalidate a prepared admission. Changes to the exact record's progress do.
- **Identity changes:** every non-progress mutation and finalization advances
  the identity revision, the counter used to reject stale plans. Finalization
  must not be treated as ordinary progress.
- **Rejected writes:** an operation rejected before writing advances neither
  the revision nor durable progress. It does not require keeping a lease for an
  uncertain write.
- **Quarantine cleanup:** report a committed quarantine even if cleanup then
  fails. Report the cleanup failure too and stop the owner; do not describe the
  quarantine as an uncommitted operation.
- **Fingerprint growth:** after empty-file registration, queue verified prefix
  growth and persist it in bounded non-progress groups at a sync boundary.
  Avoid one growth fsync per resumed file. Progress and growth never share a
  WAL transaction.
- **Early blocked results:** `Quarantined` and `ProfileBlocked` report checkpoint
  state. They do not prove current path validity, policy eligibility or a
  successful reader reconnection.

Throttling fingerprint growth at size thresholds remains an optional performance
improvement. It is not required to fix the confirmed failure paths.

[spec]: filelog-receiver-phase1-spec.md#committed-frontier-continuity-guard
[matrix]: filelog-receiver-phase1-conformance.md
