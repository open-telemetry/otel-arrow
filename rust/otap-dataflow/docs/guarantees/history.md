# Regression cases

| Case | Operator impact and why the earlier check missed it | Related components and lasting evidence |
| --- | --- | --- |
| [Receiver context declarations][4814] | A receiver addition asked the operator to restart the engine. Controller tests asserted the internal rejection. | Context-declaring nodes and the controller. The [two-pipeline scenario][live] checks delivered records, generations and committed state after rejection. |
| [Fan-out under load][2225] | Reaching the in-flight cap could refuse a record that ingress would never retry. A capacity bound alone did not establish delivery behavior. | Fan-out, routers and ingress. See `no_nacks_at_max_inflight` and `max_inflight_pauses_pdata_when_limit_reached` in the [fan-out tests][fanout]. |
| [One saturated router output][4536] | A busy destination could refuse data while other routes stayed idle. Testing saturation of every route missed the asymmetric workload. | Both [content][content] and [signal][signal] routers test `test_backpressure_pauses_on_any_blocked_route_and_resumes_after_recovery`. |
| [Structured log bodies on the wire][4218] | Arrays and maps could disappear through the wire path. In-memory representation tests did not cross Arrow IPC and protobuf serialization. | Log views, codecs and protocol consumers. Keep `test_composite_bodies_survive_otap_wire_round_trip` in the [log view tests][logs]. |

The pilot also found a documentation mismatch. Attribute action examples used a
nested string object that the component rejected. The updated scalar examples
match the [processor configuration][attribute]. The live test changes the
value and checks it on the record exported to the new destination.

## Receiver rollout verification

The same [operator scenario][live] on Linux rejected the first add with HTTP
422 and a full-engine restart instruction at both #4814 (`db57e180`) and
main (`137d2cfe`). It completed on [the proposed correction][fix]
(`bd94cf45`) in two consecutive binary runs. With the processor and exporter
checks included, two runs of both image scenarios took 3.30 and 3.36
seconds on the local Linux host, excluding the build.
The test checks that the changed address receives records, old listeners close,
the independent pipeline keeps its generation, and both pipelines continue
serving after rejection. Merge the correction, or an equivalent verified fix,
before enabling this required scenario on main.

## Backfill

The entries above seed the history. For each reviewed release, record its
tag and remaining cases in the RFC tracking issue. Prioritize repeated
failures across components and observable data loss. Where automation is
impractical, link the reviewer question and its owner.

[4814]: https://github.com/open-telemetry/otel-arrow/pull/4814#issuecomment-7050737097
[2225]: https://github.com/open-telemetry/otel-arrow/pull/2225
[4536]: https://github.com/open-telemetry/otel-arrow/pull/4536
[4218]: https://github.com/open-telemetry/otel-arrow/pull/4218
[live]: ../../scripts/guarantees/live.py
[fanout]: ../../crates/core-nodes/src/processors/fanout_processor/mod.rs
[content]: ../../crates/core-nodes/src/processors/content_router/mod.rs
[signal]: ../../crates/core-nodes/src/processors/signal_type_router/mod.rs
[logs]: ../../crates/pdata/src/views/otap/logs.rs
[fix]: https://github.com/open-telemetry/otel-arrow/pull/4860
[attribute]: ../../crates/core-nodes/src/processors/attributes_processor/README.md
