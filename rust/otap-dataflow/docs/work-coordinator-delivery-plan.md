# WorkCoordinator delivery plan

Status: proposed implementation plan for the
[WorkCoordinator RFC](../rfcs/0000-work-coordinator.md). The RFC defines runtime
contracts; this document tracks sequencing, adoption gates, estimates, and PR
workflow. Update its RFC link when a maintainer assigns the final number.

## Delivery principle

Ship useful coordination before the most advanced reliability capabilities.
Prometheus and early Filelog versions can adopt Basic, then Cooperative, without
waiting for durable ownership, multi-file conditional transactions, or fenced
takeover. Advanced Filelog requirements remain explicit commitments, with their
authority/progress extension points defined before the initial API is frozen.

Use three delivery milestones and four implementation PRs. Review and accept the
RFC separately; the implementation stack starts from `main` after that decision.
The final milestone spans two PRs so durable authority and source qualification
can be reviewed independently. Each earlier PR is usable without the later ones.

| Available after | Prometheus adoption | Filelog adoption | Remaining limits |
| --- | --- | --- | --- |
| PR 1: Basic | Distribute static/discovered targets over a fixed per-core fleet using the in-memory backend. | Host discovery outside the reader and retain a single active reader/checkpoint owner per namespace. | No coordinated resize, rolling replacement, forced takeover, or automatic per-core recovery. |
| PR 2: Cooperative | Use acknowledged resize and compatible live replacement. | Add conservative live handoff; multicore reading requires a qualified serialized local writer and source-state policy. Otherwise keep singleton support. | Failure of an owner to quiesce blocks transfer; durable grants are not required. |
| PR 3: Durable authority | Existing Basic/Cooperative integration continues unchanged. | Durable backend and conditional progress primitives are available for integration. | Backend availability alone does not qualify an advanced receiver. |
| PR 4: Advanced profile | Existing integration remains supported without a durable store. | Qualify fenced progress, supported completion topologies, migration, and state-aware handoff; enable only the modes whose tests pass. | No automatic claim of exactly-once delivery, immediate kernel-I/O cancellation, or cross-process ownership. |

Profiles describe allowed guarantees, not just test fixtures. Do not leave all
production consumers disabled until PR 4. A node implementation declares the
profile it can actually support; missing capabilities fail admission rather than
silently selecting a weaker mode. Diagnostics and receiver documentation expose
the active profile and its live-update/recovery limits.

The current checkout contains Filelog primitives without a registered receiver
and no Prometheus scrape receiver. Complete receiver implementations remain
separate work. Their adapters target the earliest ready coordinator profile:
include the thin adapter in the corresponding PR when its receiver exists, or
land it with that receiver's own implementation. A runnable conformance node
proves framework readiness meanwhile; it is not evidence that a complete
production receiver has shipped. Production advanced Filelog qualification
depends on its real reader, checkpoint, and delivery path being available.

## Milestone 1: Usable Basic coordination

PR 1 delivers the node-facing contracts and a working in-process assignment
implementation, rather than a hosting-only scaffold.

Scope:

- Side-effect-free node declarations for receivers, processors, and exporters;
  stable coordinator keys; fresh participant sessions; typed participant binding.
- Bounded catalogs, `ParticipantRequest`/`CoordinatorNotice` messages,
  `ParticipantRegistry` snapshots, and a simple sticky allocator over initial
  participants in `ReadyForAssignments`.
- In-memory backend activation/retirement, offer/accept/activate, cooperative
  catalog removal, admission tickets, and exact-session acknowledgements.
- Coordinator readiness before node preparation, `ReadyForAssignments` without
  ownership, initial activation barriers, joined shutdown, and namespace cleanup.
- Reserved control capacity, explicit item/byte limits, authority invalidation,
  and fail-closed behavior if a node or coordinator fails.
- Capability requirements and the backend/progress interfaces, including
  operation IDs and ambiguous-outcome rules. No durable implementation is
  required or constructed for a Basic consumer.
- Basic receiver examples and diagnostics; actual thin adapters when the
  receiver runtimes are available.

Basic uses a fixed participant set. Reject coordinated topology/settings
replacement and resize before mutating a participating pipeline. On an unexpected
participant exit, stop dependent admission and shut down the logical pipeline;
do not let the existing per-core recovery path launch an unsafe replacement.
Initial creation, no-op, normal shutdown, deletion, and catalog changes within
the current fleet remain supported.

Acceptance:

