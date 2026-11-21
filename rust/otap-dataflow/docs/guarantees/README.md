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
  are `released-inputs`, `release-precedence`, `live-rollout`,
  `extension-rollout`, `rejected-update`, `factory-validation`, and
  `retry-recovery`. To cite one observed component operation, use
  `rollout:URN/OPERATION`, such as `rollout:urn:otel:processor:transform/change`.
  For other Rust cases,
  use the JUnit class and full test name as `junit:CLASS/TEST`.
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

From the Rust workspace, run the fast checks on the current working tree.
The default uses Cargo, a native engine, temporary files and loopback sockets.
Add a Git base to include the evidence review for committed, staged, unstaged
and untracked changes. Use the immediate parent branch when reviewing a stack.

```sh
cargo xtask guarantees
cargo xtask guarantees --base origin/main
```

To include additional JUnit evidence on a clean checkout, pass
`--junit <directory>` containing the candidate's downloaded Linux JUnit
artifacts. The report checks the commit in each artifact's metadata.

The source inventory covers every platform and feature. The validation runner
uses the linked factories to exercise configuration validators. Per-operation
coverage comes from scenarios that observe the running engine. Linux CI runs
the native engine and fixture runner built alongside the nextest archive. It
has the OS libraries used by the journal and TLS probes. Failed, missing,
skipped and retried cases give no passing evidence. Review the summary even
when one scenario fails.

### Adding a live fixture

Add one entry to the validation crate's [fixture catalog][fixtures]. The
runner matches its URN against the production factory maps and applies one
lifecycle to each linked component. Each fixture gets an isolated engine
with a target and an observer pipeline. The runner adds the subject, changes
its configuration, and removes it. Each step checks the committed topology,
the observer's generation and delivery, and the subject's data-plane effect.
The rejected-update regression checks that the committed generation keeps serving
after an invalid graph and after startup-owned changes, including a request
to install an internal telemetry receiver in the observability pipeline.

For example, the transform processor reuses the same marked-log capture as
the attribute processor. The catalog entry supplies its query and assertion.

```yaml
- urn: urn:otel:processor:transform
  probe:
    kind: log-attribute
    key: guarantee.phase
  config:
    opl_query: 'logs | set attributes["guarantee.phase"] = "{{ phase }}"'
```

For a processor that should pass a marked log, use `probe: {kind: pipeline,
role: processor}`. Its default graph is `input -> subject -> out`.
Use `peers` and `connections` to supply a producer for a receiver, a consumer
for an exporter, or both halves of a protocol bridge. The peer and subject
use ordinary DFE node configurations. Declare required local services, such as
`services: [kafka]`, on the fixture. Templates expand only in string values.
Use `{{ phase }}` for the add or change value, `{{ address }}` for a reserved
loopback address, and `{{ dir }}` for the session directory. Use `change_config`
when the second configuration changes fields or typed values.
For the configuration-free noop exporter, the change enables its node-level
message telemetry policy.

| Probe | What an add or change must demonstrate |
| --- | --- |
| `pipeline` | The declared graph delivers the marked record, transformed body, named metric or phase-specific signal. Sinks can require a downstream OTLP acknowledgement or refusal, a Parquet row, an authenticated upload or a ClickHouse ArrowStream insert. |
| `syslog-tcp` | The listener delivers a marked record and its retired address refuses connections. |
| `log-attribute` | A fresh record carries the configured attribute value. After removal it arrives without that attribute. |
| `jsonl-file` | Each marked log appears at its active destination. The original destination resumes after removal. |
| `otlp-http-header` | The capture receives the configured test header on the marked request and observes its absence after removal. |
| `otlp-authorization` | The receiver rejects an unauthorized token, delivers a log for the allowed audience and accepts an unauthenticated request after unbinding. |

The fixtures use an in-process Kafka broker, loopback HTTP and TLS peers,
disposable credentials, and temporary files. Azure and Google token services,
Kubernetes TokenReview, and both Geneva endpoints run locally. Linux fixtures
also exercise a synthetic procfs, a two-record journal and a private CA trust
file. The journald receiver needs `libsystemd.so.0` in the engine runtime.
The journal fixture contains two records with identifiers `add` and `change`,
bodies `fixture-add-record` and `fixture-change-record`, and synthetic machine
and boot IDs. Inspect it by decompressing `sample.journal.gz` into a temporary
file and running `journalctl --file=sample.journal -o export`.
The WASM processor loads a checked-in pass-through component compiled from WAT.
On Linux, `cargo xtask guarantees` selects the Linux cases automatically.

The host-metrics fixture changes between two synthetic host roots. Overlapping
generations that claim the same root can conflict. The topic fixtures bind
two topics declared at startup with `impl_selection: force_mixed`. A change
to the broker's selected implementation requires a restart. The console
fixture uses pretty output. Structured stdout must be claimed at startup.
A successful rollout through a processor establishes the delivery behavior
named by its probe. Algorithm-specific edge cases belong in its owning crate.

Run `cargo xtask guarantees --write-support` after editing the
[catalog][fixtures] or the [prerequisite records][prerequisites]. The [support
table][live] links each inherited gap to its required environment and
acceptance criterion. A failed operation stops the remaining stages of that
fixture. Other fixtures continue. Controller extensions keep their separate
startup checks.

For a provisioned environment, pass one `--with NAME` per prerequisite declared
by its fixtures. When combining CI environments, pass their reports as repeated
`--observations FILE` options with `--junit DIR` on the clean candidate.
A new or changed fixture needs passing evidence from a build that exercises it.

### Released configurations

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
[fixtures]: ../../crates/validation/examples/guarantee_scenarios/fixtures.yaml
[prerequisites]: ../../crates/validation/examples/guarantee_scenarios/prerequisites.yaml
[live]: live-support.md
[history]: history.md
[tag]: https://github.com/open-telemetry/otel-arrow/blob/rust/otap-dataflow/v0.60.0/rust/otap-dataflow/configs/syslog-console.yaml
[continuous]: ../../../../.github/workflows/pipeline-perf-test-continuous.yml
