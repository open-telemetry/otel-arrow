# Issue Triage

This document describes how issues are triaged in the OpenTelemetry Arrow
(otel-arrow) project. It complements
[CONTRIBUTING.md](./CONTRIBUTING.md) and follows the conventions
established by the broader OpenTelemetry community and the OpenTelemetry
Collector projects.

Pull request review and merge conventions are covered in
[CONTRIBUTING.md](./CONTRIBUTING.md) and are intentionally out of scope
here; this document is focused on the issue backlog. Release planning,
detailed security response (see [SECURITY.md](./SECURITY.md)), and project
board conventions are also out of scope.

The goals of this process are:

1. Give every issue a predictable, transparent path from "filed" to
   "actioned or closed."
2. Make triage primarily an asynchronous activity so that the weekly SIG
   meeting can focus on discussion, design, and unblocking work rather than
   walking the backlog item by item.
3. Establish a shared vocabulary (labels, states, priorities) so contributors
   know what to expect and where to help.
4. Make ownership and next-steps explicit without overloading the small set of
   maintainers and approvers.

## Roles in Triage

The OpenTelemetry membership ladder is defined in the
[community membership guide](https://github.com/open-telemetry/community/blob/main/guides/contributor/membership.md).
This section describes how those roles map to triage activities in this
repository.

### Triagers

Triagers are listed in [CONTRIBUTING.md](./CONTRIBUTING.md#triagers). They are
the primary owners of the triage process and are expected to:

- Process the `triage:deciding` queue on a regular cadence (see
  [Cadence](#cadence)).
- Apply the appropriate triage labels (type, area/component, priority)
  per [Label Taxonomy](#label-taxonomy).
- Ask clarifying questions, request reproductions, or close out-of-scope
  reports.
- Identify items that need maintainer or approver judgment and escalate them
  via the `status:needs-discussion` label.
- Curate the agenda for the live triage segment of the SIG meeting (see
  [SIG Meeting Triage](#sig-meeting-triage)).

Triagers do not need merge rights. New triagers are nominated by a maintainer
after roughly one month of consistent triage participation, per the community
guide.

### Approvers and Maintainers

Approvers and maintainers are listed in [CONTRIBUTING.md](./CONTRIBUTING.md).
In triage they:

- Make final accept/reject decisions on `type:enhancement` and
  `type:proposal` issues.
- Set or adjust `priority:*` when an issue is release-impacting.
- Sponsor work on new components, large features, or breaking changes.
- Step in as a tie-breaker on disputed labels or scope.

### Contributors

Anyone can help triage, but only Triagers and above can apply labels,
assign issues, or close them on someone else's behalf (see
[GitHub Permissions](#github-permissions) below). What everyone can do:

- Reproduce reported bugs and leave findings in a comment.
- Suggest labels in a comment; a triager will apply them.
- Volunteer to work on an unassigned issue by commenting "I'd like to take
  this" and a triager will assign you. (A future slash-command bot will
  let contributors self-assign without a triager round-trip.)
- Resolve an issue by opening a PR whose description uses "Closes #N";
  the issue closes automatically when the PR merges, regardless of its
  current status label.

### GitHub Permissions

GitHub's repository permission levels determine who can perform which
triage actions. This is why most of the labeling, assigning, and closing
in this document is restricted to Triagers and above - it is a GitHub
constraint, not a project policy choice.

| Action                                | Required GitHub role |
|---------------------------------------|----------------------|
| Open an issue                         | Read (anyone)        |
| Comment on issues                     | Read (anyone)        |
| Reproduce a bug, suggest a label      | Read (anyone)        |
| Apply / remove labels                 | Triage               |
| Assign issues                         | Triage               |
| Close / reopen issues opened by others| Triage               |
| Mark duplicates, transfer issues      | Triage               |
| Change repository settings, secrets   | Admin (Maintainer)   |

The full GitHub reference is
[Repository roles for an organization](https://docs.github.com/en/organizations/managing-user-access-to-your-organizations-repositories/managing-repository-roles/repository-roles-for-an-organization).

Two practical implications:

1. **Contributors cannot drive their own issues through the status
   transitions in [Issue Lifecycle](#issue-lifecycle).** They depend on a
   triager to apply triage labels (status, type, component / area,
   priority). The async-first triage workflow is designed around this: a
   triager picks up `status:needs-triage` items and does the labeling on
   the contributor's behalf.
2. **A slash-command bot (future work) can grant the *effect* of those
   permissions to contributors without granting the role itself.** A bot
   running with Triage permissions can read a `/assign` comment and
   perform the action on behalf of the author. This is the standard
   pattern in Kubernetes (Prow) and many CNCF projects, and is listed in
   [Open Items](#open-items-for-implementation).

## Issue Lifecycle

Every issue carries exactly one `status:*` label at a time, describing
where it sits in the triage flow. Changing a status label requires the
Triage permission or higher (see [GitHub Permissions](#github-permissions))
- contributors cannot move their own issues between states.

The complete state diagram (rendered by GitHub from Mermaid source):

```mermaid
stateDiagram-v2
    direction LR
    [*] --> needs_triage : opened (auto)

    needs_triage --> accepted          : triager confirms scope
    needs_triage --> needs_info        : triager requests info
    needs_triage --> needs_discussion  : needs SIG / maintainer call
    needs_triage --> closed            : invalid / duplicate / not-planned

    needs_info --> needs_triage        : author replies (sweep / auto)
    needs_info --> closed              : stale workflow (no reply)

    needs_discussion --> accepted      : SIG decides to proceed
    needs_discussion --> needs_info    : discussion surfaces missing info
    needs_discussion --> closed        : SIG declines

    accepted --> in_progress           : assignee set (auto candidate)
    accepted --> blocked               : external dependency
    accepted --> closed                : re-decided to decline

    in_progress --> accepted           : assignee removed (auto candidate)
    in_progress --> blocked            : blocker discovered
    in_progress --> closed             : linked PR merged (Closes #N)

    blocked --> accepted               : blocker cleared, unassigned
    blocked --> in_progress            : blocker cleared, still assigned
    blocked --> closed                 : re-decided to decline

    closed --> needs_triage            : reopened (auto)
    closed --> [*]
```

### Status label definitions

- `status:needs-triage` - new or reopened, awaiting triager review. The
  existing `triage:deciding` label is treated as an alias and will be
  migrated.
- `status:needs-info` - waiting on the author for reproduction, versions,
  logs, or clarification. Stale workflow closes after the grace period if
  no response.
- `status:needs-discussion` - needs a maintainer or SIG decision. Surfaces
  on the SIG meeting agenda.
- `status:accepted` - triaged, scoped, ready for someone to pick up.
  Eligible for `help wanted` and `good first issue`.
- `status:in-progress` - someone is actively working on it; assignee is
  set.
- `status:blocked` - cannot proceed; blocker is documented in a comment.

### Notes on specific transitions

- **`needs-info` -> `needs-triage` requires a triager (or automation).**
  The author of the issue does not have permission to change labels.
  Today this relies on a triager sweep of the `needs-info` queue; the
  proposed auto-flip workflow (see
  [Automation Candidates](#automation-candidates)) would do it
  automatically when the original author comments.

### Automation candidates

Several transitions in the diagram are mechanical reflections of other
GitHub state (assignee set/unset, PR merge, author comment, reopen). These
are good targets for workflows so that triagers only intervene when
judgment is needed. Tracked in
[Open Items](#open-items-for-implementation):

| Transition                                       | Trigger                                                                   |
|--------------------------------------------------|---------------------------------------------------------------------------|
| (new / reopened) -> `status:needs-triage`        | Already implemented in [`issue_triage.yml`](./.github/workflows/issue_triage.yml). |
| `status:accepted` -> `status:in-progress`        | Workflow on `issues.assigned`: if `status:accepted`, swap labels.         |
| `status:in-progress` -> `status:accepted`        | Workflow on `issues.unassigned`: if no assignees remain, swap labels.     |
| `status:in-progress` -> closed (label cleanup)   | Workflow on `issues.closed`: remove `status:in-progress` (PR-driven closes leave the label behind otherwise). |
| `status:needs-info` -> `status:needs-triage`     | Workflow on `issue_comment.created`: if commenter is the issue author and `status:needs-info` is set, swap labels. |
| Stale `status:needs-info`                        | Handled by the stale workflow per [Stale Policy](#stale-policy).          |

None of these workflows make irreversible changes - they only adjust
labels - so the cost of getting one wrong is low. A triager can always
override.

## Label Taxonomy

Labels are grouped by prefix so they are easy to scan and filter. Only the
prefixes below are considered "triage labels"; other labels (release notes,
component-specific tags, etc.) may coexist.

### `type:*` - what kind of issue this is

Exactly one required.

- `type:bug` - something is broken or behaves contrary to documentation.
- `type:enhancement` - improvement to existing behavior.
- `type:feature` - net-new capability.
- `type:proposal` - design discussion, RFC, or larger change requiring
  consensus before implementation.
- `type:docs` - documentation gap or fix.
- `type:question` - usage question. Most should be redirected to
  [GitHub Discussions](https://github.com/open-telemetry/otel-arrow/discussions)
  or CNCF Slack `#otel-arrow`.
- `type:task` - internal work item (refactor, CI, tooling).

### Area / component labels

Issues should also carry one or more labels identifying which part of the
repo they affect. Today the repository uses a flat set of labels applied
by [`.github/labeler.yml`](./.github/labeler.yml) on PRs (`rust`, `go`,
`ci-repo`, `dependencies`, `pipelineperf`, `query-engine`,
`query-engine-columnar`, `query-engine-recordset`, `query-engine-kql`,
`query-engine-ottl`, `opl-parser`). Triagers should reuse these on issues
when applicable.

Whether to migrate to an `area:*` prefix (and to extend coverage to
`proto/`, `tools/`, `docs/`, etc.) is deferred to a separate decision and
listed in [Open Items](#open-items-for-implementation).

### `priority:*` - urgency

Exactly one for `type:bug`, optional otherwise.

- `priority:p0` - critical; data loss, crash, security, or release blocker.
  Tag `@open-telemetry/arrow-approvers` on creation.
- `priority:p1` - high; significant functional regression or impact to
  production users.
- `priority:p2` - normal.
- `priority:p3` - low; minor, cosmetic, or nice-to-have.

### Contributor-facing labels

- `good first issue` - well-scoped, mentored, suitable for first-time
  contributors. Must include enough detail in the description to start work
  without further clarification.
- `help wanted` - accepted, scoped, and not currently assigned; community
  contributions are explicitly welcome.
- `sponsor needed` - work that requires a maintainer to commit to reviewing
  before it should be started.

### Lifecycle labels

- `stale` - applied automatically by the stale workflow (see
  [Stale Policy](#stale-policy)).
- `pinned` / `keep-open` - exempt from the stale workflow. Use sparingly.
- `security` - applied to issues filed via the security process. Exempt from
  stale.

## Ownership and Assignment

A frequent source of confusion: in this project (and in most OSS projects,
including the OpenTelemetry Collector, Kubernetes, and other CNCF projects),
the GitHub "assignee" field means *"this person is actively working on this
right now"*, not *"this person owns this area of code."* As a result, most
new issues have **no assignee**, and that is intentional.

Ownership is expressed in two places instead:

1. **Code ownership** is captured in [`CODEOWNERS`](./CODEOWNERS). Today
   that file routes all paths to `@open-telemetry/arrow-approvers`;
   expanding it with path-based routing for the major components is
   tracked in [Open Items](#open-items-for-implementation). Even once
   expanded, CODEOWNERS governs PR review routing, not issue routing.
2. **Issue routing** is captured via the component / area labels
   described in [Area / component labels](#area--component-labels).
   Subject-matter experts subscribe to or filter on the labels they care
   about. There is no expectation that a triager will hand-pick an owner
   for every issue.

### When to assign

Assignees are set only on issues in `status:accepted` or
`status:in-progress` - not on `needs-triage`, `needs-info`,
`needs-discussion`, or `blocked`. Cases:

- A contributor has volunteered to work on it. Until the slash-command
  bot is in place, this means the contributor comments and a triager
  sets the assignee (contributors do not have permission to set
  assignees themselves; see [GitHub Permissions](#github-permissions)).
- A maintainer has asked a specific person to drive it and that person has
  agreed.
- It is a `priority:p0` and an approver has accepted the page.

If no one is working on it, leave it unassigned and rely on the `status:`,
component / area, and `help wanted` labels to attract a contributor.
Assigning an issue "just so it has an owner" obscures real progress and
is discouraged.

### Finding work as a contributor

Useful filters:

- All triaged, unassigned work: `is:issue is:open label:status:accepted no:assignee`
- Beginner-friendly: `is:issue is:open label:"good first issue" no:assignee`
- Work in your area: combine the above with the relevant component
  label, e.g. `label:rust` or `label:query-engine`.

## Cadence

Triagers aim for a first response on `status:needs-triage` items within
roughly one business week, and same-day attention for `priority:p0`
issues. These are guidelines for a healthy queue, not commitments - if
either consistently slips, that is a signal the triage rotation needs
more volunteers rather than a reason to feel bad.

## Async-First Triage Workflow

The bulk of triage happens asynchronously on GitHub, not in the SIG meeting.
This is what unblocks meeting time for design work.

1. **Intake (automated).** The
   [`issue_triage.yml`](./.github/workflows/issue_triage.yml) workflow tags
   every new or reopened issue with `triage:deciding` (which the proposed
   label rename will become `status:needs-triage`).
2. **Triager pass (async).** A triager opens the `triage:deciding` /
   `status:needs-triage` query and, for each item:
   - applies type, component / area, and (for bugs) priority labels,
   - asks for missing information if needed and applies `status:needs-info`,
     or
   - moves the issue to `status:accepted`, or
   - closes as `invalid`, `duplicate`, `wontfix`, or `not-planned` with a
     short explanation.
3. **Escalation (async).** Items that need maintainer judgment get
   `status:needs-discussion` and are added to the SIG meeting agenda. The
   triager should pre-summarize the question so the meeting can decide
   quickly.
4. **Follow-through.** Triagers periodically sweep `status:needs-info` and
   `status:in-progress` queues to keep them honest (close stale
   `needs-info`, check in on long-quiet assignees).

Triagers self-organize on how to share the load. A formal weekly rotation
can be introduced later if the queue becomes unmanageable, but is not
required up front.

## SIG Meeting Triage

To keep the weekly SIG meeting focused, live triage is **time-boxed and
agenda-driven**, not a walk of the open issues list.

- Live triage segment: target 15 minutes.
- Agenda: only items labeled `status:needs-discussion`, plus any
  `priority:p0` opened since the last meeting.
- The on-call triager drives the segment and captures decisions back into
  labels and a brief closing comment on each item, so the author can see
  the SIG's reasoning.
- Items not reached are deferred to the next meeting; they do not block
  closure of the segment.
- If `status:needs-discussion` is empty, skip the segment.

Anything that can be decided by the on-call triager async should be decided
async, not in the meeting.

## Stale Policy

The current stale workflow lives at
[`.github/workflows/stale.yml`](./.github/workflows/stale.yml) and runs
daily. It covers both issues and pull requests, but only the issue
behavior is in scope for this document:

- Issues: marked `stale` after 180 days of inactivity, closed 30 days
  later.
- The only exempt label today is `do-not-stale`.

Proposed target policy for issues (tracked in
[Open Items](#open-items-for-implementation)):

- Tighten the issue cycle to 60 days to stale + 14 days to close.
- Add a shorter 14 + 14 cycle for `status:needs-info` so the queue does
  not accumulate questions the author never returned to.
- Expand the exempt-label list to: `pinned`, `keep-open`, `security`,
  `priority:p0`, `priority:p1`, `status:accepted`, `status:in-progress`,
  `help wanted`, `good first issue`, and the existing `do-not-stale`.

Anyone may reopen a closed-stale issue with new information; reopening
resets the stale timer.

## Security Issues

Security vulnerabilities **must not** be filed as public issues. Follow
[SECURITY.md](./SECURITY.md) to report privately. Security-labeled issues
that are filed publicly will be redirected to the private process and
locked.

## Becoming a Triager

Help out: suggest labels in comments, reproduce bugs, point duplicates at
the canonical issue, and ask clarifying questions. After about a month of
consistent participation, ask a maintainer to nominate you per the
[community membership guide](https://github.com/open-telemetry/community/blob/main/guides/contributor/membership.md#triager).
Triagers are added to [CONTRIBUTING.md](./CONTRIBUTING.md#triagers) and
granted the GitHub "Triage" role on the repository.

## Open Items for Implementation

This document describes the target process. The following follow-up issues
should be filed to fully realize it (sequencing left to the SIG):

- Introduce the `status:*`, `type:*`, and `priority:*` label set and
  rename `triage:deciding` to `status:needs-triage`. Decide separately
  whether to migrate the existing flat component labels (`rust`, `go`,
  `ci-repo`, `query-engine*`, `opl-parser`, `pipelineperf`,
  `dependencies`) to an `area:*` prefix and extend coverage to other
  top-level directories (`proto/`, `tools/`, `docs/`).
- Update [`.github/workflows/issue_triage.yml`](./.github/workflows/issue_triage.yml)
  and the issue templates (`bug_report.yaml`, `feature_request.yaml`,
  `task.yaml`, `other.yaml`) to apply the new labels.
- Tighten [`.github/workflows/stale.yml`](./.github/workflows/stale.yml) to
  the target cadence (60 + 14 for issues, 14 + 14 for
  `status:needs-info`) and expand the exempt-label list as described in
  [Stale Policy](#stale-policy).
- Extend [`.github/labeler.yml`](./.github/labeler.yml) to cover any new
  paths added during the component-label decision.
- Expand [`CODEOWNERS`](./CODEOWNERS) with path-based routing for the
  major components (currently a single catch-all entry).
- Add slash-command handlers (`/assign`, `/label`, `/needs-info`,
  `/accepted`, etc.) so contributors and triagers can drive the
  lifecycle without manual label edits.
- Implement the status-label automation workflows listed in
  [Automation Candidates](#automation-candidates):
  `accepted` <-> `in-progress` driven by assignee changes,
  `needs-info` -> `needs-triage` driven by author comments, and label
  cleanup on PR-driven close.
- Add a GitHub Project board with columns mirroring the `status:*` labels
  to give triagers a visual queue.
- Add a weekly triage-report workflow that posts queue health metrics to
  a pinned issue or the SIG meeting notes.
