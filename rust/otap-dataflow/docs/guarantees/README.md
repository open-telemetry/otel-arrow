# Reviewing operator guarantees

Read the [AI-assisted review guide][review] and [design principles][design]
before assessing the full diff and its callers. The configuration model and
observable component behavior are user contracts. APIs can evolve through
the ordinary project review process. The [change rules][rules] find likely
areas of impact. Reviewers can add an area that automation missed.

## Reading the PR check

The job checks the release inputs, resolver, live controller and core retry
on the candidate. An absent, skipped, failed or flaky observation cannot pass
as evidence. The summary groups areas outside the diff rules on one line.
The [live operation table][live] identifies which components have direct
evidence. Blank spots in automation remain visible as inherited gaps.

When the report flags a path, add a small JSON record under
`.github/guarantee-reviews/` with a new descriptive filename. Include only
changed paths and relevant evidence.
The record travels with the code through the merge queue. Keep detailed
reasoning in component docs, benchmark results or an RFC and link to it.
Multiple affected paths can share a single assessment.

```json
{
  "evidence": [
    {
      "area": "behavior",
      "paths": ["rust/otap-dataflow/crates/core-nodes/src/processors/retry_processor/mod.rs"],
      "status": "tested",
      "source": "retry-recovery",
      "reason": "A transient refusal reaches the retry path and the recovered exporter receives the admitted records."
    }
  ]
}
```

- **Tested** records cite a passing run from this candidate. Built-in names
  are `released-inputs`, `release-precedence`, `live-rollout`, and
  `retry-recovery`. For a different Rust case, use the JUnit class and
  full test name as `junit:CLASS/TEST`.
- **Reviewed with rationale** records give a concrete argument and a source
  (repository path, optionally with a fragment, or an HTTPS evidence link).
  For example, identify which runtime owns a buffer, what bounds it, and what
  happens when it fills. The contributor supplies evidence. The native
  code-owner review decides whether that reasoning is sufficient.
- **Approved exception** records cite a decision already present on the base
  branch in `exceptions/`. Include the exact area as a bullet under
  `Scope`, plus `Justification`, `Operator procedure`, `Migration and release`,
  `Verification`, and `Decision` sections. For component exceptions also list
  the exact URN and operation on one bullet. Link the maintainer decision and
  the tested migration. An exception proposed by the same change is pending.
- **Evidence missing** blocks for a new or changed risk. Document an inherited
  gap with an owner and follow-up and supply evidence for the behavior the
  change actually touches. When a compatibility break is necessary, identify
  affected users, the replacement, timing and migration in the operator docs
  and existing breaking-change changelog. Changes to protected expectations, exceptions
  and fixtures go through maintainer review.

## From guidance to evidence

| Review guide section | What to inspect | Existing evidence or reviewer responsibility |
| --- | --- | --- |
| Agent Reviewer Quick Checklist and Agent Reviewer Instructions | Full diff, callers, test changes and remaining risk | Report specific findings, affected scenarios, evidence and residual risks. Run the async blocking script on runtime changes. |
| Review Posture | Ownership boundaries, complexity, new mechanisms | Explain the trade-off, reuse opportunities and a concrete alternative for material concerns. |
| Core Architectural Invariants | Thread-per-core locality, bounded work, backpressure and external library assumptions | Look for blocking or shared runtime paths. Link actual capacity, cancellation and admission evidence. |
| Runtime and Performance | Ingest, encode, dispatch, storage and dependency changes | Use Criterion or a base/candidate workload. Explain differences in CPU, throughput, memory or latency. |
| Design and Serviceability | Config, live topology, no-op/rejection, drain, continuity, APIs and diagnostics | Validate released inputs and effective settings. Run an operator scenario and verify actual delivery. |
| Metric Instrumentation | Units, instrument kind, aggregation, dimensions and secrets | Apply the existing metric and item attribute guides and linked semconv checks. |
| Correctness, Security, and Portability | Input fidelity, refusal, unsafe code, trust boundaries, crypto features, platform behavior and test reliability | Inspect negative tests, inventory drift, dependency and platform jobs. Record the threat, mitigation and remaining risk for new boundaries. |