- A Prometheus-style node actually executes independently identifiable targets
  on several pipeline instances, without any progress-store dependency.
- A Filelog-style node uses shared discovery and one excluded namespace owner
  with Ack-gated checkpoints. Its source-pipeline one-core guard may remain;
  adopting the coordinator does not require removing that guard.
- A processor or exporter can bind through the same public contracts.
- Tests cover empty catalogs, duplicate initialization, stale sessions, failed
  activation, catalog churn, full queues, missing capabilities, shutdown, and
  rejection of unsupported live operations before side effects.
- The API and examples are available to production node implementations with
  these declared limits; durable-progress work is not an adoption gate.

## Milestone 2: Cooperative live reconciliation

PR 2 extends the usable Basic system with acknowledged lifecycle transitions.
It does not make persistence a prerequisite for Prometheus.

Scope:

- Prepare/activate/commit/abort receipts, revisioned `ParticipantRegistry` updates
  and assignment resynchronization, and staged compatible coordinator settings.
- Source-aware departure barriers that preserve downstream completion paths
  during resize, replacement, rollback, and shutdown.
- Per-core recovery only after proven cleanup; fresh terms on every reassignment
  and no resurrection of old grants or already retired settings.
- Explicit commit-decision handling so lost receipts and panic cleanup cannot
  roll back a committed generation.
- Cooperative Filelog integration with the existing exclusive namespace. If
  enabling several readers, provide a qualified `SerializedLocal` writer that
  serializes authority checks, checkpoint commits, and ownership retirement.
  Independent per-core writers to the old store do not satisfy this contract.
- Profile-specific validation and metrics, with Prometheus target execution and
  available Filelog adapters running through the real lifecycle paths.

Acceptance:

- Prometheus can resize and apply compatible live updates before either advanced
  PR lands, with no durable grant store and no ownership/readiness deadlock.
- Early Filelog can preserve its supported local progress contract during
  cooperative handoff. Unresolved deltas, partial records, and non-reopenable
  rotated files either meet the source policy or block the move explicitly.
- Filelog with no qualified multicore writer remains usable as a singleton.
  A restricted profile's rejected update is an expected limit, not permission
  to discard data or bypass a namespace lock.
- Tests cover old/new overlap on a core, remove/re-add under one generation,
  mid-rollout failure, rollback, late completion, recovery preemption, and
  saturated queues while release and shutdown still make progress.

## Milestone 3: Durable authority and advanced Filelog

Keep the most complex mechanisms in the last two PRs. Basic and Cooperative
users remain supported throughout; neither must adopt the stronger backend.

### PR 3: Durable authority and progress backend

- Implement a qualified local durable authority backend and its progress-writer
  handle. Keep source progress opaque to generic engine code and use a suitable
  leaf storage crate to avoid engine dependencies on Filelog reader types.
- Review the storage format, transaction boundary, recovery behavior, and failure
  model before enabling the backend. Reuse existing storage where it can satisfy
  the contract; separate grant and checkpoint journals with no common atomic
  boundary are insufficient.
- Persist effective grants before activation; fence retired authority before
  successors run; condition every authoritative mutation on the current grant
  and expected checkpoint revision.
- Support bounded all-or-none transactions across the files/assignments covered
  by one supported publication, and idempotent resolution of ambiguous commits.
- Admit `AllowFencedTakeover` only with both a qualifying backend and a source
  policy that can reject obsolete execution/publication and recover needed state.
  Retain `CooperativeOnly` as the default for sources needing physical exclusion.
- Keep per-publication feedback/progress traffic outside the coordinator event
  loop and apply independent queue/item/byte bounds to the writer path.

Acceptance: test a real persistent backend across reopen/crash boundaries,
interrupted activation, retirement and multi-file commits, stale writes, unknown
outcomes, and process-incarnation changes. Verify that an invalid or unresolved
grant never becomes permission to activate another owner. An in-memory fake
alone cannot qualify this profile. This PR does not yet claim advanced Filelog
receiver conformance.

### PR 4: Advanced source profile and qualification

- Bind the advanced Filelog adapter to stable file identities, its existing
  fixed-virtual-partition target, and a fenced progress backend. Preserve
  file-centric keys across placement, service-key changes, and namespace migration.
- Define and test exclusive upgrade/migration from early checkpoint ownership;
  preserve committed progress and reject mixed old/new writers or ambiguous state.
- Validate supported end-to-end completion topologies, pending-outcome bounds,
  contiguous progress, stale-attempt rejection, and drain preservation. Reuse the
  current Ack/Nack system; do not add a coordinator-mediated feedback loop.
