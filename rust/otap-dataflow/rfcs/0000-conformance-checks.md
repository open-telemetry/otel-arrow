---
Proposal Name: conformance-checks
Start Date: 2026-11-20
RFC PR: https://github.com/open-telemetry/otel-arrow/pull/4937
Tracking Issue: after acceptance
---

# RFC 0000 - Conformance checks in pull request review

## Summary

Give authors and reviewers a local command to identify the engine guarantees
a change affects and run repeatable deployment scenarios. Record
human reasoning for properties that require judgment, including exceptions.
The configuration model and observable core component behavior are the main
compatibility contracts. Public API stability is outside this proposal.

## Motivation

[The Kafka context migration][incident] exposed a controller restriction.
Adding a receiver declaration could require an engine restart even though
local tests explicitly expected the controller to reject the update. An
operator exercising a supported topology change would notice immediately.
The [review guide][guide] and [design principles][principles] already describe
this serviceability requirement and the architecture it depends on.

## Guide-level explanation

For each affected guarantee the reviewer sees a test result, a review rationale,
a previously approved exception, or a request for missing evidence. Routine
changes use existing deployment observations. Authors and reviewers read the
full diff to identify additional risks. Owners record pre-existing gaps with a
follow-up.
Changes introducing or weakening behavior require evidence before merge.

Every registered receiver, processor and exporter should support adding,
changing and removing it in a running engine. The affected pipeline may roll.
Other pipelines must continue serving. In-place promises are additional.
Document each exception, including limits on in-flight work and the procedure
an operator can follow. Mark unverified operations explicitly.

## Reference-level explanation

From the Rust workspace, run `cargo xtask conformance` on the working tree.
The command builds the conformance binary and candidate engine with Cargo.
The binary uses the validation library to deploy pipelines and exercise local
fixtures. Add `--base <ref>` to review the change. Authors and reviewers can
supply their rationale with `--assessment <file>` and share the resulting
report. Keep lasting exceptions, operator procedures and release migrations
in the repository.

Only deployed behavior earns tested credit. Inventory and released
configuration validation are preflight checks. Unit tests in ordinary CI
provide supporting context for reviewers.

Use the source inventory for coverage across targets and Cargo features.
Use the linked pipeline factories and controller extension registry to identify
which validators ran in this build. Record operation-specific evidence for
receivers, processors, exporters and pipeline extensions. Controller extensions
have their own startup lifecycle. Keep inherited gaps visible with a follow-up.

- Validate release-pinned YAML through the candidate engine and report the
  outcome by component. Check generated JSON and the rejection of an invalid graph.
- Exercise real add, change, remove, rejection and independent pipeline
  delivery through the controller. Track retry, backpressure, and wire fidelity
  as deployment gaps with an owner and follow-up.
- Drive marked traffic through the embedded dashboard deployment. Match the
  [YAML signal catalog](../docs/conformance/telemetry-signals.yaml) to live
  metric declarations, metric samples and observable lifecycle or log events.
  Report declaration and emission separately.
- Review the complete architectural checklist, including bounded queues,
  locality, cancellation, protocol fidelity, telemetry, privacy and platform
  constraints. Explain departures and refer them to maintainers.
- Compare performance on the same workload when a hot path changes.
  Encourage Criterion microbenchmarks. Include new components in a continuous
  pipeline workload where feasible and document the alternative measurement.
- Compare the component inventory to operator documentation. Record
  reproducible evidence, confirmed restrictions and inherited gaps separately.
- Require a maintainer decision before weakening a guarantee. Explain who is
  affected, why the break is necessary, the tested migration and release timing.
  Update the operator guidance and breaking-change changelog.
- Turn past incidents and near misses into a test or a specific review question.
  Search similar components and record why the previous checks missed the case.

The command prints a local summary and can export HTML and JSON reports.
Native review rules retain approval authority. Aim for less than one minute
of execution after the Rust build.

## Drawbacks

Evidence takes time to maintain. Overbroad change detection can create rote
answers. Missing scenarios can leave an apparently healthy component untested.
Show the exact scope and provenance of each result and review missed cases.

## Rationale and alternatives

Line coverage can increase while a system-level guarantee regresses. Schema
diffs identify structural changes and need behavioral fixtures for defaults.
Running all service integrations on each PR would consume dedicated capacity.
The fast suite can surface a change while a targeted benchmark or a manual
review establishes its consequences.

## Prior art

The project compares a compiled [component inventory][inventory] to a
reviewed baseline and already runs [layered tests][testing] and continuous
performance workloads. This proposal builds on those facilities.

## Unresolved questions

The SIG should confirm the initial operator promises and the threshold for
accepting a permanent component exception during review of this proposal.

## Future possibilities

After a month of local use, examine escaped and caught regressions, flaky
failures, incremental execution time and automated coverage by operation.
Promote manual checks that repeatedly catch issues and revise noisy triggers.

[incident]: https://github.com/open-telemetry/otel-arrow/pull/4814#issuecomment-7050737097
[guide]: ../docs/ai/ai-assisted-pr-review.md
[principles]: ../docs/design-principles.md
[inventory]: ../docs/component-inventory.md
[testing]: ../docs/testing-guide.md
