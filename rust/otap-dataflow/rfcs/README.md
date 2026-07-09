# OTel-Arrow RFCs

Most changes to `otap-dataflow` can go through the normal issue and pull
request workflow described in [CONTRIBUTING.md](../../../CONTRIBUTING.md).
Some changes are "substantial" enough that they benefit from a little more
design process and a written record: that is what this RFC ("request for
comments") process is for.

An RFC is a short design document that captures the motivation, the proposed
design, the alternatives considered, and the open questions for a substantial
change -- reviewed as a pull request so the whole SIG can comment before code is
written.

## When you need an RFC

Open an RFC when a change is cross-cutting or hard to reverse. Rough guidance --
use judgment, and ask a maintainer if unsure:

- A new public API, trait, or macro used across crates.
- A new pipeline node **category** or a change to how components are
  registered, discovered, or described.
- A protocol or wire-format change, or anything that affects
  interoperability with the OpenTelemetry Collector.
- A breaking change to configuration, or to a published crate's public API.
- A new workspace-wide convention or CI gate that contributors must follow.
- Anything the CONTRIBUTING guide flags as needing "design consensus".

You do **not** need an RFC for bug fixes, a single new node that follows an
existing pattern, documentation, tests, refactors with no API change, or
performance work that does not change behavior. When in doubt, file a
[Feature Request](https://github.com/open-telemetry/otel-arrow/issues/new?template=feature_request.yaml)
first and let a maintainer tell you whether it warrants an RFC.

## The process at a glance

1. **Draft.** Copy [`0000-template.md`](./0000-template.md) to
   `NNNN-my-proposal.md`. Leave `NNNN` as `0000` in the filename for now; you
   will rename it once a number is assigned (see step 4). Fill in the template.
2. **Open a pull request** adding your RFC file. The PR *is* the RFC
   discussion. Fill in the `RFC PR` link in the document header once the PR
   exists. Announce it in the SIG so reviewers see it.
3. **Discuss and revise.** Reviewers comment on the PR. Update the document in
   place; the git history of the PR is the record of how the design evolved.
   Move settled points out of "Unresolved questions" as they are resolved.
4. **Assign a number.** RFC numbers are a **monotonic serial**, assigned by a
   maintainer when the RFC is close to acceptance. **The number is independent
   of the PR number.** The first RFC is `0001`; the next is `0002`, and so on.
   Rename the file from `0000-my-proposal.md` to `NNNN-my-proposal.md`.
5. **Merge on acceptance.** When a maintainer (or the SIG, for larger changes)
   approves, the PR is merged. A merged RFC is "accepted" -- it records a
   decision, not necessarily shipped code.
6. **File a brief tracking issue** that links back to the merged RFC and tracks
   the implementation work. Keep the issue short: the RFC holds the detail.
7. **Implement** in follow-up PRs that reference the RFC and the tracking
   issue. Implementation may reveal problems with the design; if so, amend the
   RFC in a follow-up PR rather than letting the document drift from reality.

## Numbering

- Numbers are a zero-padded four-digit **serial**: `0001`, `0002`, ...
- `0000-template.md` is reserved for the template and is never an RFC.
- The number is assigned near acceptance (step 4), not when the PR opens, so
  concurrent drafts do not fight over numbers. If two RFCs are ready at once, a
  maintainer assigns numbers in merge order.

## File naming

`NNNN-short-kebab-case-name.md`, e.g. `0001-component-inventory.md`. Keep the
short name stable once assigned; it is referenced from tracking issues and
implementation PRs.

## Status of a merged RFC

A merged RFC is a decision record, not a guarantee. An RFC may later be
superseded or amended by another RFC; when that happens, add a note to the top
of the old RFC pointing at the new one. RFCs are not deleted -- the history is
valuable.

## Prior art

This process is deliberately lightweight and borrows from the
[Rust RFC process](https://github.com/rust-lang/rfcs) and
[Apache OpenDAL's RFCs](https://github.com/apache/opendal/tree/main/core/core/src/docs/rfcs).
The document structure in [`0000-template.md`](./0000-template.md) follows the
same shape those projects use.
