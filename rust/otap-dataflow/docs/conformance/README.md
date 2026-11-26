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

### Embedded dashboard

Run the dashboard deployment alone, or omit `--check embedded-ui` to run the
full suite. Both paths use the same Rust probe and [YAML signal catalog][ui-signals].

```sh
cargo xtask conformance --check embedded-ui --html target/dashboard.html
```

The probe loads the dashboard at `/` and `/dashboard`, follows its local
asset references and module imports, and checks their content types. It compares
the API paths referenced by those modules with the endpoints it exercises.

The fixture needs two available engine cores. A marked syslog record crosses a
topic and reaches the file exporter. The probe checks the following observations.

| Dashboard surface | Evidence from the running engine |
| --- | --- |
| Channel charts and DAG | Both metrics routes expose scoped senders and receivers for data and control channels. Each pair joins on a channel ID. Sent and received counts increase during the workload. Idle gauges and error counters can be zero. |
| Charts and tooltips | Each applicable instrument has the expected unit, instrument kind and temporality, and a nonempty brief in the live schema and both metrics responses. The catalog identifies platform-specific instruments. |
| All-core selector | Two reader instances supply distinct core IDs. Removing the core, thread and NUMA dimensions leaves one series. Worker counts sum to two and the mean of the two 32-slot channel capacities is 32. |
| Status overlay | During creation, a pipeline with no observed runtime reports 0/0 cores and both conditions are unknown. A held receiver port then produces a runtime error, 0/1 running cores and a failed ready quorum. The healthy and shutdown event checks run on separate pipelines. |
| Live logs | A slow WebSocket subscriber falls behind while the fixture generates local audit events. The error supplies the last delivered sequence in `lag_before_seq`. The probe backfills from it, verifies contiguous pages against the retained HTTP logs and recovers the missed events. |

The topic topology uses an empty string when a process ID is absent, matching
the UI's single-process behavior. The log viewer renders arbitrary event names.
The probe checks the fields on real entries and observes any explicitly named
events in the stream or retained snapshot.

Terminal, HTML and JSON reports list each UI consumer and signal. The
[signal evaluator][evaluator] checks schema declarations, usable samples and
scoped traffic increases. The checker uses the same catalog to reject incomplete
observations from the deployment runner.

When a dashboard change adds, renames, or removes a telemetry dependency,
update `telemetry-signals.yaml` in the same change. Reviewers check that the
list matches what the UI reads. Under `sets`, each metric maps a catalog key
to its wire name and metadata. Use `dimensions` for
scope, `series` for required label values, and `traffic: true` when the marked
workload must increment the series. Use `channel` to identify endpoint roles and
`allCores` for the per-core and aggregate values. Under `events`, list status
kinds, conditions and expected pipeline states, the lag cursor, log fields,
and any explicitly named logs. Give a newly required event a
reproducible trigger in the deployment if the existing workload does not emit it.

[ui-signals]: telemetry-signals.yaml
[evaluator]: ../../crates/conformance/model/src/ui.rs