- Specify partial decoder/framer state, retained publications, and unlinked
  rotated-file behavior during cooperative or optionally fenced handoff.
  Durable checkpoint fencing alone does not preserve unavailable source bytes.
- Run discovery from real Filelog primitives and execution through the intended
  source/delivery path. Qualify the production profile when the real receiver is
  available; record that dependency explicitly if it is not.
- Exercise the full capability matrix and the non-receiver participant. Finish
  bounded read-only diagnostics, contributor guidance, and resource/responsiveness
  measurements under large catalogs, churn, and downstream backpressure.

Acceptance: publish the exact qualified Filelog modes and their delivery/recovery
limits, and test failure/restart/migration against their normative specifications.
Enable advanced modes only after that evidence exists. The framework and early
receiver integrations stay usable if a particular advanced mode remains blocked
or requires more source-specific work. Full receiver implementation, destructive
administration, and cross-process coordination are not hidden prerequisites for
Basic or Cooperative adoption.

## Estimate and dependencies

These are preliminary added-Rust estimates including meaningful test fixtures,
not a target, schedule, or commitment. Re-estimate after PR 1 exposes the actual
factory and lifecycle changes.

| PR | Production Rust | Tests and fixtures |
| --- | --- | --- |
| 1: Basic coordination | 1,500-2,500 lines | 1,500-2,500 lines |
| 2: Cooperative reconciliation | 2,000-3,500 lines | 2,500-4,500 lines |
| 3: Durable authority/progress | 2,000-4,000 lines | 2,500-5,000 lines |
| 4: Advanced source qualification | 1,000-3,000 lines | 2,000-4,000 lines |
| Total | 6,500-13,000 lines | 8,500-16,000 lines |

Plan approximately 15,000-30,000 added Rust lines for the expanded scope. The
earlier 8,000-15,000 estimate excluded durable authority and advanced source
qualification; it is not the estimate for this larger series. Estimates exclude
complete Prometheus/Filelog receivers, mechanical registration changes,
documentation, destructive migration tools, and cross-process storage. Backend
reuse and the state of the receiver implementations are the largest uncertainties.
Tests and validation accompany each PR; they are not deferred to the top layer.

Track these dependencies explicitly:

- Basic/Cooperative adapters can land as soon as the corresponding public API is
  ready. Complete scraping, metric translation, reading, and decoding are source
  implementation work, not an advanced coordination dependency.
- A local checkpoint adapter is required before enabling cooperative multicore
  Filelog. Singleton adoption requires only its qualified exclusive writer mode.
- A real durable backend is required before fenced progress; a qualified source
  policy is additionally required before forced takeover.
- Advanced Filelog needs its completion topology, migration format, and source
  handoff contract. Update the receiver's normative documents with any changed
  behavior; the generic RFC cannot silently override them.

## GitHub Stacks

### Review boundaries

Review the RFC as an ordinary PR from `lquerel/work-coordinator/00-rfc` to `main`.
After acceptance, create the following four-PR implementation stack in
`open-telemetry/otel-arrow`:

| Layer | Branch | Initial PR base | Milestone |
| --- | --- | --- | --- |
| 1 | `lquerel/work-coordinator/01-basic` | `main` | 1: usable Basic coordination |
| 2 | `lquerel/work-coordinator/02-reconciliation` | `lquerel/work-coordinator/01-basic` | 2: cooperative live reconciliation |
| 3 | `lquerel/work-coordinator/03-durable-authority` | `lquerel/work-coordinator/02-reconciliation` | 3: durable authority/progress |
| 4 | `lquerel/work-coordinator/04-filelog-profile` | `lquerel/work-coordinator/03-durable-authority` | 3: advanced source qualification |

The RFC review is separate from these four implementation PRs. Every branch tip
must be a coherent, testable increment. Do not submit empty placeholders, put
lower-layer fixes only in later branches, or make earlier integrations depend
on code that exists only above them. The Basic and Cooperative adoption gates
must be observable at their own branch tips.

Each PR body records its profile/delivery milestone, dependency and RFC links,
behavior, supported limitations, validation, and remaining source integration
dependencies. Populate real PR URLs in the tracking issue as they are created.
Branch names here are planning identifiers, not existing PR numbers.

### Repository and branch names

