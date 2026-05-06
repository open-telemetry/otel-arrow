window.BENCHMARK_DATA = {
  "lastUpdate": 1778083190147,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "eaa4103326057ef68125244171801bc010cb3571",
          "message": "Add mock LA server for local Azure Monitor Exporter testing (#2381)\n\nAdds a lightweight mock Azure Monitor Logs Ingestion API server and a\nlocal test config for performance testing the Azure Monitor Exporter\nwithout incurring Log Analytics costs.\n\n## New files\n\n- `crates/mock-la-server/` - Standalone Axum-based mock server that\naccepts gzip-compressed JSON POSTs, decompresses/counts log entries,\nprints periodic throughput stats, and supports error simulation flags\n(--fail-rate, --retry-after, --latency, --unauthorized-rate,\n--payload-too-large, --fail-after)\n- `fakegen-ame-local.yaml` - Pipeline config pointing the exporter at\nhttp://localhost:9999 with auth.method: dev\n\n## Usage\n\n```\n# Terminal 1: start mock server\ncargo run -p mock-la-server -- --port 9999\n\n# Terminal 2: run pipeline\ncargo run --features azure-monitor-exporter -- --config crates/contrib-nodes/src/exporters/azure_monitor_exporter/fakegen-ame-local.yaml --num-cores 1\n```",
          "timestamp": "2026-03-20T16:26:07Z",
          "tree_id": "5b29432284b38da718e93799fc9a0c0aaafc489c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaa4103326057ef68125244171801bc010cb3571"
        },
        "date": 1774032805036,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.313641905784607,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.97573030883983,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38515764195523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.554947916666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.59375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648266.7838350086,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656782.6877241696,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002556,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17040822.5351836,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17059981.69911303,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "192b3ba8d8d496790ff1fc63db52c58e51b81fd3",
          "message": "Fix flakey validation tests and improve runtime (#2345)\n\n# Change Summary\n\nFix validation tests, added a message received timeout in the validation\nexporter that will trigger the validation checks and report a finished\nsignal via the telemetry metrics, only runs the validation checks once\nvs every message received.\n\n## What issue does this PR close?\n\n* Closes #2184 and #2227\n\n## How are these changes tested?\n\nAdded unit tests and confirmed that validation tests consistently pass\nin cicd\n\n## Are there any user-facing changes?\n\nyes, changed expected_within() parameter type from duration to u64",
          "timestamp": "2026-03-20T20:03:55Z",
          "tree_id": "b40aab6ef6aabfdc799fcbbe435dd92a1414afac",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/192b3ba8d8d496790ff1fc63db52c58e51b81fd3"
        },
        "date": 1774044250940,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8642315864562988,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.97843005328228,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27773608215583,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.116015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.1953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 649677.3657655058,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 655292.0827450781,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002312,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16981791.908534262,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17001681.185060453,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "607e963919bf0e635484fb392ec35052a753b167",
          "message": "fix: Add Missing experimental-tls feature inheritance for core-nodes (#2391)\n\n# Change Summary\n\nAddress a regression introduced in the move of receivers to `core-nodes`\ncrate:\n- #2339\n- #2360\n\nBefore these PRs, the `experimental-tls` feature in the core\n`otap-dataflow` `Cargo.toml` correctly propagated to the `otap` crate.\n\nAfter these PRs, building `df_engine` with the feature\n`experimental-tls` did NOT connect to the `experimental-tls` feature\ninside the `core-nodes` crate.\n\nAs a result, configuring a receiver with `tls` settings resulted in a\nruntime config error:\n> Error: Custom { kind: Other, error: \"Invalid config for component\n`urn:otel:receiver:syslog_cef` in pipeline_group=byoc-syslog-pipeline\npipeline=main node=syslog-receiver: An invalid user configuration\noccurred: unknown field `tls`, expected `listening_addr`\" }\n\nbecause this field is gated behind the feature:\n```rust\n#[cfg(feature = \"experimental-tls\")]\ntls: Option<TlsServerConfig>,\n```\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nTested manual pipeline build of `df_engine` with `experimental-tls` and\na sample configuration to confirm `tls` config field is found and\nparsed.\n\n## Are there any user-facing changes?\n\nYes, can once again use TLS on Syslog, OTAP, and OTLP receivers.",
          "timestamp": "2026-03-20T20:15:24Z",
          "tree_id": "03a5b60ba736dc65e9da88f239de6578f6caf6e1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/607e963919bf0e635484fb392ec35052a753b167"
        },
        "date": 1774047395968,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.081695795059204,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.01262258459123,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.19304873324553,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.751822916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.54296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 645282.793153378,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 652262.7900431649,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002319,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17001743.510394134,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17020517.506334685,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c08121131bbc06a0c1712be05d5511d2d5b3e492",
          "message": "Columnar query engine `extend`/`set` operator support attiributes as destination (#2379)\n\nApologies to reviewers: I'm sorry this is long. I'm happy to break it up\nif desired.\n\n# Change Summary\n\nAdds the ability to set attribute values using the columnar query\nengine.\n\nNote that we already had this capability when the source was a static\nliteral (e.g. `logs | extend attributes[\"x\"] = \"hello\"`. This PR\nimproves the capability so the attribute can be assigned from the\nresults of the expression evaluation that was added in\nhttps://github.com/open-telemetry/otel-arrow/pull/2126\n\nIt means that now we can do things like:\n```kql\nlogs | set attributes[\"event_name\"] = event_name // use root field as source\nlogs | set attributes[\"x\"] = resource.attributes[\"y\"] // use some other attribute as the sourc\nlogs | set attributes[\"x\"] = attributes[\"y\"] * 2 // use arithmetic as the source\n// fyi ^^^ \"set\" is an alias of \"extend\" in OPL\n// etc.\n\n```\n\nNote: this does create Empty attributes if the expression evalutes to\n`null`. E.g. for something like this `logs | set attributes[\"x\"] =\nattributes[\"y\"]`, if `attributes[\"y\"]` did not exist for some row, then\nan empty attribute would be created for `attributes[\"x\"]`.\n\nThis also fixes a few bugs in the current set attributes implementation:\n- corrects the semantics of `set`/`extend` to be an \"upsert\" - e.g.\nreplace the attribute value or create a new attribute if one did not\nexist. Before this PR, we did not replace existing values.\n- fixes issue where attributes would not be inserted if the attribute\nrecord batch did not previously exist\n\nThe core of changes introduced is a optimized kernel for upserting\nattributes. Currently this lives is located\nat`otap_df_query_engine::pipeline::assign:attributes::upsert_attributes`.\nThis expects the caller to pass the attribute key, the new values, the\nparent_id associated with the attribute and a mask of which rows should\nbe updated. It then uses this to quickly merge the new values/attribute\ntypes, and append any inserts onto each column, inserting nulls where\nappropriate and maintaining correct dictionary encoding semantics. For\nbest performance, this can upsert multiple attribute keys at once.\n\nBecause the `upsert_attributes` kernel can assign multiple attributes at\nonce, the query-engine's planner code and the `AssignPipelineStage` have\nboth been modified to accomodate this. The planner attempts to coalesce\nmultiple \"set\" transformations into a single pipeline stage if possible,\nand the `AssignPipelineStage` handles evaluating the expression for each\nsource, and driving the invocation of the `upsert_attributes` kernel to\ndo all the attribute upserts in bulk.\n\n| Benchmark | 128 rows | 1536 rows | 8192 rows |\n|---|---|---|---|\n| `upsert_new_str_key` | 4.54 µs | 15.37 µs | 67.04 µs |\n| `upsert_existing_str_key` | 7.10 µs | 49.16 µs | 239.45 µs |\n| `upsert_two_new_str_keys` | 6.06 µs | 20.81 µs | 90.04 µs |\n| `upsert_two_existing_str_keys` | 9.50 µs | 58.26 µs | 271.99 µs |\n| `upsert_two_existing_one_new` | 11.41 µs | 65.20 µs | 300.78 µs |\n\nIn all cases, we see this is a lot faster than the current attribute\nupsert. (see here\nhttps://github.com/open-telemetry/otel-arrow/pull/2024). Although it's\nnot an apples-to-apples comparison, both these acheive the same result\non the same order-of-magnitude of log data, but this code is much\nfaster.\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2036 \n* Closes #2016\n\n## How are these changes tested?\n\nUnit tests - many new tests added\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nYes - in transform processors users will now have the capability to\nassign the value of attributes using a variety of new expressions for\nsources (not just static literals).\n\n## Future work\n\nIn a future PR I'll integrate the new `upsert_attributes` kernel into\nthe attribute processor to improve performance and also fix #2350",
          "timestamp": "2026-03-20T20:16:16Z",
          "tree_id": "947213420236787bbc00d1af35f9f503d6fad321",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c08121131bbc06a0c1712be05d5511d2d5b3e492"
        },
        "date": 1774048352129,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9577435255050659,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.08537494599882,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3802610316967,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.239453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.4140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648621.8960662172,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 654834.0302073555,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00128,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17034242.56679903,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17054717.382286463,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d389678b03da242781069e748a409d90ffddf610",
          "message": "fix: Temporarily disable the nightly otap-filter-otap Go collector scenario (#2396)\n\n# Change Summary\n\nThis scenario has been blocking all the nightly benchmarks for a few\nweeks now and we can't fix it until this is released and we take a\nversion bump:\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/pull/46879\n\nIt looks like it will be another couple of weeks for the next otel\ncollector contrib release as the last one was just a few days ago. I'm\nproposing to disable the scenario for now to unblock everything else.",
          "timestamp": "2026-03-21T01:34:17Z",
          "tree_id": "09bfae5af1f957c1a6958cd82687925c2d84af5b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d389678b03da242781069e748a409d90ffddf610"
        },
        "date": 1774065957446,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1851832866668701,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.0711581347592,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40858135763379,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.873046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.8203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 642936.8708090952,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 650556.8507105183,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002258,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17051952.84715796,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17071887.568125997,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "213113461+gyanranjanpanda@users.noreply.github.com",
            "name": "Google Antigravity",
            "username": "gyanranjanpanda"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "993e4a845f4a50d3cdbdbe074f40517bbe1741ee",
          "message": "feat: implement OtapMetricsView for zero-copy OTAP metrics traversal (#2367)\n\n## Summary\n\nImplement zero-copy OTAP Arrow-backed views for metrics data, following\nthe same pattern as OtapLogsView. This enables direct traversal of\nmetrics Arrow RecordBatches without intermediate conversion to protobuf\nor Prost types.\n\n## New file: views/otap/metrics.rs \n\nComplete metrics hierarchy:\n- OtapMetricsView → ResourceMetrics → ScopeMetrics → MetricView →\nDataView\n- Gauge/Sum/Histogram/ExpHistogram/Summary views\n- NumberDataPoint, HistogramDataPoint, ExpHistogramDataPoint,\nSummaryDataPoint views\n- ExemplarView, BucketsView, ValueAtQuantileView\n\n## Modified files (visibility only)\n- MetricsArrays/QuantileArrays/PositiveNegativeArrayAccess fields →\npub(crate)\n- Shared helpers in logs.rs → pub(crate) for reuse\n- views/otap.rs: added mod metrics + re-export\n\n## Design\n- Pre-computed BTreeMap indexes at construction (same as OtapLogsView)\n- Reuses RowGroup, OtapAttributeView, OtapAnyValueView from logs module\n- Introduces Otap32AttributeIter for u32-keyed dp/exemplar attributes\n\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-22T14:45:01Z",
          "tree_id": "029db550ffaeeee291f9c87484c01c2149db447d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993e4a845f4a50d3cdbdbe074f40517bbe1741ee"
        },
        "date": 1774193573716,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7755516767501831,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.03905353923163,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34407123499807,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.103385416666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.95703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 646947.3110723404,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 651964.7216650904,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002265,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16987774.64521598,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17005761.102514837,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7c80ef943e417c5671b6dcb028cccc8d89935525",
          "message": "fix(deps): update module github.com/klauspost/compress to v1.18.5 - abandoned (#2399)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.4` → `v1.18.5` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.5?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.4/v1.18.5?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.5`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.5)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.4...v1.18.5)\n\n#### What's Changed\n\n- zstd: Fix crash when changing encoder dictionary with same ID by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1135](https://redirect.github.com/klauspost/compress/pull/1135)\n- zstd: Default to full zero frames by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1134](https://redirect.github.com/klauspost/compress/pull/1134)\n- flate: Clean up histogram order by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1133](https://redirect.github.com/klauspost/compress/pull/1133)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.4...v1.18.5>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-03-23T14:17:00Z",
          "tree_id": "6df47761bcee16f4857eca76d6055c50abc1de27",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7c80ef943e417c5671b6dcb028cccc8d89935525"
        },
        "date": 1774278915857,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8138933181762695,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.04437828153763,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30967582587218,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.725390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 642680.3234742129,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 647911.0554243661,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00231,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17036321.880721588,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17056259.00938027,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a88939a5ba8b2a740f54e23fd207c32577b6b6dc",
          "message": "fix(deps): Fixes for latest Renovate config (#2413)\n\n# Change Summary\n\n1. Renovate grouping isn't working quite correctly:\n    * #2402\n    * #2403\n    * #2404\n  \nSince these are coming as git refs and not from `crates.io`, I think we\nhave to use the [cargo\nmanager](https://docs.renovatebot.com/modules/manager/cargo/) instead of\n[crate\ndataSource](https://docs.renovatebot.com/modules/datasource/crate/).\n\n2. pip_requirements manager is still trying to update indirect\ndependencies from `requirements.lock.txt` files:\n    * #2401 \n\nLooking at [Renovate job\nlogs](https://developer.mend.io/github/open-telemetry/otel-arrow) - the\nproblem is that while the `pip_compile` correctly skips indirect deps,\n`pip_requirements` was still active on lock files.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-23T17:54:50Z",
          "tree_id": "2abbe7f2720581651950d5819705e059846103e3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a88939a5ba8b2a740f54e23fd207c32577b6b6dc"
        },
        "date": 1774294430896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0484286546707153,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.0069369841174,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.22111471360198,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.376041666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.6640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 645411.04636081,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 652177.7208688435,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002295,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16979168.60903478,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16997995.133612934,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e8e67bdc23ddbeba34016a0218a1c83f892b6338",
          "message": "fix: Move batch processor to 100klrps scenario and add otap-batch-otap (#2395)\n\n# Change Summary\n\nThis PR moves the continuous batch processor benchmarks to the 1ooklrps\nscenario and adds an otap-batch-otap configuration.\n\nI think the batch processor benchmarks were mistakenly added to the\n\"passthrough\" scenario which states it's for scenarios with no processor\nin the middle. The dashboard also does not seem to be set up properly\nfor these and we want to add otap-batch-otap as mentioned here:\nhttps://github.com/open-telemetry/otel-arrow/pull/2246#issuecomment-4033755985\n\n- Closes #2277\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-23T20:20:54Z",
          "tree_id": "0e45ee80a5bc10b26def73e8582e04bb4585ea9a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8e67bdc23ddbeba34016a0218a1c83f892b6338"
        },
        "date": 1774300207029,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.220940113067627,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.11960394832786,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.39226765799256,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.869661458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.09375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648569.5776384632,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656488.2241848141,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002173,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17067017.44905925,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17088211.03484651,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "geukhanuslu@gmail.com",
            "name": "Gokhan Uslu",
            "username": "gouslu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "84c4259bea0753ed03b05d19eba9c45a5a97d075",
          "message": "azure monitor exporter: gzip batcher compression ratio configurable + bench + default to 6 + fix surfaced bugs on edge cases (#2388)\n\n# Change Summary\n\n**NOTE:** In local end-to-end tests with a basic pipeline setup, higher\ncompression levels produced modestly higher throughput at the cost of\nmore CPU, because each batch carries more log records and fewer HTTP\nrequests are needed for the same data volume. This indicates the test\nsetup was I/O bound. Making the compression level configurable allows\ntuning the exporter based on whether the workload of a given pipeline\nconfiguration is I/O bound (favor higher levels) or CPU bound (favor\nlower levels).\n\n- Make gzip batcher compression level configurable via\n`gzip_compression_level` config field (0-9).\n- Add gzip batcher benchmarks across compression levels 1, 6, 9 and\nanalysis document.\n- Set default compression level to 6 (matching `flate2::GzEncoder`\ndefault).\n- Replace fixed `GZIP_SAFETY_MARGIN` with `TARGET_LIMIT` (1020 KiB) that\nleaves 4 KiB headroom below the 1 MiB API limit.\n- Fix `is_first_entry` to use `row_count == 0` instead of\n`uncompressed_size == 0`, which was incorrect after sync flushes.\n- Fix size accounting: structural JSON bytes (`[`, `,`, `]`) are now\nincluded in size tracking.\n- Remove unused `total_uncompressed_size` field.\n- Add utilization tests enforcing that waste stays under one entry's\nworth of `TARGET_LIMIT` across entry sizes (1B-64KB), data profiles\n(json_log/hex_json), and compression levels (1/6/9).\n- Add edge case tests: flush boundary comma validity, hard limit\nenforcement, cross-batch (spillover) JSON validity.\n- Add deterministic replay test (`test_replay_seed_89`) demonstrating\ngzip framing overhead with incompressible data near the limit boundary.\n- Refactor config tests to use `test_api_config()` helper to avoid churn\nwhen new fields are added.\n- Log `gzip_compression_level` at exporter startup.\n\n## Batch Utilization Results (TARGET_LIMIT = 1020 KiB)\n\n### Batch Sizes\n\n| Profile | Entry Size | Level 1 | Level 6 | Level 9 |\n| ---------- | ---------- | -------------------- | --------------------\n| -------------------- |\n| tiny_json | 1 B | 1,044,495 (100.00%) | 1,044,491 (100.00%) |\n1,044,492 (100.00%) |\n| hex_json | 10 B | 1,044,483 (100.00%) | 1,044,488 (100.00%) |\n1,044,482 (100.00%) |\n| json_log | 256 B | 1,044,394 (99.99%) | 1,044,283 (99.98%) | 1,044,378\n(99.99%) |\n| hex_json | 256 B | 1,044,282 (99.98%) | 1,044,378 (99.99%) | 1,044,293\n(99.98%) |\n| json_log | 1 KB | 1,043,561 (99.91%) | 1,043,712 (99.93%) | 1,043,708\n(99.93%) |\n| hex_json | 1 KB | 1,044,006 (99.95%) | 1,043,676 (99.92%) | 1,044,042\n(99.96%) |\n| json_log | 2 KB | 1,042,706 (99.83%) | 1,043,156 (99.87%) | 1,043,178\n(99.88%) |\n| json_log | 16 KB | 1,037,774 (99.36%) | 1,037,962 (99.38%) | 1,037,960\n(99.38%) |\n| hex_json | 16 KB | 1,034,652 (99.06%) | 1,028,748 (98.49%) | 1,028,745\n(98.49%) |\n| json_log | 64 KB | 997,315 (95.48%) | 1,017,014 (97.37%) | 1,016,850\n(97.35%) |\n| hex_json | 64 KB | 999,133 (95.66%) | 1,010,212 (96.72%) | 1,010,178\n(96.72%) |\n| mixed_json | 1B-16KB | 1,035,417 (99.13%) | 1,037,592 (99.34%) |\n1,037,693 (99.35%) |\n\nAll batches under 1 MiB (max observed: 1,044,495 bytes, 4,081 bytes\nbelow 1 MiB).\nUtilization relative to TARGET_LIMIT: 95.48%-100.00%. Waste never\nexceeds one entry's worth of TARGET_LIMIT.\n\n### Flush Counts\n\n| Profile    | Entry Size | Level 1 | Level 6 | Level 9 |\n| ---------- | ---------- | ------- | ------- | ------- |\n| tiny_json  | 1 B        | 21      | 30      | 31      |\n| hex_json   | 10 B       | 45      | 56      | 56      |\n| json_log   | 256 B      | 10      | 9       | 10      |\n| hex_json   | 256 B      | 11      | 11      | 11      |\n| json_log   | 1 KB       | 8       | 8       | 8       |\n| hex_json   | 1 KB       | 11      | 9       | 10      |\n| json_log   | 2 KB       | 7       | 7       | 7       |\n| json_log   | 16 KB      | 6       | 6       | 6       |\n| hex_json   | 16 KB      | 7       | 6       | 6       |\n| json_log   | 64 KB      | 4       | 4       | 4       |\n| hex_json   | 64 KB      | 5       | 5       | 5       |\n| mixed_json | 1B-16KB    | 6       | 6       | 6       |\n\nWorst case: 56 flushes (hex_json/10B at level 6/9), well under\nMAX_GZIP_FLUSH_COUNT = 100.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nBenchmarks and unit tests (29 gzip_batcher tests, 24 config tests).\n\n## Are there any user-facing changes?\n\nAdded a new optional config field `gzip_compression_level` (0-9, default\n6) for tuning compression level.",
          "timestamp": "2026-03-23T20:21:49Z",
          "tree_id": "1469ff12dc799c822c7ab01ab11e5a6ecb189d56",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84c4259bea0753ed03b05d19eba9c45a5a97d075"
        },
        "date": 1774301178703,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2292693853378296,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06668103722116,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30677326709619,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.703645833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.41796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644157.3993815294,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 652075.8287068361,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003819,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17021031.114858743,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17040404.408982232,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db",
          "message": "Use OTAP spec aware `concatenate` when producing the results of `if`/`else` statements (#2393)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn the columnar query engine, when we write `if`/`else` statements in\nOPL, the results of each branch are concatenated together. Before this\nchange, we were simply using arrow's `concat_batches` helper function\nwith expects all the `RecordBatch`s to have the same schema. However,\nthis would cause a problem if some branch of the statement changes the\nschema.\n\nThis PR corrects the issue by using OTAP's\n[`concatenate`](https://github.com/open-telemetry/otel-arrow/blob/eaa4103326057ef68125244171801bc010cb3571/rust/otap-dataflow/crates/pdata/src/otap/transform/concatenate.rs#L75)\nfunction instead which correctly expands each `RecordBatch` into a\ncommon schema.\n\nThere's one pipeline stage that also writes new IDs to the rows with\nnull IDs (this happens when we assign attributes). In order for\n`concatenate` to produce a valid batch, we need to ensure the IDs are\nglobally unique. This PR adds a mechanism to initialize shared state for\ndifferent implementations of the same pipeline stage if they're being\nused in a nested branch within conditional pipeline stage, and uses it\nfor the purpose of ensuring unique IDs when filling in these null rows.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2216 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-03-24T01:20:26Z",
          "tree_id": "67a5f293ea50f9bfb31db99bc6af9abf4daa22ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db"
        },
        "date": 1774318398347,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8160354495048523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06235540219161,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3513849492366,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.036458333333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.02734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 646221.4773916886,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 651494.8740143589,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002314,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16938604.79285865,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16958767.855259307,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3bc95b9d9642d2b00009807923d34006e6502ef5",
          "message": "Remove \"Beaubourg\" Rust prototype (#2414)\n\n# Change Summary\n\nRetires the `rust/beaubourg` prototype from the repo. I believe it has\nserved it's purpose! Adds to the `rust/README.md` a permalink so we can\nfind it easily.\n\nThank you @lquerel. \n\n## What issue does this PR close?\n\nSee https://github.com/open-telemetry/otel-arrow/pull/293\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-24T16:10:35Z",
          "tree_id": "a9aa277eb0f2479c5f7bd522ae6262a46414f04d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3bc95b9d9642d2b00009807923d34006e6502ef5"
        },
        "date": 1774375021198,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3027286529541016,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.04682326687693,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.37200710479574,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.08125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.95703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 643876.4464685194,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 652264.4097705133,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00217,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17005028.498369526,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17025051.61032358,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0dde9c12802beebd157468f013b01a33cd40b3ac",
          "message": "feat: Log sampling processor (#2390)\n\n# Change Summary\n\nThis PR adds a very simple log sampling processor that's intended to be\nextensible and leave room for more sophisticated samplers in the future.\n\n## What issue does this PR close?\n\n* Closes #2382\n\n## How are these changes tested?\n\nUnit tests added - I also ran it locally and observed the metrics in the\ndashboard. This was with an `emit: 1, out_of: 10` and it dropped\nprecisely 90% according to the reported metrics.\n\n<img width=\"1745\" height=\"862\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/1faf1c68-b0fc-4193-9dd0-5de03e4399b3\"\n/>\n\nAnd the debug processor reported getting batches of exactly 10 which is\nexpected given traffic generator was spitting out batches of 100:\n\n<img width=\"407\" height=\"116\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/231c8e37-e93f-4567-b797-cab7223cc5f5\"\n/>\n\n\n## Are there any user-facing changes?\n\nYes, new processor!",
          "timestamp": "2026-03-24T19:10:26Z",
          "tree_id": "763cc0c5cc9963594254f5e2e9df43616c967976",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0dde9c12802beebd157468f013b01a33cd40b3ac"
        },
        "date": 1774383181337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2018213272094727,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.04423174488102,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.29831202046036,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.146354166666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.05078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644684.7630359232,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 652432.7215921046,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002386,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17039172.551088147,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17056541.3238352,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2abee0d4642c6bf2c2be774c5001320451913e74",
          "message": "Pipeline/group/engine policy precedence: prevent against misuse of unresolved policies (#2392)\n\n# Change Summary\n\nLike #2154 but for the other three policy fields. Make all fields Option\ntypes. Adds a ResolvedPolicies type which strips the Options after\nresolving. There was existing resolve code, but it was not used\nconsistently: this was observed for the `telemetry` policy.\n\n## What issue does this PR close?\n\nFixes #2389.\n\n## How are these changes tested?\n\nOne new test. The `configs/internal-telemetry.yaml` configuration is\nmodified to show the problem. Before the fix, no duration metrics. After\nthe fix, duration metrics, as set by the top-level policy.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-25T01:06:47Z",
          "tree_id": "2f0c86819cc64049011069f5eb68faf12ae12b7e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2abee0d4642c6bf2c2be774c5001320451913e74"
        },
        "date": 1774404943538,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.179158091545105,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06206144305598,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35219875776397,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.942838541666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.94921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 646166.4153672179,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 653785.7388886212,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007427,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16951807.108035505,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16971332.110943206,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0ed2efb7f92d2370ae0449699414a2f68c058c28",
          "message": "fix panic in `attribute_transform` benchmark (#2426)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nI noticed on main that running the `attribute_transform` benchmark\npanics because we are not adding the `type` column, which is supposed to\nbe required. The additional validation we added in\nhttps://github.com/open-telemetry/otel-arrow/pull/2356 now causes the\nbenchmarks to panic.\n\nThis PR resolves this by adding the required column.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/2429\n\n## How are these changes tested?\n\nI ran the benchmarks locally and they do not panic anymore\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n No\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-25T06:20:47Z",
          "tree_id": "91d05190696cfc73baa2cf6602704fa9a4992de6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ed2efb7f92d2370ae0449699414a2f68c058c28"
        },
        "date": 1774422500125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8525137901306152,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.00049089040597,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.30639274362355,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.34296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.25,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 653600.8703375625,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 659172.9079934815,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002466,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17018615.127219774,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17038464.204417598,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a44fe949ace10a2c36794537e4652ca89f2dce94",
          "message": "chore(deps): update dependency duckdb to v1.5.1 (#2418)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.5.0` → `==1.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.5.0/1.5.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-25T15:50:05Z",
          "tree_id": "7976e09bab150b8d799c826d7ba5dd1295cb94bf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a44fe949ace10a2c36794537e4652ca89f2dce94"
        },
        "date": 1774461480714,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.302981972694397,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.08612433252291,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31390145646111,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.6703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 642092.889880163,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 644038.315634863,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005374,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16952745.683434404,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16971848.972801346,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82a5521ad1d20c1cb9e30faade32a9ed9c0c339b",
          "message": "[query-engine] Adjust logging for conversion operations in RecordSet engine (#2435)\n\nRelates to #2430\n\n# Changes\n\n* Pass along `SelectionOptions` to `execute_convert_scalar_expression`\n* Tiddy:\n  * Reduce code duplication in conversion code\n  * Implement `Copy` for `RecordSetEngineDiagnosticLevel`\n\n# Details\n\nWe pass along `SelectionOptions` so that `toint(Something)` will emit a\nwarn if \"Something\" isn't found but `coalese(tonint(Something), null)`\nwill emit an info. This follows the pattern we've established so far\nwhen some outer function is used to handle the \"not found\" case. The\nassumption being \"not found\" is expected\\anticipated in these cases so a\nwarning is not necessary\\overkill.",
          "timestamp": "2026-03-25T18:53:09Z",
          "tree_id": "59f84c684fe305750dae4c52e1ee1e132369dae7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82a5521ad1d20c1cb9e30faade32a9ed9c0c339b"
        },
        "date": 1774467806775,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8161180019378662,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.10119245588321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35031448489542,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.440494791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.8125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 651375.4556931528,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656691.4480546138,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003096,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16974949.282004897,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16994608.2937663,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9cdb34e73656de4317d0336c5c10c3da5c7ebda5",
          "message": "fix attributes dropped when decoding OTAP -> OTLP proto bytes when IDs are out of order (#2421)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nFixes issue decoding OTAP to OTLP proto bytes when the ID column of the\nroot record batch is not in sorted order.\n\nWhen we do this decoding, we initialize cursors for the order in which\nto visit the rows of each record batch. When we visit some row in the\nroot record batch, we then try to find its attributes by advancing the\nattributes cursor until the parent_id column matches the ID of the\ncurrent row in the root record batch.\n\nThe whole scheme is predicated on the assumption that we'll visit the\n`id`/`parent_id` column in the same order. However, we initialize the\ncursor for the attribtues record batch in sorted parent_id order, but\nfor the root record batch we were only initializing it in sorted order\nof resource/scope ID.\n\nThe fix is to add the ID column to the multi-column sort which is used\nto initialize the cursor for the root record batch.\n\nThis PR also makes the change to avoid using RowConverter for the\nmulti-column sort for the root record batch cursor init, instead opting\nto simply pack the IDs into a Vec of some unsized int (width depending\non how many ID columns we're packing), and then sort this. This slightly\nimproves performance (even after adding the additional column to the\nsort) and I added a benchmark to measure. Bench: main 337µs , after:\n305µs\n\nNote: one side-effect of this change (which in my opinion is OK), is\nthat all the rows with a null ID (e.g. rows w/ no attributes) will\nappear first in the decoded OTLP message. This is b/c arrow typically\nuses 0 as a placeholder in int arrays in null rows, and the sort we are\ndoing does not bother looking at the null buffer for best performance.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #2270\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n No",
          "timestamp": "2026-03-25T22:28:13Z",
          "tree_id": "13152deb84902cb3167788d3b0e26767e81adf65",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9cdb34e73656de4317d0336c5c10c3da5c7ebda5"
        },
        "date": 1774480520337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9063722491264343,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.99610515054577,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38777769165182,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.206901041666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 647708.3410690696,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 653578.9898716243,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002908,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16980014.461707894,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16998844.090538148,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "238a1f78174fa44e3280a2662aa02277f2befa22",
          "message": "Add data sanitization step for transform processor results (#2434)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a \"sanitization\" step which can be performed on the result produced\nby the transform processor. This passes over all the the columns in all\nthe RecordBatch's and removes any values from dictionary columns that\nhave no keys pointing to them.\n\nThe procedure has some performance overhead, so there is an option to\nskip if if, for example the transformation isn't removing sensitive\ndata, or if something further along in the pipeline would remove the\nhidden arrow data (for example, serializing to OTLP in the OTLP\nexporter).\n\n**Why is this necessary?**\nSome of the arrow compute kernels will perform transformations ignoring\ncertain buffers for best performance if the result would still be a\nsemantically correct arrow array. For example when filtering, the arrow\ncompute kernels only filter dictionary key columns without touching the\ndictionaries.\n\nI'm imagining that someday someone will try to use transform processor\nto try to redact sensitive data, but the values from the rows they\ndeleted will still be present in the arrow buffers. If they then\ntransmitted the data using OTAP exporter (which does a simple arrow IPC\nserialization), the \"redacted\" data has escaped.\n\n**Why blindly do this for all transforms on all columns? Can't the\nquery-engine be smarter about this?**\nMaybe - but it's not as simple as it appears.\n\nFor example, consider when we're filtering. If we did something like:\n`logs | where event_name != \"sensitive_event_name\"`, it might be easy to\nthink that `event_name` is the sensitive column so it's the only one\nthat needs sanitizing. But _maybe_ the user actually knows a-priori that\nany log w/ this event name actually has sensitive data in some other\ncolumn.\n\nWhen it comes to the security of, I feel that it's better to be err on\nthe side of caution.\n\nIn the future we could maybe consider a better system where we let the\nuser provide hints about what fields they consider sensitive.\n\n**What's the performance impact?**\n\nThis sanitizing step adds significant overhead. When adding this\nsanitization step into the query engine's benchmarks, I saw anywhere\nbetween 5%-45% performance overhead on top of just executing the\ntransform pipeline w/ no sanitization. The actual overhead depends on\nthe complexity of the transform and size of output.\n\nThis poor performance is another reason why there's an option to skip\nthis step.\n\n**Should sanitize be on by default?**\n\nMy feeling is yes. If someone forgets to configure this or misconfigures\nit, I feel that it's best to fail on the side of worse\nperformance/better security.\n\n**Does this have to happen in the transform processor?**\n\nI feel that this is a reasonable place to do this, but open to\nsuggestions if anyone feels differently.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2313\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThere's a new user facing config field called `skip_sanitize_result`\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-26T01:03:23Z",
          "tree_id": "5b380f301b1a36964c542daace2d6319cc9d0ff1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/238a1f78174fa44e3280a2662aa02277f2befa22"
        },
        "date": 1774491484713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1322457790374756,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.0909944107824,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.37072445820434,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.690494791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.09765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 642815.1265804822,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 650093.3736964166,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005657,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17010709.8184993,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17030238.027255584,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "101909410+sjmsft@users.noreply.github.com",
            "name": "Sameer J",
            "username": "sjmsft"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f",
          "message": "fix: handle DrainIngress in fake_data_generator to unblock graceful shutdown (#2515)\n\n# Change Summary\n\nThe \"Ack nack redesign\" PR (3dca2837) introduced a two-phase\nDrainIngress/ReceiverDrained shutdown protocol but missed updating the\nfake_data_generator receiver. Without the DrainIngress handler, the\nmessage falls into the _ => {} catch-all, notify_receiver_drained() is\nnever called, the pipeline controller never removes the receiver from\nits pending set, and after the deadline expires it emits\nDrainDeadlineReached. This was causing pipeline-perf-test-basic to fail\nconsistently.\n\n## What issue does this PR close?\n\npipeline-perf-test-basic unit test is failing.\n\n* Closes #2511\n\n## How are these changes tested?\n\nfake_data_generator and runtime_control_metrics tests were executed.\n\n## Are there any user-facing changes?\n\nNo, fake_data_generator is an internal test/load-generation receiver,\nnot a user-facing component.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <josh.macdonald@gmail.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-09T00:04:43Z",
          "tree_id": "4f4595daf484dbea96a54aa48fded20f84d8862e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f"
        },
        "date": 1775697190558,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7330880761146545,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.0426822110081,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3765812631415,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.185807291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.47265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 654162.273607382,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 649366.6878182382,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001846,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16927837.03015905,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16936032.505658973,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5c4f5f23e8c1c6632f1545bd74ef04393d46166d",
          "message": "Add validation test cicd job (#2616)\n\n# Change Summary\n\ntrigger validation tests with #[cfg(validation_tests)], added dedicated\ncicd job for validation tests which runs them sequentially\n\n## What issue does this PR close?\n\n* Closes #2611 \n\n## How are these changes tested?\n\nValidation cicd\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-09T16:00:37Z",
          "tree_id": "33ebaf4ef6d4f8489a0a3e820dcb6ce2e78f3779",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5c4f5f23e8c1c6632f1545bd74ef04393d46166d"
        },
        "date": 1775756049182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4648590385913849,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.07592826310659,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32033542976939,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.030078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.40234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 647979.4582492737,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 650991.6492597428,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001507,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16938558.23000525,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16948865.304254614,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "tree_id": "395d3507ab1a26ea9714ce2ac368b63e68cf3b24",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775759018057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6854262351989746,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23044007599886,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46686798296554,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.507552083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.2265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 578888.4960330215,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 582856.3495530956,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002215,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15442187.711783348,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15436172.932545623,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "8164192+clhain@users.noreply.github.com",
            "name": "clhain",
            "username": "clhain"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fdc0cfc8ab414af51656daca330695a70c7cbbdb",
          "message": "Refactor config structs from internally-tagged enums to flat `type` + `config` for Kubernetes CRD compatibility (#2594)\n\n# Change Summary\n\n&#128680;**This PR Includes Breaking Config API Changes**&#128680;\n\nThis branch replaces three internally-tagged/adjacently-tagged enum\nconfig types with flat structs using explicit `type` discriminator\nfields, making them representable as Kubernetes CRD structural schemas.\n\n**What changed:**\n\n- **Metrics exporter configs** (`MetricsPeriodicExporterConfig`,\n`MetricsPullExporterConfig`): Converted from custom `Deserialize`\nvisitor-based enums (where the exporter name was a map key like\n`console:` or `otlp: { ... }`) to flat structs with `type` (enum\ndiscriminator) + `config` (`serde_json::Value` free-form blob).\nExporter-specific configuration is now extracted and validated at\nruntime via `from_config()` helpers. This mirrors the node specification\n/ configuration logic in the pipeline configs\n\n- **`CoreAllocation`**: Converted from a `#[serde(tag = \"type\")]` enum\nwith variants like `AllCores`, `CoreCount { count }`, `CoreSet { set }`\nto a flat struct with a `strategy` enum field plus optional `count` and\n`set` fields. Convenience constructors (`all_cores()`, `core_count(n)`,\n`core_set(ranges)`) replace direct enum variant construction.\n\n- **`PropagationSelector`**: Converted from a `#[serde(rename_all)]`\nenum with variants `AllCaptured`, `None`, `Named(Vec<String>)` to a flat\nstruct with `selector_type` enum + optional `named` list. Includes a new\n`validate()` method that rejects invalid combinations (e.g., `type:\nnamed` without a `named` list).\n\n\n## What issue does this PR close?\n\n* Closes #2592\n\n## How are these changes tested?\n\nValidation tests added for post-deserialization handling, updated\nexisting tests to new structs.\n\n## Are there any user-facing changes?\n\n**Config file format changes** (YAML, breaking):\n\n```yaml\n# Before                          # After\nexporter:                         exporter:\n  otlp:                             type: otlp\n    endpoint: \"...\"                 config:\n                                      endpoint: \"...\"\n\nselector: all_captured            selector:\n                                    type: all_captured\n\nselector: !named                  selector:\n  - tenant_id                       type: named\n  - x-request-id                    named:\n                                      - tenant_id\n                                      - x-request-id  \n\n\ncore_allocation:                  core_allocation:\n  type: core_count                  type: core_count # Non-breaking alias to new `strategy` field.\n  count: 4                          count: 4\n```",
          "timestamp": "2026-04-10T20:39:24Z",
          "tree_id": "c728603e54d29f5849d9b56e2be8077484d790b4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fdc0cfc8ab414af51656daca330695a70c7cbbdb"
        },
        "date": 1775863456247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7430185079574585,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.00453460191905,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.2472637124264,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.947916666666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.08984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 646563.9505560187,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 651368.0405122432,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002207,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16940613.900090232,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16948346.984008566,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dd5aa21c51c1f7a81251cc6bfd889ca3d6442a90",
          "message": "Reduce WARN volume on azure_monitor_exporter Heartbeat failure (#2628)\n\n# Change Summary\n\nOn heartbeat failure (simulated in this case by providing an invalid dcr\nidentifier), internal telemetry produces two almost identical WARN logs:\n\nProduced by inner `Heartbeat.send()`:\n> 2026-04-10T15:58:58.087Z WARN\notap-df-contrib-nodes::azure_monitor_exporter.heartbeat.error:\n{\"error\":{\"code\":\"NotFound\",\"message\":\"Data collection rule with\nimmutable Id 'dcr-badid' not found.\"}} [status=404] entity/node.attrs:\nnode.id=exporter node.urn=urn:microsoft:exporter:azure_monitor\nnode.type=exporter pipeline.id=main\npipeline.group.id=example-azuremonitorpipeline core.id=0 numa.node.id=0\nprocess.instance.id=AGOXQHRN2NZSBC2J3VHVEMWHMU host.id=CPC-drewr-ZFPSN\ncontainer.id=\n\nProduced by outer continuous loop:\n> 2026-04-10T15:58:58.087Z WARN\notap-df-contrib-nodes::azure_monitor_exporter.heartbeat.send_failed:\n[error=UnexpectedStatus { status: 404, body:\n\"{\\\"error\\\":{\\\"code\\\":\\\"NotFound\\\",\\\"message\\\":\\\"Data collection rule\nwith immutable Id 'dcr-badid' not found.\\\"}}\" }] entity/node.attrs:\nnode.id=exporter node.urn=urn:microsoft:exporter:azure_monitor\nnode.type=exporter pipeline.id=main\npipeline.group.id=example-azuremonitorpipeline core.id=0 numa.node.id=0\nprocess.instance.id=AGOXQHRN2NZSBC2J3VHVEMWHMU host.id=CPC-drewr-ZFPSN\ncontainer.id=\n\nSince the caller of `Heartbeat.send()` already logs the error with the\nsame details:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/b5f0814099566c119a29aa8465a137e04adbeeb4/rust/otap-dataflow/crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs#L585-L588\n\nThe inner WARN only doubles telemetry volume for the same information.\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/b5f0814099566c119a29aa8465a137e04adbeeb4/rust/otap-dataflow/crates/contrib-nodes/src/exporters/azure_monitor_exporter/heartbeat.rs#L226-L230\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nDebug run of df_engine\n\n## Are there any user-facing changes?\n\nYes, less telemetry on Heartbeat failure scenarios",
          "timestamp": "2026-04-10T22:32:57Z",
          "tree_id": "2f996c975a16e9c8c24aa1548204864e133de109",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd5aa21c51c1f7a81251cc6bfd889ca3d6442a90"
        },
        "date": 1775869140006,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4629325270652771,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.94869982054922,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.2682863340564,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.817578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648825.3629134082,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 645821.7393124572,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002192,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16863318.568242297,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16868296.442443036,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "101909410+sjmsft@users.noreply.github.com",
            "name": "Sameer J",
            "username": "sjmsft"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "tree_id": "0ef8ab22eebc52d2ab83c46aa1d36648205f5dd9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775880263392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1544549465179443,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.37359843150128,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.70328081556997,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.759895833333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.31640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 641569.3755248222,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 648976.0045691456,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002465,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16708451.090393204,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16713805.445115235,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3b657f71cb6ca6bc8125c0cec976c9d5f03aea85",
          "message": "[CI] Fix binary size display (#2633)\n\n# Change Summary\n\nFollow-up to #1689 \n\n- The JSON report stores the value in MB, but the summary step was\npassing it to `numfmt` as if it were bytes, causing the display to show\n`102B` instead of `102 MB`.\n\nCo-authored-by: Utkarsh Umesan Pillai <utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-12T05:28:30Z",
          "tree_id": "2a462842ee3e1cd2dfb1e2c7eb93b5ee2a2e5e41",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b657f71cb6ca6bc8125c0cec976c9d5f03aea85"
        },
        "date": 1775975092108,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7953976988792419,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.9424059850488,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.34455894925003,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.520052083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 631885.8483604108,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 636911.854284494,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001521,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16738060.813564569,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16742900.685368383,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "cda287f26ad0f4f06e127886950a374b46f4bfab",
          "message": "Bridge columnar query engine's expr eval into filter code (Part 1) (#2632)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nBridges the columnar query engine's expression evaluation into the\nfilter evaluation.\n\nBefore this change, we only had the capability in the columnar query\nengine to evaluate filters where at least one side of some relative\nexpression was a literal (e.g. `severity_text == \"ERROR`). This is\nclearly very limiting, and there's a whole class of logical expressions\nthat we wouldn't evaluate, such as comparing two fields\n(`attributes[\"x\"] > attributes[\"y\"]`) or using functions / arithmetic on\neither side of expression (such as `substring(event_name, 0, 3) ==\nsubstring(severity_text, 0, 3)`).\n\nThis is part 1 of what will be multiple PRs to complete this work. \n\nFuture PRs will include:\n- Type mismatches - currently if the expressions on either side of the\nrelative binary don't evaluate to the same type (or comparable types),\nthe execution fails with an error. This will include correct handling of\nnulls (e.g. something evaluating to `null == null` should probably\nresult in `true`)\n- Bridging the code in the opposite direction - currently we don't\nsupport evaluating logical expressions outside the context of a filter.\nE.g., we don't yet support `logs | set attributes[\"x\"] = attributes[\"y\"]\n> attributes[\"z\"]`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #1508\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - these new filtering capabilities are now available in transform\nprocessor.\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-04-12T05:50:07Z",
          "tree_id": "a012f63992e656420617ae014dbafc98386308ad",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cda287f26ad0f4f06e127886950a374b46f4bfab"
        },
        "date": 1775976234663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6170148253440857,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.08297891420406,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41629813664596,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.768489583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.30859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 645783.4960198447,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 641798.916202422,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007331,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17039542.309887134,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17042068.659978792,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "79b160f5914e2d75eb052a06a86d345491a862b3",
          "message": "test: improve handling of channel-send shutdown in durable_buffer_processor_tests (#2629)\n\n# Change Summary\n\nRefactor how test code determines acceptable shutdown results in\n`durable_buffer_processor_tests.rs`. Don't rely on error message\nmatching.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nVerified that the tests pass locally.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-04-12T05:52:09Z",
          "tree_id": "b9b396931afdbe8c3fb27030bb2782c69e436d87",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/79b160f5914e2d75eb052a06a86d345491a862b3"
        },
        "date": 1775977237443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.962422251701355,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.05745330397468,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38416660191156,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.862890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.71484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 649827.7033404541,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656081.7894535887,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008128,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17046236.11298101,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17055907.044953875,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9eb967e93c55492a6b5a7becfead40234b15a1ad",
          "message": "Columnar Query Engine/OPL Parser support for `TextScalarExpression::Capture` (#2626)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nPartially support `TextScalarExpression::Capture` in columnar query\nengine.\n\nhttps://github.com/open-telemetry/otel-arrow/blob/b5f0814099566c119a29aa8465a137e04adbeeb4/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L11\n\nThis currently only supports identifying the capture group by number or\nname.\n\nIn KQL, the `extract` function gets parsed into this type:\n```kql\nlogs | set severity_text = extract(\".*severity=(.*).*\", 1, event_name)\n```\n\nIn OPL, I added a function called `regexp_capture` that has similar\narguments:\n```kql\nlogs | set severity_text = regexp_capture(event_name, \".*severity=(.*).*\", 1)\n```\n\nNamed capture groups are also supported:\n```kql\nlogs | set attributes[\"x\"] = regexp_capture(attributes[\"y\"], \".*(?P<mygroup>.....+) \", \"mygroup\")\n```\n\nThe order of the arguments are slightly different from KQL's `extract`\nfunction. This follows the convention used in datafusion's [regexp\nscalar\nfunctions](https://datafusion.apache.org/user-guide/sql/scalar_functions.html#regular-expression-functions)\n(which are based on various functions available in SQL) where the first\nargument is the string and second is the pattern.\n\nUnderneath the hood, this is implemented using a ScalarUDF called\n`regexp_substr`, that accepts the same signature found in many SQL\ndialetcs (such as\nhttps://docs.snowflake.com/en/sql-reference/functions/regexp_substr),\nwith the additional capability that this can be called using a named\ngroup instead of a group number. This is probably overkill for just the\ncapture group extraction case, but I was thinking it was best to do\nsomething flexible/standard and that we could probably try donating this\nto upstream datafusion eventually.\n\nThis function is also available to be called in OPL/KQL directly:\n```kql\nlogs | extend attributes[\"s1\"] = regexp_substr(attributes[\"attr\"], \"hell.\")\n```\n\nThe `regex_substr` function is optimized for the case all the arguments\nare scalars, except for the first argument which is the source string\ncolumn. If any of the other arguments are not scalar, it casts the array\ninto either a string array or Int64 array. This keeps the imlementation\nstraight forward, and having a ton of edge cases for dict handling.\n\nThere's currently some funny logic in the handling for the optional\nparameters that I'm not proud of. Datafusion functions support optional\narguments (via multiple signatures), but our expression tree expects\nevery function to have a well-defined number of parameters, and the\ndefaults are actually supplied to the parser state. Currently KQL is\nusing these - if params are supplied in the query, it fills them in. OPL\nparser isn't doing this and just delegates to the function to figure it\nout internally. In the future, I'll probably try to fix up the\nexpressions we use for function signature in our expression tree to add\nthe concept of optional parameters.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2578\n\n## How are these changes tested?\n\nunit\n\n## Are there any user-facing changes?\n\nYes - these functions are available for use in transform processor.\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-04-12T06:24:06Z",
          "tree_id": "63b343585a6e2df6db0e7bfe1f3c4f239a103e65",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9eb967e93c55492a6b5a7becfead40234b15a1ad"
        },
        "date": 1775978393583,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8045010566711426,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.119627907324,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43969948106266,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.6625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.63671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 649123.9970325085,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 643901.7879912858,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002194,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16972637.76352961,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16977386.637951694,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "tree_id": "3689be6ec138f166b36a802675f8621ad64caadf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1775979414330,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.5720868706703186,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.10259139086904,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3822972658973,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.77890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 651791.1182312904,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 648062.3068017662,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004107,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17032186.12428739,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17041378.598481838,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "28ef25b61e3cbf4889179f13b9182654f28ac896",
          "message": "Update Rust crate zip to v4.6.1 (#2640)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [zip](https://redirect.github.com/zip-rs/zip2) |\nworkspace.dependencies | minor | `=4.2.0` → `=4.6.1` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>zip-rs/zip2 (zip)</summary>\n\n###\n[`v4.6.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#461---2025-09-03)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.6.0...v4.6.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Fixes an issue introduced by the swap from `lzma-rs` to `liblzma`\n([#&#8203;407](https://redirect.github.com/zip-rs/zip2/pull/407))\n\n###\n[`v4.6.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#460---2025-08-30)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.5.0...v4.6.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Allow to read zip files with unsupported extended timestamps\n([#&#8203;400](https://redirect.github.com/zip-rs/zip2/pull/400))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- enable clamp\\_opt for ppmd and xz\n([#&#8203;401](https://redirect.github.com/zip-rs/zip2/pull/401))\n\n###\n[`v4.5.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#450---2025-08-21)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.4.0...v4.5.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Allow reading ZIP files where the central directory comes *before* the\nfiles ([#&#8203;384](https://redirect.github.com/zip-rs/zip2/pull/384))\n([#&#8203;396](https://redirect.github.com/zip-rs/zip2/pull/396))\n\n###\n[`v4.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#440---2025-08-21)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.3.0...v4.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Add `lzma-static` and `xz-static` features that enable\n`liblzma/static`\n([#&#8203;393](https://redirect.github.com/zip-rs/zip2/pull/393))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Move deprecated annotations to fix a Clippy warning\n([#&#8203;391](https://redirect.github.com/zip-rs/zip2/pull/391))\n\n###\n[`v4.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#430---2025-07-09)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.2.0...v4.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Add support for PPMd\n([#&#8203;370](https://redirect.github.com/zip-rs/zip2/pull/370))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMTAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjExMC4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-13T11:44:21Z",
          "tree_id": "93a154fd40914c906de7488764f3d5b80219c3a8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/28ef25b61e3cbf4889179f13b9182654f28ac896"
        },
        "date": 1776089245437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0346367359161377,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.09724714182786,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42746305609285,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.507942708333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.05859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 649837.5470457979,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656561.004768443,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007219,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17048671.269985996,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17053827.903452415,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "caa18736ab59255d459edc034c5d95147478e573",
          "message": "Update Rust crate hashbrown to 0.17 (#2639)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [hashbrown](https://redirect.github.com/rust-lang/hashbrown) |\nworkspace.dependencies | minor | `0.16` → `0.17` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rust-lang/hashbrown (hashbrown)</summary>\n\n###\n[`v0.17.0`](https://redirect.github.com/rust-lang/hashbrown/blob/HEAD/CHANGELOG.md#0170---2026-04-06)\n\n[Compare\nSource](https://redirect.github.com/rust-lang/hashbrown/compare/v0.16.1...v0.17.0)\n\n##### Added\n\n- Added `hash_table::OccupiedEntry::replace_entry_with`\n([#&#8203;669](https://redirect.github.com/rust-lang/hashbrown/issues/669))\n- Added `hash_map::{OccupiedEntry::into_entry,\nVacantEntryRef::insert_entry_with_key}`\n([#&#8203;670](https://redirect.github.com/rust-lang/hashbrown/issues/670))\n- Added `hash_table::UnsafeIter`\n([#&#8203;667](https://redirect.github.com/rust-lang/hashbrown/issues/667))\n- Added `iter` methods to various `HashTable` iterators\n([#&#8203;667](https://redirect.github.com/rust-lang/hashbrown/issues/667))\n- Added\n`HashMap::{replace_key,replace_key_unchecked,insert_with_key_unchecked}`\n([#&#8203;681](https://redirect.github.com/rust-lang/hashbrown/issues/681))\n- Added `into_map` methods to all `HashMap` entry types\n([#&#8203;686](https://redirect.github.com/rust-lang/hashbrown/issues/686))\n- Added `into_table` methods to all `HashTable` entry types\n([#&#8203;686](https://redirect.github.com/rust-lang/hashbrown/issues/686))\n- Added `#[must_use]` to constructors\n([#&#8203;697](https://redirect.github.com/rust-lang/hashbrown/issues/697))\n- `TryReserveError` now implements `Error`\n([#&#8203;698](https://redirect.github.com/rust-lang/hashbrown/issues/698))\n\n##### Changed\n\n- Changed `EntryRef` to use `ToOwned`\n([#&#8203;670](https://redirect.github.com/rust-lang/hashbrown/issues/670))\n- Bumped MSRV to 1.85 (2024 edition)\n([#&#8203;676](https://redirect.github.com/rust-lang/hashbrown/issues/676))\n\n##### Fixed\n\n- `HashTable:clone_from` now forwards to `RawTable::clone_from` instead\nof using the default implementation\n([#&#8203;668](https://redirect.github.com/rust-lang/hashbrown/issues/668))\n- Fixed potential UB in `RawTableInner::fallible_with_capacity`\n([#&#8203;692](https://redirect.github.com/rust-lang/hashbrown/issues/692))\n- Fixed incorrect length if a hasher panics during rehash\n([#&#8203;710](https://redirect.github.com/rust-lang/hashbrown/issues/710))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMTAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjExMC4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-13T11:48:18Z",
          "tree_id": "78f949bc7b299d809c28683fd108d510fdf8686f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/caa18736ab59255d459edc034c5d95147478e573"
        },
        "date": 1776090271261,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0477747917175293,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15806897459922,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53323364485982,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.653645833333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.0625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 641742.527177989,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 648466.5437112406,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002232,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16959793.159753762,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16968517.531107083,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c2e39e05477b8394ce45d84f73bd89f244a9419b",
          "message": "[query-engine] Slice expression bug fix (#2636)\n\n## Changes\n\n* Switch `Slice` expression (arrays and strings) to use `Slice(source,\nstart, [length])` instead of `Slice(source, start_inclusive,\n[end_exclusive])` to match KQL\n[substring](https://learn.microsoft.com/kusto/query/substring-function?view=azure-data-explorer)\nbehavior.\n\n/cc @albertlockett @drewrelmas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-13T13:06:03Z",
          "tree_id": "cd278ea280499e9f5bed1192f47f938592e59707",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c2e39e05477b8394ce45d84f73bd89f244a9419b"
        },
        "date": 1776094171282,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.39045098423957825,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.03393008657078,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32001242525432,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.06484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.71875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 650703.4588963197,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 653244.1369544301,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.05326,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16987906.74635054,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16997623.84223251,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "8164192+clhain@users.noreply.github.com",
            "name": "clhain",
            "username": "clhain"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "28d8279b113434c11de9012042ab1cb4bc0f55fd",
          "message": "derive PartialEq for OtelDataflowSpec (#2645)\n\n# Change Summary\n\nDerive `PartialEq` on `OtelDataflowSpec` and its nested config\nstructs (where missing) to enable direct structural comparison of parsed\nconfigurations without needing to serialize to YAML.\n\n## What issue does this PR close?\n\n* Closes #2644\n\n## How are these changes tested?\n\nSeveral new unit tests against expected equal and non-equal cases.\n\n## Are there any user-facing changes?\n\nNo. This is a purely internal change. `PartialEq` is now available on\nconfig types for programmatic comparison, but there are no changes to\nconfiguration schema, behavior, or APIs.",
          "timestamp": "2026-04-13T19:35:57Z",
          "tree_id": "2b8336fe08870bd2a755dd8696052e301acf5bfb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/28d8279b113434c11de9012042ab1cb4bc0f55fd"
        },
        "date": 1776115593034,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9034562706947327,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06174467717963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41447419180618,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.553515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.53515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 649750.6106250152,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 655620.8230962572,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007368,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16957690.28736015,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16963877.38556662,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "88447796+thperapp@users.noreply.github.com",
            "name": "Thomas",
            "username": "thperapp"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163",
          "message": "Fix typo in Prometheus test config filename (#2648)\n\n# Change Summary\nFix typo in test config filename\nfake-debug-noop-prometh'~~u~~'eus-telemetry.yaml ->\nfake-debug-noop-prometheus-telemetry.yaml\n\n## What issue does this PR close?\nminor nit\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nN/A",
          "timestamp": "2026-04-13T23:38:17Z",
          "tree_id": "f6eba8352e7029468a977a34c10d5ddf7cdd5ca1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163"
        },
        "date": 1776127515092,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8355410695075989,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.58794519307042,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27574048389583,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.636588541666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.71484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 628069.4745325256,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 633317.2529013525,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002534,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16688260.10226082,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16693320.625050118,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cithomas@microsoft.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e",
          "message": "fix perf test (#2686)\n\nCloses #2667\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T02:51:58Z",
          "tree_id": "987502055306c69e3b26a1fd1d8326d084f9c0d2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e"
        },
        "date": 1776310807649,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.737657904624939,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.95408576505909,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3666060138782,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.282161458333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.5,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 639658.7198414388,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 644377.2129393661,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005598,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16946187.341695677,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16954720.548531137,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "43d170c92c488b08bc7549cb6a72279171329d9e",
          "message": "Address new rust 1.95 clippy errors (#2688)\n\n# Change Summary\n\nRust 1.95 flagged new clippy warnings from our main branch, blocking new\nPRs from going in\n\n## What issue does this PR close?\n\n* Closes #2687\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-04-16T17:20:04Z",
          "tree_id": "3273d1b8e36d714ae7ae0852e4a74793a70f2cae",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43d170c92c488b08bc7549cb6a72279171329d9e"
        },
        "date": 1776363936344,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.2931433320045471,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06099299127584,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41083894957006,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.964713541666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.36328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 649109.8361374001,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 651012.6583776283,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003503,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17018975.02154256,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17029141.142201904,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6ef19a99a26f17c89a12f5dad18c737259c1509d",
          "message": "run validation ci manually (#2675)\n\nI am seeing the validation tests acting up again in the validation ci\njob, setting the ci job to be a manual trigger instead of automatically\nrunning on every PR\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T20:11:07Z",
          "tree_id": "403e476ba1bec7e3260a1459230a81216be1b2a0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ef19a99a26f17c89a12f5dad18c737259c1509d"
        },
        "date": 1776374270154,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.07558249682188034,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23171292445896,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4614960080614,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.9984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.83203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 587031.8888590841,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 586588.1955173106,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005408,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15681781.551320113,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15652897.95819684,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b0c7885832831e2e8416305989abd580957aafd5",
          "message": "fix: Pass through generation_strategy to load generator for benchmarks  (#2683)\n\n# Change Summary\n\nThis fixes the disconnect in the attached issue by adding\n`generation_strategy` config to `df-loadgen-steps-docker.yaml`.\n\n## What issue does this PR close?\n\n* Closes #2681\n\n## How are these changes tested?\n\nResults before:\n<img width=\"2144\" height=\"593\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/9f5c741b-9455-4ce9-b33c-f6eb3e2dce52\"\n/>\n\nResults after:\n\n<img width=\"2172\" height=\"467\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/dc51f996-3755-470f-ad82-658bb93662e8\"\n/>\n\n## Are there any user-facing changes?\n\nNo\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-17T11:13:32Z",
          "tree_id": "aa3cbaecfa91cdf6c499a5a1b7dd108e3d2fa638",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b0c7885832831e2e8416305989abd580957aafd5"
        },
        "date": 1776429357310,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9289295673370361,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.09298657438235,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.37445994911727,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.688411458333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.91796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 636566.6851522574,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 642479.941000767,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003492,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16921042.421900094,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16929220.9333366,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "495588ee584201ea956c15c6fc102dc3465de675",
          "message": "update container config to allow configurable wait_for setting (#2672)\n\n# Change Summary\n\nUpdate ContainerConfig struct to have a wait_for field of type WaitFor\nfrom the test container crate. Added additional functions to allow a\nuser to configure the WaitFor enum variant to use for a test container\n\n## What issue does this PR close?\n\n* Closes #2668\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-17T12:32:22Z",
          "tree_id": "725b68172fbb97867a29dddbd980bee3aea2c180",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/495588ee584201ea956c15c6fc102dc3465de675"
        },
        "date": 1776433986377,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.32701370120048523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.93951455172969,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.24600604323236,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.229557291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644512.943078666,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 642405.2973531401,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002494,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16930647.308015116,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16938935.96388002,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1afaa2bddbff3ea3f40880dc3bb1389319c96361",
          "message": "cargo xtask check: synchronize rust-ci.yaml and clippy settings, fix default malloc selection (#2695)\n\n# Change Summary\n\nEnables clippy in the conditional for Jemalloc in src/main.rs, with\n@sapatrjv.\n\n## What issue does this PR close?\n\nIntroduced in #2420.\n\nAdds `--all-features --workspace -- -D warnings` to the clippy step in\n`cargo xtask check`.\n\nhttps://cloud-native.slack.com/archives/C08RRSJR7FD/p1776408892537689\n\n## How are these changes tested?\n\nRan `cargo xtask clippy` on Ubuntu, not sure this actually works in\nWindows.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-04-17T22:10:02Z",
          "tree_id": "c2ae98058d83346ce13cecc2fefd15cb5abf9fc8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1afaa2bddbff3ea3f40880dc3bb1389319c96361"
        },
        "date": 1776468346051,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.8432753086090088,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06698970140056,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41421024091719,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.271484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.00390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644843.3648246033,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656729.603209199,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003508,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16992634.86769098,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16999502.529401045,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "geukhanuslu@gmail.com",
            "name": "Gokhan Uslu",
            "username": "gouslu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "edac122ab94fbf853f598bbd898c2065355c193d",
          "message": "feat(engine): add ExtensionFactory, ExtensionWrapper, and extension lifecycle types (#2622)\n\n# Change Summary\n\nAdd the core engine types for the extension lifecycle system ([Phase 1,\nPR\n2](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/extension-system-architecture.md#pr-2----engine-extensionfactory--extensionwrapper--builder)).\n\nExtensions are PData-free components that provide cross-cutting services\n(e.g., authentication, storage) to data-path nodes. This PR introduces\nthe foundational types for defining, building, and starting extensions —\nfully separated from the node/DAG infrastructure.\n\n**New types (config crate):**\n- `ExtensionId` — separate ID namespace from `NodeId`\n- `ExtensionUrn` — 3-segment URN format (`urn:<namespace>:<id>`),\nsimpler than node's 4-segment `NodeUrn` (`urn:<namespace>:<kind>:<id>`)\nsince kind is implicit from the `extensions:` config section\n- `ExtensionUserConfig` — extension-specific config (`type`,\n`description`, `config` only — no output ports, wiring, or header\npolicies). Uses `#[serde(deny_unknown_fields)]` to reject node-specific\nfields at parse time\n- `PipelineExtensions` — collection type for pipeline extensions,\nparallel to `PipelineNodes`\n- `DuplicateExtension` — separate error variant for duplicate extension\nIDs (distinct from `DuplicateNode`)\n\n**New types (engine crate):**\n- `ExtensionFactory` — PData-free factory struct with `create` (returns\n`ExtensionBundle`) and `validate_config`\n- `ExtensionWrapper` — enum with `Local` / `Shared` variants (first\nlayer). Each variant holds an `ExtensionLifecycle<E, R>` with `Active` /\n`Passive` variants (second layer). This two-level structure makes\nimpossible states unrepresentable\n- `ExtensionLifecycle<E, R>` — generic lifecycle enum; `Active` bundles\nthe extension trait object with its control channel sender and receiver,\n`Passive` has no channels\n- `ExtensionBundle` — opaque return type from the builder, holding at\nmost one local and one shared `ExtensionWrapper`. Private fields enforce\nconstruction only via the builder\n- `ExtensionBundleBuilder` — builder with `with_local()` /\n`with_shared()` + `build()`. Enforces at least one variant present and\nrejects same-type dual registration via `TypeId` guard\n- `ControlChannel<R>` — generic shutdown-aware control channel. Concrete\ntypes `local::extension::ControlChannel` (with `LocalReceiver`) and\n`shared::extension::ControlChannel` (with `SharedReceiver`) are defined\nin their respective modules, matching how pipeline nodes handle control\nchannels\n- `ControlReceiver` — internal trait enabling the generic\n`ControlChannel<R>` to work with both local and shared receivers without\ncode duplication\n- `Active<E>` / `Passive<E>` — newtype wrappers that signal lifecycle\nintent at the type level\n- `EffectHandler` — extension identity and metrics reporter\n(forward-compatible extension point for the `start()` signature)\n- `ExtensionControlMsg` — PData-free control message enum (Config,\nCollectTelemetry, Shutdown)\n- `ExtensionControlSender` — control sender for extensions, using\n`ExtensionId`\n- `local::Extension` / `shared::Extension` — lifecycle traits\n(`Rc<Self>` vs `Box<Self>`), each taking their own `ControlChannel` type\n\n**Modified existing types:**\n- `ExtensionAlreadyExists` error variant added (uses `ExtensionId`, not\n`NodeId`)\n- `PipelineFactory::new()` now accepts extension factories as a 4th\nparameter\n- `pipeline_factory` proc macro generates `EXTENSION_FACTORIES`\ndistributed slice\n- `wrap_control_channel_metrics` generalized to accept `&str` name\ninstead of `&NodeId`, enabling reuse by both nodes and extensions\n- `control_channel_id()` similarly generalized from `&NodeId` to `&str`\n- `PipelineConfig` extensions field changed from `PipelineNodes` to\n`PipelineExtensions`\n- Extension URN validation moved from runtime `NodeKind` check to\nparse-time `ExtensionUrn` format enforcement\n- Removed redundant `canonicalize_plugin_urns` for extensions —\n`ExtensionUrn` normalizes at parse time via `#[serde(try_from)]`\n- `PipelineConfigBuilder` tracks `duplicate_extensions` separately from\n`duplicate_nodes`\n\n**Design decisions:**\n- Extensions are fully separated from nodes — zero `NodeType`, `NodeId`,\n`NodeUrn`, `NodeKind`, or `NodeUserConfig` references in extension code\n- `ExtensionUrn` uses a simpler 3-segment format (`urn:<ns>:<id>`) since\nthe `<kind>` segment is redundant for extensions. This also prevents\ncross-kind misconfiguration: a 4-segment node URN placed in\n`extensions:` is rejected at parse time\n- Two-level enum design: `ExtensionWrapper` (Local/Shared) ×\n`ExtensionLifecycle` (Active/Passive) — eliminates impossible states\nwithout `Option`s\n- `ExtensionBundle` replaces `Vec<ExtensionWrapper>` — exactly 0-1 local\n+ 0-1 shared, private fields, builder-enforced invariants\n- Same-type guard: runtime `TypeId` check prevents registering the same\nconcrete type as both local and shared (avoids duplicate lifecycles on\nshared `Arc` state)\n- Local extensions use local channels (`LocalSender`/`LocalReceiver` via\n`LocalMode`), shared extensions use shared channels\n(`SharedSender`/`SharedReceiver` via `SharedMode`) — matching how\npipeline nodes handle their control channels\n- Shutdown grace-period: `ControlChannel::recv()` continues delivering\nConfig/CollectTelemetry messages during the grace period, using a\n2-branch `select!` racing the deadline against incoming messages\n- `build()` and `start()` return `Result` with `Error::InternalError`\ninstead of panicking\n- `Clone` bound on `Active<E>` and `Passive<E>` shared providers is\nretained for the capability system (next PR) which needs `Box<dyn\nCloneAnySend>` for type-erased registration\n\n## What issue does this PR close?\n\n* Part of the extension system Phase 1 rollout (PR 2 of 8)\n\n## How are these changes tested?\n\n- 27 unit tests in `extension.rs` (engine) covering: all lifecycle\nvariants (local/shared × active/passive), builder validation (empty,\nsame-type guard), dual-type creation, start/shutdown for both local and\nshared, control channel behavior (immediate/delayed shutdown, config\nordering, grace-period message delivery, channel close), effect handler,\ncontrol sender, telemetry metrics for both active and passive paths,\n`ExtensionBundle` iteration\n- 10 unit tests in `extension_urn.rs` (config) covering: full URN\nparsing, short form expansion, case insensitivity, rejection of\n4-segment node URN format, empty/invalid segments, serde roundtrip,\n`From<&str>`\n- 2 unit tests in `extension.rs` (config) covering: YAML\ndeserialization, rejection of unknown fields\n- 4 unit tests in `lib.rs` (engine) covering: `ExtensionFactory`\nname/clone/validate_config, `ExtensionAlreadyExists` error variant\n- All existing engine tests and config tests pass\n- `cargo clippy` with `-D warnings` passes for both `otap-df-config` and\n`otap-df-engine`\n- `cargo fmt --all -- --check` passes\n\n## Are there any user-facing changes?\n\nExtension URN format changed from `urn:<namespace>:extension:<id>` to\n`urn:<namespace>:<id>`. This is a simplification — the `:extension:`\nsegment was redundant since extensions are always declared in the\n`extensions:` config section.",
          "timestamp": "2026-04-17T23:00:50Z",
          "tree_id": "700a5d935e56110c122d39f6502f6c5e13459998",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/edac122ab94fbf853f598bbd898c2065355c193d"
        },
        "date": 1776469797997,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1423351764678955,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.11078007532794,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43919926085618,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.968229166666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.66796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644639.358457572,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 652003.3009880349,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002641,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16983313.958444048,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16989364.42853016,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "tree_id": "0be50b21ac7f629ac332644b1a64bcd628e6fa8c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776476670467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4078134298324585,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.04256426297704,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35356194006798,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.594661458333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.20703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 641239.4940474262,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 650266.950198988,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005387,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16996435.814965766,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17004477.852082975,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e6b0c45febd8d61b394478536242d1b6b5f6d349",
          "message": "Update Rust crate arrow to v58 (#2711)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [arrow](https://redirect.github.com/apache/arrow-rs) | dependencies |\nmajor | `55.2.0` → `58.0.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>apache/arrow-rs (arrow)</summary>\n\n###\n[`v58.1.0`](https://redirect.github.com/apache/arrow-rs/blob/HEAD/CHANGELOG.md#5810-2026-03-20)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-rs/compare/58.0.0...58.1.0)\n\n[Full\nChangelog](https://redirect.github.com/apache/arrow-rs/compare/58.0.0...58.1.0)\n\n**Implemented enhancements:**\n\n- Reuse compression dict lz4\\_block\n[#&#8203;9566](https://redirect.github.com/apache/arrow-rs/issues/9566)\n- \\[Variant] Add `variant_to_arrow` `Struct` type support\n[#&#8203;9529](https://redirect.github.com/apache/arrow-rs/issues/9529)\n- \\[Variant] Add `unshred_variant` support for `Binary` and\n`LargeBinary` types\n[#&#8203;9526](https://redirect.github.com/apache/arrow-rs/issues/9526)\n- \\[Variant] Add `shred_variant` support for `LargeUtf8` and\n`LargeBinary` types\n[#&#8203;9525](https://redirect.github.com/apache/arrow-rs/issues/9525)\n- \\[Variant] `variant_get` tests clean up\n[#&#8203;9517](https://redirect.github.com/apache/arrow-rs/issues/9517)\n- parquet\\_variant: Support LargeUtf8 typed value in `unshred_variant`\n[#&#8203;9513](https://redirect.github.com/apache/arrow-rs/issues/9513)\n- parquet-variant: Support string view typed value in `unshred_variant`\n[#&#8203;9512](https://redirect.github.com/apache/arrow-rs/issues/9512)\n- Deprecate ArrowTimestampType::make\\_value in favor of\nfrom\\_naive\\_datetime\n[#&#8203;9490](https://redirect.github.com/apache/arrow-rs/issues/9490)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Followup for support \\['fieldName'] in VariantPath\n[#&#8203;9478](https://redirect.github.com/apache/arrow-rs/issues/9478)\n- Speedup DELTA\\_BINARY\\_PACKED decoding when bitwidth is 0\n[#&#8203;9476](https://redirect.github.com/apache/arrow-rs/issues/9476)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Support CSV files encoded with charsets other than UTF-8\n[#&#8203;9465](https://redirect.github.com/apache/arrow-rs/issues/9465)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Expose Avro writer schema when building the reader\n[#&#8203;9460](https://redirect.github.com/apache/arrow-rs/issues/9460)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Python: avoid importing pyarrow classes ever time\n[#&#8203;9438](https://redirect.github.com/apache/arrow-rs/issues/9438)\n- Add `append_nulls` to `MapBuilder`\n[#&#8203;9431](https://redirect.github.com/apache/arrow-rs/issues/9431)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Add `append_non_nulls` to `StructBuilder`\n[#&#8203;9429](https://redirect.github.com/apache/arrow-rs/issues/9429)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Add `append_value_n` to GenericByteBuilder\n[#&#8203;9425](https://redirect.github.com/apache/arrow-rs/issues/9425)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Optimize `from_bitwise_binary_op`\n[#&#8203;9378](https://redirect.github.com/apache/arrow-rs/issues/9378)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Configurable Arrow representation of UTC timestamps for Avro reader\n[#&#8203;9279](https://redirect.github.com/apache/arrow-rs/issues/9279)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Fixed bugs:**\n\n- MutableArrayData::extend does not copy child values for ListView\narrays\n[#&#8203;9561](https://redirect.github.com/apache/arrow-rs/issues/9561)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- ListView interleave bug\n[#&#8203;9559](https://redirect.github.com/apache/arrow-rs/issues/9559)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Flight encoding panics with \"no dict id for field\" with nested dict\narrays\n[#&#8203;9555](https://redirect.github.com/apache/arrow-rs/issues/9555)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\\[[arrow-flight](https://redirect.github.com/apache/arrow-rs/labels/arrow-flight)]\n- \"DeltaBitPackDecoder only supports Int32Type and Int64Type\" but\nunsigned types are supported too\n[#&#8203;9551](https://redirect.github.com/apache/arrow-rs/issues/9551)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Potential overflow when calling `util::bit_mask::set_bits` (soundness\nissue)\n[#&#8203;9543](https://redirect.github.com/apache/arrow-rs/issues/9543)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- handle Null type in try\\_merge for Struct, List, LargeList, and Union\n[#&#8203;9523](https://redirect.github.com/apache/arrow-rs/issues/9523)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Invalid offset in sparse column chunk data for multiple predicates\n[#&#8203;9516](https://redirect.github.com/apache/arrow-rs/issues/9516)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- debug\\_assert\\_eq! in BatchCoalescer panics in debug mode when\nbatch\\_size < 4\n[#&#8203;9506](https://redirect.github.com/apache/arrow-rs/issues/9506)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Parquet Statistics::null\\_count\\_opt wrongly returns Some(0) when\nstats are missing\n[#&#8203;9451](https://redirect.github.com/apache/arrow-rs/issues/9451)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Error \"Not all children array length are the same!\" when decoding rows\nspanning across page boundaries in parquet file when using\n`RowSelection`\n[#&#8203;9370](https://redirect.github.com/apache/arrow-rs/issues/9370)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Avro schema resolution not properly supported for complex types\n[#&#8203;9336](https://redirect.github.com/apache/arrow-rs/issues/9336)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Documentation updates:**\n\n- Update planned release schedule in README.md\n[#&#8203;9466](https://redirect.github.com/apache/arrow-rs/pull/9466)\n([alamb](https://redirect.github.com/alamb))\n\n**Performance improvements:**\n\n- Introduce `NullBuffer::try_from_unsliced` to simplify array\nconstruction\n[#&#8203;9385](https://redirect.github.com/apache/arrow-rs/issues/9385)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- perf: Coalesce page fetches when RowSelection selects all rows\n[#&#8203;9578](https://redirect.github.com/apache/arrow-rs/pull/9578)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- Use chunks\\_exact for has\\_true/has\\_false to enable compiler\nunrolling\n[#&#8203;9570](https://redirect.github.com/apache/arrow-rs/pull/9570)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([adriangb](https://redirect.github.com/adriangb))\n- pyarrow: Cache the imported classes to avoid importing them each time\n[#&#8203;9439](https://redirect.github.com/apache/arrow-rs/pull/9439)\n([Tpt](https://redirect.github.com/Tpt))\n\n**Closed issues:**\n\n- Duplicate macro definition: `partially_shredded_variant_array_gen`\n[#&#8203;9492](https://redirect.github.com/apache/arrow-rs/issues/9492)\n- Enable `LargeList` / `ListView` / `LargeListView` for\n`VariantArray::try_new`\n[#&#8203;9455](https://redirect.github.com/apache/arrow-rs/issues/9455)\n- Support variables/expressions in record\\_batch! macro\n[#&#8203;9245](https://redirect.github.com/apache/arrow-rs/issues/9245)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Merged pull requests:**\n\n- \\[Variant] Add unshred\\_variant support for Binary and LargeBinary\ntypes\n[#&#8203;9576](https://redirect.github.com/apache/arrow-rs/pull/9576)\n([kunalsinghdadhwal](https://redirect.github.com/kunalsinghdadhwal))\n- \\[Variant] Add `variant_to_arrow` `Struct` type support\n[#&#8203;9572](https://redirect.github.com/apache/arrow-rs/pull/9572)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- Make Sbbf Constructers Public\n[#&#8203;9569](https://redirect.github.com/apache/arrow-rs/pull/9569)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([cetra3](https://redirect.github.com/cetra3))\n- fix: Used `checked_add` for bounds checks to avoid UB\n[#&#8203;9568](https://redirect.github.com/apache/arrow-rs/pull/9568)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([etseidl](https://redirect.github.com/etseidl))\n- Add mutable operations to BooleanBuffer (Bit\\*Assign)\n[#&#8203;9567](https://redirect.github.com/apache/arrow-rs/pull/9567)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- chore(deps): update lz4\\_flex requirement from 0.12 to 0.13\n[#&#8203;9565](https://redirect.github.com/apache/arrow-rs/pull/9565)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- arrow-select: fix MutableArrayData interleave for ListView\n[#&#8203;9560](https://redirect.github.com/apache/arrow-rs/pull/9560)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([asubiotto](https://redirect.github.com/asubiotto))\n- Move `ValueIter` into own module, and add public `record_count`\nfunction\n[#&#8203;9557](https://redirect.github.com/apache/arrow-rs/pull/9557)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Rafferty97](https://redirect.github.com/Rafferty97))\n- arrow-flight: generate dict\\_ids for dicts nested inside complex types\n[#&#8203;9556](https://redirect.github.com/apache/arrow-rs/pull/9556)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\\[[arrow-flight](https://redirect.github.com/apache/arrow-rs/labels/arrow-flight)]\n([asubiotto](https://redirect.github.com/asubiotto))\n- add `shred_variant` support for `LargeUtf8` and `LargeBinary`\n[#&#8203;9554](https://redirect.github.com/apache/arrow-rs/pull/9554)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- \\[minor] Download clickbench file when missing\n[#&#8203;9553](https://redirect.github.com/apache/arrow-rs/pull/9553)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- DeltaBitPackEncoderConversion: Fix panic message on invalid type\n[#&#8203;9552](https://redirect.github.com/apache/arrow-rs/pull/9552)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([progval](https://redirect.github.com/progval))\n- Replace interleave overflow panic with error\n[#&#8203;9549](https://redirect.github.com/apache/arrow-rs/pull/9549)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([xudong963](https://redirect.github.com/xudong963))\n- feat(arrow-avro): `HeaderInfo` to expose OCF header\n[#&#8203;9548](https://redirect.github.com/apache/arrow-rs/pull/9548)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- chore: Protect `main` branch with required reviews\n[#&#8203;9547](https://redirect.github.com/apache/arrow-rs/pull/9547)\n([comphead](https://redirect.github.com/comphead))\n- Add benchmark for `infer_json_schema`\n[#&#8203;9546](https://redirect.github.com/apache/arrow-rs/pull/9546)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Rafferty97](https://redirect.github.com/Rafferty97))\n- chore(deps): bump black from 24.3.0 to 26.3.1 in /parquet/pytest\n[#&#8203;9545](https://redirect.github.com/apache/arrow-rs/pull/9545)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- Unroll interleave -25-30%\n[#&#8203;9542](https://redirect.github.com/apache/arrow-rs/pull/9542)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- Optimize `take_fixed_size_binary` For Predefined Value Lengths\n[#&#8203;9535](https://redirect.github.com/apache/arrow-rs/pull/9535)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([tobixdev](https://redirect.github.com/tobixdev))\n- feat: expose arrow schema on async avro reader\n[#&#8203;9534](https://redirect.github.com/apache/arrow-rs/pull/9534)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- Make with\\_file\\_decryption\\_properties pub instead of pub(crate)\n[#&#8203;9532](https://redirect.github.com/apache/arrow-rs/pull/9532)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- fix: handle Null type in try\\_merge for Struct, List, LargeList, and\nUnion\n[#&#8203;9524](https://redirect.github.com/apache/arrow-rs/pull/9524)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([zhuqi-lucas](https://redirect.github.com/zhuqi-lucas))\n- chore: extend record\\_batch macro to support variables and expressions\n[#&#8203;9522](https://redirect.github.com/apache/arrow-rs/pull/9522)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([buraksenn](https://redirect.github.com/buraksenn))\n- \\[Variant] clean up `variant_get` tests\n[#&#8203;9518](https://redirect.github.com/apache/arrow-rs/pull/9518)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- support large string for unshred variant\n[#&#8203;9515](https://redirect.github.com/apache/arrow-rs/pull/9515)\n([friendlymatthew](https://redirect.github.com/friendlymatthew))\n- support string view unshred variant\n[#&#8203;9514](https://redirect.github.com/apache/arrow-rs/pull/9514)\n([friendlymatthew](https://redirect.github.com/friendlymatthew))\n- Add has\\_true() and has\\_false() to BooleanArray\n[#&#8203;9511](https://redirect.github.com/apache/arrow-rs/pull/9511)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([adriangb](https://redirect.github.com/adriangb))\n- Fix Invalid offset in sparse column chunk data error for multiple\npredicates\n[#&#8203;9509](https://redirect.github.com/apache/arrow-rs/pull/9509)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([cetra3](https://redirect.github.com/cetra3))\n- fix: remove incorrect debug assertion in BatchCoalescer\n[#&#8203;9508](https://redirect.github.com/apache/arrow-rs/pull/9508)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Tim-53](https://redirect.github.com/Tim-53))\n- \\[Json] Add benchmarks for list json reader\n[#&#8203;9507](https://redirect.github.com/apache/arrow-rs/pull/9507)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- fix: first next\\_back() on new RowsIter panics\n[#&#8203;9505](https://redirect.github.com/apache/arrow-rs/pull/9505)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([rluvaton](https://redirect.github.com/rluvaton))\n- Add some benchmarks for decoding delta encoded Parquet\n[#&#8203;9500](https://redirect.github.com/apache/arrow-rs/pull/9500)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([etseidl](https://redirect.github.com/etseidl))\n- chore: remove duplicate macro `partially_shredded_variant_array_gen`\n[#&#8203;9498](https://redirect.github.com/apache/arrow-rs/pull/9498)\n([codephage2020](https://redirect.github.com/codephage2020))\n- Deprecate ArrowTimestampType::make\\_value in favor of\nfrom\\_naive\\_datetime\n[#&#8203;9491](https://redirect.github.com/apache/arrow-rs/pull/9491)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([codephage2020](https://redirect.github.com/codephage2020))\n- fix: Do not assume missing nullcount stat means zero nullcount\n[#&#8203;9481](https://redirect.github.com/apache/arrow-rs/pull/9481)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Enahcne bracket access for VariantPath\n[#&#8203;9479](https://redirect.github.com/apache/arrow-rs/pull/9479)\n([klion26](https://redirect.github.com/klion26))\n- Optimize delta binary decoder in the case where bitwidth=0\n[#&#8203;9477](https://redirect.github.com/apache/arrow-rs/pull/9477)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([etseidl](https://redirect.github.com/etseidl))\n- Add PrimitiveRunBuilder::with\\_data\\_type() to customize the values'\nDataType\n[#&#8203;9473](https://redirect.github.com/apache/arrow-rs/pull/9473)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([brunal](https://redirect.github.com/brunal))\n- Convert `prettyprint` tests in `arrow-cast` to `insta` inline\nsnapshots\n[#&#8203;9472](https://redirect.github.com/apache/arrow-rs/pull/9472)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([grtlr](https://redirect.github.com/grtlr))\n- Update strum\\_macros requirement from 0.27 to 0.28\n[#&#8203;9471](https://redirect.github.com/apache/arrow-rs/pull/9471)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- docs(parquet): Fix broken links in README\n[#&#8203;9467](https://redirect.github.com/apache/arrow-rs/pull/9467)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([SYaoJun](https://redirect.github.com/SYaoJun))\n- Add list-like types support to VariantArray::try\\_new\n[#&#8203;9457](https://redirect.github.com/apache/arrow-rs/pull/9457)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- Simplify downcast\\_...!() macro definitions\n[#&#8203;9454](https://redirect.github.com/apache/arrow-rs/pull/9454)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([brunal](https://redirect.github.com/brunal))\n- feat(parquet): add content defined chunking for arrow writer\n[#&#8203;9450](https://redirect.github.com/apache/arrow-rs/pull/9450)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([kszucs](https://redirect.github.com/kszucs))\n- refactor: simplify iterator using cloned().map(Some)\n[#&#8203;9449](https://redirect.github.com/apache/arrow-rs/pull/9449)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([SYaoJun](https://redirect.github.com/SYaoJun))\n- feat: Optimize from\\_bitwise\\_binary\\_op with 64-bit alignment\n[#&#8203;9441](https://redirect.github.com/apache/arrow-rs/pull/9441)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([kunalsinghdadhwal](https://redirect.github.com/kunalsinghdadhwal))\n- docs: fix markdown link syntax in README\n[#&#8203;9440](https://redirect.github.com/apache/arrow-rs/pull/9440)\n([SYaoJun](https://redirect.github.com/SYaoJun))\n- Move `ListLikeArray` to arrow-array to be shared with json writer and\nparquet unshredding\n[#&#8203;9437](https://redirect.github.com/apache/arrow-rs/pull/9437)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Add `claim` method to recordbatch for memory accounting\n[#&#8203;9433](https://redirect.github.com/apache/arrow-rs/pull/9433)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([cetra3](https://redirect.github.com/cetra3))\n- Add `append_nulls` to `MapBuilder`\n[#&#8203;9432](https://redirect.github.com/apache/arrow-rs/pull/9432)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Fokko](https://redirect.github.com/Fokko))\n- Add `append_non_nulls` to `StructBuilder`\n[#&#8203;9430](https://redirect.github.com/apache/arrow-rs/pull/9430)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Fokko](https://redirect.github.com/Fokko))\n- Add `append_value_n` to GenericByteBuilder\n[#&#8203;9426](https://redirect.github.com/apache/arrow-rs/pull/9426)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Fokko](https://redirect.github.com/Fokko))\n- refactor: simplify dynamic state for Avro record projection\n[#&#8203;9419](https://redirect.github.com/apache/arrow-rs/pull/9419)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- Add `NullBuffer::from_unsliced_buffer` helper and refactor call sites\n[#&#8203;9411](https://redirect.github.com/apache/arrow-rs/pull/9411)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Eyad3skr](https://redirect.github.com/Eyad3skr))\n- Implement min, max, sum for run-end-encoded arrays.\n[#&#8203;9409](https://redirect.github.com/apache/arrow-rs/pull/9409)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([brunal](https://redirect.github.com/brunal))\n- feat: add `RunArray::new_unchecked` and `RunArray::into_parts`\n[#&#8203;9376](https://redirect.github.com/apache/arrow-rs/pull/9376)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([rluvaton](https://redirect.github.com/rluvaton))\n- Fix skip\\_records over-counting when partial record precedes num\\_rows\npage skip\n[#&#8203;9374](https://redirect.github.com/apache/arrow-rs/pull/9374)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([jonded94](https://redirect.github.com/jonded94))\n- fix: resolution of complex type variants in Avro unions\n[#&#8203;9328](https://redirect.github.com/apache/arrow-rs/pull/9328)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- feat(arrow-avro): Configurable Arrow timezone ID for Avro timestamps\n[#&#8203;9280](https://redirect.github.com/apache/arrow-rs/pull/9280)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n\n\\* *This Changelog was automatically generated by\n[github\\_changelog\\_generator](https://redirect.github.com/github-changelog-generator/github-changelog-generator)*\n\n###\n[`v58.0.0`](https://redirect.github.com/apache/arrow-rs/blob/HEAD/CHANGELOG.md#5810-2026-03-20)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-rs/compare/57.3.0...58.0.0)\n\n[Full\nChangelog](https://redirect.github.com/apache/arrow-rs/compare/58.0.0...58.1.0)\n\n**Implemented enhancements:**\n\n- Reuse compression dict lz4\\_block\n[#&#8203;9566](https://redirect.github.com/apache/arrow-rs/issues/9566)\n- \\[Variant] Add `variant_to_arrow` `Struct` type support\n[#&#8203;9529](https://redirect.github.com/apache/arrow-rs/issues/9529)\n- \\[Variant] Add `unshred_variant` support for `Binary` and\n`LargeBinary` types\n[#&#8203;9526](https://redirect.github.com/apache/arrow-rs/issues/9526)\n- \\[Variant] Add `shred_variant` support for `LargeUtf8` and\n`LargeBinary` types\n[#&#8203;9525](https://redirect.github.com/apache/arrow-rs/issues/9525)\n- \\[Variant] `variant_get` tests clean up\n[#&#8203;9517](https://redirect.github.com/apache/arrow-rs/issues/9517)\n- parquet\\_variant: Support LargeUtf8 typed value in `unshred_variant`\n[#&#8203;9513](https://redirect.github.com/apache/arrow-rs/issues/9513)\n- parquet-variant: Support string view typed value in `unshred_variant`\n[#&#8203;9512](https://redirect.github.com/apache/arrow-rs/issues/9512)\n- Deprecate ArrowTimestampType::make\\_value in favor of\nfrom\\_naive\\_datetime\n[#&#8203;9490](https://redirect.github.com/apache/arrow-rs/issues/9490)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Followup for support \\['fieldName'] in VariantPath\n[#&#8203;9478](https://redirect.github.com/apache/arrow-rs/issues/9478)\n- Speedup DELTA\\_BINARY\\_PACKED decoding when bitwidth is 0\n[#&#8203;9476](https://redirect.github.com/apache/arrow-rs/issues/9476)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Support CSV files encoded with charsets other than UTF-8\n[#&#8203;9465](https://redirect.github.com/apache/arrow-rs/issues/9465)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Expose Avro writer schema when building the reader\n[#&#8203;9460](https://redirect.github.com/apache/arrow-rs/issues/9460)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Python: avoid importing pyarrow classes ever time\n[#&#8203;9438](https://redirect.github.com/apache/arrow-rs/issues/9438)\n- Add `append_nulls` to `MapBuilder`\n[#&#8203;9431](https://redirect.github.com/apache/arrow-rs/issues/9431)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Add `append_non_nulls` to `StructBuilder`\n[#&#8203;9429](https://redirect.github.com/apache/arrow-rs/issues/9429)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Add `append_value_n` to GenericByteBuilder\n[#&#8203;9425](https://redirect.github.com/apache/arrow-rs/issues/9425)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Optimize `from_bitwise_binary_op`\n[#&#8203;9378](https://redirect.github.com/apache/arrow-rs/issues/9378)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Configurable Arrow representation of UTC timestamps for Avro reader\n[#&#8203;9279](https://redirect.github.com/apache/arrow-rs/issues/9279)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Fixed bugs:**\n\n- MutableArrayData::extend does not copy child values for ListView\narrays\n[#&#8203;9561](https://redirect.github.com/apache/arrow-rs/issues/9561)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- ListView interleave bug\n[#&#8203;9559](https://redirect.github.com/apache/arrow-rs/issues/9559)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Flight encoding panics with \"no dict id for field\" with nested dict\narrays\n[#&#8203;9555](https://redirect.github.com/apache/arrow-rs/issues/9555)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\\[[arrow-flight](https://redirect.github.com/apache/arrow-rs/labels/arrow-flight)]\n- \"DeltaBitPackDecoder only supports Int32Type and Int64Type\" but\nunsigned types are supported too\n[#&#8203;9551](https://redirect.github.com/apache/arrow-rs/issues/9551)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Potential overflow when calling `util::bit_mask::set_bits` (soundness\nissue)\n[#&#8203;9543](https://redirect.github.com/apache/arrow-rs/issues/9543)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- handle Null type in try\\_merge for Struct, List, LargeList, and Union\n[#&#8203;9523](https://redirect.github.com/apache/arrow-rs/issues/9523)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Invalid offset in sparse column chunk data for multiple predicates\n[#&#8203;9516](https://redirect.github.com/apache/arrow-rs/issues/9516)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- debug\\_assert\\_eq! in BatchCoalescer panics in debug mode when\nbatch\\_size < 4\n[#&#8203;9506](https://redirect.github.com/apache/arrow-rs/issues/9506)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Parquet Statistics::null\\_count\\_opt wrongly returns Some(0) when\nstats are missing\n[#&#8203;9451](https://redirect.github.com/apache/arrow-rs/issues/9451)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Error \"Not all children array length are the same!\" when decoding rows\nspanning across page boundaries in parquet file when using\n`RowSelection`\n[#&#8203;9370](https://redirect.github.com/apache/arrow-rs/issues/9370)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Avro schema resolution not properly supported for complex types\n[#&#8203;9336](https://redirect.github.com/apache/arrow-rs/issues/9336)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Documentation updates:**\n\n- Update planned release schedule in README.md\n[#&#8203;9466](https://redirect.github.com/apache/arrow-rs/pull/9466)\n([alamb](https://redirect.github.com/alamb))\n\n**Performance improvements:**\n\n- Introduce `NullBuffer::try_from_unsliced` to simplify array\nconstruction\n[#&#8203;9385](https://redirect.github.com/apache/arrow-rs/issues/9385)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- perf: Coalesce page fetches when RowSelection selects all rows\n[#&#8203;9578](https://redirect.github.com/apache/arrow-rs/pull/9578)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- Use chunks\\_exact for has\\_true/has\\_false to enable compiler\nunrolling\n[#&#8203;9570](https://redirect.github.com/apache/arrow-rs/pull/9570)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([adriangb](https://redirect.github.com/adriangb))\n- pyarrow: Cache the imported classes to avoid importing them each time\n[#&#8203;9439](https://redirect.github.com/apache/arrow-rs/pull/9439)\n([Tpt](https://redirect.github.com/Tpt))\n\n**Closed issues:**\n\n- Duplicate macro definition: `partially_shredded_variant_array_gen`\n[#&#8203;9492](https://redirect.github.com/apache/arrow-rs/issues/9492)\n- Enable `LargeList` / `ListView` / `LargeListView` for\n`VariantArray::try_new`\n[#&#8203;9455](https://redirect.github.com/apache/arrow-rs/issues/9455)\n- Support variables/expressions in record\\_batch! macro\n[#&#8203;9245](https://redirect.github.com/apache/arrow-rs/issues/9245)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\n**Merged pull requests:**\n\n- \\[Variant] Add unshred\\_variant support for Binary and LargeBinary\ntypes\n[#&#8203;9576](https://redirect.github.com/apache/arrow-rs/pull/9576)\n([kunalsinghdadhwal](https://redirect.github.com/kunalsinghdadhwal))\n- \\[Variant] Add `variant_to_arrow` `Struct` type support\n[#&#8203;9572](https://redirect.github.com/apache/arrow-rs/pull/9572)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- Make Sbbf Constructers Public\n[#&#8203;9569](https://redirect.github.com/apache/arrow-rs/pull/9569)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([cetra3](https://redirect.github.com/cetra3))\n- fix: Used `checked_add` for bounds checks to avoid UB\n[#&#8203;9568](https://redirect.github.com/apache/arrow-rs/pull/9568)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([etseidl](https://redirect.github.com/etseidl))\n- Add mutable operations to BooleanBuffer (Bit\\*Assign)\n[#&#8203;9567](https://redirect.github.com/apache/arrow-rs/pull/9567)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- chore(deps): update lz4\\_flex requirement from 0.12 to 0.13\n[#&#8203;9565](https://redirect.github.com/apache/arrow-rs/pull/9565)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- arrow-select: fix MutableArrayData interleave for ListView\n[#&#8203;9560](https://redirect.github.com/apache/arrow-rs/pull/9560)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([asubiotto](https://redirect.github.com/asubiotto))\n- Move `ValueIter` into own module, and add public `record_count`\nfunction\n[#&#8203;9557](https://redirect.github.com/apache/arrow-rs/pull/9557)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Rafferty97](https://redirect.github.com/Rafferty97))\n- arrow-flight: generate dict\\_ids for dicts nested inside complex types\n[#&#8203;9556](https://redirect.github.com/apache/arrow-rs/pull/9556)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n\\[[arrow-flight](https://redirect.github.com/apache/arrow-rs/labels/arrow-flight)]\n([asubiotto](https://redirect.github.com/asubiotto))\n- add `shred_variant` support for `LargeUtf8` and `LargeBinary`\n[#&#8203;9554](https://redirect.github.com/apache/arrow-rs/pull/9554)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- \\[minor] Download clickbench file when missing\n[#&#8203;9553](https://redirect.github.com/apache/arrow-rs/pull/9553)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- DeltaBitPackEncoderConversion: Fix panic message on invalid type\n[#&#8203;9552](https://redirect.github.com/apache/arrow-rs/pull/9552)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([progval](https://redirect.github.com/progval))\n- Replace interleave overflow panic with error\n[#&#8203;9549](https://redirect.github.com/apache/arrow-rs/pull/9549)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([xudong963](https://redirect.github.com/xudong963))\n- feat(arrow-avro): `HeaderInfo` to expose OCF header\n[#&#8203;9548](https://redirect.github.com/apache/arrow-rs/pull/9548)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- chore: Protect `main` branch with required reviews\n[#&#8203;9547](https://redirect.github.com/apache/arrow-rs/pull/9547)\n([comphead](https://redirect.github.com/comphead))\n- Add benchmark for `infer_json_schema`\n[#&#8203;9546](https://redirect.github.com/apache/arrow-rs/pull/9546)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Rafferty97](https://redirect.github.com/Rafferty97))\n- chore(deps): bump black from 24.3.0 to 26.3.1 in /parquet/pytest\n[#&#8203;9545](https://redirect.github.com/apache/arrow-rs/pull/9545)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- Unroll interleave -25-30%\n[#&#8203;9542](https://redirect.github.com/apache/arrow-rs/pull/9542)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- Optimize `take_fixed_size_binary` For Predefined Value Lengths\n[#&#8203;9535](https://redirect.github.com/apache/arrow-rs/pull/9535)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([tobixdev](https://redirect.github.com/tobixdev))\n- feat: expose arrow schema on async avro reader\n[#&#8203;9534](https://redirect.github.com/apache/arrow-rs/pull/9534)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- Make with\\_file\\_decryption\\_properties pub instead of pub(crate)\n[#&#8203;9532](https://redirect.github.com/apache/arrow-rs/pull/9532)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([Dandandan](https://redirect.github.com/Dandandan))\n- fix: handle Null type in try\\_merge for Struct, List, LargeList, and\nUnion\n[#&#8203;9524](https://redirect.github.com/apache/arrow-rs/pull/9524)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([zhuqi-lucas](https://redirect.github.com/zhuqi-lucas))\n- chore: extend record\\_batch macro to support variables and expressions\n[#&#8203;9522](https://redirect.github.com/apache/arrow-rs/pull/9522)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([buraksenn](https://redirect.github.com/buraksenn))\n- \\[Variant] clean up `variant_get` tests\n[#&#8203;9518](https://redirect.github.com/apache/arrow-rs/pull/9518)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- support large string for unshred variant\n[#&#8203;9515](https://redirect.github.com/apache/arrow-rs/pull/9515)\n([friendlymatthew](https://redirect.github.com/friendlymatthew))\n- support string view unshred variant\n[#&#8203;9514](https://redirect.github.com/apache/arrow-rs/pull/9514)\n([friendlymatthew](https://redirect.github.com/friendlymatthew))\n- Add has\\_true() and has\\_false() to BooleanArray\n[#&#8203;9511](https://redirect.github.com/apache/arrow-rs/pull/9511)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([adriangb](https://redirect.github.com/adriangb))\n- Fix Invalid offset in sparse column chunk data error for multiple\npredicates\n[#&#8203;9509](https://redirect.github.com/apache/arrow-rs/pull/9509)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([cetra3](https://redirect.github.com/cetra3))\n- fix: remove incorrect debug assertion in BatchCoalescer\n[#&#8203;9508](https://redirect.github.com/apache/arrow-rs/pull/9508)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Tim-53](https://redirect.github.com/Tim-53))\n- \\[Json] Add benchmarks for list json reader\n[#&#8203;9507](https://redirect.github.com/apache/arrow-rs/pull/9507)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- fix: first next\\_back() on new RowsIter panics\n[#&#8203;9505](https://redirect.github.com/apache/arrow-rs/pull/9505)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([rluvaton](https://redirect.github.com/rluvaton))\n- Add some benchmarks for decoding delta encoded Parquet\n[#&#8203;9500](https://redirect.github.com/apache/arrow-rs/pull/9500)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([etseidl](https://redirect.github.com/etseidl))\n- chore: remove duplicate macro `partially_shredded_variant_array_gen`\n[#&#8203;9498](https://redirect.github.com/apache/arrow-rs/pull/9498)\n([codephage2020](https://redirect.github.com/codephage2020))\n- Deprecate ArrowTimestampType::make\\_value in favor of\nfrom\\_naive\\_datetime\n[#&#8203;9491](https://redirect.github.com/apache/arrow-rs/pull/9491)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([codephage2020](https://redirect.github.com/codephage2020))\n- fix: Do not assume missing nullcount stat means zero nullcount\n[#&#8203;9481](https://redirect.github.com/apache/arrow-rs/pull/9481)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([scovich](https://redirect.github.com/scovich))\n- \\[Variant] Enahcne bracket access for VariantPath\n[#&#8203;9479](https://redirect.github.com/apache/arrow-rs/pull/9479)\n([klion26](https://redirect.github.com/klion26))\n- Optimize delta binary decoder in the case where bitwidth=0\n[#&#8203;9477](https://redirect.github.com/apache/arrow-rs/pull/9477)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([etseidl](https://redirect.github.com/etseidl))\n- Add PrimitiveRunBuilder::with\\_data\\_type() to customize the values'\nDataType\n[#&#8203;9473](https://redirect.github.com/apache/arrow-rs/pull/9473)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([brunal](https://redirect.github.com/brunal))\n- Convert `prettyprint` tests in `arrow-cast` to `insta` inline\nsnapshots\n[#&#8203;9472](https://redirect.github.com/apache/arrow-rs/pull/9472)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([grtlr](https://redirect.github.com/grtlr))\n- Update strum\\_macros requirement from 0.27 to 0.28\n[#&#8203;9471](https://redirect.github.com/apache/arrow-rs/pull/9471)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([dependabot\\[bot\\]](https://redirect.github.com/apps/dependabot))\n- docs(parquet): Fix broken links in README\n[#&#8203;9467](https://redirect.github.com/apache/arrow-rs/pull/9467)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([SYaoJun](https://redirect.github.com/SYaoJun))\n- Add list-like types support to VariantArray::try\\_new\n[#&#8203;9457](https://redirect.github.com/apache/arrow-rs/pull/9457)\n([sdf-jkl](https://redirect.github.com/sdf-jkl))\n- Simplify downcast\\_...!() macro definitions\n[#&#8203;9454](https://redirect.github.com/apache/arrow-rs/pull/9454)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([brunal](https://redirect.github.com/brunal))\n- feat(parquet): add content defined chunking for arrow writer\n[#&#8203;9450](https://redirect.github.com/apache/arrow-rs/pull/9450)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([kszucs](https://redirect.github.com/kszucs))\n- refactor: simplify iterator using cloned().map(Some)\n[#&#8203;9449](https://redirect.github.com/apache/arrow-rs/pull/9449)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([SYaoJun](https://redirect.github.com/SYaoJun))\n- feat: Optimize from\\_bitwise\\_binary\\_op with 64-bit alignment\n[#&#8203;9441](https://redirect.github.com/apache/arrow-rs/pull/9441)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([kunalsinghdadhwal](https://redirect.github.com/kunalsinghdadhwal))\n- docs: fix markdown link syntax in README\n[#&#8203;9440](https://redirect.github.com/apache/arrow-rs/pull/9440)\n([SYaoJun](https://redirect.github.com/SYaoJun))\n- Move `ListLikeArray` to arrow-array to be shared with json writer and\nparquet unshredding\n[#&#8203;9437](https://redirect.github.com/apache/arrow-rs/pull/9437)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([liamzwbao](https://redirect.github.com/liamzwbao))\n- Add `claim` method to recordbatch for memory accounting\n[#&#8203;9433](https://redirect.github.com/apache/arrow-rs/pull/9433)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([cetra3](https://redirect.github.com/cetra3))\n- Add `append_nulls` to `MapBuilder`\n[#&#8203;9432](https://redirect.github.com/apache/arrow-rs/pull/9432)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Fokko](https://redirect.github.com/Fokko))\n- Add `append_non_nulls` to `StructBuilder`\n[#&#8203;9430](https://redirect.github.com/apache/arrow-rs/pull/9430)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Fokko](https://redirect.github.com/Fokko))\n- Add `append_value_n` to GenericByteBuilder\n[#&#8203;9426](https://redirect.github.com/apache/arrow-rs/pull/9426)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Fokko](https://redirect.github.com/Fokko))\n- refactor: simplify dynamic state for Avro record projection\n[#&#8203;9419](https://redirect.github.com/apache/arrow-rs/pull/9419)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- Add `NullBuffer::from_unsliced_buffer` helper and refactor call sites\n[#&#8203;9411](https://redirect.github.com/apache/arrow-rs/pull/9411)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([Eyad3skr](https://redirect.github.com/Eyad3skr))\n- Implement min, max, sum for run-end-encoded arrays.\n[#&#8203;9409](https://redirect.github.com/apache/arrow-rs/pull/9409)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([brunal](https://redirect.github.com/brunal))\n- feat: add `RunArray::new_unchecked` and `RunArray::into_parts`\n[#&#8203;9376](https://redirect.github.com/apache/arrow-rs/pull/9376)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([rluvaton](https://redirect.github.com/rluvaton))\n- Fix skip\\_records over-counting when partial record precedes num\\_rows\npage skip\n[#&#8203;9374](https://redirect.github.com/apache/arrow-rs/pull/9374)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n([jonded94](https://redirect.github.com/jonded94))\n- fix: resolution of complex type variants in Avro unions\n[#&#8203;9328](https://redirect.github.com/apache/arrow-rs/pull/9328)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n- feat(arrow-avro): Configurable Arrow timezone ID for Avro timestamps\n[#&#8203;9280](https://redirect.github.com/apache/arrow-rs/pull/9280)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([mzabaluev](https://redirect.github.com/mzabaluev))\n\n\\* *This Changelog was automatically generated by\n[github\\_changelog\\_generator](https://redirect.github.com/github-changelog-generator/github-changelog-generator)*\n\n###\n[`v57.3.0`](https://redirect.github.com/apache/arrow-rs/releases/tag/57.3.0):\narrow 57.3.0\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-rs/compare/57.2.0...57.3.0)\n\n<!---\n  Licensed to the Apache Software Foundation (ASF) under one\n  or more contributor license agreements.  See the NOTICE file\n  distributed with this work for additional information\n  regarding copyright ownership.  The ASF licenses this file\n  to you under the Apache License, Version 2.0 (the\n  \"License\"); you may not use this file except in compliance\n  with the License.  You may obtain a copy of the License at\n\n    http://www.apache.org/licenses/LICENSE-2.0\n\n  Unless required by applicable law or agreed to in writing,\n  software distributed under the License is distributed on an\n  \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY\n  KIND, either express or implied.  See the License for the\n  specific language governing permissions and limitations\n  under the License.\n-->\n\n### Changelog\n\n#### [57.3.0](https://redirect.github.com/apache/arrow-rs/tree/57.3.0)\n(2026-02-02)\n\n[Full\nChangelog](https://redirect.github.com/apache/arrow-rs/compare/57.2.0...57.3.0)\n\n**Breaking changes:**\n\n- Revert \"Seal Array trait\", mark `Array` as `unsafe`\n[#&#8203;9313](https://redirect.github.com/apache/arrow-rs/pull/9313)\n([alamb](https://redirect.github.com/alamb),\n[gabotechs](https://redirect.github.com/gabotechs))\n- Mark `BufferBuilder::new_from_buffer` as unsafe\n[#&#8203;9312](https://redirect.github.com/apache/arrow-rs/pull/9312)\n([alamb](https://redirect.github.com/alamb),\n[Jefffrey](https://redirect.github.com/Jefffrey))\n\n**Fixed bugs:**\n\n- Fix string array equality when the values buffer is the same and only\nthe offsets to access it differ\n[#&#8203;9330](https://redirect.github.com/apache/arrow-rs/pull/9330)\n([alamb](https://redirect.github.com/alamb),\n[jhorstmann](https://redirect.github.com/jhorstmann))\n- Ensure `BufferBuilder::truncate` doesn't overset length\n[#&#8203;9311](https://redirect.github.com/apache/arrow-rs/pull/9311)\n([alamb](https://redirect.github.com/alamb),\n[Jefffrey](https://redirect.github.com/Jefffrey))\n- \\[parquet] Provide only encrypted column stats in plaintext footer\n[#&#8203;9310](https://redirect.github.com/apache/arrow-rs/pull/9310)\n([alamb](https://redirect.github.com/alamb),\n[rok](https://redirect.github.com/rok),\n[adamreeve](https://redirect.github.com/adamreeve))\n- \\[regression] Error with adaptive predicate pushdown: \"Invalid offset\n…\" [#&#8203;9309](https://redirect.github.com/apache/arrow-rs/pull/9309)\n([alamb](https://redirect.github.com/alamb),\n[erratic-pattern](https://redirect.github.com/erratic-pattern),\n[sdf-jkl](https://redirect.github.com/sdf-jkl))\n\n###\n[`v57.2.0`](https://redirect.github.com/apache/arrow-rs/releases/tag/57.2.0):\narrow 57.2.0\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-rs/compare/57.1.0...57.2.0)\n\n<!---\n  Licensed to the Apache Software Foundation (ASF) under one\n  or more contributor license agreements.  See the NOTICE file\n  distributed with this work for additional information\n  regarding copyright ownership.  The ASF licenses this file\n  to you under the Apache License, Version 2.0 (the\n  \"License\"); you may not use this file except in compliance\n  with the License.  You may obtain a copy of the License at\n\n    http://www.apache.org/licenses/LICENSE-2.0\n\n  Unless required by applicable law or agreed to in writing,\n  software distributed under the License is distributed on an\n  \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY\n  KIND, either express or implied.  See the License for the\n  specific language governing permissions and limitations\n  under the License.\n-->\n\n### Changelog\n\n#### [57.2.0](https://redirect.github.com/apache/arrow-rs/tree/57.2.0)\n(2026-01-07)\n\n[Full\nChangelog](https://redirect.github.com/apache/arrow-rs/compare/57.1.0...57.2.0)\n\n**Breaking changes:**\n\n- Seal Array trait\n[#&#8203;9092](https://redirect.github.com/apache/arrow-rs/pull/9092)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n([tustvold](https://redirect.github.com/tustvold))\n- \\[Variant] Unify the CastOptions usage in parquet-variant-compute\n[#&#8203;8984](https://redirect.github.com/apache/arrow-rs/pull/8984)\n([klion26](https://redirect.github.com/klion26))\n\n**Implemented enhancements:**\n\n- \\[parquet] further relax `LevelInfoBuilder::types_compatible` for\n`ArrowWriter`\n[#&#8203;9098](https://redirect.github.com/apache/arrow-rs/issues/9098)\n- Update arrow-row documentation with Union encoding\n[#&#8203;9084](https://redirect.github.com/apache/arrow-rs/issues/9084)\n- Add code examples for min and max compute functions\n[#&#8203;9055](https://redirect.github.com/apache/arrow-rs/issues/9055)\n- Add `append_n` to bytes view builder API\n[#&#8203;9034](https://redirect.github.com/apache/arrow-rs/issues/9034)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Move `RunArray::get_physical_indices` to `RunEndBuffer`\n[#&#8203;9025](https://redirect.github.com/apache/arrow-rs/issues/9025)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Allow quote style in csv writer\n[#&#8203;9003](https://redirect.github.com/apache/arrow-rs/issues/9003)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- IPC support for ListView\n[#&#8203;9002](https://redirect.github.com/apache/arrow-rs/issues/9002)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Implement `BinaryArrayType` for `&FixedSizeBinaryArray`s\n[#&#8203;8992](https://redirect.github.com/apache/arrow-rs/issues/8992)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- arrow-buffer: implement num-traits for i256\n[#&#8203;8976](https://redirect.github.com/apache/arrow-rs/issues/8976)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Support for `Arc<str>` in `ParquetRecordWriter` derive macro\n[#&#8203;8972](https://redirect.github.com/apache/arrow-rs/issues/8972)\n- \\[arrow-avro] suggest switching from xz to liblzma\n[#&#8203;8970](https://redirect.github.com/apache/arrow-rs/issues/8970)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- arrow-buffer: add i256::trailing\\_zeros\n[#&#8203;8968](https://redirect.github.com/apache/arrow-rs/issues/8968)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- arrow-buffer: make i256::leading\\_zeros public\n[#&#8203;8965](https://redirect.github.com/apache/arrow-rs/issues/8965)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Add spark like `ignoreLeadingWhiteSpace` and\n`ignoreTrailingWhiteSpace` options to the csv writer\n[#&#8203;8961](https://redirect.github.com/apache/arrow-rs/issues/8961)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Add round trip benchmark for Parquet writer/reader\n[#&#8203;8955](https://redirect.github.com/apache/arrow-rs/issues/8955)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Support performant `interleave` for List/LargeList\n[#&#8203;8952](https://redirect.github.com/apache/arrow-rs/issues/8952)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Variant] Support array access when parsing `VariantPath`\n[#&#8203;8946](https://redirect.github.com/apache/arrow-rs/issues/8946)\n- Some panic!s could be represented as unimplemented!s\n[#&#8203;8932](https://redirect.github.com/apache/arrow-rs/issues/8932)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Variant] easier way to construct a shredded schema\n[#&#8203;8922](https://redirect.github.com/apache/arrow-rs/issues/8922)\n- Support `DataType::ListView` and `DataType::LargeListView` in\n`ArrayData::new_null`\n[#&#8203;8908](https://redirect.github.com/apache/arrow-rs/issues/8908)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Add `GenericListViewArray::from_iter_primitive`\n[#&#8203;8906](https://redirect.github.com/apache/arrow-rs/issues/8906)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Variant] Unify the cast option usage in ParquentVariant\n[#&#8203;8873](https://redirect.github.com/apache/arrow-rs/issues/8873)\n- Blog post about efficient filter representation in Parquet filter\npushdown\n[#&#8203;8843](https://redirect.github.com/apache/arrow-rs/issues/8843)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n- Add comparison support for Union arrays in the `cmp` kernel\n[#&#8203;8837](https://redirect.github.com/apache/arrow-rs/issues/8837)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Variant] Support array shredding into\n`List/LargeList/ListView/LargeListView`\n[#&#8203;8830](https://redirect.github.com/apache/arrow-rs/issues/8830)\n- Support `Union` data types for row format\n[#&#8203;8828](https://redirect.github.com/apache/arrow-rs/issues/8828)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- FFI support for ListView\n[#&#8203;8819](https://redirect.github.com/apache/arrow-rs/issues/8819)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[Variant] Support more Arrow Datatypes from Variant primitive types\n[#&#8203;8805](https://redirect.github.com/apache/arrow-rs/issues/8805)\n- `FixedSizeBinaryBuilder` supports `append_array`\n[#&#8203;8750](https://redirect.github.com/apache/arrow-rs/issues/8750)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Implement special case `zip` with scalar for Utf8View\n[#&#8203;8724](https://redirect.github.com/apache/arrow-rs/issues/8724)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[geometry] Wire up arrow reader/writer for `GEOMETRY` and `GEOGRAPHY`\n[#&#8203;8717](https://redirect.github.com/apache/arrow-rs/issues/8717)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\n**Fixed bugs:**\n\n- Soundness Bug in `try_binary` when `Array` is implemented incorrectly\nin external crate\n[#&#8203;9106](https://redirect.github.com/apache/arrow-rs/issues/9106)\n- casting `Dict(_, LargeUtf8)` to `Utf8View` (`StringViewArray`) panics\n[#&#8203;9101](https://redirect.github.com/apache/arrow-rs/issues/9101)\n- wrong results for null count of `nullif` kernel\n[#&#8203;9085](https://redirect.github.com/apache/arrow-rs/issues/9085)\n\\[[parquet](https://redirect.github.com/apache/arrow-rs/labels/parquet)]\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Empty first line in some code examples\n[#&#8203;9063](https://redirect.github.com/apache/arrow-rs/issues/9063)\n- GenericByteViewArray::slice is not zero-copy but ought to be\n[#&#8203;9014](https://redirect.github.com/apache/arrow-rs/issues/9014)\n- Regression in struct casting in 57.2.0 (not yet released)\n[#&#8203;9005](https://redirect.github.com/apache/arrow-rs/issues/9005)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Fix panic when decoding multiple Union columns in RowConverter\n[#&#8203;8999](https://redirect.github.com/apache/arrow-rs/issues/8999)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- `take_fixed_size_binary` Does Not Consider NULL Indices\n[#&#8203;8947](https://redirect.github.com/apache/arrow-rs/issues/8947)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- \\[arrow-avro] RecordEncoder Bugs\n[#&#8203;8934](https://redirect.github.com/apache/arrow-rs/issues/8934)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- `FixedSizeBinaryArray::try_new(...)` Panics with Item Length of Zero\n[#&#8203;8926](https://redirect.github.com/apache/arrow-rs/issues/8926)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- `cargo test -p arrow-cast` fails on main\n[#&#8203;8910](https://redirect.github.com/apache/arrow-rs/issues/8910)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- `GenericListViewArray::new_null` ignores `len` and returns an empty\narray\n[#&#8203;8904](https://redirect.github.com/apache/arrow-rs/issues/8904)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- `FixedSizeBinaryArray::new_null` Does Not Properly Set the Length of\nthe Values Buffer\n[#&#8203;8900](https://redirect.github.com/apache/arrow-rs/issues/8900)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Struct casting requires same order of fields\n[#&#8203;8870](https://redirect.github.com/apache/arrow-rs/issues/8870)\n\\[[arrow](https://redirect.github.com/apache/arrow-rs/labels/arrow)]\n- Cannot cast string dictionary to binary view\n[#&#8203;8841](https://redirect.github.com/apache/arrow-rs/issues/8841)\n\\[[arrow](https://r\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-20T12:19:40Z",
          "tree_id": "eb5f830458b7a0dde746cbc6d47fb91511350f00",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e6b0c45febd8d61b394478536242d1b6b5f6d349"
        },
        "date": 1776698698652,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7914937734603882,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.12862973922611,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.36041230721537,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.470052083333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.1484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 643552.5419381408,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 638458.8637123452,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008502,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16931825.7348066,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16938422.582094226,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3358bf14fb0429edd59e77df130f503744feee21",
          "message": "Update module go.opentelemetry.io/collector/pdata to v1.56.0 (#2708)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.55.0` → `v1.56.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.56.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.55.0/v1.56.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.56.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1560v01500)\n\n##### 💡 Enhancements 💡\n\n- `all`: Update semconv package from 1.38.0 to 1.40.0\n([#&#8203;15095](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15095))\n- `cmd/mdatagen`: Only allow the `ToVersion` feature flag attribute to\nbe set for the `Stable` and `Deprecated` stages.\n([#&#8203;15040](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15040))\n  To better match the feature flag README\n\n(<https://github.com/open-telemetry/opentelemetry-collector/blob/main/featuregate/README.md#feature-lifecycle>).\n\n##### 🧰 Bug fixes 🧰\n\n- `exporter/debug`: Guard from out of bounds profiles dictionary indices\n([#&#8203;14803](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14803))\n\n- `pdata/pprofile`: create a copy when the input is marked as read-only\n([#&#8203;15080](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15080))\n\n- `pkg/otelcol`: Fix missing default values in unredacted print-config\ncommand by introducing confmap.WithUnredacted MarshalOption.\n([#&#8203;14750](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14750))\nResolves an issue where the unredacted mode output omitted all\ndefault-valued options. By introducing a new MarshalOption to disable\nredaction directly at the confmap encoding level, the unredacted mode\nnow preserves all component defaults natively without requiring\npost-processing.\n\n- `pkg/service`: Headers on the internal telemetry OTLP exporter are now\nredacted when the configuration is marshaled\n([#&#8203;14756](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14756))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-20T14:06:29Z",
          "tree_id": "d69c74ae358d381032ef8d55d2454dc7ad881fa2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3358bf14fb0429edd59e77df130f503744feee21"
        },
        "date": 1776702447148,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6853066682815552,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.95008159792712,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.21034267912772,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.175911458333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 638755.8651558387,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 643133.3015856307,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002242,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16961662.724418245,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16969029.921479974,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c7a9bac88f1b3ad20eb2c7877032eac9860021af",
          "message": "Update Rust crate datafusion to v53 (#2712)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [datafusion](https://datafusion.apache.org)\n([source](https://redirect.github.com/apache/datafusion)) | dependencies\n| major | `49.0.2` → `53.0.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>apache/datafusion (datafusion)</summary>\n\n###\n[`v53.1.0`](https://redirect.github.com/apache/datafusion/compare/53.0.0...53.1.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/53.0.0...53.1.0)\n\n###\n[`v53.0.0`](https://redirect.github.com/apache/datafusion/compare/52.5.0...53.0.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/52.5.0...53.0.0)\n\n###\n[`v52.5.0`](https://redirect.github.com/apache/datafusion/compare/52.4.0...52.5.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/52.4.0...52.5.0)\n\n###\n[`v52.4.0`](https://redirect.github.com/apache/datafusion/compare/52.3.0...52.4.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/52.3.0...52.4.0)\n\n###\n[`v52.3.0`](https://redirect.github.com/apache/datafusion/compare/52.2.0...52.3.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/52.2.0...52.3.0)\n\n###\n[`v52.2.0`](https://redirect.github.com/apache/datafusion/compare/52.1.0...52.2.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/52.1.0...52.2.0)\n\n###\n[`v52.1.0`](https://redirect.github.com/apache/datafusion/compare/52.0.0...52.1.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/52.0.0...52.1.0)\n\n###\n[`v52.0.0`](https://redirect.github.com/apache/datafusion/compare/51.0.0...52.0.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/51.0.0...52.0.0)\n\n###\n[`v51.0.0`](https://redirect.github.com/apache/datafusion/compare/50.3.0...51.0.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/50.3.0...51.0.0)\n\n###\n[`v50.3.0`](https://redirect.github.com/apache/datafusion/compare/50.2.0...50.3.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/50.2.0...50.3.0)\n\n###\n[`v50.2.0`](https://redirect.github.com/apache/datafusion/compare/50.1.0...50.2.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/50.1.0...50.2.0)\n\n###\n[`v50.1.0`](https://redirect.github.com/apache/datafusion/compare/50.0.0...50.1.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/50.0.0...50.1.0)\n\n###\n[`v50.0.0`](https://redirect.github.com/apache/datafusion/compare/49.0.2...50.0.0)\n\n[Compare\nSource](https://redirect.github.com/apache/datafusion/compare/49.0.2...50.0.0)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-20T14:15:39Z",
          "tree_id": "cbee85adc83c0a88d85db05d695115974ea726b0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c7a9bac88f1b3ad20eb2c7877032eac9860021af"
        },
        "date": 1776706002242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8346166610717773,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.82207686504158,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.29022535429412,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.70546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 620589.6048029312,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 625769.1489965757,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002191,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16610302.05083665,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16610868.145672232,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sapatrjv@gmail.com",
            "name": "sapatrjv",
            "username": "sapatrjv"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6a056e9564e989c4239ffd5a194783acb67bb57e",
          "message": "Crypto library selection condition simplification (#2705)\n\n# Change Summary\n\n\n## What issue does this PR close?\n\nPart of #2537\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-20T15:43:37Z",
          "tree_id": "013f011240f4a688f09efc00cd7bacd27d4bbd80",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6a056e9564e989c4239ffd5a194783acb67bb57e"
        },
        "date": 1776709898088,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6934775710105896,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.00670065686236,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.20700973141301,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.912369791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.41796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 643521.0221539632,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 647983.6962803077,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003485,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16983732.816497825,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16993813.324129123,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1e251126b4f960b4022cfa65b59ee21a50764def",
          "message": "test: Add JUnit XML upload and flaky test tracking workflow (#2699)\n\n# Change Summary\n\n- Implemented JUnit XML result uploads for both required and\nnon-required tests in the Rust-CI workflow.\n- Created a new workflow for detecting flaky tests from JUnit XML\nartifacts, which runs daily and on-demand.\n- The flaky test tracker parses JUnit results, identifies flaky tests,\nand creates or updates a tracking issue with a summary.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a. This is adding a new flaky test detection workflow. No changes to\nproduct code.\n\n## Are there any user-facing changes?\n\nNo. Test/infra only.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-20T18:48:02Z",
          "tree_id": "403cb31eede9ba6233f67c162ad67bac545e7b6f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1e251126b4f960b4022cfa65b59ee21a50764def"
        },
        "date": 1776715766373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4278502464294434,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.09116645590419,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.36040055461409,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.072395833333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 640706.0612878907,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656261.4457050045,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003403,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17038567.65305444,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17045926.3258866,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "cff3b1956af17af2a1a15d758165e08daf6a384a",
          "message": "fix(dataflow): isolate blocked routes in content and signal type routers (#2694)\n\n# Change Summary\n\nThis PR fixes a router liveness bug identified during the live\nreconfiguration and shutdown work.\n\nThe issue was that `content_router` could block the whole router task\nwhen one selected output route became blocked, which prevented unrelated\nhealthy routes from continuing to make progress. While addressing that\nbug, this PR also applies the same selected-route admission model to\n`signal_type_router`, since it has the same exclusive-routing shape:\neach input message selects at most one output route.\n\nFor both `content_router` and `signal_type_router`:\n\n- selected matched/named/default routes now use non-blocking admission\n- `Closed` selected routes produce an immediate retryable route-local\nNACK with `NackCause::RouteClosed`\n- `Full` selected routes follow `admission_policy.on_full`\n- the default `reject_immediately` policy emits an immediate retryable\nroute-local NACK with `NackCause::RouteFull`\n- the optional `backpressure` policy parks at most one message per\nblocked selectable route, keeps healthy routes flowing, and pauses pdata\nadmission only when all selectable routes are currently blocked\n- the processor returns `Ok(())` for route-local rejection paths, so one\nblocked route does not fail or stall the whole router task\n- router-owned parked work is retryable-NACKed with\n`NackCause::NodeShutdown` when shutdown starts\n\nThis PR also:\n\n- adds engine-level selected-route admission outcomes used by\nnon-blocking sends\n- adds a shared core-nodes exclusive-router scheduler for the two\nexclusive routers\n- adds route-full and route-closed rejection telemetry\n- adds regression coverage for route `Full` / `Closed` behavior, route\nisolation, backpressure, shutdown, and Ack/Nack propagation behavior\n- documents processor classes in `docs/processors.md` and the\nrouter-specific guarantees in the `content_router` and\n`signal_type_router` READMEs\n\n## What issue does this PR close?\n\n* Closes #2693\n\n## How are these changes tested?\n\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nYes.\n\nOperators now get explicit retryable route-local NACKs when the selected\nroute in `content_router` or `signal_type_router` is unavailable\n(`Closed`) or rejected by the configured full-route admission policy.\n\nThis PR also adds optional router-local configuration:\n\n```yaml\n  admission_policy:\n    on_full: reject_immediately # or \"backpressure\"\n```\n\nreject_immediately is the default. backpressure allows a router to\ntolerate temporarily slow routes by parking one message per blocked\nroute while unrelated healthy routes continue to flow.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-20T20:52:02Z",
          "tree_id": "ca4fa19f0c27fb93ebf60941d095bd81f8cbaf1d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cff3b1956af17af2a1a15d758165e08daf6a384a"
        },
        "date": 1776723144464,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6942582726478577,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.02701945960501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.24810873786407,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.99921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 636650.3901554648,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 641070.3883072437,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003645,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17060514.982611712,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17068019.35050667,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "913bdecd515798e0311df8a7422bbfbeba42777e",
          "message": "Fix flaky durable_buffer tests subject to timing issues to use condition-based shutdowns (#2715)\n\n# Change Summary\n\nRefactor `test_durable_buffer_recovery_after_outage` and\n`test_durable_buffer_otlp_item_count_metrics` to use condition-based\nshutdown instead of relying on internal quiver timings to improve test\nreliability.\n\n## What issue does this PR close?\n\n* Closes #2701 \n\n## How are these changes tested?\n\n* Verified that tests pass locally when run in a loop. Will need to rely\non CI results over time via #2699 to gain confidence in test\nreliability.\n\n## Are there any user-facing changes?\n\nNo. Test-only change.",
          "timestamp": "2026-04-20T21:10:13Z",
          "tree_id": "edfaac6b301cb214e1bf57576414817280922f26",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/913bdecd515798e0311df8a7422bbfbeba42777e"
        },
        "date": 1776727523449,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.089410662651062,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.01350793868777,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.37353159851301,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.66822916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.90625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 604650.9488567343,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 611238.0810044736,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005476,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16022585.154076297,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16023291.997171313,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "044a9084f0abaab469f17ea9da155c4a6ab2b010",
          "message": "Include response body in RateLimited/ServerError Display for azure_monitor_exporter (#2716)\n\nThe `Display` impl for `Error::RateLimited` and `Error::ServerError`\npreviously omitted the HTTP response body, losing actionable details\nwhen these errors were logged at WARN level via Display formatting\n(`%`).\n\nFor example, when the main export path exhausts retries on a 429, the\nWARN log showed:\n```\nExport failed after 5 attempts: Rate limited\n```\n\nWith this change it now shows:\n```\nExport failed after 5 attempts: Rate limited: {\"error\":{\"code\":\"QuotaLimitExceeded\",\"message\":\"Allowed data-rate per data collection rule: 2GB/minute\"}}\n```\n\nThis lets operators identify *which* Azure Monitor quota was hit\n(data-rate vs request-rate per DCR) without enabling debug logging.\n\nAdditionally, the heartbeat failure WARN log in `exporter.rs` is changed\nfrom Debug format (`?e`) to Display format (`%e`), aligning with the\nproject's [events\nguide](rust/otap-dataflow/docs/telemetry/events-guide.md) recommendation\nto prefer Display for `thiserror` types at warn/error severity.\n\n**Changes:**\n- `error.rs`: Include `{body}` in `#[error(...)]` for `RateLimited` and\n`ServerError` variants; update corresponding test assertions.\n- `exporter.rs`: Change heartbeat send failure WARN log from `error =\n?e` (Debug) to `error = %e` (Display).",
          "timestamp": "2026-04-20T21:32:46Z",
          "tree_id": "fb081413ce25a0da69c1a02a5adf035bbda60685",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/044a9084f0abaab469f17ea9da155c4a6ab2b010"
        },
        "date": 1776728533786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9671982526779175,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.05419815547873,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3662497100889,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.515234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 637827.2218414781,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 643996.275263345,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005316,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16999612.54614763,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17008909.307954,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5a0d3fb508582e0922747323347ab66c85351dd3",
          "message": "Update docker.io/rust Docker tag to v1.95 (#2707)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.94` → `1.95` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-20T21:52:46Z",
          "tree_id": "22b951c113a9a67388f778cdbdd9e641f89203f5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a0d3fb508582e0922747323347ab66c85351dd3"
        },
        "date": 1776729546667,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4980173110961914,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.07279198343551,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.36946716232961,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.424869791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.3515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 639093.2767606822,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 642276.0716954825,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002609,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17100280.822534688,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17108232.837533403,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0be250ec882dbe231ec43d2b64b995d670e8290a",
          "message": "Update pipeline perf python dependencies (major) (#2642)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pandas](https://redirect.github.com/pandas-dev/pandas) | `==2.3.3` →\n`==3.0.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pandas/3.0.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pandas/2.3.3/3.0.2?slim=true)\n|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==23.0.1` →\n`==24.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/24.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/23.0.1/24.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pandas-dev/pandas (pandas)</summary>\n\n###\n[`v3.0.2`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v3.0.2):\npandas 3.0.2\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v3.0.1...v3.0.2)\n\nWe are pleased to announce the release of pandas 3.0.2.\nThis is a patch release in the 3.0.x series and includes some regression\nfixes and bug fixes. We recommend that all users of the 3.0.x series\nupgrade to this version.\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/docs/dev/whatsnew/v3.0.2.html) for a\nlist of all the changes.\n\nPandas 3.0 supports Python 3.11 and higher.\nThe release can be installed from PyPI:\n\n```\npython -m pip install --upgrade pandas==3.0.*\n```\n\nOr from conda-forge\n\n```\nconda install -c conda-forge pandas=3.0\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues).\n\nThanks to all the contributors who made this release possible.\n\n###\n[`v3.0.1`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v3.0.1):\npandas 3.0.1\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v3.0.0...v3.0.1)\n\nWe are pleased to announce the release of pandas 3.0.1.\nThis is a patch release in the 3.0.x series and includes some regression\nfixes and bug fixes. We recommend that all users of the 3.0.x series\nupgrade to this version.\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/docs/dev/whatsnew/v3.0.1.html) for a\nlist of all the changes.\n\nPandas 3.0.0 supports Python 3.11 and higher.\nThe release can be installed from PyPI:\n\n```\npython -m pip install --upgrade pandas==3.0.*\n```\n\nOr from conda-forge\n\n```\nconda install -c conda-forge pandas=3.0\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues).\n\nThanks to all the contributors who made this release possible.\n\n###\n[`v3.0.0`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v3.0.0):\npandas 3.0.0\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v2.3.3...v3.0.0)\n\nWe are pleased to announce the release of pandas 3.0.0, a major release\nfrom the pandas 2.x series. This release includes some new features, bug\nfixes, and performance improvements, as well as possible breaking\nchanges.\n\nThe pandas 3.0 release removed a functionality that was deprecated in\nprevious releases. It is recommended to first upgrade to pandas 2.3 and\nto ensure your code is working without warnings, before upgrading to\npandas 3.0.\n\nHighlights include:\n\n- [Dedicated string data type by\ndefault](https://pandas.pydata.org/docs/whatsnew/v3.0.0.html#whatsnew-300-enhancements-string-dtype)\n- [Consistent copy/view behaviour with\nCopy-on-Write](https://pandas.pydata.org/docs/whatsnew/v3.0.0.html#whatsnew-300-enhancements-copy_on_write)\n(CoW) (a.k.a. getting rid of the SettingWithCopyWarning)\n- [New default resolution for datetime-like\ndata](https://pandas.pydata.org/docs/whatsnew/v3.0.0.html#whatsnew-300-api-breaking-datetime-resolution-inference)\n- [Initial support for the new `pd.col`\nsyntax](https://pandas.pydata.org/docs/whatsnew/v3.0.0.html#whatsnew-300-enhancements-col)\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/docs/whatsnew/v3.0.0.html) for a\nlist of all the changes.\n\nPandas 3.0.0 supports Python 3.11 and higher.\nThe release can be installed from PyPI\n\n```\npython -m pip install --upgrade pandas==3.0.*\n```\n\nOr from conda-forge\n\n```\nconda install -c conda-forge pandas=3.0\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues/new/choose).\n\nThanks to all the contributors who made this release possible.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMTAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-21T13:07:33Z",
          "tree_id": "5b75e0f59356088f299be4ab8fecb70940410563",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0be250ec882dbe231ec43d2b64b995d670e8290a"
        },
        "date": 1776779747588,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7270122170448303,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.0851629229857,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42170985259891,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.200130208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.51953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 643179.9230261119,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 647855.9196539733,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003465,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16996445.066286094,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17004714.19307852,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "693b34e1972d186270e39913fdadd6b8c3751bf9",
          "message": "Transport header support in OTLP/Fake Data receivers (#2702)\n\n# Change Summary\n\nUpdate the OTLP receiver to use the capture header policy when defined,\nextracts data from grpc/http headers\n\nUpdate the Fake Data Generator to allow user to specify transport\nheaders to tack on to generated OtapPdata, users can provide key/value\npairs or just the key (random value will be generated).\n\n## What issue does this PR close?\n\n* Closes #2692\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-21T17:41:03Z",
          "tree_id": "16f6fe477cca8f5b9341f48e97be70ac7bfbd2a6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/693b34e1972d186270e39913fdadd6b8c3751bf9"
        },
        "date": 1776796349481,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5823151469230652,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.09017798151017,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35427863777089,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.458203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.41015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 641825.4933677461,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 645562.9405684464,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002453,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17076795.743833102,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17084349.281636372,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dd344850b0de4426753c6c5ac7ca8786a2545458",
          "message": "[query-engine] Tweak slice validation errors (#2721)\n\nRelates to #2636\n\n# Changes\n\n* Tweak the slice validation error messages\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-21T19:39:49Z",
          "tree_id": "35ab1c2a68ff14f3578a6ca9a77b2fe11985a7f5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd344850b0de4426753c6c5ac7ca8786a2545458"
        },
        "date": 1776803768852,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0892438888549805,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.02137437251974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.2631756078734,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.035677083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644728.0189831292,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 651750.679486102,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002331,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17029581.354624867,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17034248.91299256,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3a6d2a3b70b838918f335bbc21b28c0fdab54f25",
          "message": "fix(benchmarks): Bump max_decoding_message_size to 32MiB to fix batch processor benchmarks (#2730)\n\n# Change Summary\n\nBatch processor benchmarks had 100% signal drop rate due to being over\nthe decompression limit on the backend engine. Bumping the limit fixes\nthe issue for both continuous and nightly.\n\n## What issue does this PR close?\n\n* Closes #2729\n\n## How are these changes tested?\n\nRan all scenarios locally and observed the dropped rate being 0 (or\nless):\n\n<img width=\"1891\" height=\"466\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6af43bf9-2e61-4af9-a48b-8285f8768a92\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-04-22T15:57:57Z",
          "tree_id": "f4b3db7e6103917dd690d8754f8c8704c9ded822",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3a6d2a3b70b838918f335bbc21b28c0fdab54f25"
        },
        "date": 1776899377494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6432966589927673,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.0679941582389,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43394861445316,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.611197916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.09765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 640673.0920105905,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 644794.5206278506,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002495,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16970659.758545212,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16978142.24030389,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ebf2403b5810741def8e0725e17e3fafbefc2458",
          "message": "test: enhance flaky test reporting by adding JUnit artifact metadata and improving job link resolution (#2722)\n\n# Change Summary\n\nFix misc. issues in the flaky test reporting:\n\n- Add links to the failed jobs to ease investigations (relies on a new\n`metadata.json` file included in test output)\n- Fix an issue where the :new: icon was missing from new flaky test\nresults\n- Fix truncation of test names in output\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nValidated flaky test report output with local runs. Will monitor Flaky\nTest report post PR completion.\n\n## Are there any user-facing changes?\n\nNo. Test infra only.",
          "timestamp": "2026-04-22T18:36:38Z",
          "tree_id": "3fcf1e63c528169db56aee69033d704dbd2276cd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ebf2403b5810741def8e0725e17e3fafbefc2458"
        },
        "date": 1776901788420,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.3920433521270752,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.01031097936723,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.32971238080471,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.828385416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.83984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648609.3349334016,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 651152.1647735604,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002442,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16985170.31397602,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16992997.33870363,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pritishnahar@gmail.com",
            "name": "Pritish Nahar",
            "username": "pritishnahar95"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "446e2510d411d9944a3a37faba577e2029677e8b",
          "message": "feat(config): add optional field to EngineConfig (#2727)\n\nAllow applications embedding the dataflow engine to carry their own\nengine-level configuration under `engine.custom`. The engine ignores\nthis field entirely; embedding binaries can read namespaced keys for\nconcerns like remote management, auth, or fleet coordination.\n\n# Change Summary\n\nAdd an optional `custom: HashMap<String, serde_json::Value>` field to\n`EngineConfig`. This gives embedding applications an escape hatch for\nengine-level config without forking the config crate or pre-parsing\nYAML. The field defaults to an empty map and is omitted from serialized\noutput when empty.\n\n## What issue does this PR close?\n\n* Closes #2561\n\n## How are these changes tested?\n\nThree new unit tests in `engine.rs`:\n- `from_yaml_accepts_custom_config` — parses a config with multiple\nnamespaced custom keys and verifies values\n- `custom_defaults_to_empty` — confirms the field defaults to an empty\nmap when omitted\n- `custom_roundtrips_through_json` — serializes to JSON and deserializes\nback, verifying data is preserved\n\n## Are there any user-facing changes?\n\nYes. A new optional `custom` key is available under the `engine` section\nof the YAML/JSON config. Example:\n\n```yaml\nengine:\n  custom:\n    remote_management:\n      server_url: \"ws://mgmt.example.com/v1\"\n      heartbeat_interval_secs: 10\n    custom_auth:\n      provider: \"oidc\"\n      token_endpoint: \"https://auth.example.com/token\"\n```\n\nExisting configs are unaffected since the field defaults to empty.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-22T23:05:42Z",
          "tree_id": "7ff523f0db00e256fa8054e2b84e659ca88ddb4f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/446e2510d411d9944a3a37faba577e2029677e8b"
        },
        "date": 1776903539568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6024177074432373,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.07009810388064,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43102066083726,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.366666666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.6328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 634572.6892298746,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 638395.467676968,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002431,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17052210.041635428,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17058733.44872035,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8f6fa92280658b62989971f21ab1000610b55d5d",
          "message": "feat: Make fake data generator pacing more reliable and update sql reports to query correct time frame (#2723)\n\n# Change Summary\n\nThis PR is an attempt to solve #2713 with three connected changes.\n\nThe first is to make the reported data production rate coming out of the\nfake data generator smoother and more reliable especially under\nbackpressure by using tokio intervals which are less prone to drift and\nhave a catch-up mechanism.\n\nThe second makes sure we report metrics in smaller increments after\nevery batch is exported. #2685 made some progress in this area, but as\nnoted there it was an incomplete solution which held back metrics for\nprolonged periods of time in addition to having to heap allocate the\nsend futures. I went after the full solution to the problem in this PR.\nNow we exclusively do non-blocking sends one batch at a time and always\nbump counters in-between on successful sends.\n\nThe third is against the rate calculations which were using the total\nproduced signals from the run, but computing the rate from the\nobservation time interval only. This means that we artificially inflated\nthe overall rate by making it look like it took a shorter time to\nachieve than it did. We correct this by computing the rate for the\nobservation window only.\n\nOther small fixes:\n\n- Added two production modes for the fake data generator which can do\neither a smooth production loop or an open loop in case the interval to\ndo a smooth loop is too small.\n- Updated some try_from implementations.\n\n## What issue does this PR close?\n\n* Closes #2713\n\n## How are these changes tested?\n\nRan the benchmarks locally and we can see that the produced and received\nrates are _very_ close to 100klrps now as opposed to 120k.\n\n<img width=\"1393\" height=\"310\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/1360f96e-8b23-48ef-98e7-f3a73cd27721\"\n/>\n\n\n## Are there any user-facing changes?\n\nNew fake data generator `production_mode` setting.\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-04-23T14:45:02Z",
          "tree_id": "0350a7d43d3b13f9ade00ffdf398db468dd41399",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8f6fa92280658b62989971f21ab1000610b55d5d"
        },
        "date": 1776959422735,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.749362657324763,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.109238954429794,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.851692708333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.346968251147,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.346968251147,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002513,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 210979.93663970375,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175318.30224909083,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gscalderon99@hotmail.com",
            "name": "gscalderon",
            "username": "gscalderon"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "fdb01e34326b9333bdae32a5da036701f186dd24",
          "message": "perf: [durable_buffer] Avoid unnecessary copy of OTLP bytes in OtlpBytesAdapter::new() (#2726)\n\nReplace `BinaryArray::from_vec()` which deep-copies the entire OTLP\npayload with a zero-copy construction using\n`Buffer::from(bytes::Bytes)`. The `clone_bytes()` call is just an Arc\nrefcount bump, and `Buffer::from(Bytes)` wraps the data without copying,\neliminating a full memcpy on the ingest hot path.\n \n # Change Summary\n \n- **Zero-copy wrapping**: Use `Buffer::from(bytes::Bytes)` instead of\n`BinaryArray::from_vec()` to avoid deep-copying the OTLP payload into\nArrow.\n- **Bounds check**: Added explicit `i32::try_from` guard on payload\nlength for a clear error on oversized payloads.\n- **New tests**: `test_otlp_bytes_adapter_zero_copy` (pointer equality\nassertion) and `test_otlp_bytes_adapter_empty_payload`.\n \n ## What issue does this PR close?\n \n * Closes #2703\n \n ## How are these changes tested?\n \n- Existing tests (`test_otlp_bytes_adapter`, `test_extract_otlp_bytes`)\nverify correctness is preserved.\n- New `test_otlp_bytes_adapter_zero_copy` asserts the Arrow buffer\npoints to the same memory as the original `OtlpProtoBytes` (no copy).\n- New `test_otlp_bytes_adapter_empty_payload` verifies empty payload\nhandling.\n \n ## Are there any user-facing changes?\n \nNo. This is a transparent performance improvement with no API or\nbehavior changes.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-04-23T17:12:28Z",
          "tree_id": "97ad4f4c3830e21488120cedb141c8109a833aa0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fdb01e34326b9333bdae32a5da036701f186dd24"
        },
        "date": 1776986852222,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.731981107520921,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.031980319803198,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.971354166666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.90625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.340725793852,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.340725793852,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002575,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 210520.52323074316,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174869.9956056961,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "101909410+sjmsft@users.noreply.github.com",
            "name": "Sameer J",
            "username": "sjmsft"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9c54c8e469806b54ec6fa8b9721790c951ca0a97",
          "message": "Task 1614: Fix tests that are ignored on Windows and MacOS (#2724)\n\n# Change Summary\n\nFix tests that are ignored on Windows and MacOS\n\n## What issue does this PR close?\n\n* Closes #1614\n\n## How are these changes tested?\n\nBy running the reenabled tests.\n\n## Are there any user-facing changes?\n\nYes, one production code change:\n\nWindows CA certificate hot-reload fix - The get_file_identity() function\nnow uses MetadataExt::last_write_time() (100ns-precision FILETIME) on\nWindows instead of the platform-fallback get_mtime() which truncated to\nwhole seconds. Previously, if a CA certificate file was replaced within\nthe same second (e.g., by automation or cert-manager), the file watcher\ncould miss the change entirely, leaving stale CA certificates in memory\nuntil the next reload event. This affects any Windows deployment using\nmTLS with watch_client_ca: true.\n\nAll other changes are test-only (removing #[ignore] attributes and\nfixing test assertions for cross-platform compatibility).",
          "timestamp": "2026-04-23T17:12:49Z",
          "tree_id": "085e4dabbf1d45dbb12dbf594bc1cd0752232a94",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9c54c8e469806b54ec6fa8b9721790c951ca0a97"
        },
        "date": 1776988037713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.7120410949253735,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.322890370600093,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.698828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.16015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.361567596799,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.361567596799,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002368,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211181.20381954865,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175492.17509181474,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0c30214c41fdfb38302b635e7fe451b4cf3ff081",
          "message": "[query-engine] Rename log-specific methods in bridge (#2746)\n\nRelates to #2736\n\n# Changes\n\n* Adds \"logs\" into names of bridge methods which are specific to logs\nprocessing\n\n# Details\n\nPrepping for #2736 which seems like it will require \"metrics\" APIs in\nbridge.",
          "timestamp": "2026-04-23T20:33:44Z",
          "tree_id": "b9d86a856696104df84a80c427fdb00431aa6605",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c30214c41fdfb38302b635e7fe451b4cf3ff081"
        },
        "date": 1776993109901,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5617977380752563,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.7608202955962255,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.3160089444058904,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.032942708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.27734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6075.4966927371515,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.6286966289335,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002337,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 209855.8830600124,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174173.7821801354,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sapatrjv@gmail.com",
            "name": "sapatrjv",
            "username": "sapatrjv"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "da7688be82431d7cf4508c7376036fb240785034",
          "message": "Use latest weaver build and simplify crypto selection. (#2740)\n\n# Change Summary\n\n<!--\nUse latest weaver build that has the exclusion of openssl build in case\nof windows platforms. In case of windows platforms it uses SChannel TLS\ninstead of natively building openssl.\n\nSimplification of crypto selection.\n\n-->\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/2697\n\n## How are these changes tested?\n\nSearch cargo tree and check on windows platform no openssl dependency.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-24T00:33:27Z",
          "tree_id": "0eddea84eced611b6b262fd7a578e17671ac76d6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/da7688be82431d7cf4508c7376036fb240785034"
        },
        "date": 1776994142559,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.42553189396858215,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.721445592233943,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.944230411892227,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.794010416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.08984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6015.732099397173,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.330959394609,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002672,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 209868.94766511724,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174247.4122471642,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e07aba3d39195c8c4127fcb32f77cc10fd5617b2",
          "message": "fix: update shutdown endpoint URL in perf test templates (#2780)\n\n## What issue does this PR close?\n\nCloses #2774\n\n## Change Summary\n\nThe shutdown API endpoint was renamed from\n`/api/v1/pipeline-groups/shutdown` to `/api/v1/groups/shutdown`, causing\npipeline perf tests to fail with 404 on the shutdown endpoint.\n\nThis PR updates all 3 perf test template files:\n- `df-loadgen-steps-docker-filtered.yaml` (3 URLs)\n- `df-loadgen-steps-docker-otel.yaml` (1 URL)\n- `df-loadgen-steps-docker.yaml` (3 URLs)\n\n## How are these changes tested?\n\nThe pipeline perf test CI job will validate the fix.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-04-29T17:37:42Z",
          "tree_id": "dd40c734024bcd5cec6a406eb010024ce225c0c6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e07aba3d39195c8c4127fcb32f77cc10fd5617b2"
        },
        "date": 1777489896835,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5384615659713745,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.709888483101367,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.1149725423466625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.934505208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.11328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6101.074139366979,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6007.211460299794,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002549,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212343.6546661269,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175374.73168152483,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pritishnahar@gmail.com",
            "name": "Pritish Nahar",
            "username": "pritishnahar95"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c77442588054522e7de6eb3a4083ff45140adb6b",
          "message": "feat(metrics): add scope_attributes to MetricSelector for view filtering (#2755)\n\nExpand MetricSelector to support scope_attributes as a new selector\ndimension. When configured, the view only applies to instruments whose\nscope contains all specified attribute key-value pairs.\n\n# Change Summary\n\nAdd `scope_attributes: Option<HashMap<String, String>>` to\n`MetricSelector` in the config crate, and implement corresponding\nAND-based filter logic in `DeclarativeView::to_view_funtion()` in the\ntelemetry crate. Each configured key-value pair must be present on the\ninstrument's scope for the view to match.\n\n## What issue does this PR close?\n\n* Closes #2742\n\n## How are these changes tested?\n\n- Added `test_view_config_with_scope_attributes` in `views.rs` to verify\nYAML deserialization of the new field.\n- Added `test_views_provider_configure_with_scope_attributes` in\n`views_provider.rs` to verify that a view with scope attributes can be\nregistered on the meter provider.\n- Updated all existing test struct literals to include\n`scope_attributes: None`.\n\n## Are there any user-facing changes?\n\nYes. Users can now configure `scope_attributes` in their YAML view\nselectors to filter by scope-level metadata:\n\n```yaml\nviews:\n  - selector:\n      scope_name: \"my.library\"\n      scope_attributes:\n        env: \"production\"\n    stream:\n      description: \"Production histograms\"\n```\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-29T18:30:13Z",
          "tree_id": "38b9d594a016cac42dbca29f3c455e2cb5e3a6be",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c77442588054522e7de6eb3a4083ff45140adb6b"
        },
        "date": 1777493992181,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9900990128517151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.7542782909503645,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.243101876882676,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.769661458333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.208886562998,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6091.933727024017,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008532,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213273.56237122478,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176378.75664862656,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30638925+marcsnid@users.noreply.github.com",
            "name": "Marc Snider",
            "username": "marcsnid"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "92b39f8c06f710fc7d0f4ac4f39e8ac9dc02dfa8",
          "message": "Fix: skip boolean column when all values are false for optional arrays (#2759)\n\n# Change Summary\n\nFix `AdaptiveBooleanArrayBuilder` to skip producing an array when all\nvalues are `false` or null for optional columns. Previously, the builder\nonly skipped on all-null:`false` values triggered array creation even\nthough `false` is the boolean default value. This makes boolean columns\nconsistent with how integer (`0`) and string (`\"\"`) columns handle\ndefaults.\n\nChanges:\n- Added `has_true_value` and `optional` fields to\n`AdaptiveBooleanArrayBuilder`\n- `finish()` now returns `None` when `optional` and  `!has_true_value`\n- Updated `test_metrics_round_trip` to reflect that the `is_monotonic`\ncolumn (all `false`/null) is now correctly omitted\n\n## What issue does this PR close?\ncloses #1449\n\n## How are these changes tested?\n\n- Added 4 new unit tests in `boolean.rs`:\n- `test_adaptive_boolean_builder_all_false` — all-false optional →\n`None`\n- `test_adaptive_boolean_builder_false_and_null` — mixed false+null\noptional → `None`\n- `test_adaptive_boolean_builder_false_then_true` — false then true →\n`Some(array)` with correct values\n- `test_adaptive_boolean_builder_all_false_non_optional` — non-optional\nalways produces array\n- All existing tests in `otap-df-pdata` continue to pass (including the\nupdated `test_metrics_round_trip`).\n\n## Are there any user-facing changes?\n\nNo. This is an internal encoding optimization. Optional boolean columns\nthat contain only default values (`false`/null) are no longer included\nin Arrow record batches, reducing payload size.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-29T19:14:26Z",
          "tree_id": "77767c46ca2232bb5db0e6c40f21986f84856cee",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/92b39f8c06f710fc7d0f4ac4f39e8ac9dc02dfa8"
        },
        "date": 1777495028864,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2955524921417236,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.750827733963406,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.211584632001235,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.037239583333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5946.068533245167,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6082.563650220666,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.016799,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212233.66375381665,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175266.55041791502,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "77f6f4b3305f44a949e94c45ef4fef3e0958f3de",
          "message": "Add capability in query-engine to filter by type of signal (#2754)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds the capability to use a syntax like `is <signal type>` in logical\nexpressions in OPL. This can be used for example when we want to have a\nsingle program that does different operations to each signal:\n```kotlin\nsignals |\nif (is Log) {\n // ...\n} else  if (is Metric) {\n  // ...\n} else if (is Span) {\n  // ...\n}\n```\n\nAlternatively, we can also do things like this to just keep/drop certain\nsignal types:\n```kql\nsignals | where is Log\nsignals | where not(is Log)\n```\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Part of #2752\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes this syntax is now available for use in transform processor\n\n <!-- If yes, provide further info below -->\n\n## Future work:\n\nIn a followup, I'll add support for checking where a particular field is\nof some type.\n```\nlogs | if (attributes[\"x\"] is String) {\n  // ...\n```\nThere are some TODOs in this PR to add support for this in the near\nfuture.\n\nAlso, when we eventually add more capability to the parser/planner to be\ntype aware, it may have the capability to reject the use of invalid\nfield accesses for certain signal types, and the syntax in this PR will\nallow users to get around these compile time checks. However, that's not\nimplemented as part of this PR.",
          "timestamp": "2026-04-29T19:14:44Z",
          "tree_id": "4048b83ad8419e262dd9414d082e83a63e3841ad",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/77f6f4b3305f44a949e94c45ef4fef3e0958f3de"
        },
        "date": 1777497490933,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.28328612446784973,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.710018634662272,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.127743831696187,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.17734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.4140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6024.170778655304,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6041.2364182548945,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003611,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213193.72908230912,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176312.62138650287,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "77021922+luckymachi@users.noreply.github.com",
            "name": "Max Jacinto",
            "username": "luckymachi"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "775794600fb4ba7bea406b4e0bb08cd598b10cda",
          "message": "Isolation of setup-protoc jobs from RUST-CI (#2772)\n\n# Change Summary\n\nAs mentioned in issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768),\n`setup-protoc` jobs are dropped in favour of a targeted `compile_proto`\njob.\n\n## What issue does this PR close?\n\nThis issue closes issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768)\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-29T20:45:20Z",
          "tree_id": "3218d1260faf004c7f0e40d88f9d2177b8c90d78",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/775794600fb4ba7bea406b4e0bb08cd598b10cda"
        },
        "date": 1777498711779,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7052186131477356,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.73637703759269,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.126636384633151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.880598958333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.30078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.81128171277,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.47567721145,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003194,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212324.60309199654,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175358.85094290556,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "524b78de5762aaebe99e31d67d3a2351d4d9e303",
          "message": "Make pipeline perf test a required CI check (#2779)\n\n## Summary\n\nMake the pipeline performance test a required CI check so that PRs which\nbreak the perf test are caught before merge.\n\n> **Dependency**: #2780 must be merged first (it fixes the currently\nbroken perf test).\n\n#2774 is an example of the kind of breakage this prevents — a route\nrename broke the perf test but the PR still merged because the perf test\nwas not a required check.\n\n### Changes\n\n- **rust-ci.yml**: Add `pipeline_perf_test` job (runs on\n`ubuntu-latest`) and include it in `rust-required-status-check`\naggregator\n- **pipeline-perf-on-label.yaml**: Simplify to only run on dedicated\nOracle bare-metal hardware when `pipelineperf` label is present — the\nbasic validation path is removed since `rust-ci.yml` now covers it\n\n### Motivation\n\nThe pipeline perf test has been broken by merged PRs several times\nbecause it was not a required check. This change ensures that if a PR\nbreaks the perf test (e.g. build failures, config issues, test\ninfrastructure breakage), it is caught before merge.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-30T12:25:55Z",
          "tree_id": "a4bb78d0fc1fd33cd2619de4d3d5919d2a29c650",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/524b78de5762aaebe99e31d67d3a2351d4d9e303"
        },
        "date": 1777555539040,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.5665722489356995,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.670716913199773,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.059177409453652,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.207552083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.54296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6023.921202540469,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5989.791337370268,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006097,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211662.28747753275,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 174940.0717428281,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "42ae9be9d1c6474c3b9fe07d61958ef88286f340",
          "message": "fix(tests): increase timeouts for CI stability in durable buffer tests (#2793)\n\n# Change Summary\n\nThis change increases some of the timeouts for the\ndurable_buffer_processor tests to improve stability on slow CI systems\nwhere pipeline initialization takes longer than expected.\n\n## What issue does this PR close?\n\n* Addresses a flaky test reported in #2720. Includes fixes for similar\ntests with the same potential timing issue.\n\n## How are these changes tested?\n\nWas able to manually reproduce the failure reported in #2720. Validated\nthat tests pass on local runs.\n\n## Are there any user-facing changes?\n\nNo. This is a test-only change.",
          "timestamp": "2026-04-30T18:23:10Z",
          "tree_id": "82377ac06cdabb023198543c68aa755fe3734b08",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/42ae9be9d1c6474c3b9fe07d61958ef88286f340"
        },
        "date": 1777578614697,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5602837800979614,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.740982775770179,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.3131065339813315,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.903515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6015.660516224868,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.521885981568,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003386,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214127.5359762369,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177228.18833103057,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "0884c8c9d267ef131a68e208191adba0c4679ce9",
          "message": "Validation framework fix equivalence check for metrics (#2776)\n\n# Change Summary\n\nThis adds additional validation tests to check that all signals are\npreserved through otlp/otap receiver/exporters.\n\nAdds canonicalize_buckets fn to set values from None to Some(Buckets\n{offset: 0, bucket_counts: vec![]}). Before the validation tests\ninvolving metrics and otap receiver/exporters would fail, but now they\nare succeeding\n\n## What issue does this PR close?\n\n* Closes #2786\n\n## How are these changes tested?\n\nvalidation test\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-30T18:31:18Z",
          "tree_id": "1738573548c52c15c3eb31768b9559338bd5b52c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0884c8c9d267ef131a68e208191adba0c4679ce9"
        },
        "date": 1777586944071,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.703234851360321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.73870144245342,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.233022037293882,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.167708333333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.4375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6066.5595735276875,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.221736492017,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006334,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214024.00320831416,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176900.03338986047,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c8804f567d53c195a25f3be40080e281edcb2314",
          "message": "docs: consolidate CLAUDE.md and copilot-instructions.md into AGENTS.md (#2791)\n\nBoth `CLAUDE.md` and `.github/copilot-instructions.md` were stubs that\nsimply redirected to\n[rust/otap-dataflow/AGENTS.md](rust/otap-dataflow/AGENTS.md).\n\nModern AI coding agents (Claude Code, GitHub Copilot, Cursor, etc.) all\nsupport a single top-level `AGENTS.md` file, so this consolidates the\ntwo stubs into one.",
          "timestamp": "2026-04-30T18:59:36Z",
          "tree_id": "4647005286ec671e7c219b6c24b342409c74ba3e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c8804f567d53c195a25f3be40080e281edcb2314"
        },
        "date": 1777589045242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9887005090713501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.740904093832289,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.202992944095526,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.544921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.75,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.3450552386685,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5981.614242545631,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002532,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212410.94907589044,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175491.70640271768,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "107717825+opentelemetrybot@users.noreply.github.com",
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7af33d651e8e9db5b2a7775d0cb106c1e12027c1",
          "message": "chore: Move inactive members to emeritus (#2789)\n\n## Move inactive members to emeritus\n\nThe following members have had no activity in\n`open-telemetry/otel-arrow` since **2025-12-31** and are being moved to\nemeritus:\n\n- @v0y4g3r (Approver, Remove from team(s): arrow-approvers,\narrow-triagers)\n\n> [!IMPORTANT]\n> After merging, remove the user(s) from:\n> - The listed team(s) in GitHub\n> - Any relevant private channels on Slack\n> - Any relevant package managers used for publishing\n\nThis PR was automatically generated by the [move-to-emeritus\nworkflow](https://github.com/open-telemetry/community/actions/workflows/move-to-emeritus.yml).",
          "timestamp": "2026-04-30T19:03:28Z",
          "tree_id": "1b55f64569a732ae3704f30b49e3638d170946dc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7af33d651e8e9db5b2a7775d0cb106c1e12027c1"
        },
        "date": 1777591981281,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8462623953819275,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.755992704319522,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.221231816774992,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.548697916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.779522060951,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.976527889394,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003509,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213217.70165803612,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176318.92045398292,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "43051891+hestolz@users.noreply.github.com",
            "name": "Henry Stolz",
            "username": "hestolz"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "74bcd9886b81855f59df803b3e8979f35115cceb",
          "message": "fix crate name in otap-dataflow readme (#2790)\n\n# Change Summary\n\nfix crate name in otap-dataflow readme.\n\n## What issue does this PR close?\n\nminor nit.\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nN/A",
          "timestamp": "2026-04-30T19:39:37Z",
          "tree_id": "6ed9c7961fd84cc2a5d0ccb4882e64ef1be54275",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74bcd9886b81855f59df803b3e8979f35115cceb"
        },
        "date": 1777599774122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1299434900283813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.786249524290974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.187954791763431,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.77109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.257057974342,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.519849589872,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003406,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214190.9700314921,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177284.5967679228,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "geukhanuslu@gmail.com",
            "name": "Gokhan Uslu",
            "username": "gouslu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "993fa369f3ef1c12b381f01f758302e8243f211a",
          "message": "feat(engine): Extension System - Capability Registry & Resolver (#2732)\n\n# Extension system — Phase 1 (capabilities, registry, builder, proc\nmacro)\n\nImplements the Phase-1 extension/capability system for the OTAP dataflow\nengine. Extensions are first-class config siblings of nodes; nodes\nexplicitly bind to extension instances via named capabilities, and\nreceive typed handles resolved once at factory time — no hot-path\nregistry lookups.\n\nTracking docs:\n\n[`docs/extension-system-architecture.md`](rust/otap-dataflow/docs/extension-system-architecture.md)\n(rewritten in this PR).\n\n## What's in this PR\n\n### `#[capability]` proc macro (`engine-macros`)\n\n- New `capability.rs` expansion: from a single `#[capability] trait\n  Foo { ... }` source it generates `local::Foo` (`!Send`-friendly) and\n  `shared::Foo` (`Send + Clone`) trait variants plus a `SharedAsLocal`\n  adapter and an `ExtensionCapability` impl. The dual variants are\n  derived from one source, so authors can't accidentally let local\n  and shared semantics diverge.\n- New `pipeline_factory.rs` expansion to build the static\n  `PipelineFactory` registry used by `main.rs`.\n- All emitted paths use fully-qualified `::std::...` /\n  `::async_trait::...` / `::otap_df_engine::...` so generated code is\n  hygienic in any caller crate.\n\n### Capability registry (`engine::capability`)\n\n- `CapabilityRegistry`: typed-keyed (`(extension_name, TypeId)`)\n  storage with **typestate-enforced** single `.shared()` / `.local()`\n  registration per builder — duplicates are unrepresentable rather\n  than runtime errors.\n- Two execution models: native local (`Rc<dyn Local>`, lock-free) and\n  native shared (`Box<dyn Shared>`, `Send + Clone`). A shared-only\n  extension serves local consumers transparently via the\n  `SharedAsLocal` adapter generated by the proc macro.\n- Two **instance policies** chosen at build time, invisible to\n  consumers: `.cloned()` (clone a stored prototype) and\n  `.constructed()` (per-consumer construction via a closure;\n  Passive-only — `Active + Constructed` is statically rejected).\n- `resolve_bindings`: walks a node's `capabilities:` declaration and\n  produces a per-node `Capabilities` bundle with all bindings\n  resolved, surfacing config errors (unknown extension, unknown\n  capability, capability not provided by bound extension, multiple\n  bindings for the same capability).\n- `Capabilities`: per-node consumer API with `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`. Instances\n  are minted lazily at the call site, not at resolution time.\n- `ConsumedTracker`: cross-node, per-(capability, extension)\n  consumption flags driving `drop_local()` / `drop_shared()` cleanup\n  for extensions no node ever claimed.\n\n### One-shot consumption contract\n\nA binding is claimable **at most once per node**, regardless of\nexecution model. The guard is the `Cell<Option<_>>::take()` on each\nresolved entry's `produce` closure — no auxiliary flag.\n\n- Same accessor twice → `CapabilityAlreadyConsumed`.\n- Different accessors on a SharedAsLocal-fallback binding share one\n  underlying entry, so claiming either side consumes the other\n  naturally.\n- Different accessors on a native-dual binding (extension registered\n  both native local **and** native shared) take and drop the\n  alternative entry's `produce` closure on success, so the\n  per-binding contract holds uniformly. The cross-node tracker is\n  only flipped by actual consumption, not by invalidation, so\n  `drop_*` cleanup remains correct.\n\n### Documentation\n\n- `docs/extension-system-architecture.md`: rewritten to describe the\n  capability-based design, the local/shared duality, instance\n  policies, Active vs Passive lifecycle, and the typestate builder.\n\n## Tests\n\nNew, focused unit tests cover:\n\n- Registry: typestate single-registration, duplicate rejection,\n  `SharedAsLocal` adapter freshness per node, double-`Box` envelope\n  for shared `produce`.\n- `resolve_bindings`: every error path (unknown extension / unknown\n  capability / capability not provided / wrong extension), local-only\n  and shared-only binding shapes, fallback path, native-dual path.\n- One-shot contract: second-call rejection on each of `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`; fallback\n  cross-side rejection; native-dual cross-side rejection (both\n  directions).\n- `ConsumedTracker`: per-extension consumption flags, with the\n  invariant that mere invalidation does not flip a bucket.\n- Proc-macro end-to-end: `local-only`, `shared-only`, and `dual`\n  forms of `extension_capabilities!` against the registry.\n\n## Validation\n\n```text\ncargo xtask check\n✅ Cargo workspace structure complies with project policies.\n✅ Formatting completed successfully.\n✅ Clippy linting passed without warnings.\n✅ All tests passed successfully.\n```\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-01T02:10:49Z",
          "tree_id": "29c54668ae7178d41f61630f34daf75d7a38356a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993fa369f3ef1c12b381f01f758302e8243f211a"
        },
        "date": 1777613850314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2747875452041626,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.777533315105428,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.252938271604938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.255859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.66015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6024.176400881582,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.971850751176,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003555,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214246.1013853716,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177289.22975114422,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cithomas@microsoft.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ce3582596886edf41cb83c87829be8cd8d15fcce",
          "message": "fix(perf-test): add missing /api/v1 prefix to idle-state template endpoints (#2798)\n\nThe idle-state-template.yaml.j2 was using /telemetry/metrics instead of\n/api/v1/telemetry/metrics for both the Prometheus scraping endpoint and\nthe ready-check URL. This caused 404 errors during idle state\nbenchmarks.\n\nAll other test configs already had the correct /api/v1 prefix.\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-01T16:38:12Z",
          "tree_id": "a86e34dbb8f8829478ea49a921a5480b991c9701",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce3582596886edf41cb83c87829be8cd8d15fcce"
        },
        "date": 1777660662196,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7062146663665771,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.717974139325335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.146508254898935,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.654557291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.84765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.235512124102,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.899604723849,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00362,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212600.57802859842,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175627.8204025501,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "213113461+gyanranjanpanda@users.noreply.github.com",
            "name": "Gyan ranjan",
            "username": "gyanranjanpanda"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f018901f3a2ba93bee9d45f6afb1204f90679863",
          "message": "Fix duplicate attribute keys in transform_attributes (#2423)\n\n# Fix Duplicate Attribute Keys in `transform_attributes`\n\n## Changes Made\nThis PR resolves issue #1650 by ensuring that dictionary keys are\ndeduplicated when transformations such as `rename` are applied, as\nrequired by the OpenTelemetry specification (\"Exported maps MUST contain\nonly unique keys by default\").\n\nTo accomplish this while maintaining strict performance requirements, we\nreplaced the previous `RowConverter` deduplication strategy with a new\nhigh-performance, proactive pre-filter:\n- We injected `filter_rename_collisions` into\n`transform_attributes_impl` inside\n`otap-dataflow/crates/pdata/src/otap/transform.rs`.\n- Before a rename is processed, this function reads the `parent_id`s and\ntarget keys. It uses the `IdBitmap` type to find any existing target\nkeys whose `parent_id` maps back to an old key that will be renamed.\n- It proactively strips those collision rows from the batch via\n`arrow::compute::filter_record_batch` *before* the actual transform\nhappens.\n\n## Testing\n- Extended the `AttributesProcessor` unit tests\n(`test_rename_removes_duplicate_keys`) to explicitly verify that\nrenaming an attribute resulting in a collision automatically discards\nduplicate keys.\n- Extended the `AttributesTransformPipelineStage` in `query-engine`\ntests with a parallel case ensuring OPL/KQL query pipelines\n(`project-rename`) properly drop duplicates when resolving duplicates.\n- Refactored `otap_df_pdata` `transform.rs` tests to properly expect\ndeduplicated keys using this plan-based method.\n- Validated logic with `cargo test --workspace --all-features`.\n\n## Validation Results\nAll tests pass. OTel semantic rules surrounding unique mapped keys map\ncleanly through down/upstream processors. The `IdBitmap` intersection\napproach completely resolves the multi-thousand percent `RowConverter`\nperformance regressions, dropping collision resolution overhead to\nessentially zero through efficient bitmap operations.\n\n---------\n\nSigned-off-by: Gyanranjan Panda <gyanranjanpanda438@gmail.com>\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-01T20:08:07Z",
          "tree_id": "e2085c4a3fb15d509fdc667a56549abc71c1a226",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f018901f3a2ba93bee9d45f6afb1204f90679863"
        },
        "date": 1777672940645,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8486563563346863,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.7666135124231,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.32263137528055,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.584635416666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.59765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.7244101018,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6083.92150551992,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003404,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213617.5025300227,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176650.40525475284,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9aa767ee7b26712bbab69e4ecab5db2b22f80f32",
          "message": "Update github workflow dependencies (#2802)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[DavidAnson/markdownlint-cli2-action](https://redirect.github.com/DavidAnson/markdownlint-cli2-action)\n| action | minor | `v23.0.0` → `v23.1.0` |\n|\n[EmbarkStudios/cargo-deny-action](https://redirect.github.com/EmbarkStudios/cargo-deny-action)\n| action | patch | `v2.0.15` → `v2.0.17` |\n| [Swatinem/rust-cache](https://redirect.github.com/Swatinem/rust-cache)\n| action | minor | `v2` → `v2.9.1` |\n|\n[actions/create-github-app-token](https://redirect.github.com/actions/create-github-app-token)\n| action | minor | `v3.0.0` → `v3.1.1` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.3.0` → `v6.4.0` |\n|\n[actions/upload-artifact](https://redirect.github.com/actions/upload-artifact)\n| action | patch | `v7.0.0` → `v7.0.1` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.35.1` → `v4.35.3` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.26.1` → `1.26.2` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | minor | `v2.16.1` → `v2.19.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.71.2` → `v2.75.28` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>DavidAnson/markdownlint-cli2-action\n(DavidAnson/markdownlint-cli2-action)</summary>\n\n###\n[`v23.1.0`](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/releases/tag/v23.1.0):\nUpdate markdownlint-cli2 version (markdownlint-cli2 v0.22.1,\nmarkdownlint v0.40.0).\n\n[Compare\nSource](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/compare/v23.0.0...v23.1.0)\n\n</details>\n\n<details>\n<summary>EmbarkStudios/cargo-deny-action\n(EmbarkStudios/cargo-deny-action)</summary>\n\n###\n[`v2.0.17`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.17):\nRelease 2.0.17 - cargo-deny 0.19.2\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.16...v2.0.17)\n\n##### Fixed\n\n-\n[PR#845](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/845)\nfixed structural issues with SARIF output, resolving\n[#&#8203;818](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/818).\nThanks\n[@&#8203;KyleChamberlin](https://redirect.github.com/KyleChamberlin)!\n\n###\n[`v2.0.16`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.16):\nRelease 2.0.16 - cargo-deny 0.19.1\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.15...v2.0.16)\n\n##### Fixed\n\n-\n[PR#833](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/833)\nfixed an issue where the maximum advisory database staleness was over 14\nyears instead of the intended 90 days.\n-\n[PR#839](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/839)\nfixed an issue where unsound advisories would appear for transitive\ndependencies despite requesting them only for workspace dependencies,\nresolving\n[#&#8203;829](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/829).\n-\n[PR#840](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/840)\nresolved\n[#&#8203;797](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/797)\nby passing `--filter-platform` when collecting cargo metadata if only a\nsingle target was requested either in the config or via the command\nline.\n-\n[PR#841](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/841)\nfixed an issue where `--frozen` would not disable fetching of the\nadvisory DB, resolving\n[#&#8203;759](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/759).\n-\n[PR#842](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/842)\nand\n[PR#844](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/844)\nupdated crates. Notably `krates` was updated to resolve two issues with\ncrates being pruned from the graph used when running checks. Resolving\nthese two issues may mean that updating cargo-deny may highlight issues\nthat were previously hidden.\n-\n[EmbarkStudios/krates#106](https://redirect.github.com/EmbarkStudios/krates/issues/106)\nwould fail to pull in crates brought in via a feature if that crate had\nits `lib` target renamed by the package author.\n-\n[EmbarkStudios/krates#109](https://redirect.github.com/EmbarkStudios/krates/issues/109)\nwould fail to bring in optional dependencies if they were brought in by\na weak feature in a crate *also* brought in by a weak feature.\n\n##### Changed\n\n-\n[PR#830](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/830)\nremoved `gix` in favor of shelling out to `git`. This massively improves\nbuild times and eases maintenance as `gix` bumps minor versions quite\nfrequently. If cargo-deny is used in an environment that for some reason\nallows internet access but doesn't have `git` available, the advisory\ndatabase would need to be updated before calling cargo-deny.\n-\n[PR#838](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/838)\nremoved `rustsec` in favor of manually implemented advisory parsing and\nchecking, with a nightly cron job that checks that the implementation\nexactly matches rustsec on the official rustsec advisory db.\n\n</details>\n\n<details>\n<summary>Swatinem/rust-cache (Swatinem/rust-cache)</summary>\n\n###\n[`v2.9.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1)\n\nFix regression in hash calculation\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1>\n\n###\n[`v2.9.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0)\n\n##### What's Changed\n\n- Add support for running rust-cache commands from within a Nix shell by\n[@&#8203;marc0246](https://redirect.github.com/marc0246) in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- Bump taiki-e/install-action from 2.62.57 to 2.62.60 in the actions\ngroup by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;291](https://redirect.github.com/Swatinem/rust-cache/pull/291)\n- Bump the actions group across 1 directory with 5 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;296](https://redirect.github.com/Swatinem/rust-cache/pull/296)\n- Bump the prd-major group with 3 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;294](https://redirect.github.com/Swatinem/rust-cache/pull/294)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.10.1 to 25.0.2 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;295](https://redirect.github.com/Swatinem/rust-cache/pull/295)\n- Consider all installed toolchains in cache key by\n[@&#8203;tamird](https://redirect.github.com/tamird) in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- Compare case-insenitively for full cache key match by\n[@&#8203;kbriggs](https://redirect.github.com/kbriggs) in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n- Migrate to `node24` runner by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;314](https://redirect.github.com/Swatinem/rust-cache/pull/314)\n- Bump the actions group across 1 directory with 7 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;312](https://redirect.github.com/Swatinem/rust-cache/pull/312)\n- Bump the prd-minor group across 1 directory with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;307](https://redirect.github.com/Swatinem/rust-cache/pull/307)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n25.0.2 to 25.2.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;309](https://redirect.github.com/Swatinem/rust-cache/pull/309)\n\n##### New Contributors\n\n- [@&#8203;marc0246](https://redirect.github.com/marc0246) made their\nfirst contribution in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- [@&#8203;tamird](https://redirect.github.com/tamird) made their first\ncontribution in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- [@&#8203;kbriggs](https://redirect.github.com/kbriggs) made their\nfirst contribution in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0>\n\n###\n[`v2.8.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2)\n\n##### What's Changed\n\n- ci: address lint findings, add zizmor workflow by\n[@&#8203;woodruffw](https://redirect.github.com/woodruffw) in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- feat: Implement ability to disable adding job ID + rust environment\nhashes to cache names by\n[@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- Don't overwrite env for cargo-metadata call by\n[@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n##### New Contributors\n\n- [@&#8203;woodruffw](https://redirect.github.com/woodruffw) made their\nfirst contribution in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- [@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) made\ntheir first contribution in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- [@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) made their\nfirst contribution in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2>\n\n###\n[`v2.8.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.0...v2.8.1)\n\n##### What's Changed\n\n- Set empty `CARGO_ENCODED_RUSTFLAGS` in workspace metadata retrieval by\n[@&#8203;ark0f](https://redirect.github.com/ark0f) in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- chore(deps): update dependencies by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- chore: fix dependabot groups by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;253](https://redirect.github.com/Swatinem/rust-cache/pull/253)\n- Bump the prd-patch group with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- chore(dependabot): regenerate and commit dist/ by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;257](https://redirect.github.com/Swatinem/rust-cache/pull/257)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n22.16.3 to 24.2.1 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;255](https://redirect.github.com/Swatinem/rust-cache/pull/255)\n- Bump typescript from 5.8.3 to 5.9.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;256](https://redirect.github.com/Swatinem/rust-cache/pull/256)\n- Bump actions/setup-node from 4 to 5 in the actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;259](https://redirect.github.com/Swatinem/rust-cache/pull/259)\n- Update README.md by\n[@&#8203;Propfend](https://redirect.github.com/Propfend) in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.2.1 to 24.3.0 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;258](https://redirect.github.com/Swatinem/rust-cache/pull/258)\n\n##### New Contributors\n\n- [@&#8203;ark0f](https://redirect.github.com/ark0f) made their first\ncontribution in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- [@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt)\nmade their first contribution in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- [@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot]\nmade their first contribution in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- [@&#8203;Propfend](https://redirect.github.com/Propfend) made their\nfirst contribution in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.8.1>\n\n###\n[`v2.8.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0)\n\n##### What's Changed\n\n- Add cache-workspace-crates feature by\n[@&#8203;jbransen](https://redirect.github.com/jbransen) in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- Feat: support warpbuild cache provider by\n[@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n##### New Contributors\n\n- [@&#8203;jbransen](https://redirect.github.com/jbransen) made their\nfirst contribution in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- [@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) made their\nfirst contribution in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0>\n\n###\n[`v2.7.8`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.8)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8)\n\n##### What's Changed\n\n- Include CPU arch in the cache key for arm64 Linux runners by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;228](https://redirect.github.com/Swatinem/rust-cache/pull/228)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8>\n\n###\n[`v2.7.7`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.7)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7>\n\n###\n[`v2.7.6`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.6)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6)\n\n##### What's Changed\n\n- Updated artifact upload action to v4 by\n[@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- Adds an option to do lookup-only of the cache by\n[@&#8203;danlec](https://redirect.github.com/danlec) in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- add runner OS in cache key by\n[@&#8203;rnbguy](https://redirect.github.com/rnbguy) in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- Allow opting out of caching $CARGO\\_HOME/bin. by\n[@&#8203;benjyw](https://redirect.github.com/benjyw) in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n##### New Contributors\n\n- [@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) made\ntheir first contribution in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- [@&#8203;danlec](https://redirect.github.com/danlec) made their first\ncontribution in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- [@&#8203;rnbguy](https://redirect.github.com/rnbguy) made their first\ncontribution in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- [@&#8203;benjyw](https://redirect.github.com/benjyw) made their first\ncontribution in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6>\n\n###\n[`v2.7.5`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.5)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5)\n\n##### What's Changed\n\n- Upgrade checkout action from version 3 to 4 by\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nin\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- fix: usage of `deprecated` version of `node` by\n[@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- Only run macOsWorkaround() on macOS by\n[@&#8203;heksesang](https://redirect.github.com/heksesang) in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n- Support Cargo.lock format cargo-lock v4 by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;211](https://redirect.github.com/Swatinem/rust-cache/pull/211)\n\n##### New Contributors\n\n-\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nmade their first contribution in\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- [@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) made\ntheir first contribution in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- [@&#8203;heksesang](https://redirect.github.com/heksesang) made their\nfirst contribution in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5>\n\n###\n[`v2.7.3`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.3)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3)\n\n- Work around upstream problem that causes cache saving to hang for\nminutes.\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3>\n\n###\n[`v2.7.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2)\n\n##### What's Changed\n\n- Update action runtime to `node20` by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- Only key by `Cargo.toml` and `Cargo.lock` files of workspace members\nby [@&#8203;max-heller](https://redirect.github.com/max-heller) in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n##### New Contributors\n\n- [@&#8203;rhysd](https://redirect.github.com/rhysd) made their first\ncontribution in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- [@&#8203;max-heller](https://redirect.github.com/max-heller) made\ntheir first contribution in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2>\n\n###\n[`v2.7.1`](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n###\n[`v2.7.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0)\n\n##### What's Changed\n\n- Fix save-if documentation in readme by\n[@&#8203;rukai](https://redirect.github.com/rukai) in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- Support for `trybuild` and similar macro testing tools by\n[@&#8203;neysofu](https://redirect.github.com/neysofu) in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n##### New Contributors\n\n- [@&#8203;rukai](https://redirect.github.com/rukai) made their first\ncontribution in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- [@&#8203;neysofu](https://redirect.github.com/neysofu) made their\nfirst contribution in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0>\n\n###\n[`v2.6.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.1...v2.6.2)\n\n##### What's Changed\n\n- dep: Use `smol-toml` instead of `toml` by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;164](https://redirect.github.com/Swatinem/rust-cache/pull/164)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.6.2>\n\n###\n[`v2.6.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.0...v2.6.1)\n\n- Fix hash contributions of `Cargo.lock`/`Cargo.toml` files.\n\n###\n[`v2.6.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0)\n\n##### What's Changed\n\n- Add \"buildjet\" as a second `cache-provider` backend\n[@&#8203;joroshiba](https://redirect.github.com/joroshiba) in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n- Clean up sparse registry index.\n- Do not clean up src of `-sys` crates.\n- Remove `.cargo/credentials.toml` before saving.\n\n##### New Contributors\n\n- [@&#8203;joroshiba](https://redirect.github.com/joroshiba) made their\nfirst contribution in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0>\n\n###\n[`v2.5.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.0...v2.5.1)\n\n- Fix hash contribution of `Cargo.lock`.\n\n###\n[`v2.5.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0)\n\n##### What's Changed\n\n- feat: Rm workspace crates version before caching by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n- feat: Add hash of `.cargo/config.toml` to key by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;149](https://redirect.github.com/Swatinem/rust-cache/pull/149)\n\n##### New Contributors\n\n- [@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) made their\nfirst contribution in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0>\n\n###\n[`v2.4.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.4.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.3.0...v2.4.0)\n\n- Fix cache key stability.\n- Use 8 character hash components to reduce the key length, making it\nmore readable.\n\n###\n[`v2.3.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.3.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.1...v2.3.0)\n\n- Add `cache-all-crates` option, which enables caching of crates\ninstalled by workflows.\n- Add installed packages to cache key, so changes to workflows that\ninstall rust tools are detected and cached properly.\n- Fix cache restore failures due to upstream bug.\n- Fix `EISDIR` error due to globed directories.\n- Update runtime `@actions/cache`, `@actions/io` and dev `typescript`\ndependencies.\n- Update `npm run prepare` so it creates distribution files with the\nright line endings.\n\n###\n[`v2.2.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.0...v2.2.1)\n\n- Update `@actions/cache` dependency to fix usage of `zstd` compression.\n\n###\n[`v2.2.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.1.0...v2.2.0)\n\n- Add new `save-if` option to always restore, but only conditionally\nsave the cache.\n\n###\n[`v2.1.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.1.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.2...v2.1.0)\n\n- Only hash `Cargo.{lock,toml}` files in the configured workspace\ndirectories.\n\n###\n[`v2.0.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.1...v2.0.2)\n\n- Avoid calling cargo metadata on pre-cleanup.\n- Added `prefix-key`, `cache-directories` and `cache-targets` options.\n\n###\n[`v2.0.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2...v2.0.1)\n\n- Primarily just updating dependencies to fix GitHub deprecation\nnotices.\n\n</details>\n\n<details>\n<summary>actions/create-github-app-token\n(actions/create-github-app-token)</summary>\n\n###\n[`v3.1.1`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.1)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3.1.0...v3.1.1)\n\n##### Bug Fixes\n\n- improve error message when app identifier is empty\n([#&#8203;362](https://redirect.github.com/actions/create-github-app-token/issues/362))\n([07e2b76](https://redirect.github.com/actions/create-github-app-token/commit/07e2b760664f080c40eec4eacf7477256582db36)),\ncloses\n[#&#8203;249](https://redirect.github.com/actions/create-github-app-token/issues/249)\n\n###\n[`v3.1.0`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3...v3.1.0)\n\n##### Bug Fixes\n\n- **deps:** bump p-retry from 7.1.1 to 8.0.0\n([#&#8203;357](https://redirect.github.com/actions/create-github-app-token/issues/357))\n([3bbe07d](https://redirect.github.com/actions/create-github-app-token/commit/3bbe07d928e2d6c30bf3e37c6b89edbc4045facf))\n\n##### Features\n\n- add `client-id` input and deprecate `app-id`\n([#&#8203;353](https://redirect.github.com/actions/create-github-app-token/issues/353))\n([e6bd4e6](https://redirect.github.com/actions/create-github-app-token/commit/e6bd4e6970172bed9fe138b2eaf4cbffa4cca8f9))\n- update permission inputs\n([#&#8203;358](https://redirect.github.com/actions/create-github-app-token/issues/358))\n([076e948](https://redirect.github.com/actions/create-github-app-token/commit/076e9480ca6e9633bff412d05eff0fc2f1e7d2be))\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.4.0`](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n</details>\n\n<details>\n<summary>actions/upload-artifact (actions/upload-artifact)</summary>\n\n###\n[`v7.0.1`](https://redirect.github.com/actions/upload-artifact/releases/tag/v7.0.1)\n\n[Compare\nSource](https://redirect.github.com/actions/upload-artifact/compare/v7...v7.0.1)\n\n##### What's Changed\n\n- Update the readme with direct upload details by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;795](https://redirect.github.com/actions/upload-artifact/pull/795)\n- Readme: bump all the example versions to v7 by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;796](https://redirect.github.com/actions/upload-artifact/pull/796)\n- Include changes in typespec/ts-http-runtime 0.3.5 by\n[@&#8203;yacaovsnc](https://redirect.github.com/yacaovsnc) in\n[#&#8203;797](https://redirect.github.com/actions/upload-artifact/pull/797)\n\n**Full Changelog**:\n<https://github.com/actions/upload-artifact/compare/v7...v7.0.1>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.35.3`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.3)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.2...v4.35.3)\n\n- *Upcoming breaking change*: Add a deprecation warning for customers\nusing CodeQL version 2.19.3 and earlier. These versions of CodeQL were\ndiscontinued on 9 April 2026 alongside GitHub Enterprise Server 3.15,\nand will be unsupported by the next minor release of the CodeQL Action.\n[#&#8203;3837](https://redirect.github.com/github/codeql-action/pull/3837)\n- Configurations for private registries that use Cloudsmith or GCP OIDC\nare now accepted.\n[#&#8203;3850](https://redirect.github.com/github/codeql-action/pull/3850)\n- Best-effort connection tests for private registries now use `GET`\nrequests instead of `HEAD` for better compatibility with various\nregistry implementations. For NuGet feeds, the test is now always\nperformed against the service index.\n[#&#8203;3853](https://redirect.github.com/github/codeql-action/pull/3853)\n- Fixed a bug where two diagnostics produced within the same millisecond\ncould overwrite each other on disk, causing one of them to be lost.\n[#&#8203;3852](https://redirect.github.com/github/codeql-action/pull/3852)\n- Update default CodeQL bundle version to\n[2.25.3](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.3).\n[#&#8203;3865](https://redirect.github.com/github/codeql-action/pull/3865)\n\n###\n[`v4.35.2`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.1...v4.35.2)\n\n- The undocumented TRAP cache cleanup feature that could be enabled\nusing the `CODEQL_ACTION_CLEANUP_TRAP_CACHES` environment variable is\ndeprecated and will be removed in May 2026. If you are affected by this,\nwe recommend disabling TRAP caching by passing the `trap-caching: false`\ninput to the `init` Action.\n[#&#8203;3795](https://redirect.github.com/github/codeql-action/pull/3795)\n- The Git version 2.36.0 requirement for improved incremental analysis\nnow only applies to repositories that contain submodules.\n[#&#8203;3789](https://redirect.github.com/github/codeql-action/pull/3789)\n- Python analysis on GHES no longer extracts the standard library,\nrelying instead on models of the standard library. This should result in\nsignificantly faster extraction and analysis times, while the effect on\nalerts should be minimal.\n[#&#8203;3794](https://redirect.github.com/github/codeql-action/pull/3794)\n- Fixed a bug in the validation of OIDC configurations for private\nregistries that was added in CodeQL Action 4.33.0 / 3.33.0.\n[#&#8203;3807](https://redirect.github.com/github/codeql-action/pull/3807)\n- Update default CodeQL bundle version to\n[2.25.2](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.2).\n[#&#8203;3823](https://redirect.github.com/github/codeql-action/pull/3823)\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.26.2`](https://redirect.github.com/actions/go-versions/releases/tag/1.26.2-24114135105):\n1.26.2\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.26.1-22746851271...1.26.2-24114135105)\n\nGo 1.26.2\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.19.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.19.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0)\n\n##### What's Changed\n\n##### New Runner Support\n\nHarden-Runner now supports Depot, Blacksmith, Namespace, and WarpBuild\nrunners with the same egress monitoring, runtime monitoring, and policy\nenforcement available on GitHub-hosted runners.\n\n##### Automated Incident Response for Supply Chain Attacks\n\n- Global block list: Outbound connections to known malicious domains and\nIPs are now blocked even in audit mode.\n- System-defined detection rules: Harden-Runner will trigger lockdown\nmode when a high risk event is detected during an active supply chain\nattack (for example, a process reading the memory of the runner worker\nprocess, a common technique for stealing GitHub Actions secrets).\n\n##### Bug Fixes\n\nWindows and macOS: stability and reliability fixes\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0>\n\n###\n[`v2.18.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.18.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0)\n\n##### What's Changed\n\nGlobal Block List: During supply chain incidents like the recent axios\nand trivy compromises, StepSecurity will add known malicious domains and\nIP addresses (IOCs) to a global block list. These will be automatically\nblocked, even in audit mode, providing immediate protection without\nrequiring any workflow changes.\n\nDeploy on Self-Hosted VM: Added `deploy-on-self-hosted-vm` input that\nallows the Harden Runner agent to be installed directly on ephemeral\nself-hosted Linux runner VMs at workflow runtime. This is intended as an\nalternative when baking the agent into the VM image is not possible.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0>\n\n###\n[`v2.17.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.17.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0)\n\n##### What's Changed\n\n##### Policy Store Support\n\nAdded `use-policy-store` and `api-key` inputs to fetch security policies\ndirectly from the [StepSecurity Policy\nStore](https://docs.stepsecurity.io/harden-runner/policy-store).\nPolicies can be defined and attached at the workflow, repo, org, or\ncluster (ARC) level, with the most granular policy taking precedence.\nThis is the preferred method over the existing `policy` input which\nrequires `id-token: write` permission. If no policy is found in the\nstore, the action defaults to audit mode.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.75.28`](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n###\n[`v2.75.27`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.27):\n2.75.27\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.26...v2.75.27)\n\n- Update `cargo-udeps@latest` to 0.1.61.\n\n- Update `wasm-tools@latest` to 1.248.0.\n\n- Update `cargo-deb@latest` to 3.6.4.\n\n###\n[`v2.75.26`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.26):\n2.75.26\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.25...v2.75.26)\n\n- Update `wasm-bindgen@latest` to 0.2.120.\n\n- Update `mise@latest` to 2026.4.25.\n\n- Update `martin@latest` to 1.8.0.\n\n- Update `vacuum@latest` to 0.26.4.\n\n###\n[`v2.75.25`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.25):\n2.75.25\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.24...v2.75.25)\n\n- Update `uv@latest` to 0.11.8.\n\n- Update `typos@latest` to 1.45.2.\n\n- Update `tombi@latest` to 0.9.25.\n\n- Update `mise@latest` to 2026.4.24.\n\n###\n[`v2.75.24`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.24):\n2.75.24\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.23...v2.75.24)\n\n- Update `prek@latest` to 0.3.11.\n\n- Update `mise@latest` to 2026.4.23.\n\n- Update `vacuum@latest` to 0.26.3.\n\n###\n[`v2.75.23`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.23):\n2.75.23\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.22...v2.75.23)\n\n- Update `vacuum@latest` to 0.26.2.\n\n- Update `tombi@latest` to 0.9.24.\n\n- Update `mise@latest` to 2026.4.22.\n\n- Update `martin@latest` to 1.7.0.\n\n- Update `git-cliff@latest` to 2.13.1.\n\n- Update `cargo-tarpaulin@latest` to 0.35.4.\n\n- Update `cargo-sort@latest` to 2.1.4.\n\n###\n[`v2.75.22`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.22):\n2.75.22\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.21...v2.75.22)\n\n- Update `tombi@latest` to 0.9.22.\n\n- Update `biome@latest` to 2.4.13.\n\n###\n[`v2.75.21`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.21):\n2.75.21\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21)\n\n- Update `mise@latest` to 2026.4.19.\n\n- Update `tombi@latest` to 0.9.21.\n\n- Update `syft@latest` to 1.43.0.\n\n###\n[`v2.75.20`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.20):\n2.75.20\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20)\n\n- Update `prek@latest` to 0.3.10.\n\n- Update `cargo-xwin@latest` to 0.22.0.\n\n###\n[`v2.75.19`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.19):\n2.75.19\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19)\n\n- Update `wasmtime@latest` to 44.0.0.\n\n- Update `tombi@latest` to 0.9.20.\n\n- Update `martin@latest` to 1.6.0.\n\n- Update `just@latest` to 1.50.0.\n\n- Update `mise@latest` to 2026.4.18.\n\n- Update `rclone@latest` to 1.73.5.\n\n###\n[`v2.75.18`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.18):\n2.75.18\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18)\n\n- Update `vacuum@latest` to 0.26.1.\n\n- Update `wasm-tools@latest` to 1.247.0.\n\n- Update `mise@latest` to 2026.4.16.\n\n- Update `espup@latest` to 0.17.1.\n\n- Update `trivy@latest` to 0.70.0.\n\n###\n[`v2.75.17`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.21...HEAD\n\n[2.75.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21\n\n[2.75.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20\n\n[2.75.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19\n\n[2.75.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18\n\n[2.75.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17\n\n[2.75.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.15...v2.75.16\n\n[2.75.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.14...v2.75.15\n\n[2.75.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.13...v2.75.14\n\n[2.75.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.12...v2.75.13\n\n[2.75.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.11...v2.75.12\n\n[2.75.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.10...v2.75.11\n\n[2.75.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.9...v2.75.10\n\n[2.75.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.8...v2.75.9\n\n[2.75.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.7...v2.75.8\n\n[2.75.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.6...v2.75.7\n\n[2.75.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.5...v2.75.6\n\n[2.75.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.4...v2.75.5\n\n[2.75.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.3...v2.75.4\n\n[2.75.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.2...v2.75.3\n\n[2.75.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.1...v2.75.2\n\n[2.75.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.0...v2.75.1\n\n[2.75.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.1...v2.75.0\n\n[2.74.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.0...v2.74.1\n\n[2.74.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.73.0...v2.74.0\n\n[2.73.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.72.0...v2.73.0\n\n[2.72.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.3...v2.72.0\n\n[2.71.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.2...v2.71.3\n\n[2.71.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.1...v2.71.2\n\n[2.71.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.0...v2.71.1\n\n[2.71.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.4...v2.71.0\n\n[2.70.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.3...v2.70.4\n\n[2.70.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.2...v2.70.3\n\n[2.70.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.1...v2.70.2\n\n[2.70.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.0...v2.70.1\n\n[2.70.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.14...v2.70.0\n\n[2.69.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.13...v2.69.14\n\n[2.69.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.12...v2.69.13\n\n[2.69.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.11...v2.69.12\n\n[2.69.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.10...v2.69.11\n\n[2.69.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.9...v2.69.10\n\n[2.69.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.8...v2.69.9\n\n[2.69.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.7...v2.69.8\n\n[2.69.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.6...v2.69.7\n\n[2.69.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.5...v2.69.6\n\n[2.69.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.4...v2.69.5\n\n[2.69.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.3...v2.69.4\n\n[2.69.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.2...v2.69.3\n\n[2.69.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.1...v2.69.2\n\n[2.69.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.0...v2.69.1\n\n[2.69.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.36...v2.69.0\n\n[2.68.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.35...v2.68.36\n\n[2.68.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.34...v2.68.35\n\n[2.68.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.33...v2.68.34\n\n[2.68.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.32...v2.68.33\n\n[2.68.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.31...v2.68.32\n\n[2.68.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.30...v2.68.31\n\n[2.68.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.29...v2.68.30\n\n[2.68.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.28...v2.68.29\n\n[2.68.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.27...v2.68.28\n\n[2.68.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.26...v2.68.27\n\n[2.68.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.25...v2.68.26\n\n[2.68.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.24...v2.68.25\n\n[2.68.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.23...v2.68.24\n\n[2.68.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.22...v2.68.23\n\n[2.68.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.21...v2.68.22\n\n[2.68.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.20...v2.68.21\n\n[2.68.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.19...v2.68.20\n\n[2.68.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.18...v2.68.19\n\n[2.68.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.17...v2.68.18\n\n[2.68.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.16...v2.68.17\n\n[2.68.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.15...v2.68.16\n\n[2.68.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.14...v2.68.15\n\n[2.68.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.13...v2.68.14\n\n[2.68.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.12...v2.68.13\n\n[2.68.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.11...v2.68.12\n\n[2.68.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.10...v2.68.11\n\n[2.68.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.9...v2.68.10\n\n[2.68.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.8...v2.68.9\n\n[2.68.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.7...v2.68.8\n\n[2.68.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.6...v2.68.7\n\n[2.68.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.5...v2.68.6\n\n[2.68.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.4...v2.68.5\n\n[2.68.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.3...v2.68.4\n\n[2.68.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.2...v2.68.3\n\n[2.68.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.1...v2.68.2\n\n[2.68.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.0...v2.68.1\n\n[2.68.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.30...v2.68.0\n\n[2.67.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.29...v2.67.30\n\n[2.67.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.28...v2.67.29\n\n[2.67.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.27...v2.67.28\n\n[2.67.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.26...v2.67.27\n\n[2.67.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.25...v2.67.26\n\n[2.67.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.24...v2.67.25\n\n[2.67.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.23...v2.67.24\n\n[2.67.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.22...v2.67.23\n\n[2.67.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.21...v2.67.22\n\n[2.67.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.20...v2.67.21\n\n[2.67.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.19...v2.67.20\n\n[2.67.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...v2.67.19\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62\n\n> ✂ **Note**\n> \n> PR body was truncated to here.\n\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-02T11:20:16Z",
          "tree_id": "b4b323c10b769203f65d4896f59fc3f9dcafc342",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9aa767ee7b26712bbab69e4ecab5db2b22f80f32"
        },
        "date": 1777734220337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9887005090713501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.76603143711191,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.236069310743165,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.661067708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.2645084442975,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.994524770725,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003332,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214192.37834134945,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177281.12712983164,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bca750e798b12053aa0c0c44e7393dfc4f7578e6",
          "message": "Add host metrics receiver design (#2799)\n\n# Change Summary\n\n\nAdds a design doc for the Linux v1 OTAP-native `host_metrics` receiver\nproposed in open-telemetry/otel-arrow#2741. Most of the direction is\nalready established in the issue. This document mainly makes the Rust\nOTAP Dataflow implementation choices explicit before code is added. So\nspecifically covering:\n\n\n  - singleton one-core receiver deployment\n  - per-family interval scheduling\n  - shared Linux scrape snapshots\n  - `HostView` / `root_path` handling\n  - duplicate host collection guard\n  - direct OTAP metrics emission\n  - semantic convention projection\n  - receiver self-observability through `MetricSet`\n  - partial failure behavior\n  - validation and benchmark scope\n\n## What issue does this PR close?\n\n\n* Towards #2741 \n\n## How are these changes tested?\n\n  - `npx --yes markdownlint-cli docs/host-metrics-receiver.md`\n  - `python3 tools/sanitycheck.py`\n\n## Are there any user-facing changes?\n\nNot yet.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-03T03:10:18Z",
          "tree_id": "7fbf755a5ba6651876e4826091980e5923cc0492",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bca750e798b12053aa0c0c44e7393dfc4f7578e6"
        },
        "date": 1777780505609,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.578796625137329,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.760963573772745,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.260943190661479,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.576171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5955.889658851261,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.47993658668,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003798,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212952.4821777077,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176021.97242392425,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "140f30edbbf4ea059eee3823f63c32d7fcab3786",
          "message": "Update docker digest updates (#2800)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| alpine | final | digest | `2510918` → `5b10f43` |\n| docker.io/alpine | final | digest | `2510918` → `5b10f43` |\n| docker.io/rust | stage | digest | `4a7e3a0` → `a9cfb75` |\n| golang | stage | digest | `595c784` → `b54cbf5` |\n| python | final | digest | `fb83750` → `5b3879b` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-03T03:11:09Z",
          "tree_id": "450e5055d076e18cb3446c6a400d1dfefc144353",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/140f30edbbf4ea059eee3823f63c32d7fcab3786"
        },
        "date": 1777781540335,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.860832154750824,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.729241953897027,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.2430795393058665,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.715625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5946.850820671546,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.043223718933,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008904,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213305.14952179143,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176073.20670369727,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fe469abf54b3700a301deeab1cd987722df96382",
          "message": "Update github workflow dependencies (major) (#2803)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[actions/github-script](https://redirect.github.com/actions/github-script)\n| action | major | `v8.0.0` → `v9.0.0` |\n| [dorny/test-reporter](https://redirect.github.com/dorny/test-reporter)\n| action | major | `v2.7.0` → `v3.0.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/github-script (actions/github-script)</summary>\n\n###\n[`v9.0.0`](https://redirect.github.com/actions/github-script/releases/tag/v9.0.0)\n\n[Compare\nSource](https://redirect.github.com/actions/github-script/compare/v8.0.0...v9.0.0)\n\n**New features:**\n\n- **`getOctokit` factory function** — Available directly in the script\ncontext. Create additional authenticated Octokit clients with different\ntokens for multi-token workflows, GitHub App tokens, and cross-org\naccess. See [Creating additional clients with\n`getOctokit`](https://redirect.github.com/actions/github-script#creating-additional-clients-with-getoctokit)\nfor details and examples.\n- **Orchestration ID in user-agent** — The `ACTIONS_ORCHESTRATION_ID`\nenvironment variable is automatically appended to the user-agent string\nfor request tracing.\n\n**Breaking changes:**\n\n- **`require('@&#8203;actions/github')` no longer works in scripts.**\nThe upgrade to `@actions/github` v9 (ESM-only) means\n`require('@&#8203;actions/github')` will fail at runtime. If you\npreviously used patterns like `const { getOctokit } =\nrequire('@&#8203;actions/github')` to create secondary clients, use the\nnew injected `getOctokit` function instead — it's available directly in\nthe script context with no imports needed.\n- `getOctokit` is now an injected function parameter. Scripts that\ndeclare `const getOctokit = ...` or `let getOctokit = ...` will get a\n`SyntaxError` because JavaScript does not allow `const`/`let`\nredeclaration of function parameters. Use the injected `getOctokit`\ndirectly, or use `var getOctokit = ...` if you need to redeclare it.\n- If your script accesses other `@actions/github` internals beyond the\nstandard `github`/`octokit` client, you may need to update those\nreferences for v9 compatibility.\n\n##### What's Changed\n\n- Add ACTIONS\\_ORCHESTRATION\\_ID to user-agent string by\n[@&#8203;Copilot](https://redirect.github.com/Copilot) in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n- ci: use deployment: false for integration test environments by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;712](https://redirect.github.com/actions/github-script/pull/712)\n- feat!: add getOctokit to script context, upgrade\n[@&#8203;actions/github](https://redirect.github.com/actions/github) v9,\n[@&#8203;octokit/core](https://redirect.github.com/octokit/core) v7, and\nrelated packages by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;700](https://redirect.github.com/actions/github-script/pull/700)\n\n##### New Contributors\n\n- [@&#8203;Copilot](https://redirect.github.com/Copilot) made their\nfirst contribution in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n\n**Full Changelog**:\n<https://github.com/actions/github-script/compare/v8.0.0...v9.0.0>\n\n</details>\n\n<details>\n<summary>dorny/test-reporter (dorny/test-reporter)</summary>\n\n###\n[`v3.0.0`](https://redirect.github.com/dorny/test-reporter/releases/tag/v3.0.0)\n\n[Compare\nSource](https://redirect.github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0)\n\n**Note:** The v3 release requires NodeJS 24 runtime on GitHub Actions\nrunners.\n\n#### What's Changed\n\n- Upgrade action runtime to Node.js 24 by\n[@&#8203;dav-tb](https://redirect.github.com/dav-tb) in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n- Explicitly use lowest permissions required to run workflow by\n[@&#8203;jozefizso](https://redirect.github.com/jozefizso) in\n[#&#8203;745](https://redirect.github.com/dorny/test-reporter/pull/745)\n\n##### Other Changes\n\n- Bump\n[@&#8203;typescript-eslint/parser](https://redirect.github.com/typescript-eslint/parser)\nfrom 8.57.0 to 8.57.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;742](https://redirect.github.com/dorny/test-reporter/pull/742)\n- Bump\n[@&#8203;types/adm-zip](https://redirect.github.com/types/adm-zip) from\n0.5.7 to 0.5.8 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;743](https://redirect.github.com/dorny/test-reporter/pull/743)\n- Bump flatted from 3.4.1 to 3.4.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;744](https://redirect.github.com/dorny/test-reporter/pull/744)\n\n#### New Contributors\n\n- [@&#8203;dav-tb](https://redirect.github.com/dav-tb) made their first\ncontribution in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n\n**Full Changelog**:\n<https://github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-03T03:11:43Z",
          "tree_id": "b2739e400bbc869d5f779c7cb6de470a71baf203",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fe469abf54b3700a301deeab1cd987722df96382"
        },
        "date": 1777782577113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4184396266937256,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.762075568083611,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.220285119827387,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.979166666666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6015.193262330301,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.51515257612,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008047,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212549.1895025614,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175588.1405715943,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "00b00e301d89f9b298ef3b5d8256d210b497fc2f",
          "message": "fix(tests): increase shutdown deadlines from 200ms to 1s in parquet adaptive_schema tests (#2812)\n\n# Change Summary\n\nIncrease shutdown timeouts in parquet adaptive_schema tests to address\ntest flakiness on slow CI runs.\n\n## What issue does this PR close?\n\n* Addresses a flaky test reported in #2720 \n\n## How are these changes tested?\n\n* Verified tests pass locally\n\n## Are there any user-facing changes?\n\nNo. This is a test-only change.",
          "timestamp": "2026-05-04T16:09:28Z",
          "tree_id": "e000a86f0b4799ab6c345f344796d646e134257b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00b00e301d89f9b298ef3b5d8256d210b497fc2f"
        },
        "date": 1777921854288,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.42796003818511963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.813802893372647,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.517434826332483,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.577734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5981.311301912284,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6006.908925172964,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005571,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213465.2698569727,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176525.44291488072,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "34c171b819eb69c1fce176e54072729c747b0cd0",
          "message": "fix(tests): increase timeout duration for logs and metrics watch tests (#2817)\n\n# Change Summary\n\nThe `metrics_watch_human_color_always_styles_stream_header` and\n`logs_watch_uses_next_seq_as_after_cursor` tests were using a 10ms\ntimeout that was too tight for HTTP requests to the mock server.\n\nIncrease the timeouts from 10ms to 200ms to provide sufficient time for:\n - Mock server HTTP connection establishment\n - Request/response round-trip\n - Output rendering and writing to stdout\n\n## What issue does this PR close?\n\n* Addresses flaky test\n`metrics_watch_human_color_always_styles_stream_header` from #2720\n\n## How are these changes tested?\n\nValidated that tests pass locally\n\n## Are there any user-facing changes?\n\nNo. This is a test only change.",
          "timestamp": "2026-05-04T16:35:35Z",
          "tree_id": "a745055661847578c3c0253e7f8d41887ab3f8bc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/34c171b819eb69c1fce176e54072729c747b0cd0"
        },
        "date": 1777922905488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4164305925369263,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.753190380968878,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.137920792079208,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.070052083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.19921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6024.1824247070945,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.510787663286,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003495,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213513.90237574186,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176579.02801431302,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b91157891d6b1a4b3d3198d24bb42c6d0209be02",
          "message": "Add try_into_with_options with empty ConversionOptions struct (#2811)\n\n# Change Summary\n\nReplaces TryInto by a new TryIntoWithOptions and a default path, then\nmechanically converts the code.\n\n## What issue does this PR close?\n\nPart of #2725 \nSplit from #2792 \n\n## Are there any user-facing changes?\n\nNone.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-04T19:29:08Z",
          "tree_id": "f765582949669a202a6dc8062f3577e5ebffcb02",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b91157891d6b1a4b3d3198d24bb42c6d0209be02"
        },
        "date": 1777926859299,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1363636255264282,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.743218680874361,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.254475659778655,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.274869791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.69921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6006.40053057249,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5938.145979088712,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.01065,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213042.1333061851,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176116.32083185203,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f5f798f0fc2c2e0760b0879f452431172680393f",
          "message": "Implement pdata-propagated stopwatch timer across processors (#2747)\n\n# Change Summary\n\nAlternative to the full `ProcessorChain` implementation under discussion\nat #2556\n\nAfter some conversations, trying to simplify back to focusing on ONLY\nthe composite duration problem in a way that isn't directly tied to the\nchannel elimination strategy / full implementation of `ProcessorChain`\n\n## Motivation\n\nPer-node `processor.compute.*.duration` reports each processor in\nisolation. Operators also need a single metric for the combined compute\nof a *range* of processors (e.g. five sequential attribute processors as\none number).\n\nDistributed **stopwatches** are engine-managed metrics that sum\nper-message compute time across configurable processor node pairs on the\nforward path.\n\n## Design\n\n**What's measured.** The wall-clock time spent inside each processor's\n`process()` between successive sends. Queue wait and channel latency\nbetween nodes are excluded.\n\n**How it's measured.** Immediately before each `process()` call, the\nengine arms an `Instant` send-marker on the EffectHandler. On every send\n(`send_message`, `try_send_message`, `_to`, and `_with_source_node*`\nvariants), the unified `ProcessorSendHook` reads the elapsed time since\nthe marker, advances the marker to \"now\", and adds the delta to the\nmessage's on-PData `stopwatch_compute_ns` accumulator. No processor\ncooperation needed — `timed()` is not required for stopwatch\nparticipation, and is used solely to split per-node compute into\nsuccess/failed.\n\n**Non-overlapping ranges.** At most one stopwatch is active per message.\nThe on-PData accumulator is a single `Option<NonZeroU64>`.\n\n**Declarative YAML config.**\n\n```yaml\npolicies:\n  telemetry:\n    runtime_metrics: normal\n    stopwatches:\n      - name: ingest_pipeline\n        start_node: attr1\n        stop_node: attr5\n```\n\nEach stopwatch gets its own metric entity with `stopwatch.name`,\n`stopwatch.start_node`, and `stopwatch.stop_node` attributes.\n\n## How It Works\n\n1. **Build time** — `build_stopwatch_state()` resolves config node names\nto indices, validates that endpoints are processor nodes (local or\nshared; receivers and exporters are rejected) and that no node is reused\nacross roles. If `runtime_metrics` does not include `PROCESS_DURATION`,\nall stopwatches are skipped with a single warning. Each processor's\n`EffectHandler` is assigned its start/stop role and (for stop nodes) the\nmetric set to report into.\n2. **Marker arming** — the engine loop calls\n`effect_handler.begin_process_timing()` before each `process()` call\nwhen stopwatches are active.\n   - Local: `Rc<Cell<Option<Instant>>>`.\n- Shared: `Arc<Mutex<Option<Instant>>>` (mutex contention is bounded to\nthe per-processor sequential `process()` loop).\n3. **Forward-path accumulation** — `OtapPdata::before_processor_send`\n(the unified `ProcessorSendHook` impl, called by both the plain\n`send_message*` family and the `send_message_with_source_node*` family\non local and shared processor handlers) invokes `stopwatch_accumulate`,\nwhich calls `take_elapsed_since_send_marker_ns()` (single\nread-and-advance step) and:\n- **Start node:** initialises the on-PData accumulator with the 1 ns\nsentinel.\n   - **Any node within an active range:** adds the delta.\n- **Stop node:** takes the total via `take_stopwatch_compute()` and\nrecords into the local stop accumulator (`Cell<Mmsc>` for local,\n`Arc<Mutex<Mmsc>>` for shared).\n4. **Periodic reporting** — on `CollectTelemetry` and at shutdown, the\nengine loop calls `effect_handler.report_stopwatch()`, which drains the\nstop accumulator into its `MetricSet` and reports it.\n\n## Metrics Produced\n\n| Metric Set | Metric Name | Unit | Attributes |\n\n|-------------|------------------------------|------|---------------------------------------------------------------------------------|\n| `stopwatch` | `stopwatch.compute.duration` | ns | `stopwatch.name`,\n`stopwatch.start_node`, `stopwatch.stop_node`, pipeline attrs |\n\n`Mmsc` (min/max/sum/count), delta temporality.\n\n## Live Validation\n\nDemo: `configs/fake-stopwatch-demo.yaml` (5 attribute processors, one\nstopwatch spanning the chain).\n\n```bash\ncurl -s 'http://127.0.0.1:8080/api/v1/telemetry/metrics?format=json&reset=true' | jq '\n  [.metric_sets[] | select(.name == \"stopwatch\" or .name == \"processor.compute\")]\n  | .[] | {\n      name: .name,\n      identity: (.attributes[\"node.id\"].String // .attributes[\"stopwatch.name\"].String),\n      duration: (\n        .metrics[]\n        | select(\n            (.name == \"compute.success.duration\") or\n            (.name == \"stopwatch.compute.duration\")\n          )\n        | .value\n        | if type == \"object\" and .count > 0 then {\n            avg_ms: (.sum / .count / 1e6),\n            count\n          } else empty end\n      )\n    }'\n```\n\nThe stopwatch average is at least the sum of per-processor\n`processor.compute` averages — it also covers `process()` time outside\n`timed()` closures (message handling, send-side bookkeeping):\n\n| Metric | Node / Stopwatch | Avg (ms) | Count |\n\n|---------------------|-----------------------------------|-----------|--------|\n| `processor.compute` | attr1 | 1.305 | 40 |\n| `processor.compute` | attr2 | 0.908 | 40 |\n| `processor.compute` | attr3 | 1.207 | 40 |\n| `processor.compute` | attr4 | 0.521 | 40 |\n| `processor.compute` | attr5 | 0.519 | 40 |\n| **`stopwatch`** | **ingest_pipeline** (attr1→attr5) | **6.121** |\n**40** |\n\n## Limitations / Follow-ups\n\n- **Non-overlapping only.** Build-time validation rejects nodes reused\nacross roles (e.g. `a→b` + `b→c`); runtime detection warns and resets if\ntopologically overlapping ranges (e.g. 1→3 + 2→4) appear.\n- **Requires `runtime_metrics: normal` or higher.** Gated behind the\nsame `PROCESS_DURATION` interest as `processor.compute`. Lower levels\nskip all stopwatches at build time with one warning.\n- **Processors only.** Receivers and exporters are rejected at build\ntime (no incoming-message frame / no further send path).\n- **No topology ordering validation.** Start-before-stop is not\nenforced; a misconfigured stopwatch simply never fires.\n- **Shared processors with worker tasks.** The marker captures only time\nspent inside `process()` itself. Processors that fan work out to spawned\ntasks and return early can still report accurate per-message compute via\n`timed()` from inside their workers.\n- **Send-bounded measurement.** Each processor's contribution is the\nwall-clock span from `begin_process_timing` (or its previous send)\nthrough its next `send_message`. Two consequences:\n- **Dropped messages contribute nothing.** A processor that consumes a\nmessage without sending it never reaches a stop node, so no stopwatch\nobservation is recorded — by design, the metric counts only complete\ntransits.\n- **Post-final-send work is not attributed.** Cleanup performed after\nthe last `send_message` in `process()` is not captured. For typical\nstateless processors this is a trivial return path; processors that do\nsignificant post-send work should perform it before the final send if\nthey want it measured.\n\n## What issue does this PR close?\n\n* Related to #2556\n* Contributes to #2782\n\n## How are these changes tested?\n\nUnit tests, local run of sample config\n\n## Are there any user-facing changes?\n\nYes, users could configure stopwatch metrics across multiple processors",
          "timestamp": "2026-05-04T23:17:49Z",
          "tree_id": "cf6955d2d8ba7477dddc57643ae666943bba9521",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f5f798f0fc2c2e0760b0879f452431172680393f"
        },
        "date": 1777940295737,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.435530185699463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.789262284676699,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.3603939511438545,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.002734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5954.5924337607075,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6099.618323981241,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.01687,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214579.3973644053,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177697.69492671313,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1884c67d731ae445dcab2d6642d7344b37bbad38",
          "message": "fix(otlp_grpc_exporter): Set accept_compressed in addition to send_compressed (#2829)\n\n# Change Summary\n\nSet `accept_compressed` so that we can process responses if the replies\nare compressed with the same codec.\n\n## What issue does this PR close?\n\n\n* Closes #2828\n\n## How are these changes tested?\n\nI tried them locally and verified the warnings in the issue went away.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-04T23:33:25Z",
          "tree_id": "5320b9c5a6c8d4ab00dc7b87d62023ae0e477125",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1884c67d731ae445dcab2d6642d7344b37bbad38"
        },
        "date": 1777941342653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9900990128517151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.78102318177053,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.248904713987151,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.01328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.76171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.745422972894,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.475575675596,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003195,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213528.98204685643,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176473.58791924306,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33842784+JakeDern@users.noreply.github.com",
            "name": "Jake Dern",
            "username": "JakeDern"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "11fdb97ee8b5899cc280f6a64a812a9f024987a8",
          "message": "feat(orchestrator): Allow providing a `--tests` flag to filter the tests to be run (#2830)\n\n# Change Summary\n\nWhen working on benchmarks I often need to run or re-run just a single\ntest of a much larger suite. Rather than play around with commenting\nstuff in and out, especially when templates are involved, it's much\neasier to pass this argument.\n\n## What issue does this PR close?\n\nNone :(\n\n## How are these changes tested?\n\nI've been running this daily locally.\n\n## Are there any user-facing changes?\n\nJust for the orchestrator - New `--tests` flag.",
          "timestamp": "2026-05-05T01:48:31Z",
          "tree_id": "09deb0da8cf0950d8a85e84475ad48841f470366",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/11fdb97ee8b5899cc280f6a64a812a9f024987a8"
        },
        "date": 1777948949283,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.755670186209963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.260938271604938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.992578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.37109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5972.989189606193,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.448973398316,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003457,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214398.53746347738,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176843.54187603868,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8b1e57dbc1f5a79d100ed8f3e1c2aeb42fe89417",
          "message": "  [Geneva Exporter] Encode logs using LogsDataView (#2810)\n\n## Description\n\nUpdates the otel-arrow Geneva exporter to encode log payloads through\nthe shared `LogsDataView` abstraction instead of converting logs back\nthrough OTLP proto objects.\n\n  ## Changes\n\n- Encode OTAP Arrow logs through `OtapLogsView`, the OTAP implementation\nof `LogsDataView`.\n- Encode OTLP byte logs through `RawLogsData`, the OTLP bytes\nimplementation of `LogsDataView`.\n- Call the existing `geneva-uploader` `encode_and_compress_logs(&view)`\nAPI for both log paths.\n- Patch `otap-df-pdata` / `otap-df-pdata-views` so `geneva-uploader`\nuses the workspace view traits.\n- Add a TODO for compressed byte accounting until\n`EncodedBatch::data_len()` is exposed upstream.\n\n## How are these changes tested?\n\n  - `cargo fmt --all`\n- `CARGO_NET_GIT_FETCH_WITH_CLI=true cargo check -p\notap-df-contrib-nodes --features \"geneva-exporter\notap-df-otap/crypto-ring\"`\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\nNone.",
          "timestamp": "2026-05-05T03:06:26Z",
          "tree_id": "15db94aa98fd779b4698b4f3b6525799f4b6f2bc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b1e57dbc1f5a79d100ed8f3e1c2aeb42fe89417"
        },
        "date": 1777958279151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.750804913493768,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.522168767886146,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.658984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5972.687586260466,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.284818773011,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006487,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212755.42665633943,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175713.9953720253,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8b4e674f6909f04cdd6726c42e3ad0f3281b3d6e",
          "message": "Update module go.opentelemetry.io/collector/pdata to v1.57.0 (#2815)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.56.0` → `v1.57.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.57.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.56.0/v1.57.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.57.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1570v01510)\n\n##### 🛑 Breaking changes 🛑\n\n- `cmd/builder`: In the generated Collector source, the `replace`\nstatements in the Go module will now use relative paths by default.\n([#&#8203;15097](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15097))\nWe expect that this will not break existing use-cases where the\ngenerated collector is only used in an interim manner for builds. It\nenables the possibility of tracking the generated Collector code as a\nlonger living artifact, allowing it to be run on any machine (whereas\nabsolute paths will be different depending on the machine the Collector\nsource is generated on.) We have added\n`dist::use_absolute_replace_paths` to go back to the absolute path\nbehaviour in the case where there is an unforeseen use-case that\nrequires absolute paths.\n\n- `pkg/confighttp`: Stabilize framedSnappy feature gate.\n([#&#8203;15096](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15096))\n\n##### 💡 Enhancements 💡\n\n- `all`: Add declarative schema support for service telemetry resource\nconfiguration.\n([#&#8203;14411](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14411))\nThe `service::telemetry::resource` configuration now accepts the\ndeclarative schema with explicit name/value pairs:\n\n  ```yaml\n  service:\n    telemetry:\n      resource:\n        schema_url: https://opentelemetry.io/schemas/1.38.0\n        attributes:\n          - name: service.name\n            value: my-collector\n          - name: host.name\n            value: collector-host\n  ```\n\nThe legacy inline attribute map format is still supported for backward\ncompatibility:\n\n  ```yaml\n  service:\n    telemetry:\n      resource:\n        service.name: my-collector\n        host.name: collector-host\n  ```\n\nNote: `resource.detectors` is accepted for forward compatibility but is\nnot yet applied by the collector.\n\n- `exporter/otlp_grpc`: Added the `server.address` and `url.path`\nattributes to metrics generated by the otlp exporter.\n([#&#8203;14998](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14998))\n\n- `exporter/otlp_http`: Added the `server.address` and `url.path`\nattributes to metrics generated by the otlp\\_http exporter.\n([#&#8203;14998](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14998))\n\n- `pkg/config/configgrpc`: Add `UserAgent` field to `ClientConfig` to\nallow overriding the default gRPC user-agent string.\n([#&#8203;14686](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14686))\n  The otlp gRPC exporter was unconditionally setting the User-Agent via\ngrpc.WithUserAgent() at dial time, which takes precedence over per-call\nmetadata, causing any user-configured User-Agent to be silently\ndiscarded.\nA dedicated `UserAgent` field has been added to `ClientConfig` which,\nwhen\nset, is used in the dial option directly instead of the default\nBuildInfo-derived string.\n\n- `pkg/config/configgrpc`: Accept gRPC resolver scheme URIs in client\nendpoint (e.g. passthrough:///host:port) to allow control over name\nresolution\n([#&#8203;14990](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14990))\nAfter the migration to grpc.NewClient, some gRPC client components such\nas the OTLP\nexporter experienced connection issues in dual-stack DNS environments.\nThis can now be\nfixed by using the passthrough:/// gRPC resolver scheme in the endpoint\nfield.\n\n- `pkg/config/confignet`: Add support for Windows Named Pipe (npipe)\ntransport\n([#&#8203;15085](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15085))\n\n- `pkg/service`: Emit a warning when using the old v0.2.0 declarative\nconfig format\n([#&#8203;15088](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15088))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/otelcol`: Print components exactly once in the `otelcol\ncomponents` command\n([#&#8203;14682](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14682))\n  This resolves an issue where aliased components were skipped.\n\n- `pkg/otelcol`: Synchronize Collector Run and Shutdown lifecycles so\nthat Shutdown blocks until Run completes all cleanup.\n([#&#8203;4947](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/4947))\nShutdown now blocks until Run finishes cleanup, matching http.Server\nsemantics.\nIf Shutdown is called before Run, the next Run call returns nil after\ncleaning up\n  the config provider.\n\n- `pkg/pdata`: Use spec-compliant string representation for NaN,\nInfinity, and -Infinity in Value.AsString().\n([#&#8203;14487](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14487))\n\n- `pkg/pprofile`: Fix data corruption of resource and scope attributes\nafter marshal-unmarshal-merge round-trip.\n([#&#8203;15084](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15084))\n\n- `pkg/service`: Non-string resource attributes in telemetry\nconfiguration now return an error instead of panicking\n([#&#8203;15171](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15171))\n\n- `pkg/xscraperhelper`: fix the merge of profiles in the profiling\nscraper helpers\n([#&#8203;14790](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14790))\n\n- `receiver/otlp`: Fix profiles receiver reporting its samples as spans\n([#&#8203;15089](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15089))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-05T12:14:50Z",
          "tree_id": "96351c4e7feacba2bb2ed0e4535202693efbe663",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4e674f6909f04cdd6726c42e3ad0f3281b3d6e"
        },
        "date": 1777988777639,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1299434900283813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.814225004285036,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.463173203372785,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.235677083333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.245781622338,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.50844582146,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003518,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213831.81655335153,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176905.14712334104,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f0f16046495687c7e36810683ff10e0fde7b1eaa",
          "message": "Update geneva-uploader digest to ce866b4 (#2831)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `0022519` →\n`ce866b4` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-05-05T16:57:34Z",
          "tree_id": "718c7b68ba983548ed2496f6d9c8ce68080be9ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0f16046495687c7e36810683ff10e0fde7b1eaa"
        },
        "date": 1778014467176,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.133144497871399,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.682583046823742,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.0351350932879155,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.691666666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6024.195074779885,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6092.457908488438,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003369,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212632.376024336,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175635.90974702814,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "29de46bb4dbff6e48b595459188f912b49373eed",
          "message": "Add capability to evaluate field-level type checks (#2794)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nPart 2 of what was started in\nhttps://github.com/open-telemetry/otel-arrow/pull/2754 ...\n\nAdds the capability to OPL and the OTAP query-engine to evaluate logical\nexpressions that check the type of some field. For example:\n```kql\nlogs | where attributes[\"x\"] is String\nlogs | where attributes[\"y\"] is Integer\n```\n\nThis i useful for cases where users may wish to do something with a\nfield of unknown type (an `AnyValue`) and the operation would fail if\nthe value was not the correct type. For example, maybe I wish to redact\nlog body only if it is text, when I know that sometimes the body will\nactually be an int:\n```kotlin\nlogs | if (body is String) {\n  set body = concat(substring(body, 0, 3), \"****\") // <-- substring only accepts string type\n}\n```\n\nIn this case we use the `GetType` expression which is only supposed to\nreturn a value from the `ValueType` enum (see discussion here:\nhttps://github.com/open-telemetry/otel-arrow/pull/2754#discussion_r3139794049),\nso currently only these values are supported. Although there are in fact\nmany other types supported by OTel (e.g. specific int types like uint32,\nsome etc.) the set of types in `ValueType` seem to be enough for most\nuseful expressions, at least in the context of what the engine currently\nsupports.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2752\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this type of expressions is now available in transform processor.\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-05-05T18:19:33Z",
          "tree_id": "95bf22e723ad09cb009d8b25f8b0c193629d3c0b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/29de46bb4dbff6e48b595459188f912b49373eed"
        },
        "date": 1778020367685,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2729843854904175,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.766881139687753,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.425954647473106,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.635286458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6032.722801466984,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6109.5184241164925,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00342,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213622.2294589405,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176760.9256996319,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "d.dahl@f5.com",
            "name": "David Dahl",
            "username": "daviddahl"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "21b02e791e3af690fcb61d44e27b9311f597b85a",
          "message": "[PerfTest] Fix shutdown hooks: use wait=false with readyz polling (#2795) (#2796)\n\n## Summary\n\nFixes #2795.\n\nThe `df-loadgen-steps-docker*.yaml` test step templates used `wait=true`\non shutdown hooks, which blocks until all pipelines terminate. The admin\nserver **stays alive after pipeline shutdown by design** (`run_forever`\nparks the main thread), so `wait=true` returns **504 Gateway Timeout**\nwhen drain takes longer than `timeout_secs`.\n\nSince `send_http_request` defaults to `raise_for_status: true` with no\n`on_error` configured, this 504 aborts the entire suite before the\nreport step -- losing all metrics collected during the observation\nwindow.\n\n## Changes\n\nSwitch from blocking `wait=true` to non-blocking `wait=false` with\nactive readyz polling across all three affected template files:\n\n| File | Hooks fixed |\n|---|---|\n| `df-loadgen-steps-docker.yaml` | 3 (ports 8085, 8086, 8087) |\n| `df-loadgen-steps-docker-filtered.yaml` | 3 (ports 8085, 8086, 8087) |\n| `df-loadgen-steps-docker-otel.yaml` | 1 (port 8085) |\n\n**Pattern for each shutdown hook:**\n1. `POST .../shutdown?wait=false` -- returns 202 immediately, drain runs\nin background\n2. Poll `/readyz` every 5s until 503 (pipelines terminated) or endpoint\nstops responding\n3. `on_error: continue: true` so 404/connection errors (service already\nexited) are non-fatal\n4. Polling loop exits with a warning after 300s (container will be\nkilled by `destroy` regardless)\n\n## Testing\n\nVerified in downstream deployments using the orchestrator framework\nacross a range of workloads. The polling approach provides clear log\noutput (\"Engine pipelines stopped after Ns\") and never aborts the suite\ndue to shutdown timing.\n\n/cc @cijothomas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-05T20:00:56Z",
          "tree_id": "e3c117b03d4a3966f95e81030e882cc3b33aa25e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/21b02e791e3af690fcb61d44e27b9311f597b85a"
        },
        "date": 1778021613467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.787649060136712,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.308656254832225,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.743880208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5972.982420616122,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5998.580916704476,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003525,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213980.78378728038,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177652.54382963944,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "AaronRM@users.noreply.github.com",
            "name": "Aaron Marten",
            "username": "AaronRM"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "93af768ae44406440fd200341f8e7fb63784f132",
          "message": "fix(tests): extend backdating duration to prevent flaky tests due to timing jitter (#2842)\n\n# Change Summary\n\nExtends the backdating durations used in the Quiver\n`startup_preserves_fresh_segments_deletes_expired` test to prevent flaky\ntest failures.\n\n## What issue does this PR close?\n\n* Addresses the flaky test result for\n`startup_preserves_fresh_segments_deletes_expired` reported in #2720.\n\n## How are these changes tested?\n\nValidated that test passes locally on re-run.\n\n## Are there any user-facing changes?\n\nNo. This is a test only change.",
          "timestamp": "2026-05-05T20:05:37Z",
          "tree_id": "829dfa521597283206d12e4943acdb5485bb44e7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/93af768ae44406440fd200341f8e7fb63784f132"
        },
        "date": 1778022651153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.845070481300354,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.76439902938709,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.097792900781069,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.417057291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.35546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6057.7412457290275,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6006.549066187656,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009166,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212650.40268972504,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175491.78054221734,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a43d415c137bd8d7d82ac55cadcf556cae36ea43",
          "message": "ci(rust): speed up Linux ARM build_nonrequired (#2836)\n\nThe ubuntu-24.04-arm build_nonrequired job has been running ~55 minutes\n(vs ~13 minutes on macos-latest with the same default features), tying\nup the merge queue. Apply the same pattern used to fix the Windows build\nin #2695:\n\n* Use a reduced feature set ('crypto-ring,jemalloc,azure,aws,\nazure-monitor-exporter,contrib-processors,unsafe-optimizations'),\nmirroring the Windows 'Incomplete' variant.\n\nPart of #2591\nFixes #2737\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-05T20:15:52Z",
          "tree_id": "38927bb1bb2d46279c888f66e5bf30ad4a5fdb01",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a43d415c137bd8d7d82ac55cadcf556cae36ea43"
        },
        "date": 1778023706239,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.773123608858828,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.241242621932277,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.662109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.07421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6007.118353920779,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.9795782007905,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003479,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213401.48837835185,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176297.43742972155,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jmacd@users.noreply.github.com",
            "name": "Joshua MacDonald",
            "username": "jmacd"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3b7ca8b02321fd1e267c17eaab46027a7b474b9e",
          "message": "ci(codecov): wait for all uploads; use auto project target; raise patch to 80% (#2843)\n\nCodecov was reporting a failing project status (e.g. 56% vs target 85%)\nbefore all test jobs had even finished, because it computed the status\nfrom the first upload that arrived. We upload coverage from three jobs:\nrust-ci 'test' (Linux x86_64), rust-ci 'coverage', and go-ci\n'test_and_coverage'. Set codecov.notify.after_n_builds: 3 (with\nwait_for_ci: true) so the status is computed only after all three\nreports have been received.\n\nAlso relax the thresholds:\n- project: switch to 'target: auto' with a 1% threshold so PRs only fail\nif they meaningfully decrease overall coverage, rather than needing the\nwhole repo to hit a fixed 85%.\n- patch: raise to 80% (from 70%) for changed lines in a PR.\n\nFixes #2591\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-05T20:15:58Z",
          "tree_id": "ac11a002a5f77911a9527222697441a1b4741898",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b7ca8b02321fd1e267c17eaab46027a7b474b9e"
        },
        "date": 1778024755734,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4184396266937256,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.753230316809878,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.297904408925952,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.905338541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.19140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6015.635251982555,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.963411585144,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003638,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213264.26097467577,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176385.6063760156,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5e496a9aaf2b81c147e775b770a542ca6510b8c1",
          "message": "fix(perf-test): flush loadgen metrics during run, not only on thread exit (#2822)\n\nThe Python load generator's three worker threads only flushed\n`logs_produced` / `failed` / `bytes_sent` to the shared metrics dict on\nthread exit, so every Prometheus scrape during the observation window\nsaw `0` — causing the [nightly syslog\ndashboard](https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/syslog/)\nto publish `logs_produced_rate=0` and `dropped_logs_percentage=0` for\nevery syslog test since #2723 (2026-04-23). This PR flushes accumulators\nonce per send-loop iteration; verified locally that `logs_produced_rate`\nis now `~12k` (UDP) / `~95k–200k` (TCP) instead of `0`.\n\nCurrent state where logs count shows 0.\n<img width=\"457\" height=\"667\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/c4d1bfbd-bddc-42c3-919a-d8ee056e64a5\"\n/>",
          "timestamp": "2026-05-05T21:48:25Z",
          "tree_id": "e8921a6d1af4161bbfd118121b6b8c4a1f15f817",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5e496a9aaf2b81c147e775b770a542ca6510b8c1"
        },
        "date": 1778025789337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.14064697921276093,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.706393195542407,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.230330418633445,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.501171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.69921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6066.5087213312045,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6075.041082401994,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006837,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 212288.54085377904,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 175445.69880153637,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "43b48f2f0173874c81756d913b4cef9cd9a58c25",
          "message": "Update all patch versions (#2801)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/fxamacker/cbor/v2](https://redirect.github.com/fxamacker/cbor)\n| `v2.9.1` → `v2.9.2` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2ffxamacker%2fcbor%2fv2/v2.9.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2ffxamacker%2fcbor%2fv2/v2.9.1/v2.9.2?slim=true)\n|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.5` → `v1.18.6` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.6?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.5/v1.18.6?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>fxamacker/cbor (github.com/fxamacker/cbor/v2)</summary>\n\n###\n[`v2.9.2`](https://redirect.github.com/fxamacker/cbor/releases/tag/v2.9.2)\n\n[Compare\nSource](https://redirect.github.com/fxamacker/cbor/compare/v2.9.1...v2.9.2)\n\nThis release refactors and hardens the streaming encoder by adding\nstricter checks for encoding CBOR indefinite-length data. Other changes\ninclude minor bugfixes, defensive checks, and more tests.\n\nProjects that don't use CBOR indefinite-length data may also want to\nupgrade ([summary of prior\nreleases](https://redirect.github.com/fxamacker/cbor#prior-releases)).\n\nThe stricter checks in the encoder prevent improper use of the library\nand bad inputs from producing malformed CBOR indefinite-length data that\nwould be rejected by the decoder.\n\nThis release passed fuzz tests (billions of execs) and it is production\nquality.\n\n#### What's Changed\n\n- Reject encoding indefinite-length map with odd item count by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;764](https://redirect.github.com/fxamacker/cbor/pull/764)\n- Reject encoding indefinite-length data item as a chunk inside\nindefinite-length byte string or text string by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;765](https://redirect.github.com/fxamacker/cbor/pull/765)\n- Make TagSet.Remove a no-op when contentType is nil by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;766](https://redirect.github.com/fxamacker/cbor/pull/766)\n- Refactor indefinite-length encoding and improve chunk validation\nduring encoding by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;767](https://redirect.github.com/fxamacker/cbor/pull/767)\n- Add more tests, fix a nit in unreachable panic message, update docs &\nci by [@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;768](https://redirect.github.com/fxamacker/cbor/pull/768)\n\n##### CI / GitHub Actions and Docs\n\n<details><summary>:mag_right: Details...</summary><p>\n\n- Bump actions/setup-go from 6.3.0 to 6.4.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;760](https://redirect.github.com/fxamacker/cbor/pull/760)\n- Bump github/codeql-action from 4.34.1 to 4.35.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;761](https://redirect.github.com/fxamacker/cbor/pull/761)\n- Bump github/codeql-action from 4.35.1 to 4.35.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;763](https://redirect.github.com/fxamacker/cbor/pull/763)\n- Update README for v2.9.2 release by\n[@&#8203;fxamacker](https://redirect.github.com/fxamacker) in\n[#&#8203;769](https://redirect.github.com/fxamacker/cbor/pull/769)\n\n</details>\n\n**Full Changelog**:\n<https://github.com/fxamacker/cbor/compare/v2.9.1...v2.9.2>\n\n</details>\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.6`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.6)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.5...v1.18.6)\n\n#### What's Changed\n\n- s2: Fix amd64 stack frame corruption by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1145](https://redirect.github.com/klauspost/compress/pull/1145)\n- gzhttp: Canonicalize ETag header by\n[@&#8203;justinmayhew](https://redirect.github.com/justinmayhew) in\n[#&#8203;1139](https://redirect.github.com/klauspost/compress/pull/1139)\n- perf: pool hash tables in Go encode paths to reduce allocations by\n[@&#8203;huynhanx03](https://redirect.github.com/huynhanx03) in\n[#&#8203;1143](https://redirect.github.com/klauspost/compress/pull/1143)\n\n#### New Contributors\n\n- [@&#8203;justinmayhew](https://redirect.github.com/justinmayhew) made\ntheir first contribution in\n[#&#8203;1139](https://redirect.github.com/klauspost/compress/pull/1139)\n- [@&#8203;huynhanx03](https://redirect.github.com/huynhanx03) made\ntheir first contribution in\n[#&#8203;1143](https://redirect.github.com/klauspost/compress/pull/1143)\n- [@&#8203;thaJeztah](https://redirect.github.com/thaJeztah) made their\nfirst contribution in\n[#&#8203;1144](https://redirect.github.com/klauspost/compress/pull/1144)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.5...v1.18.6>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-05-05T22:21:05Z",
          "tree_id": "351e4b7415aaad326fafd205990c3385cf037324",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43b48f2f0173874c81756d913b4cef9cd9a58c25"
        },
        "date": 1778026840403,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8547009229660034,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.75180746704229,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.206558848564575,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.9078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 5989.7163737045485,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6040.910530744758,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006848,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213548.01947030422,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176601.55405596356,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "78856dcb2ecd93270265296c7c279cd9ab877e24",
          "message": "feat(otap-dataflow): Add stopwatch `signals.incoming` and `signals.outgoing` metrics (#2839)\n\n# Change Summary\n\nFollow-up to #2747\n\nAdds two MMSC metrics on the existing `stopwatch` metric set so\noperators can compare signal volume in vs. out across a stopwatch range,\nalongside the existing combined compute duration. \"Signals\" here means\nindividual log records, spans, or metric data points\n(`OtapPdata::num_items()`).\n\n| Metric | Recorded | Why |\n|---|---|---|\n| `stopwatch.signals.incoming` | At the start node, **before**\n`process()` runs | Any filter/drop in the start processor itself does\nnot undercount entry volume. |\n| `stopwatch.signals.outgoing` | At the stop node, **after** `process()`\ncompletes | Reflects what actually leaves the range. |\n\nImplemented as two metric set types (`StopwatchStartMetrics`,\n`StopwatchStopMetrics`) sharing one entity per stopwatch — mirrors the\n`ChannelSenderMetrics` / `ChannelReceiverMetrics` precedent. Each role\nregisters its own `MetricSet` against the same entity and drains its own\naccumulator on the periodic `CollectTelemetry` tick and at shutdown.\n\nTo capture the incoming count pre-process, the existing\n`ProcessorSendHook` trait is renamed to `FlowMeasurementHook` and gains\na default-no-op `after_processor_receive` method. The engine run loops\n(Local + Shared) call it immediately after `inbox.recv_when(...)`\nreturns a `Message::PData`, before `begin_process_timing` and\n`process()`. `OtapPdata` overrides it to drive the start-side counter;\ntest PData stand-ins (`()`, `String`, `TestMsg`) get blanket no-op\nimpls.\n\nThe two hooks fire from different surfaces by design, matching the\nasymmetric flow control of a processor:\n\n| Hook | Fires from | Cardinality per `process()` | Captures |\n|---|---|---|---|\n| `after_processor_receive` | Engine run loop | Exactly 1 (1 dequeue per\niteration) | True pre-process input volume |\n| `before_processor_send` | Effect handler `send_message[_to]` | 0..N\n(drop, pass-through, or fan out) | What actually leaves |\n\n**Behavior change:** removed the `PROCESS_DURATION` gate in\n`build_stopwatch_state`. Stopwatches are already explicit opt-in via the\ntelemetry policy YAML; the gate was redundant and signal counts don't\nneed the timing path. Pipelines with stopwatches under `runtime_metrics:\nbasic`/`none` will now run them instead of silently skipping.\n\n## Demo\n\n`configs/fake-stopwatch-demo.yaml` now includes a 1-in-3\n`processor:log_sampling` node inside the stopwatch range so\n`signals.outgoing` is visibly smaller than `signals.incoming`.\n\n```bash\ncargo run --bin df_engine -- --config configs/fake-stopwatch-demo.yaml\ncurl -s 'http://127.0.0.1:8080/api/v1/telemetry/metrics?format=json' \\\n  | jq '.metric_sets[] | select(.name == \"stopwatch\")'\n```\n\nSample output (truncated, after ~38 collection cycles at 10\nsignals/sec):\n\n```json\n{\n  \"name\": \"stopwatch.signals.incoming\",\n  \"value\": { \"min\": 10.0, \"max\": 10.0, \"sum\": 380.0, \"count\": 38 }\n}\n{\n  \"name\": \"stopwatch.compute.duration\",\n  \"value\": { \"min\": 2859829.0, \"max\": 6619768.0, \"sum\": 170014602.0, \"count\": 38 }\n}\n{\n  \"name\": \"stopwatch.signals.outgoing\",\n  \"value\": { \"min\": 3.0, \"max\": 4.0, \"sum\": 127.0, \"count\": 38 }\n}\n```\n\nReading: 380 signals entered the range (38 batches × 10 signals), 127\nleft it (≈1/3, matching the sampler ratio), and `compute.duration`\naverages ~4.47 ms per batch across the chain (170014602 ns / 38). Both\nsignal-count metrics share the same `stopwatch.name` / `start_node` /\n`stop_node` attributes as the duration metric, so they correlate without\njoins.\n\n## What issue does this PR close?\n\n* Related to #2782 \n* Closes #2837 \n\n## How are these changes tested?\n\nUnit Tests / Local runs\n\n## Are there any user-facing changes?\n\n1. Stopwatch duration metric will now be tracked and emitted even on\n`runtime_metrics: basic/none`.\n2. New Stopwatch metrics for `consumed` and `produced`",
          "timestamp": "2026-05-05T23:00:46Z",
          "tree_id": "d012eaced09e897594792ffe8c20e93222c0c231",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/78856dcb2ecd93270265296c7c279cd9ab877e24"
        },
        "date": 1778027887590,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2729843854904175,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.779009317539973,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.174735050669143,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.4953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.78125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6031.150570130537,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6107.926178519751,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.019062,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214427.07831625224,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177503.78557134996,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7615259cc149f767ce7d03e179311e07e5bfe2e2",
          "message": "Add node-local delayed resume to ProcessorInbox (#2471)\n\n# Change Summary\n\nAdd processor-local delayed `pdata` resume via\n`EffectHandler::requeue_later(...)`, integrated into `ProcessorInbox`,\nand migrate `retry_processor` from the old global `delay_data(...)`\npath.\n\nThis PR completes the delayed-payload side of the node-local scheduling\nwork. The existing wakeup path remains for lightweight timer-style\ncallbacks, while `requeue_later(...)` is for retry-style retained\npayloads that must be delivered back to the same processor later.\n\nThe goal is to move retry-style deferred work out of the global runtime\npath and into node-local inbox orchestration, while keeping the old\nglobal delayed-data plumbing in place temporarily for compatibility.\nThat keeps this PR focused on local delayed resume and retry migration\nonly; the cleanup and removal of the old global path happens in the next\nPR.\n\n## What Changed\n\n- Added a bounded processor-local delayed-resume queue in the node-local\nscheduler.\n- Exposed `EffectHandler::requeue_later(when, data)` with\npayload-preserving rejection.\n- Delivered due local delayed resumes through `ProcessorInbox` as\n`NodeControlMsg::DelayedData`.\n- Preserved inbox control/pdata fairness for delayed resumes.\n- On processor shutdown, pending delayed resumes are surfaced\nimmediately for drain; new delayed resumes are rejected.\n- Migrated `retry_processor` to schedule retries with\n`requeue_later(...)`.\n\n## Benchmark\n\nResults based on a local Criterion benchmark.\n\n| Scenario | `delay_data` | `requeue_later` | Result |\n| --- | ---: | ---: | ---: |\n| Schedule only | ~1.247 ms / 10k | ~286.9 us / 10k | ~4.35x faster |\n| Schedule + due delivery | ~2.720 ms / 10k | ~1.911 ms / 10k | ~1.42x\nfaster |\n\nThe new path removes the runtime-control scheduling hop for retry-style\nretained pdata and keeps delayed resume state local to the processor\ninbox/scheduler.\n\n## What issue does this PR close?\n\n* Part of #2465\n\n## How are these changes tested?\n\n- Added scheduler tests for delayed resume ordering, capacity,\npayload-preserving rejection, and shutdown drain conversion.\n- Added inbox tests for delayed-resume delivery, fairness, shutdown\nrejection, and shutdown drain behavior.\n- Updated retry tests for local delayed retry and terminal NACK behavior\nwhen `requeue_later(...)` rejects.\n- Ran targeted engine checks/tests for the scheduler and inbox behavior.\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nNo user-facing changes.\n\nThis PR is an internal runtime change only.",
          "timestamp": "2026-05-06T03:13:41Z",
          "tree_id": "5dbf69cbd93a5c6cd0f6c83a4febda12d9578c9e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7615259cc149f767ce7d03e179311e07e5bfe2e2"
        },
        "date": 1778039890968,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9887005090713501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.803074946238963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.320074280408543,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.043619791666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6041.250513657785,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6100.980391617679,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003471,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214399.87059795065,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 177408.6571240466,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "cijo.thomas@gmail.com",
            "name": "Cijo Thomas",
            "username": "cijothomas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ed9830013345f296033c4c29d53917561622a5cf",
          "message": "refactor(perf-test): factor loadgen flush_batch_metrics into _BatchMetricsAccumulator (#2847)\n\nFollow-up to #2822 addressing review feedback from @jmacd — the\nper-thread accumulator and flush logic was duplicated across all three\nworker functions. Extract it into a small `_BatchMetricsAccumulator`\nhelper class (with `__slots__`) so each worker is just `acc =\n_BatchMetricsAccumulator(self); …; acc.flush()`.\n\nBehavior unchanged. Smoke-tested with the rebuilt loadgen container —\n`/metrics` counters tick up at the expected ~5000 logs/sec during a\nsyslog UDP run, same as before. Net diff: `+58 / -84`.",
          "timestamp": "2026-05-06T15:00:46Z",
          "tree_id": "d096833928a9fb6b09c912bf7096d31e79ffbd99",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed9830013345f296033c4c29d53917561622a5cf"
        },
        "date": 1778083189489,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.8335683345794678,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 5.709314579388946,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 6.180060535506403,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.831640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.75390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 6049.429986273596,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5938.509549289735,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006976,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214047.67800631875,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 176852.93284520117,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}