# Proposed Task 12 identity clarifications

Status: implementation follow-up for review after discovery design PR #4443
merges. That PR was still open at `d9781ee3f122647b050840f93869ea8bb068aa31`
when checked. This document does not amend it or assert that the changes below
were already unambiguously required by the accepted specification.

The Task 12 follow-up implements these safer semantics on its feature branch.
The normative specification and conformance matrix should adopt the agreed
wording together before claiming Phase 1 conformance.

## Environmental frontier failures

The current frontier section says that failure to read the window follows the
identity-mismatch policy. That includes environmental I/O failures literally;
the narrower rule below is a recorded behavioral clarification/change.

Proposed rule:

- A digest mismatch, offset beyond size, or EOF before collecting the required
  frontier bytes is failed continuity evidence and follows mismatch policy.
- A source I/O error is not evidence of replacement. Preserve the record, ID,
  offset, epoch, prefix and frontier; return the original error for caller-owned
  environmental retry or failure handling. Do not supersede or quarantine solely
  because of that error.
- Interrupted reads may retry with a bounded budget and cancellation checks.
  Budget exhaustion remains a source error, never a continuity mismatch.

Regression cases cover EIO, WouldBlock, Interrupted followed by success,
Interrupted budget exhaustion, cancellation between retries, successful partial
reads and early EOF. Fault injection wraps a real SourceFile behind a private
read-evidence seam; it does not add a public source backend framework.

## Changed-locator association

The baseline conformance includes short and full equal fingerprints and favors
replay of copied bytes over skipping them. The implementation restores that
conservative policy rather than adopting the earlier path-only proposal.

The explicit comparison is a nonempty equal common raw prefix at the same
`ignored_header_bytes` offset. It permits short files and evidence that grew;
empty fingerprints do not associate. Retained Active records participate.
A full native path match also associates, using kind, full length and digest.
Neither kind of association transfers identity or progress across locators.
The candidate's own exact-locator record and validated replacement context keep
their existing precedence.

Common headers can falsely associate independent files, particularly with small
fingerprint windows. Association bypasses start_at and ignore_older_than and
selects mismatch policy; the config guide must state this, including possible
quarantine under fail. Neither a larger threshold nor requiring the old locator
to be absent resolves the general ambiguity. Copytruncate can leave that locator
present, and incomplete inventories cannot prove disappearance.

This is a comparison clarification for review, not a claim that fingerprints
uniquely identify files or guarantee lossless copytruncate capture. The
frontier-I/O change above still requires normative wording to be reconciled.

## Additional conformance cases

- Ordinary progress on another file between prepare and commit does not invalidate
  admission; changes to the exact record's progress do.
- Any non-progress mutation or finalization advances identity revision. In
  particular, finalization cannot masquerade as harmless ordinary progress.
- Rejected pre-write operations advance neither identity revision nor durable
  progress, and never require retaining an uncertain-write lease.
- A committed quarantine and a subsequent cleanup failure are both reported;
  the owner stops, but the durable outcome is not hidden as an uncommitted error.
- Growth after empty registration is queued from verified reader evidence and
  persisted in bounded non-progress groups at a sync boundary, not one fsync per
  resumed file. Progress and growth never mix in one WAL transaction.
- Quarantined/ProfileBlocked checkpoint status is not current path validation,
  policy eligibility, or a successful reader reconnection.

No wire-format change or modification to #4443 is proposed here.

## Directional-prefix alternative not adopted

A proposed alternative requires the candidate to cover the entire stored
fingerprint. That reduces matches between a tiny candidate and a long stored
prefix, but it is not capture-equivalent even for append-only sources. A complete
copy can be created first, then the original grows and a longer fingerprint is
persisted before the original is unlinked. After restart, the shorter copy may
contain the only surviving unread bytes. The directional rule rejects it.

A regression covers that ordering explicitly. Keep the conservative common-
prefix rule for this MVP. Narrowing it requires accepting that capture gap or
additional evidence, not an assumption that all copies postdate the latest
fingerprint observation. Size-threshold growth throttling remains a separate
optimization; it is not required to fix the confirmed failure paths.
