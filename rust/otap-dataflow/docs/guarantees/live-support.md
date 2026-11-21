# Live reconfiguration support

Each operation applies to a running engine. A rolling replacement of the affected
pipeline qualifies. In-place updates have a separate column. A scenario link
identifies the check to run against the candidate. Current main has a known
receiver-add restriction under [discussion][correction].

An **untested** cell is an inherited evidence gap. The backlog below gives its
owner and follow-up. An **exception** links to a reviewed decision with an
operator procedure and reason. A **manual** cell links the steps and expected
result for a specific environment. A dash in the in-place column means that
this index has no recorded in-place promise. The component catalog and its
owners should be consulted when introducing or finding such a promise.

| Registered component | Add | Change | Remove | In-place |
| --- | --- | --- | --- | --- |
| `urn:microsoft:exporter:azure_data_explorer` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:microsoft:exporter:azure_monitor` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:microsoft:exporter:geneva` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:microsoft:processor:recordset_kql` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:microsoft:receiver:json` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:clickhouse` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:console` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:error` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:file` | [scenario][syslog] | [scenario][syslog] | [scenario][syslog] | - |
| `urn:otel:exporter:geneva_metrics` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:kafka` | [untested][backlog] | [untested][backlog] | [untested][backlog] | [scenario][kafka] |
| `urn:otel:exporter:noop` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:otap` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:otlp_grpc` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:otlp_http` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:parquet` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:perf` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:exporter:topic` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:attribute` | [scenario][syslog] | [scenario][syslog] | [scenario][syslog] | - |
| `urn:otel:processor:batch` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:condense_attributes` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:content_router` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:debug` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:delay` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:durable_buffer` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:fanout` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:filter` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:log_parser` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:log_sampling` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:partition` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:resource_validator` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:retry` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:temporal_reaggregation` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:transform` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:type_router` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:processor:wasm_processor` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:etw` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:host_metrics` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:internal_telemetry` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:journald` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:kafka` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:oracle` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:otap` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:otlp` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:postgresql` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:prometheus` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:sql_server` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:syslog_cef` | [scenario][syslog] | [scenario][syslog] | [scenario][syslog] | - |
| `urn:otel:receiver:topic` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:traffic_generator` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:user_events` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |
| `urn:otel:receiver:windows_perf_counters` | [untested][backlog] | [untested][backlog] | [untested][backlog] | - |

## Manual verification for an untested operation

Start an engine with two regular pipelines. Use the component README to
configure any required broker, credentials, platform or listening address.
Use a uniquely identifiable input and a capture sink on each pipeline.
From the target pipeline's committed config, prepare three candidate files
named `add.json`, `change.json` and `remove.json`. Each file contains
`{"pipeline": ...}`. Keep the observer pipeline outside every candidate.

- For a receiver, add a second input wired to the capture sink. Send a record
  through it, change its source or listening address, then remove it. Verify
  the new source delivers and the removed source stops.
- For a processor, insert it between the input and sink. Exercise a record
  whose result distinguishes the chosen settings. Change one setting and
  observe the new result. Remove the processor and verify the original path.
- For an exporter, add a reachable second destination, change the destination
  or a verifiable output setting, then remove the exporter. Check the actual
  receiving sinks and the remaining original route after removal.

For each candidate, run the following with the engine's admin URL as `BASE`
and the group and pipeline names as `GROUP` and `PIPE`. Verify the terminal
rollout state, committed configuration and sample delivery before submitting
the next candidate. Send a distinct observer probe during each rollout and
record its generation before and after. Finally, submit a candidate with a
missing connection target and verify both pipelines still deliver with the
previous committed configuration. Record build features, platform, commands,
observed values and a reference before promoting a cell to manual coverage.

```sh
curl --fail-with-body -X PUT -H 'Content-Type: application/json' \
  --data-binary "@$CANDIDATE" \
  "$BASE/groups/$GROUP/pipelines/$PIPE?wait=true&timeout_secs=30"
curl --fail-with-body "$BASE/groups/$GROUP/pipelines/$PIPE"
```

## Coverage backlog

The Arrow maintainers own the initial inventory backfill with the owners in
each component's metadata. Verify high-traffic defaults first (OTLP, OTAP,
batch, retry and their exporters), followed by stateful or service-dependent
nodes. Use the same add, change, remove and rejection assertions as the syslog
scenario. For platform or external service tests, record setup, input, observed
delivery and cleanup. Update the relevant cell with the evidence. A PR that
changes one of these behaviors supplies evidence for its change.

The Kafka exporter has node-harness tests for switching topics and retaining
the running configuration after invalid input. The README documents ordering
and drain considerations for operators. An in-place scenario covers only the
specific behaviors it asserts.

[backlog]: #coverage-backlog
[syslog]: ../../scripts/guarantees/live.py
[kafka]: ../../crates/contrib-nodes/src/exporters/kafka_exporter/exporter.rs
[correction]: https://github.com/open-telemetry/otel-arrow/pull/4860
