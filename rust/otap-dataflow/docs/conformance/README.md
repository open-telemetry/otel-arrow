# Engine conformance

From `rust/otap-dataflow`, run the candidate on your working tree:

```sh
cargo xtask conformance --open
cargo xtask conformance --component urn:otel:receiver:syslog_cef
cargo xtask conformance --base origin/main --metrics target/live.json --html target/live.html
```

## How the checker works

The validation library starts a native engine. The conformance driver discovers
registered factories through linkme and selects the same YAML rollout fixture
for a full run or a single URN. Each fixture changes a running pipeline and
checks its effect on traffic. An independent observer must keep delivering
through its original generation. The suite also checks identical updates,
rejected configurations and rollback after an accepted replacement fails to start.

The report names failed operations first. A failure stops later stages of that
fixture. Unlinked components, missing fixtures and unmet prerequisites remain
visible. Controller extensions get startup configuration preflight. The
[check catalog](checks.yaml) describes each observation. The generated
[support table](live-support.md) lists known gaps and their required environment.

Use `--base REF` to identify new nodes without rollout evidence and lost
coverage. Documented inherited gaps remain visible. The command writes requested
reports before exiting unsuccessfully for a regression. Local runs require the
normal Rust build prerequisites. Python and Docker are unnecessary.

### Checking one component

Use the full registered URN to check its factory's rejection of invalid
configuration and its add, change, and remove fixture. The observer pipeline
and delivery checks run at each stage. Focused JSON reports include
`behavior_checks` and `deployment_gaps` with an owner and follow-up.

```sh
cargo xtask conformance --component urn:otel:processor:transform
cargo xtask conformance --component urn:otel:exporter:otlp_http --metrics target/otlp-http.json
```

A controller extension receives startup configuration validation. For a
pipeline component, a missing fixture, missing prerequisite or unlinked
factory makes the focused check exit unsuccessfully and explains the gap.
Unknown URNs are rejected against the source inventory. Use the full suite
to review a change against a base revision.

Each catalog fixture also reapplies the changed configuration with the subject
installed. The admin operation must complete as a no-op, retain the committed
generation and pass the normal delivery probe. A focused report records
`reconfiguration-noop` when the fixture reaches this step. A failed add leaves
the no-op in `unexercised_checks` and later operations unexercised.
Independent behavior checks still run.

### Add a live fixture for a component

Follow the [fixture authoring guide][authoring] to configure a component,
prepare its input files, and choose an observable result for each rollout.
It includes a complete processor fixture and follows the host-metrics receiver
from its checked-in assets to the two running pipelines. The generated
[support table](live-support.md) shows which components have a fixture and
what each remaining gap needs.

The component fixture checks the behavior specified by its probe. Engine-wide
scenarios also check that the committed generation keeps serving after an
invalid graph or an attempt to change startup-owned settings, such as the
observability pipeline. Controller extensions receive startup checks.

[authoring]: ../../crates/conformance/fixtures/README.md
