# Regression cases

| Case | Operator impact and why the earlier check missed it | Related components and lasting evidence |
| --- | --- | --- |
| [Receiver context declarations][4814] | A receiver addition asked the operator to restart the engine. Controller tests asserted the internal rejection. | Context-declaring nodes and the controller. The [component rollout][live] checks delivery and observer continuity. The [rejection scenario][rejection] checks the committed state. |
| [Fan-out under load][2225] | Reaching the in-flight cap could refuse a record that ingress would never retry. A capacity bound alone did not establish delivery behavior. | Fan-out, routers and ingress. See `no_nacks_at_max_inflight` and `max_inflight_pauses_pdata_when_limit_reached` in the [fan-out tests][fanout]. |
| [One saturated router output][4536] | A busy destination could refuse data while other routes stayed idle. Testing saturation of every route missed the asymmetric workload. | Both [content][content] and [signal][signal] routers test `test_backpressure_pauses_on_any_blocked_route_and_resumes_after_recovery`. |
| [Structured log bodies on the wire][4218] | Arrays and maps could disappear through the wire path. In-memory representation tests did not cross Arrow IPC and protobuf serialization. | Log views, codecs and protocol consumers. Keep `test_composite_bodies_survive_otap_wire_round_trip` in the [log view tests][logs]. |

The pilot also found a documentation mismatch. Attribute action examples used a
nested string object that the component rejected. The updated scalar examples
match the [processor configuration][attribute]. The live test changes the
value and checks it on the record exported to the new destination.

## Receiver rollout verification

The Rust [operator scenario][live] rejected the receiver addition with HTTP
422 at #4814 (`db57e180`) and the local stack's base (`137d2cfe`). The
diagnostic asked for an engine restart. The same runner, fixtures and build
features passed all phases against [the correction][fix] (`bd94cf45`).
The extension scenario passed on all three engines. It observed a synthetic
API key change and removal at a loopback OTLP HTTP destination. The corrected
engine also passed both scenarios through the CI image runner.

The expanded YAML catalog reproduced the same HTTP 422 when adding the
syslog and traffic-generator receivers and the batch, partition, and temporal
reaggregation processors. The remaining 44 components completed all three
operations on the stack base. The corrected engine completed all 147
operations with the same 49 fixtures on Linux.

On the stack base, the no-argument Rust command took 17.89 seconds after
the build on a Linux host with Python and Docker absent from the command
search path. Five seed checks passed and the receiver rollout failed.
The local runner exercised 37 linked factory validators. Coverage records
included 63 source components across platforms and build features.

The correction merged in #4860. The live rollout remains a required scenario.

### Kafka receiver regression in #4814

The merge candidate (`d1716f43`) completed 42 of 49 rollout fixtures. Its
first parent (`4046dd6a`) completed 44. Both additional failures involved
Kafka receivers. The exporter fixture also adds a Kafka receiver to read
back the exported records. The PR head (`db57e180`)
produced the same rollout results as the merge candidate.

Separate admin API requests isolated the receiver. For removal, the engine
started with the receiver already configured. After each request, the probe
read back the committed configuration.

| Live operation | First parent | PR head and merge candidate |
| --- | --- | --- |
| Add a Kafka receiver | HTTP 200, receiver added | HTTP 422, receiver remained absent |
| Remove a Kafka receiver | HTTP 200, receiver removed | HTTP 422, receiver remained configured |
| Change the receiver's `client_id` | HTTP 200, new value committed | HTTP 200, new value committed |
| Add or remove a static-topic Kafka exporter alone | Both succeeded | Both succeeded |

The receiver declares arrival context in this PR. The controller requested
an engine restart when the set of context declarations changed. Updating
`client_id` preserved that set and succeeded through pipeline replacement.

## Controller extension measurements

The monitor and OpAMP controller extensions already ship. Their inventory
annotations add static metadata. As a local startup reference, the candidate
debug engine validated `configs/opamp-controller-extension.yaml` in 30.4 ms
and the same pipelines with only the documented controller monitor extension
in 30.1 ms. These are medians of ten warm process launches on Linux using
`df_engine --config <file> --validate-and-exit`. Runtime sampling and OpAMP
network activity require separate workloads.

## Backfill

The entries above seed the history. For each reviewed release, record its
tag and remaining cases in the RFC tracking issue. Prioritize repeated
failures across components and observable data loss. Where automation is
impractical, link the reviewer question and its owner.

[4814]: https://github.com/open-telemetry/otel-arrow/pull/4814#issuecomment-7050737097
[2225]: https://github.com/open-telemetry/otel-arrow/pull/2225
[4536]: https://github.com/open-telemetry/otel-arrow/pull/4536
[4218]: https://github.com/open-telemetry/otel-arrow/pull/4218
[live]: ../../crates/conformance/src/live.rs
[rejection]: ../../crates/conformance/src/reconfiguration.rs
[fanout]: ../../crates/core-nodes/src/processors/fanout_processor/mod.rs
[content]: ../../crates/core-nodes/src/processors/content_router/mod.rs
[signal]: ../../crates/core-nodes/src/processors/signal_type_router/mod.rs
[logs]: ../../crates/pdata/src/views/otap/logs.rs
[fix]: https://github.com/open-telemetry/otel-arrow/pull/4860
[attribute]: ../../crates/core-nodes/src/processors/attributes_processor/README.md