Native [GitHub Stacks][github-stacks] require all heads to be in the same
repository. To have the four PRs reviewed upstream as a native stack, push all
four branches to `open-telemetry/otel-arrow` with the appropriate repository
access. A stack entirely in the personal fork is not an upstream native stack.
If upstream hosting is unavailable, agree a maintainer-hosted workflow or use
ordinary dependency-linked PRs and label that fallback explicitly.

Use the exact lowercase `lquerel/` prefix. The repository's live
[EasyCLA ruleset][easycla-rule] includes the personal exclusions
`refs/heads/lquerel/**/*` and `refs/heads/lquerel-*`; the planned names match the
former. This is repository configuration, not a universal Stacks naming rule.
Git ref names are [case-sensitive][git-case]: `LQuerel/` is a different namespace.
The stack still follows its trunk's required reviews and checks.

The current checkout points `origin` at the personal fork. Before publication,
verify an `upstream` remote points to `open-telemetry/otel-arrow`, use it explicitly,
and check push access. Writing this plan does not create or rename local/remote
branches, install tools, change remotes, or publish PRs.

### Working with the stack

Use the current [GitHub quickstart][github-stack-quickstart] prerequisites and
`github/gh-stack` extension, or the native web UI. The examples below assume an
authenticated environment, a clean current `main` after RFC acceptance, and an
explicit `upstream` remote. Commit each layer before adding its successor.

```sh
gh extension install github/gh-stack
gh stack init --base main lquerel/work-coordinator/01-basic
# Implement, validate, and commit Basic coordination.
gh stack submit --remote upstream

gh stack add lquerel/work-coordinator/02-reconciliation
# Implement, validate, and commit cooperative reconciliation.
gh stack submit --remote upstream

gh stack add lquerel/work-coordinator/03-durable-authority
# Implement, validate, and commit the durable authority/progress backend.
gh stack submit --remote upstream

gh stack add lquerel/work-coordinator/04-filelog-profile
# Implement, validate, and commit advanced qualification.
gh stack submit --remote upstream
gh stack view --short
```

Review titles/bodies during submission and keep incomplete layers in draft.
Confirm native stack membership on GitHub; parent-base selection alone is not
proof the stack object exists. Check the [CLI reference][github-stack-cli] before
using non-interactive submission, including its draft behavior.

For feedback, commit changes on the owning branch, then run
`gh stack rebase --upstack` and `gh stack push --remote upstream`. Re-run checks
on affected descendants. Use the [maintenance workflow][github-stack-manage]
for conflict resolution, preserving lease checks on rewritten branches.
After a lower PR merges, use `gh stack sync --remote upstream` and inspect the
remaining bases/diffs. Merge from the lowest accepted layer upward; inspect the
[selected merge range][github-stack-merge] because merging a higher PR can also
land its lower dependencies. RFC acceptance and checks remain separate gates.

## Validation per PR

Follow [CONTRIBUTING](../CONTRIBUTING.md#reliable-tests). Tests use explicit
readiness/transition barriers, controlled clocks, isolated resources, and checked
worker joins. Immediately document every test with specific `Scenario:` and
`Guarantees:` comments. Include failure paths and the promised limitations of the
profile introduced by that PR.

Run an affected-crate `cargo check` after Rust changes, the relevant focused
tests, and `cargo xtask check` from `rust/otap-dataflow` before each implementation
PR is ready. Quick checks are iteration aids. Run Markdown lint and repository
sanity checks for changed documentation. Full validation failures caused by
environment prerequisites, such as missing submodules or disk capacity, must be
reported rather than represented as passing.

Apply changelog rules to each PR's actual observable behavior. API changes alone
are not a user-facing breaking change. Documentation-only changes are exempt;
internal-only implementation PRs include `chore` in their titles. Copy the
appropriate `.chloggen/TEMPLATE.yaml` for user-facing receiver behavior changes.

[github-stacks]: https://docs.github.com/en/pull-requests/get-started/about-stacked-prs
[github-stack-quickstart]: https://docs.github.com/en/pull-requests/get-started/stacked-prs-quickstart
[github-stack-cli]: https://docs.github.com/en/pull-requests/reference/stacked-prs-cli-commands
[github-stack-manage]: https://docs.github.com/en/pull-requests/how-tos/create-pull-requests/managing-stacked-pull-requests
[github-stack-merge]: https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/merging-stacked-pull-requests
[easycla-rule]: https://github.com/open-telemetry/otel-arrow/rules/10644352
[git-case]: https://learn.microsoft.com/en-us/azure/devops/repos/git/case-sensitivity?view=azure-devops