Follow the design guide for predictability, easy defaults, scalability,
allocation, synchronization, context switches, extensibility, programmability,
observability, interoperability, reliability, security and privacy. Explain
specific departures with a maintainer-reviewed decision. Automated triggers
help locate evidence and the full-diff review covers interactions across paths.

## Operator scenarios and performance

Run from `rust/otap-dataflow` with a binary built with `syslog-cef` and `file`.
The live test uses two available CPU cores, unprivileged loopback sockets,
unique capture files and bounded readiness polling. CI supplies `--image
df_engine` to the same scripts, reusing the pipeline performance job's image
on PRs and in the merge queue.

```sh
python3 scripts/guarantees/released.py target/debug/df_engine
python3 scripts/guarantees/live.py target/debug/df_engine
cargo test -p otel-arrow-dfe-config --test release_guarantees
cargo test -p otel-arrow-dfe-validation --features integration-tests --test core_node_liveness_tests
```

The YAML fixture is a byte-for-byte copy of the [v0.60.0 syslog example][tag].
The JSON fixture encodes the same released input. Negative and inheritance
cases derive from that fixed input. Keep the released data intact when adding
new releases. To retire an accepted input at the approved release boundary,
add an entry to `release-migrations.json` in this guide's directory. Key it
by the old release-relative filename (for example, `v0.60.0/example.yaml`).
Set `replacement` to an accepted fixture, `diagnostic` to the expected
rejection text, and `decision` to a protected decision already on the base.
Its scope names `configuration` and `configuration v0.60.0/example.yaml`.
CI exercises the old input and the proposed replacement on the same binary.

The live test sends uniquely identified records to the primary
and observer pipelines and checks the committed configuration after rejection.

For each added URN, include a `benchmarks` entry in the same JSON record.
Use its full URN as the key, `measurement` for the evidence link, and
either `workload` for the repository-relative continuous suite path or
`obstacle` for the reason and reproduction steps.

For new components, include a representative pipeline in the existing
[continuous benchmark][continuous] where feasible. Identify the actual
workload invoked by CI and its tracked signals. If an external service or
platform prevents continuous coverage, document that obstacle, the alternative
measurement and how to reproduce it. Use the existing `cargobench` and
`pipelineperf` PR labels for focused runs. Record the same input, batch size,
build features and hardware for a base/candidate comparison.

## Regression history and the first month

For a bug fix or near miss, record the broken operator workflow, why coverage
missed it, sibling components at risk and the lasting check. Link an existing
test when it already establishes the property. Start with the [incident
history][history]. The Arrow maintainers own backfill from release bug-fix
entries, reverts, closed issues and recurring review threads. Work backwards
by release and link each entry to a case, a prior equivalent case, or a
reasoned review question.

At one month from **enablement**, the RFC's tracking issue should report
caught regressions, escapes, flaky failures, added execution time after the
build, and the fraction of add/change/remove operations with scenario links.
Each run retains its `operator-guarantees` JSON artifact for 60 days.
Use the candidate, seed outcomes, affected areas, coverage and blocking
findings together with the Actions step durations. Count unique incidents
and classify flakes and misses using reviewer follow-up.
Record which checks to promote or revise and why.

[review]: ../ai/ai-assisted-pr-review.md
[design]: ../design-principles.md
[rules]: review-map.json
[live]: live-support.md
[history]: history.md
[tag]: https://github.com/open-telemetry/otel-arrow/blob/rust/otap-dataflow/v0.60.0/rust/otap-dataflow/configs/syslog-console.yaml
[continuous]: ../../../../.github/workflows/pipeline-perf-test-continuous.yml
