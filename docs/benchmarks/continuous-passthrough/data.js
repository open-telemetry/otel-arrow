window.BENCHMARK_DATA = {
  "lastUpdate": 1789006651836,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla",
            "email": "66651184+utpilla@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "d25a5680f19dfa4f53f2e5c3179ce07ea2c8d3f8",
          "message": "chore: Fix flaky quiver WAL replay tests by disabling time-based segment finalization (#3533)\n\n# Change Summary\n\n## Problem\n`wal_replay_reads_from_rotated_files` intermittently fails in CI with:\n\n```\nassertion `left == right` failed: no segments should be finalized\n  left: 2, right: 0\n```\n\nBoth this test and `wal_replay_finalizes_segments_if_threshold_exceeded`\nset a large `target_size_bytes` and assert that ingesting 20 bundles\nfinalizes zero segments (so the data stays in the WAL). But segment\nfinalization also triggers on `max_open_duration`, which defaulted to 5\nseconds. On a loaded CI runner, ingesting the bundles occasionally\nexceeded 5 seconds of wall-clock time, causing time-based finalization\nand breaking the assertion.\n\nHere's a sample CI run that fails with these tests:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/29763886758/job/88433587864?pr=3528\n\n## Fix\n\nSet max_open_duration: `Duration::from_secs(3600)` in both tests'\n`SegmentConfig` so only size/stream limits (which the tests never hit)\ncan trigger finalization. This removes the wall-clock dependency and\nmakes the tests deterministic. It matches the convention already used\nelsewhere in this file.\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nTest-only change; no production behavior is affected.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-21T21:54:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d25a5680f19dfa4f53f2e5c3179ce07ea2c8d3f8"
        },
        "date": 1784689502021,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.10049628466367722,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.24403476384174,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49595402655898,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.438151041666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.8125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2063321.0376210727,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2065394.5984653323,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001133,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23428846.156722102,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23373717.61601297,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.34352059123745,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4855319261550903,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22660098244418,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51066872110941,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.496354166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.14453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 539931.6433959942,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531910.7867689064,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003566,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15769141.217759324,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15757676.819932958,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.646214384086885,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "93e7bef509ab3321aa38fff99acef9d0b2efbf97",
          "message": "feat(engine): Add per-signal produced/consumed metrics for all nodes (#3437)\n\n# Change Summary\n\n### Motivation\n\nOur end goal is surfacing universal node telemetry, aligned with the\nOpenTelemetry Collector [component universal telemetry\nRFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).\n\nRather than introduce receiver-only, processor-only, or exporter-only\ncounters, this PR extends the **existing** `node.producer` /\n`node.consumer` metric sets with per-signal item counts. Because these\nmetric sets are emitted by every node, **all nodes** get the per-signal\nbreakdown uniformly.\n\n### Methodology\n\nFollow the same pattern as other produced and consumed metrics but\nsimply recording into per-signal counters during the same Ack/Nack\nunwinding with `Frames`.\n\nCounting items is expensive to have on the hot path, so it is **off by\ndefault**. Enable it either:\n\n- **broadly**, via `telemetry.runtime_metrics: detailed`, or\n- **per node**, via a narrow telemetry override:\n  ```yaml\n  nodes:\n    my_receiver:\n      policies:\n        telemetry:\n          item_counts: true\n  ```\n\nThis mirrors the per-node `header_capture` / `header_propagation`\nprecedent: the node exposes only the honored knob, not the full\n`TelemetryPolicy`. Counts require `runtime_metrics: normal` or higher;\nwhen a node hasn't opted in, the fields read 0.\n\n### Demo & verification\n\n`configs/trafficgen-per-signal-metrics-demo.yaml` runs two pipelines\nwith `receiver → sampler(emit 1/3 logs) → noop`, traffic 50:30:20\nlogs:metrics:spans. `main` opts every node in; `partial` opts in only\n`sampler`.\n\n```\ncurl -s 'http://127.0.0.1:8080/api/v1/telemetry/metrics?format=json' | jq '[.metric_sets[] | select(.name==\"node.producer\" or .name==\"node.consumer\")]'\n```\n\n### `full` pipeline\n\n| Node | Scope | Evidence |\n| --- | --- | --- |\n| `receiver` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`produced_items_total{signal=\"logs\",outcome=\"success\"} =\n3360`<br>`produced_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`produced_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n| `sampler` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`consumed_items_total{signal=\"logs\",outcome=\"success\"} =\n3360`<br>`consumed_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`consumed_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n| `sampler` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`produced_items_total{signal=\"logs\",outcome=\"success\"} =\n1120`<br>`produced_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`produced_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n| `noop` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`consumed_items_total{signal=\"logs\",outcome=\"success\"} =\n1120`<br>`consumed_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`consumed_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n\n### `partial` pipeline\n\n| Node | Scope | Evidence |\n| --- | --- | --- |\n| `receiver` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`produced_items_total{signal=\"logs\"} =\n0`<br>`produced_items_total{signal=\"metrics\"} =\n0`<br>`produced_items_total{signal=\"traces\"} = 0` |\n| `sampler` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`consumed_items_total{signal=\"logs\",outcome=\"success\"} =\n4380`<br>`consumed_items_total{signal=\"metrics\",outcome=\"success\"} =\n2628`<br>`consumed_items_total{signal=\"traces\",outcome=\"success\"} =\n1752` |\n| `sampler` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`produced_items_total{signal=\"logs\",outcome=\"success\"} =\n1460`<br>`produced_items_total{signal=\"metrics\",outcome=\"success\"} =\n2628`<br>`produced_items_total{signal=\"traces\",outcome=\"success\"} =\n1752` |\n| `noop` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`consumed_items_total{signal=\"logs\"} =\n0`<br>`consumed_items_total{signal=\"metrics\"} =\n0`<br>`consumed_items_total{signal=\"traces\"} = 0` |\n\nThe partial/sampler rows (both consumer and producer) show full counts\nbecause sampler is the one opted-in node, while its neighbors\nreceiver / noop read 0.\n\n### Performance\n\nI extended the existing item count benchmark to prove the following:\n\n| Payload | Log records per batch | Item counting disabled | Item\ncounting enabled | Incremental overhead |\n| --- | ---: | ---: | ---: | ---: |\n| OTLP | 10 | 0.98 ns | 251 ns | ~250 ns |\n| OTLP | 100 | 0.94 ns | 2.01 µs | ~2.00 µs |\n| OTLP | 1,000 | 0.94 ns | 18.95 µs | ~18.01 µs |\n| OTAP | 10 | 1.30 ns | 1.91 ns | ~0.62 ns |\n| OTAP | 100 | 1.24 ns | 1.87 ns | ~0.64 ns |\n| OTAP | 1,000 | 1.24 ns | 1.88 ns | ~0.64 ns |\n\nOTLP item-count cost scales approximately linearly with the number of\nlog records because it traverses the encoded protobuf payload. OTAP item\ncounting stays effectively constant because it reads Arrow batch\nmetadata.\n\nFollow-up issue https://github.com/open-telemetry/otel-arrow/issues/3548\nwould help optimize the OTLP path on certain pipelines.\n\n## What issue does this PR close?\n\n- Related to #3300\n- Closes #3436 \n\n## How are these changes tested?\n\nUnit tests / sample config run\n\n## Are there any user-facing changes?\n\nYes, users will see new per-signal `produced` and `consumed` metrics for\neach node depending on telemetry policy configuration.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-07-22T17:06:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/93e7bef509ab3321aa38fff99acef9d0b2efbf97"
        },
        "date": 1784743728688,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6588415503501892,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20413604186705,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4740749884277,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.772265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.84765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 533572.5638440759,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 530057.1658797084,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005724,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15753229.147617184,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15687170.270595295,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.719868273966952,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9919167757034302,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21726586332863,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5558852192745,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.6359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.60546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2007639.6028782318,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1987725.4889602973,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008093,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23012701.603278898,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22905766.43631008,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.57740429002395,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "7987c8b5f859c8febe4d0a1fc3e1b28dc48bc71e",
          "message": "feat(wasm-host): introduce experimental WASM host-kernel processor plugin (#3478)\n\n# Change Summary\n\n- Added `otap-df-wasm-host` crate for WASM host-kernel runtime.\n- Implemented simple `severity-filter` reference guest plugin to filter\nlog records by severity.\n- Created integration tests to validate the functionality of the WASM\nprocessor.\n- Established WIT contract for OTAP dataflow WASM plugins.\n- Introduced bridge and host modules for managing data between the host\nand guest.\n- Until stabilized, the binary plugins feature is disabled by default in\nbuilds and must be enabled with the `wasm` flag\n\n## What issue does this PR close?\n\n- Starts implementation of #2973 and #3227 \n\n## How are these changes tested?\n\n- Integration and unit tests included\n\n## Are there any user-facing changes?\n\n- When the `wasm` flag is enabled, builds the experimental\n`otap-df-wasm-host` crate and support for WASM binary plugins.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-22T21:56:11Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7987c8b5f859c8febe4d0a1fc3e1b28dc48bc71e"
        },
        "date": 1784773589944,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.71977299451828,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.15589682927111,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.04824023650225,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.15390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.8515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 369860.2310789855,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 367198.0770176515,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005543,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10915393.981432581,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10840274.928826781,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.726174140361493,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.13069595396518707,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18838046049198,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57265627411638,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.86536458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.01953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2036252.8845579084,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2033591.58438456,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.024796,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23105704.801861,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23000308.30098212,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.362018302634569,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "257cceb07ac9fe49f0ee6489f4dcdd1ea41db676",
          "message": "chore(deps): update geneva-uploader digest to 16f987e (#3553)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `c0a28d7` →\n`16f987e` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNjUuMSIsInVwZGF0ZWRJblZlciI6IjQzLjI3Mi40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->",
          "timestamp": "2026-07-23T16:03:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/257cceb07ac9fe49f0ee6489f4dcdd1ea41db676"
        },
        "date": 1784830292756,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.18912918865680695,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19558301536442,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48578715511245,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.452083333333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.98828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 518821.1694347708,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519802.41167611605,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00557,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15935289.744523836,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15873116.485619968,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.65643672783297,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.07942641526460648,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21231138008856,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57198049686558,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.15065104166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.26171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1987486.1825550487,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1989064.771528828,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002953,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22988275.626606796,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22885157.691338293,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.557328828933825,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Thomas",
            "username": "thperapp",
            "email": "88447796+thperapp@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9d6e1ce4a5b728dac98d1e60a36ba5285e50dc82",
          "message": "refactor(geneva): use geneva-uploader tls-rustls feature to enable the SymCrypt path (#3408)\n\n## Change Summary\n\nEnables geneva-uploader's `tls-rustls` feature (default features off),\nswitching the\nGeneva exporter from native-tls to rustls:\n\n```toml\ngeneva-uploader = { git = \"...\", rev = \"70f2dd38...\", default-features = false, features = [\"tls-rustls\"] }\n```\n\nThe Geneva exporter was the only TLS component still on native-tls\n(OpenSSL), which\nbypassed the pluggable rustls crypto provider. It now rides the\nprocess-wide provider\ninstalled at startup, so Geneva uploads work end-to-end with SymCrypt\n(`crypto-symcrypt`),\nconsistent with the rest of otap-dataflow.\n\n## Changes\n\n- **`Cargo.toml`**: enable `tls-rustls`.\n- **`Cargo.lock`**: drops `native-tls`/`hyper-tls`/`tokio-native-tls`\nfrom the Geneva path\nand adds the rustls stack + `p12-keystore` and its closure (`cbc`,\n`des`, `rc2`, `scrypt`,\n`pkcs12`, `x509-cert`, …). These parse the PKCS#12 client cert for\nGeneva mTLS — previously\n  handled by the OS cert store via native-tls, and unique to Geneva.\n- **`geneva_exporter/mod.rs` (tests)**: added the idempotent\n`otap_df_otap::crypto::ensure_crypto_provider()` to the 3 tests that\nbuild a `GenevaClient`\n(reqwest/rustls now needs a provider; production installs it at\nstartup).\n- **`rust-ci.yml`**: added `geneva-exporter` to the Windows\n`crypto-symcrypt` build so the\n  Geneva + SymCrypt path is compiled/linked in CI.\n\n## Note\n\nBuilds enabling `geneva-exporter` must also enable one `crypto-*`\nfeature (`crypto-ring` by\ndefault), else TLS fails at runtime — the same contract documented in\n`crypto.rs`.\n\n## Validation\n\n`cargo test -p otap-df-contrib-nodes --features\n\"geneva-exporter,otap-df-otap/crypto-ring\"`,\nclippy `--all-targets -- -D warnings`, and `cargo fmt --check` all pass.\nNo default behavior\nchange (`crypto-ring` stays default); SymCrypt routing is opt-in via\n`crypto-symcrypt`.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: 1e52f75b-8536-477a-8685-1236e3d714e3\nCopilot-Session: f07b7fb6-592f-442c-8227-5a77c7895d58",
          "timestamp": "2026-07-23T22:06:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9d6e1ce4a5b728dac98d1e60a36ba5285e50dc82"
        },
        "date": 1784859232593,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6102948188781738,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18381556918538,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3863064178989,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.447526041666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.18359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521520.26636445156,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518337.4550919305,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002301,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15903150.384863483,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15842968.64233395,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.68107509622849,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.41353800892829895,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19580148437434,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54379198266523,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.83541666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.15625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1993237.798014564,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1984995.0019115729,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00294,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22892184.996188004,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22788617.85798197,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.532615938147233,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "renovate[bot]",
            "username": "renovate[bot]",
            "email": "29139614+renovate[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "617ee3f63f6854b0d7d2a64b1c2320174acfc937",
          "message": "chore(deps): update rust crate rustls-pki-types to v1.15.1 (#3570)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [rustls-pki-types](https://redirect.github.com/rustls/pki-types) |\nworkspace.dependencies | patch | `1.15.0` → `1.15.1` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNzUuMiIsInVwZGF0ZWRJblZlciI6IjQzLjI3NS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-24T14:39:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/617ee3f63f6854b0d7d2a64b1c2320174acfc937"
        },
        "date": 1784916770281,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.37385836243629456,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18267665658749,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56164731382566,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.577083333333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.3046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1978833.5445645757,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1986231.5794715532,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002961,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22881039.05432449,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22787356.63768929,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.519824420681148,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.22829917073249817,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23831219153259,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57654404945905,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.970182291666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.3359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 515789.30685022176,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 516966.84956276877,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002919,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15834960.726433838,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15774872.569341112,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.6305147802542,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eaf8f4cca694c396409f772a902b1ef512813f3e",
          "message": "chore: improve GUID formatting and clarify comments in encoder and tests (#3537)\n\n# Change Summary\nchore: improve GUID formatting and clarify comments in encoder and tests\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-25T00:36:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaf8f4cca694c396409f772a902b1ef512813f3e"
        },
        "date": 1784945888367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.07728991657495499,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15672310646464,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48476057647606,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.285807291666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.8671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 518890.379567518,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518489.3296112662,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0025,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15865425.754137639,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15801735.343447907,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.599329336309836,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.14530885219573975,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1898133441762,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5931918847762,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.707682291666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1984727.9567090238,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1981843.9714231882,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005854,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22895121.729591116,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22803130.225374695,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.552434025948989,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eaf8f4cca694c396409f772a902b1ef512813f3e",
          "message": "chore: improve GUID formatting and clarify comments in encoder and tests (#3537)\n\n# Change Summary\nchore: improve GUID formatting and clarify comments in encoder and tests\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-25T00:36:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaf8f4cca694c396409f772a902b1ef512813f3e"
        },
        "date": 1785002323204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.16707879304885864,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21107040504609,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49443161565286,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.690625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 510716.17837497115,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509862.87994154374,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002454,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15604438.563745158,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15541212.987891497,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.605166952993756,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.7164745926856995,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18950959424942,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55739973625009,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.29114583333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.5078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1976990.040143511,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1991154.6716193547,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002973,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23011239.146207776,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22909402.00606817,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.556731113958781,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eaf8f4cca694c396409f772a902b1ef512813f3e",
          "message": "chore: improve GUID formatting and clarify comments in encoder and tests (#3537)\n\n# Change Summary\nchore: improve GUID formatting and clarify comments in encoder and tests\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-25T00:36:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaf8f4cca694c396409f772a902b1ef512813f3e"
        },
        "date": 1785032388438,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6146734952926636,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23301212860207,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52162923958494,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.691796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.51953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 520574.6949645152,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 517374.8603426346,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003101,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15863318.248621965,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15804323.210498482,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.66116942388037,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6657987833023071,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21052838884543,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.66506501547987,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.25755208333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.30859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2005710.1456371623,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1992356.1517892191,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003023,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22979858.269779146,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22883160.414517894,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.534011250519779,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eaf8f4cca694c396409f772a902b1ef512813f3e",
          "message": "chore: improve GUID formatting and clarify comments in encoder and tests (#3537)\n\n# Change Summary\nchore: improve GUID formatting and clarify comments in encoder and tests\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-25T00:36:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaf8f4cca694c396409f772a902b1ef512813f3e"
        },
        "date": 1785088861395,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.08910584449768066,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20598132055352,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57553422112109,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.286067708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.4921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1982266.3521017816,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1984032.6672303397,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002883,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22928033.22461195,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22831691.226720057,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.55627808115625,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5487416386604309,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.24390081122658,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53718264098399,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.236848958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.1640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522493.30843542377,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519626.17003111384,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001289,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15921768.005215278,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15853674.300642801,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.640812421479705,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eaf8f4cca694c396409f772a902b1ef512813f3e",
          "message": "chore: improve GUID formatting and clarify comments in encoder and tests (#3537)\n\n# Change Summary\nchore: improve GUID formatting and clarify comments in encoder and tests\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-25T00:36:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaf8f4cca694c396409f772a902b1ef512813f3e"
        },
        "date": 1785118765671,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.3735647201538086,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16111111842459,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55327866315953,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.186848958333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.61328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1982522.9220460623,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1975116.916191529,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007514,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23213140.148679506,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23116304.245558545,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.752792940197017,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.028848171234131,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20788735216124,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46540973505853,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.358072916666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522373.66150839045,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511775.4929017383,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00131,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15953258.296143832,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15891255.769340642,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.172376398271343,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mikel Blanchard",
            "username": "CodeBlanch",
            "email": "mblanchard@macrosssoftware.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "af5cdf9235b7cec83a86cb9a0fddb663dfb18d32",
          "message": "[query-engine] Add empty structure for columnar engine (#3554)\n\n# Change Summary\n\n* Add empty project structure for general purpose columnar engine\nimplementation\n\n# Details\n\nThis will be filled in over subsequent PRs.",
          "timestamp": "2026-07-27T17:24:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af5cdf9235b7cec83a86cb9a0fddb663dfb18d32"
        },
        "date": 1785176601058,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.331237316131592,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19128890845954,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49283950617284,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.968619791666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.51953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 529143.8988614277,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511516.86045307363,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009627,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16017637.468625305,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15949075.691031495,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.313997068322944,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.2645365297794342,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2391382250461,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58774928333463,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.586588541666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.94921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1970803.3831786104,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1976016.877862431,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004281,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22681820.810808178,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22578611.773115825,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.478556213216349,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "David Dahl",
            "username": "daviddahl",
            "email": "d.dahl@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "537d4ecc1d18287999b4e8573ac2aa29ec447e5c",
          "message": "feat(engine): #[component_inventory] macro + inventory module (RFC 0001 Phase 1) (#3487)\n\n# Change Summary\n\nImplements **Phase 1 of RFC 0001 (component inventory)** — the\n`#[component_inventory]` attribute macro and its runtime metadata\nsurface,\nmirroring the existing `#[capability]` → `KNOWN_CAPABILITIES` mechanism.\n\nThis is the first of the four staged PRs from the RFC / tracking issue\n#3435.\nIt adds **no component annotations** (Phase 2), **no `xtask` command**\n(Phase 3), and **no CI enforcement** (Phase 4). Zero runtime cost; no\nnew\ncrates.\n\n## What's in this PR\n\n**`otap-df-engine` — new `inventory` module\n(`crates/engine/src/inventory.rs`):**\n\n- `ComponentMeta { id, category, description, file, line, attributes }`\nand a\n`#[linkme::distributed_slice] COMPONENT_INVENTORY` populated at link\ntime,\nplus a `components()` accessor and `ComponentMeta::attribute(key)`\nhelper.\n- `Category` enum — **Phase 1 ships only the four factory categories**\n  (`Receiver`, `Exporter`, `Processor`, `Extension`), each with\n  `urn_segment()` for the macro's URN cross-check.\n- `attrs` key constants (RFC **Option A**): the attribute map stays\nfree-form\n  `&[(&str, &str)]`, with `PORT`/`PROTOCOL`/`AUTH`/… key constants for\nconsistency. Value validation (Option C) is intentionally **not** in\nPhase 1.\n\n**`otap-df-engine-macros` — new `#[component_inventory]` proc macro:**\n\n- Re-emits the annotated item unchanged and appends one\n`COMPONENT_INVENTORY`\nentry using fully-qualified `::otap_df_engine::inventory::*` paths, so\nit can\nbe invoked from any node crate (unlike `#[capability]`, which is\nengine-local).\n- **Factory case:** `id` is derived from the factory static's `name`\n(URN)\nfield — contributors write no `id`. **Non-factory items require an\nexplicit\n  URN-shaped `id`.**\n- `category` is a validated bare identifier (a misspelling like\n`Reciever` is a\ncompile error). When the URN is a string literal, `category` is\ncross-checked\nagainst the URN segment; for `const`-path URNs the value isn't visible\nat\n  macro time, so the full cross-check is deferred to the Phase 3 `xtask`\n  scanner.\n- Propagates the annotated item's `#[cfg(...)]` onto the emitted entry,\nso the\n  inventory reflects exactly what was compiled.\n\n**Tests:**\n\n- Hand-rolled macro-expansion unit tests (arg parsing, `id` derivation,\n`cfg`\n  propagation, literal-URN cross-check, attribute ordering).\n- The compile-fail paths (unknown/missing `category`, missing `id` on a\nnon-factory item, URN/category mismatch) are covered by the same unit\ntests,\n  which assert on the generated error text. `trybuild` UI tests are\nintentionally **not** used: this repo runs tests via `cargo nextest`\nfrom a\nprebuilt archive in `--offline` mode, and trybuild spawns a nested\n`cargo`\n  build for its fixture crate that cannot resolve dependencies offline.\n- End-to-end test (`crates/engine/tests/component_inventory_e2e.rs`)\nthat\nannotates a factory-style static and a non-factory struct, then reads\nback\n  `COMPONENT_INVENTORY` — validating the cross-crate link-time path.\n\n## Deferred (called out explicitly for reviewers)\n\n- **Non-factory `Category` variants** (`Admin`, `Controller`, `Cli`,\n`Subsystem`, `Safety`) from the RFC are **deferred to Phase 2**, when\nthe\nnon-factory components (admin server, controller, `dfctl`, memory\nlimiter) are\nactually annotated and their synthetic-URN scheme is settled with the\nSIG.\n- **Per-signal stability** attribute: intentionally **omitted** (a\n`TODO(stability)` documents this). Per the SIG discussion, stability is\nnot\nmodeled per-signal because many components have no signal type, or\nhandle\nmultiple signal types, so a single per-signal stability field doesn't\nfit.\n- **Attribute value validation** (RFC Option C) and the `xtask`\n  `component-inventory` command are later phases.\n\n## What issue does this PR close?\n\nRelates to #3435 (Phase 1 of 4). Does not close it — Phases 2–4 follow.\n\n## How are these changes tested?\n\n`cargo xtask check` (structure, fmt, `clippy --workspace --all-targets\n-D\nwarnings`, `test --workspace`) passes, plus `python3\ntools/sanitycheck.py` for\nthe changelog YAML. New unit and end-to-end tests included.\n\n## Are there any user-facing changes?\n\nAdds new public API (`otap_df_engine::inventory`, the\n`#[component_inventory]`\nmacro) but changes no existing runtime, API, or config behavior. A\n`.chloggen`\nentry is included.\n\n### Changelog\n\n- [x] Added a `.chloggen/*.yaml` entry",
          "timestamp": "2026-07-27T22:09:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/537d4ecc1d18287999b4e8573ac2aa29ec447e5c"
        },
        "date": 1785204925838,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.09757065773010254,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21023495260238,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43717472118958,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.02890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 515965.70615447883,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515462.27503004955,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004236,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15784346.448038852,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15722148.13497346,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.62172968355188,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.3380208909511566,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17440058575251,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55274078662124,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.8984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.84765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1984188.438383673,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1990895.4100112892,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002043,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22983972.500235625,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22887258.996821363,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.544540403609297,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Dipanshu singh",
            "username": "Dipanshusinghh",
            "email": "161134993+Dipanshusinghh@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "090c1315fb1bc011e32a14c701e621e179a0e783",
          "message": "security(admin): attach hardened response headers to API routes (#3467)\n\nCloses #3445\n\nApplies hardened security headers (`X-Content-Type-Options`,\n`X-Frame-Options`, `Referrer-Policy`, `Cache-Control`) to all\n`/api/v1/*` endpoints using `axum::middleware::map_response`.\n\nPreviously these headers were only set on static UI routes in\n`dashboard.rs`. Adding a single layer on `api_routes` in `lib.rs` means\nany new endpoint added in future automatically inherits them — no\nper-handler changes needed.\n\nAlso marks the corresponding security checklist item in\n`crates/admin/README.md` as done.",
          "timestamp": "2026-07-28T19:30:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/090c1315fb1bc011e32a14c701e621e179a0e783"
        },
        "date": 1785272783565,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.3583056628704071,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22320213304738,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.62481607418856,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.257421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.2421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521538.4250532018,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519669.7233780549,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003157,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15923253.299254268,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15873599.907275738,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.641102575202844,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.39923200011253357,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.88875195055464,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61811670268601,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.79466145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.02734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1848442.3773003868,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1855821.9507284504,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.014309,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21734280.72610061,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21776561.391042992,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.711404058761904,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Pritish Nahar",
            "username": "pritishnahar95",
            "email": "pritishnahar@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "33e8e7dead37ed702c18a0c651b7030b84c09a32",
          "message": "OAuth 2.0 Client Auth Extension design doc (#3571)\n\n# Change Summary\n\nAdds a design doc for a proposed OAuth 2.0 Client Auth extension\n(urn:otel:extension:oauth2_client_auth) — the generic, provider-neutral\ncounterpart to the Azure Identity Auth extension, modeled on the Go\ncollector's oauth2clientauthextension.\n\nThe extension acquires and background-refreshes OAuth 2.0 access tokens\n(client-credentials + JWT-bearer grants) and exposes them to data-path\nnodes through the existing `BearerTokenProvider` capability — so the\nOTLP exporters can inject a refreshed Authorization: Bearer header\nwithout embedding static credentials or doing token work on the hot\npath.\n\nKey design points covered:\n\n- Reuses the existing `BearerTokenProvider` capability (no new\ncapability machinery).\n- `Active + Shared` execution, watch-based token cache, slow-path\ncoalescing via `fetch_lock`, background refresh with expiry_buffer,\njittered scheduling + bounded exponential-backoff retry.\n- Readiness-gated startup (blocks data-path spawn until the first token\nis published, bounded by `startup_timeout`.\n- Token-endpoint TLS via the shared\n`otap_df_config::tls::TlsClientConfig`; custom CA, mTLS client cert,\nSNI.\n- Config schema, telemetry, lifecycle, security/performance\nconsiderations, validation expectations, and open questions.\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Related to #3479\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nno\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [x] This is a documentation-only PR.",
          "timestamp": "2026-07-28T23:50:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/33e8e7dead37ed702c18a0c651b7030b84c09a32"
        },
        "date": 1785291955930,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.29510682821273804,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22190561712337,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51264849755417,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.5328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.3515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 517550.38141917146,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519077.7079536286,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005505,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15911105.748129986,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15849800.911586436,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.652647001268242,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6015552282333374,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20982653589347,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54567025478201,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.491796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.09765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1984107.380327626,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1952330.807224032,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002946,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22947968.31064051,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22854564.22235362,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.754139321946994,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Pritish Nahar",
            "username": "pritishnahar95",
            "email": "pritishnahar@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "33e8e7dead37ed702c18a0c651b7030b84c09a32",
          "message": "OAuth 2.0 Client Auth Extension design doc (#3571)\n\n# Change Summary\n\nAdds a design doc for a proposed OAuth 2.0 Client Auth extension\n(urn:otel:extension:oauth2_client_auth) — the generic, provider-neutral\ncounterpart to the Azure Identity Auth extension, modeled on the Go\ncollector's oauth2clientauthextension.\n\nThe extension acquires and background-refreshes OAuth 2.0 access tokens\n(client-credentials + JWT-bearer grants) and exposes them to data-path\nnodes through the existing `BearerTokenProvider` capability — so the\nOTLP exporters can inject a refreshed Authorization: Bearer header\nwithout embedding static credentials or doing token work on the hot\npath.\n\nKey design points covered:\n\n- Reuses the existing `BearerTokenProvider` capability (no new\ncapability machinery).\n- `Active + Shared` execution, watch-based token cache, slow-path\ncoalescing via `fetch_lock`, background refresh with expiry_buffer,\njittered scheduling + bounded exponential-backoff retry.\n- Readiness-gated startup (blocks data-path spawn until the first token\nis published, bounded by `startup_timeout`.\n- Token-endpoint TLS via the shared\n`otap_df_config::tls::TlsClientConfig`; custom CA, mTLS client cert,\nSNI.\n- Config schema, telemetry, lifecycle, security/performance\nconsiderations, validation expectations, and open questions.\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Related to #3479\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nno\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [x] This is a documentation-only PR.",
          "timestamp": "2026-07-28T23:50:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/33e8e7dead37ed702c18a0c651b7030b84c09a32"
        },
        "date": 1785348611804,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.1166781634092331,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.11673711648668,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5077744256208,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.11354166666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2003813.7037849403,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2001475.690742378,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003087,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23111926.92973694,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23005692.169068035,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.54744323732674,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.11856770515441895,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.83135857702716,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 79.88460478439266,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.615885416666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.42578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 359818.16309260065,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 360244.7912205264,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00542,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11063473.462995226,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10988698.350501467,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.710988007659054,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Jake Dern",
            "username": "JakeDern",
            "email": "33842784+JakeDern@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "0909afeb86bb385afaeb146b5b86d470aad6ed28",
          "message": "fix(go): Avoid panicking when a batch has > u16 max records (#3615)\n\n# Change Summary\n\nBuilding arrow record batches can panic when the number of input records\nis too large as seen in the attached issue. This PR does not implement\nsupport for larger batches but at least avoids the panic and produces a\nmore descriptive error noting the limitation.\n\n## What issue does this PR close?\n\n* Closes #1883\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\nNew error handling behavior.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-29T23:22:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0909afeb86bb385afaeb146b5b86d470aad6ed28"
        },
        "date": 1785380802042,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7261187434196472,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23728114877346,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.63812814751685,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.398046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1978906.4222805947,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1993275.632152707,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003856,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23049981.88295611,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22941929.30878404,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.563870801983608,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.18223309516906738,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22160854337191,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5252314383349,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.653255208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.76953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519725.332982655,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518778.22142708907,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005603,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15870758.272239063,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15805101.088324668,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.592568494068125,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c52449cc22f84f5c766a7032c733060d82c32d2a",
          "message": "chore(docs): Add metric guide for node and flow configurations (#3618)\n\n# Change Summary\n\nAdd missing user-facing documentation about node metrics from #3437 as\nwell as `flow_metrics` solution.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nN/A\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [x] This is a documentation-only PR.",
          "timestamp": "2026-07-30T17:12:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c52449cc22f84f5c766a7032c733060d82c32d2a"
        },
        "date": 1785440213218,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6169615387916565,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2221910769507,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52460741084552,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.48046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522771.06519658054,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519545.7687507311,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005647,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15925711.769733546,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15864306.126259765,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.65314497320913,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.5884268879890442,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2002007903658,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47881408276713,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.94908854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.5703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1991019.0741471108,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2002734.7657685804,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002945,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23120093.781351488,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23009465.899245217,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.544261465136545,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "75e0ce7f9fd562eb66464293fd651e559e0d0b64",
          "message": "chore(changelog): Tighten breaking change 'Migration' enforcement (#3612)\n\n# Change Summary\n\nReceived some offline feedback that breaking change line items need to\nbe more specific about what action is required from a consumer.\n\nI had separately noticed that the Changelog can get very verbose:\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/CHANGELOG.md\n\nThis PR proposes:\n- Every `breaking` changelog entry MUST have a `subtext` starting with\n`Migration:`\n- Limit the `note` to 200 characters and `subtext` to 300 characters\n\nI am hoping the limits will help force authors to carefully choose\nwordings in changelog entries. I did have to touch basically all\nexisting changelog entries to make this new validation pass - please\nfeel free to point out places where my summarization lost accuracy.\n\n## What issue does this PR close?\n\nSemi-related to #3286\n\n## How are these changes tested?\n\nCI runs\n\n## Are there any user-facing changes?\n\nAffects repo-wide PR changelog enforcement\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-30T19:30:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/75e0ce7f9fd562eb66464293fd651e559e0d0b64"
        },
        "date": 1785464510355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.04083597660064697,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.24626375938281,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.6133519163763,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.20260416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.22265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1985051.2845699252,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1984240.6694853983,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003818,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22958856.045031913,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22862580.26943072,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.57060048113325,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.17738650739192963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20884043361711,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50685003476782,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.941536458333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.4375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519497.54308385804,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520419.0616216193,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005304,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15945492.816248894,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15884435.561897842,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.639717089844744,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "James Thompson",
            "username": "thompson-tomo",
            "email": "thompson.tomo@outlook.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ba5a5186ec3b0df993b94b68ed746dc6cdbd9f8a",
          "message": "chore: Update Renovate configuration to best practices (#3621)\n\n# Change Summary\n\nChange Renovate config to extend best practices like most other repos\nand recommendation from security SIG.\n\nsee docs at https://docs.renovatebot.com/presets-config/\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-31T15:38:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ba5a5186ec3b0df993b94b68ed746dc6cdbd9f8a"
        },
        "date": 1785525871037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7320267558097839,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.5959517343872,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.95093895258753,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.270833333333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.96484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 364850.8511193732,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 362180.04527250567,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002864,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11125302.486473775,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11047926.901645387,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.71760200952831,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.24526198208332062,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2210692335673,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58165171667181,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.057421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.08984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1989947.5921726527,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1985067.0072197209,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005922,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22871290.48590245,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22768575.59690878,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.521671763582386,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "369d5de7684e87ce3fbee4b0a3ee5068eac73aff",
          "message": "feat(metrics): refactor retry processor telemetry (#3544)\n\n# Change Summary\n\nRefactor the retry processor's internal telemetry to use bounded\nenum-based\nattributes and align its metrics with the engine-owned node telemetry\nmodel.\n\nThis PR:\n\n- Replaces `retry_attempts_{signal}` with operationally precise metrics:\n  - `processor.retry.retries.scheduled{signal}`\n  - `processor.retry.requests.recovered{signal}`\n- Adds `processor.retry.requests.terminated{signal, reason}` with\nbounded termination reasons\n- Keeps termination metrics in a separate metric set so `reason` is not\nattached to unrelated operational metrics.\n- Removes duplicated consumed/produced item counters and the\ncorresponding\n`num_items` retry state. Item and message outcomes are already recorded\nby\n  the engine-owned node consumer and producer metrics.\n- Records a retry as scheduled only after the local scheduler accepts\nit.\n- Records a request as recovered when it is acknowledged after one or\nmore\n  retries.\n- Reports the deadline reason when both the deadline and retry-count\nguards\napply, while retaining the retry limit as protection against a stalled\nclock.\n- Forwards local scheduling failures upstream after removing the retry\nframe,\n  preventing them from being routed back into the retry processor.\n- Converts channel send failures that retain their payload into\nretryable\n  NACKs.\n- Updates the retry processor documentation and expands test coverage\nfor the\n  new metric and control-flow semantics.\n\n## What issue does this PR close?\n\n* Related to #3530\n\n## How are these changes tested?\n\nAdded additional tests\n\n## Are there any user-facing changes?\n\nYes. This is a breaking change to the retry processor's internal\ntelemetry\nschema. Existing per-signal metric names are replaced with bounded\nattributes,\nand retry-specific operational metrics are added. Consumers of the\nprevious\nretry metric names will need to update their queries and dashboards.\n\nLocal retry scheduling failures are also now forwarded upstream instead\nof\nbeing routed back to the retry processor.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-07-31T22:39:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/369d5de7684e87ce3fbee4b0a3ee5068eac73aff"
        },
        "date": 1785550620534,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.379097580909729,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21403080932465,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48926530928235,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.626822916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.2109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525301.5933933322,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518057.1722765566,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003138,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15870247.174846135,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15810739.44919995,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.634161679695953,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1631042957305908,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21960005209138,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.63061720794212,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.07174479166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1961567.8114556398,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1984382.890610377,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008032,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22977792.20989604,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22874385.77561009,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.579313810163063,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a956ea9c21e62d7e83b77ccf3604394ac440ca5c",
          "message": "Refactor transform processor internal telemetry (#3620)\n\n# Change Summary\n\n- Replace transform processor message counters with operation outcome\nand failure metric sets.\n- Reuse bounded shared `signal` and `outcome` attributes and add bounded\n`language` and `error.type` attributes.\n- Classify conversion, query, routing, capacity, send, and internal\nfailures.\n- Document the metric contract and migration from the removed counters.\n\n## What issue does this PR close?\n\nContributes to #3530.\n\n## How are these changes tested?\n\n- Focused transform processor tests\n- `cargo xtask check`\n- Changelog note and subtext length controls\n\n## Are there any user-facing changes?\n\nYes. This is a breaking internal-metric change. Consumers must replace\n`msgs_transformed` and `msgs_transform_failed` queries with\n`processor.transform.operations` and `processor.transform.failures`,\nusing their bounded attributes.\n\n### Changelog\n\n- [x] Added a `.chloggen/*.yaml` entry\n- [ ] This PR is a `chore` (indicated in title)\n- [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-01T07:53:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a956ea9c21e62d7e83b77ccf3604394ac440ca5c"
        },
        "date": 1785607253108,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.05922932177782059,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.93994287125724,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51803643099721,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.948177083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 489826.09392143524,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 489535.973247034,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002618,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14989451.21092431,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14924972.755878968,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.61971342269509,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.8448700904846191,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1810685755845,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60019704433498,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.57630208333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.73828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1996264.7060553855,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1959436.2137052917,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002239,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23050144.77906022,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22937030.22322441,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.763661719547594,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a956ea9c21e62d7e83b77ccf3604394ac440ca5c",
          "message": "Refactor transform processor internal telemetry (#3620)\n\n# Change Summary\n\n- Replace transform processor message counters with operation outcome\nand failure metric sets.\n- Reuse bounded shared `signal` and `outcome` attributes and add bounded\n`language` and `error.type` attributes.\n- Classify conversion, query, routing, capacity, send, and internal\nfailures.\n- Document the metric contract and migration from the removed counters.\n\n## What issue does this PR close?\n\nContributes to #3530.\n\n## How are these changes tested?\n\n- Focused transform processor tests\n- `cargo xtask check`\n- Changelog note and subtext length controls\n\n## Are there any user-facing changes?\n\nYes. This is a breaking internal-metric change. Consumers must replace\n`msgs_transformed` and `msgs_transform_failed` queries with\n`processor.transform.operations` and `processor.transform.failures`,\nusing their bounded attributes.\n\n### Changelog\n\n- [x] Added a `.chloggen/*.yaml` entry\n- [ ] This PR is a `chore` (indicated in title)\n- [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-01T07:53:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a956ea9c21e62d7e83b77ccf3604394ac440ca5c"
        },
        "date": 1785637123434,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.4313148260116577,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21040955885765,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56822129500851,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.14049479166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.46484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1999422.6558291812,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1970804.623192937,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005802,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23135264.769999277,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23039921.270975992,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.738994569901813,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.17716243863105774,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18041791412142,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46720991479474,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.581380208333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 520175.8509892039,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519254.294717049,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002847,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15901506.298960844,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15833468.375382198,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.623735731691657,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a956ea9c21e62d7e83b77ccf3604394ac440ca5c",
          "message": "Refactor transform processor internal telemetry (#3620)\n\n# Change Summary\n\n- Replace transform processor message counters with operation outcome\nand failure metric sets.\n- Reuse bounded shared `signal` and `outcome` attributes and add bounded\n`language` and `error.type` attributes.\n- Classify conversion, query, routing, capacity, send, and internal\nfailures.\n- Document the metric contract and migration from the removed counters.\n\n## What issue does this PR close?\n\nContributes to #3530.\n\n## How are these changes tested?\n\n- Focused transform processor tests\n- `cargo xtask check`\n- Changelog note and subtext length controls\n\n## Are there any user-facing changes?\n\nYes. This is a breaking internal-metric change. Consumers must replace\n`msgs_transformed` and `msgs_transform_failed` queries with\n`processor.transform.operations` and `processor.transform.failures`,\nusing their bounded attributes.\n\n### Changelog\n\n- [x] Added a `.chloggen/*.yaml` entry\n- [ ] This PR is a `chore` (indicated in title)\n- [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-01T07:53:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a956ea9c21e62d7e83b77ccf3604394ac440ca5c"
        },
        "date": 1785693624415,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.14894673228263855,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.53871707055215,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 79.86341312262762,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.685807291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.22265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 360917.0098154104,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 361454.58391329844,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002891,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11080498.858448042,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11003327.501754113,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.655300421106027,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.5156542658805847,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17959168486581,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.62600478653594,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.708203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.15625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1987289.031692829,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1997536.5720395027,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005814,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23068872.420504805,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22976199.22222382,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.548660857283469,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a956ea9c21e62d7e83b77ccf3604394ac440ca5c",
          "message": "Refactor transform processor internal telemetry (#3620)\n\n# Change Summary\n\n- Replace transform processor message counters with operation outcome\nand failure metric sets.\n- Reuse bounded shared `signal` and `outcome` attributes and add bounded\n`language` and `error.type` attributes.\n- Classify conversion, query, routing, capacity, send, and internal\nfailures.\n- Document the metric contract and migration from the removed counters.\n\n## What issue does this PR close?\n\nContributes to #3530.\n\n## How are these changes tested?\n\n- Focused transform processor tests\n- `cargo xtask check`\n- Changelog note and subtext length controls\n\n## Are there any user-facing changes?\n\nYes. This is a breaking internal-metric change. Consumers must replace\n`msgs_transformed` and `msgs_transform_failed` queries with\n`processor.transform.operations` and `processor.transform.failures`,\nusing their bounded attributes.\n\n### Changelog\n\n- [x] Added a `.chloggen/*.yaml` entry\n- [ ] This PR is a `chore` (indicated in title)\n- [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-01T07:53:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a956ea9c21e62d7e83b77ccf3604394ac440ca5c"
        },
        "date": 1785723486128,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.22834017872810364,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21387134221264,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74087918844144,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.54674479166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1969191.422786358,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1964694.967562661,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008159,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22675662.031782463,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22572222.663985524,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.541568745357544,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.16308914124965668,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20397186216519,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53188385598142,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.524739583333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.22265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 517973.4576721068,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518818.2161240703,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002951,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15902902.37301601,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15836400.539164722,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.65216655618157,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Matthew Wear",
            "username": "mwear",
            "email": "matthew.wear@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9708f63b7e682499e629b048128d218a4f3f58e9",
          "message": "feat(engine)!: adopt ResourceDetectors for self-telemetry (#3555)\n\n# Change Summary\n\n- Replaces bespoke resource detection by adopting resource detectors\nfrom upstream Rust SDK and contrib projects.\n- Default detectors were changed to `env`, `service_instance` which\nmirrors the Go collector's defaults for self-telemetry. `host`, `os`,\n`process`, `container`, and `k8s` are opt-in.\n- Supporting work was done upstream in both opentelemetry-rust and\nopentelemetry-rust-contrib, but is not yet released. We'll update the\ndeps in this PR to use the pending releases before merging.\n- https://github.com/open-telemetry/opentelemetry-rust/pull/3593 enables\na slim build (the only dep is the API). The dep added in this PR is the\ncurrent release which results in a heavier build.\n- `opentelemetry-resource-detectors` is pinned to\nhttps://github.com/open-telemetry/opentelemetry-rust-contrib/commit/4f5296b92b2a8e50d2078e011fa04371b69a5cf6\nas it contains new detectors and updates to existing ones.\n\n## What issue does this PR close?\n\n* Closes #3177\n\n## How are these changes tested?\n- New unit tests\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nYes, breaking. Default self-telemetry no longer emits `host.id` and\n`container.id` (now opt-in). When opted in, `host.id` derives from\nmachine-id rather than hostname, and `service.instance.id` is a plain\nUUIDv7 instead of a base32 string.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2026-08-03T17:35:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9708f63b7e682499e629b048128d218a4f3f58e9"
        },
        "date": 1785786117221,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9680731296539307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19007338213942,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50561579393846,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.74375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.37890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523574.2077729346,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518505.6264822173,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002589,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15880747.224511858,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15820661.307920398,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.627916869975383,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8064515590667725,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17421616196911,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52802101846842,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.916927083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1983802.0826788847,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1999800.4865714563,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005986,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23212915.843081154,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23101214.19419957,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.607615859159216,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4c05b596db372a0ca5249a4698f9a65f1a0d6840",
          "message": "chore(deps): update all patch versions plus test fix (#3654)\n\n# Change Summary\n\nManual fix needed to unblock #3601 \n\n> TRY 3 FAIL [ 0.010s] (1469/1718) otap-df-query-engine-languages\nopl::parser::temporal::test::test_parse_from_invalid_time_literal\n\nTest failure caused by `iso8601` update\n\n## What issue does this PR close?\n\nNA\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nN/A\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-04T01:12:52Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4c05b596db372a0ca5249a4698f9a65f1a0d6840"
        },
        "date": 1785810874511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.060528725385665894,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.24273862445932,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51033949423865,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.815755208333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.33984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521601.9888166396,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 521917.7078584243,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002716,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15934692.812862268,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15869716.99198563,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.53104459369047,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1014989614486694,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21268810071051,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58184753920199,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.7234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.75,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1970671.3011550587,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1992378.225315029,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005185,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23004314.346993543,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22898160.405370586,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.54615828194778,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "albertlockett",
            "username": "albertlockett",
            "email": "a.lockett@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4997cf418cc578d77a5702941952399b1a0c6d0f",
          "message": "fix: two OPL parser panics (#3659)\n\n# Change Summary\n\n<!--Replace with a brief summary of the change in this PR-->\n\nFixes a few panics that occured when parsing OPL.\n\n**Parsing empty string `\"\"`**\n\n```rs\nOplParser::parse(\"\"); // This panics with \"Query line did not exist\"\n```\n\nRoot cause: Pest would identify the position of the error as line `1`.\nWhen we take this line to produce the error content, we take it from an\nempyt iterator and panic when we don't fine the line:\n\nhttps://github.com/open-telemetry/otel-arrow/blob/4c05b596db372a0ca5249a4698f9a65f1a0d6840/rust/experimental/query_engine/parser-abstractions/src/parser_error.rs#L46-L58\n\n> An empty string returns an empty iterator.\n - https://doc.rust-lang.org/std/primitive.str.html#method.lines\n \nSolution: make `ParserError::from_pest_error` robust over invalid error\npositions.\n \n **Parsing escape double backslash escape sequences (in regexps)**:\n\n```rs\nOplParser::parse(r#\"logs | where (matches(attributes[\"code\"], \"\\\\d+\"))\"#); // // This panics with \"Unexpected escape character\"\n```\n\nRoot case: when parsing the string literal, we look back a single\ncharacter. and if the previous character is `\\`, we assume we're parsing\nan escape sequence:\n\nhttps://github.com/open-telemetry/otel-arrow/blob/4c05b596db372a0ca5249a4698f9a65f1a0d6840/rust/experimental/query_engine/parser-abstractions/src/parser_abstractions.rs#L117-L150\n\nSolution: if it the previous character was `\\` but was already escaped\nby a previous `\\`, we are not parsing an escape sequence and should take\nthe current character.\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/3658\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n No\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-04T17:22:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4997cf418cc578d77a5702941952399b1a0c6d0f"
        },
        "date": 1785872260920,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.712634801864624,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20042301469283,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58506427133345,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.34192708333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.76953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1993595.6916992608,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2007802.7488242364,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003982,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23224985.2260799,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 23105704.904953763,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.567363995133677,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.27151867747306824,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18705220933994,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51735562780617,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.089322916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.62890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 515375.38839622313,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 516774.72886570176,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005411,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15845769.57247908,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15785369.912029842,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.66281822112289,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Gokhan Uslu",
            "username": "gouslu",
            "email": "geukhanuslu@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1ba295db09906ce56c3b0c39c846b432a3d70999",
          "message": "feat(engine): generalize AuthorizedIdentity into a scheme-agnostic claims model (#3648)\n\n## Description\n\nGeneralizes `capability::auth::AuthorizedIdentity` into a\n**scheme-agnostic claims model** so a single identity type serves every\nauthentication method (Kubernetes SAT, OIDC/JWT, mutual TLS) rather than\nbeing limited to a subject and audience.\n\n`AuthorizedIdentity` now carries:\n- an optional **`principal`** (the primary human/service identity),\n- an optional **`scheme`** tag (which auth method produced it),\n- a **`claims`** map of **`ClaimValue`** (single- or multi-valued) keyed\nby standard or namespaced claim names.\n\nClaim names follow JWT registered names where they exist (`sub`, `aud`,\n`groups`) and are otherwise namespaced by scheme (`k8s.*`, `x509.*`).\nThis lets an authorizer emit the full set of verified claims for a\ndownstream tenant / per-route authorization resolver to match on.\n\n`subject()` / `audience()` and their `with_*` builders are kept as thin,\ntyped accessors over the near-universal `sub` / `aud` claims (returning\n`Option<&str>` directly instead of going through the generic claims\nmap). `AuthorizedIdentity` stays `#[non_exhaustive]`.\n\nEngine-only; no new dependencies.\n\n### Testing\n- `cargo test -p otap-df-engine --lib capability::auth`\n- `cargo fmt --all --check`, `cargo clippy -p otap-df-engine\n--all-targets -- -D warnings`\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: 10d746ae-67a2-4f55-be24-202717eaadd9",
          "timestamp": "2026-08-05T01:22:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1ba295db09906ce56c3b0c39c846b432a3d70999"
        },
        "date": 1785896105149,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3640782833099365,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22950144962581,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.64359188728906,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.152734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.01953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1965934.856036044,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1992751.746009315,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007555,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23005492.55323194,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22898933.134060998,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.544585319923941,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.14942283928394318,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17052119590464,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51852909231962,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.619270833333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.84375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519652.58203553286,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520429.0616494182,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004151,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15942236.076842407,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15880709.353152271,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.632870551686704,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tom Tan",
            "username": "ThomsonTan",
            "email": "totan@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "60cb3546684054e31004c6cc84315480fd4e2fdd",
          "message": "chore(ci): ask contributors to claim help wanted issues before starting (#3663)\n\n# Change Summary\n\nThe auto comment posted when an issue is labeled `help wanted` invited\nanyone to work on it and only suggested commenting to be assigned, so\ntwo\ncontributors could independently start the same issue and duplicate\nwork.\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-05T15:08:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/60cb3546684054e31004c6cc84315480fd4e2fdd"
        },
        "date": 1785954981195,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.0359293222427368,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.15902842130771,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.13813791502206,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.348697916666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 364066.2790326576,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 360294.8093879103,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004195,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11086660.700626373,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11008641.466020813,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.771080825341432,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.07610152661800385,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22587838718326,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.62574257425743,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.04661458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.15234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1973381.5845623708,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1974883.358069021,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003722,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22795339.596747696,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22691250.8966554,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.542625797928777,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Manish Goel",
            "username": "manishgoel3",
            "email": "manish_vit@hotmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "538053869042e3e6a0566d6bd300010accca6872",
          "message": "feat(geneva-exporter): support agent-fed credentials (#3579)\n\n# Change Summary\n\nAdds agent-fed authentication to the Geneva exporter. An embedding host\nsupplies one atomic token-and-routing snapshot through the\n`agent_fed_credential_provider` capability, bypassing the Geneva Config\nService handshake.\n\nEach upload reads one immutable snapshot, preventing token, endpoint,\nand moniker generations from being mixed during rotation. Invalid,\nunavailable, or near-expiry credentials fail closed.\n\nThe exporter validates that:\n\n- The endpoint is an absolute HTTPS URL without credentials, query, or\nfragment.\n- The moniker matches the configured account or an explicit `default`.\n- Monikers contain only URL-unreserved ASCII characters.\n- Tokens with known expiry remain valid for more than 30 seconds.\n\nToken zeroization is preserved through the merged\n[geneva-uploader\nhardening](https://github.com/open-telemetry/opentelemetry-rust-contrib/pull/716).\n\n## What issue does this PR close?\n\n* Closes #3275 \n\n## How are these changes tested?\n\n- All 77 Geneva exporter unit and regression tests pass.\n- All 23 engine authentication capability tests pass.\n- Engine and contrib-nodes all-target clippy pass with warnings denied.\n- Formatting, Markdown lint, sanity, and focused dependency checks pass.\n- Coverage includes capability binding, atomic rotation, routing\nprecedence, endpoint validation, expiry handling, recovery, existing\nauthentication modes, logs/spans routing and ACK/NACK behavior.\n\n## Are there any user-facing changes?\n\nYes. Geneva exporters can select `auth.type: agentfed` and bind one\ncapability:\n\n```yaml\ncapabilities: agent_fed_credential_provider: agent-auth\nconfig: account: my-account auth:\n    type: agentfed\n```\n\nThe host extension must provide this capability using the shared\nexecution model.\nendpoint region are  are optional in agent-fed mode because the endpoint\ncomes from the credential snapshot. For other authentication modes they\nremain required, and blank values now fail during configuration\nvalidation.\n\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Manish Goel <292890742+manishgoel3@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-06T00:21:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/538053869042e3e6a0566d6bd300010accca6872"
        },
        "date": 1785983346032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.278795063495636,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.13824862245504,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55859823228408,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.680208333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.53125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 517262.87541638303,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518704.9788501052,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001244,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15892816.657488707,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15826425.105937857,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.639414128472048,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.4283459186553955,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23982576195294,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61896236642403,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.37330729166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.4296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2011215.1617209136,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1962375.8992792869,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006803,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23043970.30946437,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22931280.66019217,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.742893050168231,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1a7bd6347bf423e6694c33490fab6f7930f90498",
          "message": "chore(opamp): improve reconciliation diagnostics and example (#3676)\n\n## Summary\n\nAdd local OpAMP lifecycle logs for WebSocket connection and remote\nconfiguration reconciliation, including complete failure reasons.\n\nMake the OpAMP controller example self-contained by generating\nlightweight log traffic and forwarding it through a local OTLP loopback\npipeline. This makes it relatively easy to test.",
          "timestamp": "2026-08-07T01:02:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1a7bd6347bf423e6694c33490fab6f7930f90498"
        },
        "date": 1786071539784,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.1171019077301025,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22507286137336,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50447437683852,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.13203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.5390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526307.3503394977,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509901.8137240289,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.04607,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15916005.08887092,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15856361.751086006,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.21386247409005,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.16432107985019684,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.13087961537852,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.41748097530672,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.22252604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.28125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1968082.731144795,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1964848.7563301271,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002941,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22690702.222140636,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22595310.137332566,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.548320016509313,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Dipanshu singh",
            "username": "Dipanshusinghh",
            "email": "161134993+Dipanshusinghh@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8d38d41cd89a7c3db8c271f1a4ec392eb171fe8e",
          "message": "fix(metrics): include custom identity attributes in channel metrics (#3647)\n\nFixes #3645\n\nThis PR addresses the issue where custom telemetry identity attributes\nconfigured via `entity.extend.identity_attributes` were dropped from\nchannel-backed metrics (e.g., `produced.items`).\n\n**Changes**\n* Introduced `NodeWithCustomChannelAttributeSet` to carry custom\nattributes alongside the base channel attributes.\n* Updated `PipelineContext::register_node_channel_entity` to construct\n`NodeWithCustomChannelAttributeSet` when custom attributes are present\non the node.\n\nThis ensures that producers and consumers correctly see the configured\ncomponent identity on channel metrics, bringing them into parity with\nstandard node-local metrics and logs.",
          "timestamp": "2026-08-07T16:26:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8d38d41cd89a7c3db8c271f1a4ec392eb171fe8e"
        },
        "date": 1786125483979,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.03204101324081421,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 80.41125766296634,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4047816661505,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.619010416666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.4453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 372836.36501689896,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 372716.9044705943,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003074,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11667557.32610036,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11592769.109664818,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.30407337620738,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5264838933944702,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.14774012223336,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56807248509254,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.141276041666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.1015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1977087.2686579064,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1966678.2232000697,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009345,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22708182.999926753,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22619260.413468186,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.546465879393965,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Pritish Nahar",
            "username": "pritishnahar95",
            "email": "pritishnahar@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "30bb3bcf917814b46140bb48a008acb0953cdee9",
          "message": "doc: make auth extension READMEs the full config reference (#3696)\n\n# Change Summary\n\nThe oauth2_client_auth and azure_identity_auth READMEs only summarized\nthe extensions and deferred to design.md, so operators had to read\ndesign docs or consumer exporter READMEs to find configuration options.\nConsumer READMEs in turn duplicated provider details that drift out of\nsync.\n\nRewrite both extension READMEs as standalone usage guides: metadata,\ngetting started, build/feature gates, complete config field tables\n(including grant-specific and method-specific fields), validation rules,\ntelemetry, and troubleshooting. Design rationale and lifecycle detail\nstay in design.md.\n\nRemove the duplicated provider configuration from the Azure Monitor and\nOTLP HTTP exporter READMEs, which now document only the capability\nbinding and link to the extension. Update the contrib-extensions catalog\nto link usage and design docs and state the ownership rule.\n\n## What issue does this PR close?\n\n* Related to #3479 \n* Related to #3356\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nn/a\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [x] This is a documentation-only PR.",
          "timestamp": "2026-08-07T22:28:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30bb3bcf917814b46140bb48a008acb0953cdee9"
        },
        "date": 1786157763426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4979534447193146,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21798605328549,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59959620867689,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.73072916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.60546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1962793.9236439548,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1953020.123831248,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.033151,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22578241.853710584,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22476873.7157009,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.56068059832315,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.04975242167711258,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.78808396918036,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.07233566433565,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.941276041666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.59375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 360165.4283814619,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 360344.619406978,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003005,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11025851.797309188,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10951870.26257022,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.598075296516207,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Pritish Nahar",
            "username": "pritishnahar95",
            "email": "pritishnahar@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "30bb3bcf917814b46140bb48a008acb0953cdee9",
          "message": "doc: make auth extension READMEs the full config reference (#3696)\n\n# Change Summary\n\nThe oauth2_client_auth and azure_identity_auth READMEs only summarized\nthe extensions and deferred to design.md, so operators had to read\ndesign docs or consumer exporter READMEs to find configuration options.\nConsumer READMEs in turn duplicated provider details that drift out of\nsync.\n\nRewrite both extension READMEs as standalone usage guides: metadata,\ngetting started, build/feature gates, complete config field tables\n(including grant-specific and method-specific fields), validation rules,\ntelemetry, and troubleshooting. Design rationale and lifecycle detail\nstay in design.md.\n\nRemove the duplicated provider configuration from the Azure Monitor and\nOTLP HTTP exporter READMEs, which now document only the capability\nbinding and link to the extension. Update the contrib-extensions catalog\nto link usage and design docs and state the ownership rule.\n\n## What issue does this PR close?\n\n* Related to #3479 \n* Related to #3356\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nn/a\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [x] This is a documentation-only PR.",
          "timestamp": "2026-08-07T22:28:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30bb3bcf917814b46140bb48a008acb0953cdee9"
        },
        "date": 1786211220278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9984925389289856,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18079726787323,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4621145784816,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.859114583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.66796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526421.6387735903,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 521165.3580045479,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002883,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15922779.68921583,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15858234.880190687,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.55226032325211,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9138608574867249,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22388172076802,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56063732693943,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.334765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.49609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1964158.601211389,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1946208.925679974,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.014901,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22490322.563278776,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22380159.60830719,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.555965172352202,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Pritish Nahar",
            "username": "pritishnahar95",
            "email": "pritishnahar@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "30bb3bcf917814b46140bb48a008acb0953cdee9",
          "message": "doc: make auth extension READMEs the full config reference (#3696)\n\n# Change Summary\n\nThe oauth2_client_auth and azure_identity_auth READMEs only summarized\nthe extensions and deferred to design.md, so operators had to read\ndesign docs or consumer exporter READMEs to find configuration options.\nConsumer READMEs in turn duplicated provider details that drift out of\nsync.\n\nRewrite both extension READMEs as standalone usage guides: metadata,\ngetting started, build/feature gates, complete config field tables\n(including grant-specific and method-specific fields), validation rules,\ntelemetry, and troubleshooting. Design rationale and lifecycle detail\nstay in design.md.\n\nRemove the duplicated provider configuration from the Azure Monitor and\nOTLP HTTP exporter READMEs, which now document only the capability\nbinding and link to the extension. Update the contrib-extensions catalog\nto link usage and design docs and state the ownership rule.\n\n## What issue does this PR close?\n\n* Related to #3479 \n* Related to #3356\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nn/a\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [x] This is a documentation-only PR.",
          "timestamp": "2026-08-07T22:28:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30bb3bcf917814b46140bb48a008acb0953cdee9"
        },
        "date": 1786240793779,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.604736864566803,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19519129379486,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46974057151708,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.265885416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.8984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523473.71548721805,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520308.07711583865,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004327,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15913642.607954195,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15854489.999364771,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.585038572082873,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.14067628979682922,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20650657948352,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5830311461473,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.68971354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.5703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1965164.4563679793,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1962399.935953144,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006068,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22715522.232419107,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22619356.41184794,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.575378604660473,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shaurya Srivastava",
            "username": "Shaurya2k06",
            "email": "104617579+Shaurya2k06@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "e19f973a1001c6d1a0f7204d83d3b087257076df",
          "message": "chore(ci): add offline markdown link checker (#3594)\n\n# Change Summary\n\nAdd a Repo Lint lychee job that fails PRs on broken relative Markdown\nlinks and anchors across the whole tree (not just changed files). Fix\nthe three existing broken relative links so the check is green.\n\n## What issue does this PR close?\n\n* Closes #3580\n\n## How are these changes tested?\n\n- Ran lychee offline locally on all `**/*.md` (0 errors after link\nfixes)\n- markdownlint + sanitycheck on touched Markdown\n\n## Are there any user-facing changes?\n\nNo. CI + doc link fixes only.\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nSigned-off-by: shaurya2k06 <shaurya2k06@gmail.com>",
          "timestamp": "2026-08-09T13:27:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e19f973a1001c6d1a0f7204d83d3b087257076df"
        },
        "date": 1786297675962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.37222161889076233,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17059766413561,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49743075379986,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.50052083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.8125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1925554.2002194296,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1918386.8709619925,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005615,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22548544.817349494,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22453255.446439218,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.753909056958006,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9596336483955383,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19280648852391,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48376179427687,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.927994791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.46484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525508.6860421271,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520465.72768388176,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002875,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15928869.166693395,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15871176.146899337,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.605029917297845,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shaurya Srivastava",
            "username": "Shaurya2k06",
            "email": "104617579+Shaurya2k06@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "e19f973a1001c6d1a0f7204d83d3b087257076df",
          "message": "chore(ci): add offline markdown link checker (#3594)\n\n# Change Summary\n\nAdd a Repo Lint lychee job that fails PRs on broken relative Markdown\nlinks and anchors across the whole tree (not just changed files). Fix\nthe three existing broken relative links so the check is green.\n\n## What issue does this PR close?\n\n* Closes #3580\n\n## How are these changes tested?\n\n- Ran lychee offline locally on all `**/*.md` (0 errors after link\nfixes)\n- markdownlint + sanitycheck on touched Markdown\n\n## Are there any user-facing changes?\n\nNo. CI + doc link fixes only.\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nSigned-off-by: shaurya2k06 <shaurya2k06@gmail.com>",
          "timestamp": "2026-08-09T13:27:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e19f973a1001c6d1a0f7204d83d3b087257076df"
        },
        "date": 1786327267724,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.2583014965057373,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20024186851005,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54936058213346,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.41223958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1940031.7751849422,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1915620.327285905,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005945,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22544016.067322448,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22449160.65296222,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.768519965155793,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.17909662425518036,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18132021300103,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46160378089408,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.079557291666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.26171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519307.1008366971,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520237.16235378024,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004633,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15914159.00377451,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15847630.872550076,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.59020030743652,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "renovate[bot]",
            "username": "renovate[bot]",
            "email": "29139614+renovate[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "cfda6b46cb336af4ea1c62c84040d793f2f46d69",
          "message": "chore(deps): update opentelemetry-resource-detectors digest to 24d1f73 (#3702)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| opentelemetry-resource-detectors | workspace.dependencies | digest |\n`4f5296b` → `24d1f73` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4xMi4wIiwidXBkYXRlZEluVmVyIjoiNDQuMTIuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-10T12:14:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cfda6b46cb336af4ea1c62c84040d793f2f46d69"
        },
        "date": 1786385552437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.1970055103302002,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20560053002876,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45675065697944,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.667838541666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.55078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519743.71374290704,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520767.6375288466,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004466,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15949227.63306189,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15884535.698550228,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.62638014286827,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.1473325788974762,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19116083111261,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54817733990147,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.541015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.08203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1934392.0743995823,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1937242.0640459377,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003025,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22359645.239496753,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22253615.240731858,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.541998624992969,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c190552e0036de9e9a4ce7b98d2dbbc9681927e3",
          "message": "feat(telemetry): reload internal log levels during reconciliation (#3613)\n\n# Change Summary\n\nChanging internal log verbosity previously required restarting the\nengine, which discards the in-process state an operator is usually\ntrying to inspect.\n\nExtends `df_engine` full-engine reconciliation to apply changes to\n`engine.telemetry.logs.level` to existing tracing subscribers. Internal\nlog level can now be changed dynamically through the admin control plane\nor OpAMP, without restarting the engine.\n\n```yaml\nengine:\n  telemetry:\n    logs:\n      level: warn\n```\n\nUpdate flow: Admin or OpAMP submits a full desired config -> the\ncontroller validates and reconciles it -> on success, every live tracing\ndispatcher atomically receives the new filter -> tracing rebuilds its\ncallsite-interest cache so the new level takes effect immediately.\nFailed reconciliation keeps the previous filter.\n\nReconciling the same configuration with a different `level` applies the\nnew value to every live tracing dispatcher.\n\nEach tracing dispatcher retains its own `EnvFilter` instance, so filter\nstate is never shared across engine threads. A shared update registry\nreplaces every live dispatcher filter only after successful\nreconciliation and rebuilds tracing's callsite-interest cache so both\nverbosity increases and decreases take effect. Failed reconciliation\npreserves the active filter, and reconciliations that do not change\n`logs.level` skip filter parsing and cache rebuilding.\n\nA valid `RUST_LOG` value supplies the startup filter. After startup, a\nsuccessful reconciliation makes `engine.telemetry.logs.level`\nauthoritative, allowing OpAMP and admin updates to replace the\nenvironment-derived filter.\n\nOnly `engine.telemetry.logs.level` is applied to running components.\nOther `engine.telemetry` fields are recorded in the controller's live\nconfiguration but are not applied at runtime; extending live reload to\nmore settings is left for follow-up work.\n\n## Performance\n\n`df_engine` does not compile out debug or trace callsites today;\ndisabled callsites are filtered at runtime and cached as uninterested.\nThis change preserves that steady-state behavior. Benchmarks comparing\nthe static `EnvFilter` with the reloadable filter (committed in\n`benches/self_tracing/main.rs`) show disabled logs unchanged at the ~1\nns cached-interest floor, and enabled logs ~1.2% slower (135.3 ns to\n136.9 ns) from the added indirection. Both figures assume level and\ntarget directives; span directives disable the cached-interest floor.\nBuilds that use tracing's compile-time max-level features cannot\ndynamically enable callsites that were removed during compilation.\n\n## What issue does this PR close?\n\nRelates to #3387\n\n## Are there any user-facing changes?\n\nYes. Internal log severity and target directives can be changed through\nsuccessful full-engine reconciliation without restarting `df_engine`.\n\n### Changelog\n\n- [x] Added a `.chloggen/*.yaml` entry\n- [ ] This PR is a `chore` (indicated in title)\n- [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-11T00:01:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c190552e0036de9e9a4ce7b98d2dbbc9681927e3"
        },
        "date": 1786413892316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.09157264977693558,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15767847747254,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51791422400494,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.488932291666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.93359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1919518.0470217068,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1921275.8005476382,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003862,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22201515.693533186,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22097065.724892724,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.55561095767973,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.09030012041330338,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21301643049871,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53881558760355,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.497265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.3046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519699.3802782952,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520168.6694671588,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005644,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15953197.824225245,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15892480.705749597,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.669278564137862,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tim R",
            "username": "timr-dev",
            "email": "68666585+timr-dev@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "7d6ec63f00f178b8754a32b1b2cec2ab2c2e0874",
          "message": "fix(journald): bound filtered follow waits (#3674)\n\n## Summary\n\n- bound each journald follow call to one blocking wait so non-matching\njournal activity cannot indefinitely delay batch flushes, checkpoint\ncommands, drain, or shutdown\n- exercise the production `next()` / `wait()` control flow with\nbehavior-level tests for `start_at: end`, idle timeouts, non-matching\nwakes, invalidation, errors, immediate reads, raw-tail stalls, cursor\nadvancement, and current-entry failures\n- emit and document `journald_receiver.start_at_end_head_recovery` from\nthe pipeline thread when fresh `start_at: end` cannot establish a\nmatching tail anchor\n- avoid the libsystemd infinite-wait sentinel and remove one per-record\ncursor clone\n\nRefs #3399.\n\n## Scope\n\nThis PR addresses the behavior-level follow-test work in #3399 and the\nclosely coupled wait-budget defect found by the mandatory review panel.\n\nIt deliberately does **not** close #3399. We are knowingly retaining the\ncurrent best-effort head recovery for fresh `start_at: end`: buggy\nlibsystemd behavior or a later `SD_JOURNAL_INVALIDATE` can expose\npre-startup matching history. The warning and operator documentation\nmake that risk explicit, while #3399 remains open for the dedicated\nmonotonic/boot-aware durable boundary guard. This PR does not claim that\n`start_at: end` is fully replay-safe.\n\nThe diff is larger than a typical test-only change because the\nproduction follow seam, faithful fake state machine, branch/error\ncoverage, operator telemetry, documentation, and changelog form one\nreviewable behavior contract.\n\n## Validation\n\n- `cargo test -p otap-df-core-nodes --lib` (850 tests: 846 passed, 4\nignored)\n- `cargo clippy -p otap-df-core-nodes --lib --tests -- -D warnings`\n- Linux target `cargo-zigbuild check`\n- Linux target test compilation with `cargo-zigbuild test --no-run`\n- `cargo xtask check` with constrained build parallelism\n- `markdownlint-cli2` on both changed Markdown files\n- pinned `chloggen v0.30.0` validation for Go and Rust entries\n- ASCII/LF checks for changed Rust, YAML, and Markdown files\n\n## Review panel\n\nTwenty independent reviews ran across SRE, SDET, Security, Performance\nArchitect, and Adversarial personas using Claude Opus 5, GPT-5.6 Sol,\nGemini 3.1 Pro, and MAI Code Flash. Material findings were deduplicated\nand resolved; MAI findings were advisory and independently verified by\nfrontier models. Resolution re-reviews found no remaining defect in this\nPR's stated scope.",
          "timestamp": "2026-08-11T07:08:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7d6ec63f00f178b8754a32b1b2cec2ab2c2e0874"
        },
        "date": 1786479149472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6237518787384033,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22764377278118,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52462121798345,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.600520833333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.38671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1939819.2270928773,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1951918.8851467439,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003018,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22562429.10890994,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22463609.23947764,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.559101805203197,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.05607243627309799,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.16792221157266,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38390214067279,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.699348958333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.27734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 410879.7138467519,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 410649.32357690093,
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
            "value": 12606545.766190631,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12536914.72748035,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.699054016169214,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f2b0348342bd6f0aae619c950dfc93b1f1bb1052",
          "message": "feat(metrics): Add durable_buffer.loss bytes metric (#3715)\n\n# Change Summary\n\nAdd `processor.durable_buffer.loss.bytes{reason}` for persisted bytes\nremoved by durable-buffer runtime retention.\n\nThe durable buffer already reports lost segments and bundles by\nretention reason and lost items by reason and signal. Those counts do\nnot show the persisted volume discarded when the buffer applies\nDropOldest or max-age expiry.\n\nBytes are aggregate rather than signal-specific because a segment may\ncontain bundles from multiple signals. The value includes the complete\npersisted file representation, including metadata and encoding overhead.\n\n### Validation\n\nRunning the sample config:\n\n```text\nprocessor.durable_buffer.loss.segments{reason}\nprocessor.durable_buffer.loss.bundles{reason}\nprocessor.durable_buffer.loss.bytes{reason}\nprocessor.durable_buffer.loss.items{reason,signal}\n```\n\nAn unreachable-exporter run exercised both runtime retention paths:\n\n| Pipeline | Traffic | Retention |\n| --- | --- | --- |\n| DropOldest | 50,000 logs/s, batches up to 1,000 | 192 MiB cap with\n`drop_oldest` |\n| Max age | 1,000 logs/s, batches up to 100 | 5-second max age with a\n192 MiB cap |\n\nThe admin accumulator was reset, then sampled and reset again five\nseconds later. Loss values are deltas for that window; storage used and\nutilization are point-in-time gauges at the end of the window.\n\n| Pipeline | Reason | Utilization | Storage used | Segments lost |\nBundles lost | Items lost | Storage lost |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n| DropOldest | `drop_oldest` | 82.66% | 158.70 MiB | 30 | 175 | 175,000\n| 62.73 MiB |\n| Max age | `expired` | 14.54% | 27.92 MiB | 48 | 50 | 5,000 | 2.04 MiB\n|\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Follow-up to:\n  * #3516\n  * #3705\n\n## How are these changes tested?\n\nUnit tests and local engine runs\n\n## Are there any user-facing changes?\n\nYes, added a new `loss.bytes` metric.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-11T23:33:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f2b0348342bd6f0aae619c950dfc93b1f1bb1052"
        },
        "date": 1786501372553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.25430166721343994,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1899448491713,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47683511049297,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.876822916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.79296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516739.0874004256,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518053.1634838442,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002614,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15869527.63649103,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15805762.054021116,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.63300980495978,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.35903000831604004,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23210156943381,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60474870285758,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.86888020833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.41796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1934599.5078941376,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1941545.300536342,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002943,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22470163.241081018,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22358796.075881492,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.573339666539715,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "48b3bbc76d8522f8891f5ebb53c4a0602a368efc",
          "message": "Introduce `max_in_flight`  in the ClickHouse exporter (#3709)\n\n# Change Summary\n\nAdds configurable, bounded concurrency to the df_engine ClickHouse\nexporter.\n\nThe new `max_in_flight` setting controls how many ClickHouse HTTP insert\nrequests may execute concurrently. It defaults to 10, enabling\nconcurrent inserts without requiring additional configuration.\n\nThe exporter applies backpressure when the configured limit is reached\nand drains accepted requests during shutdown. Concurrent requests may\ncomplete out of order.\n\nUsers who require serialized insertion behavior can set:\n\n```yaml\nexporter:\n  type: urn:otel:exporter:clickhouse\n  config:\n    endpoint: http://clickhouse:8123\n    max_in_flight: 1\n```\n\n## Performance impact\n\nThe change was benchmarked against the serialized implementation using:\n\n- 8,192-log input batches\n- synchronous ClickHouse inserts\n- `max_in_flight: 1` for the baseline\n- `max_in_flight: 10` for this change and the new default\n- one df_engine core\n- six ClickHouse cores\n- twelve traffic-generator cores\n- ClickHouse 25.6\n- three 60-second repetitions per scenario\n\nMedian ClickHouse written throughput under maximum offered load:\n\n| Input path | Serialized baseline | New default | Gain |\n| --- | --- | --- | --- |\n| DFE OTAP | 180,271 logs/s | 682,597 logs/s | +278.6% / 3.79x |\n| DFE OTLP | 179,810 logs/s | 437,543 logs/s | +143.3% / 2.43x |\n\nAt a fixed offered load of approximately 100,000 logs/s, written\nthroughput remained unchanged, as expected. DFE CPU usage decreased by\napproximately 1.0% for OTAP and 9.8% for OTLP.\n\nThese results indicate that the new default can provide up to\napproximately 3.8x OTAP throughput and 2.4x OTLP throughput when\nserialized synchronous inserts are the bottleneck. The actual gain\ndepends on the workload, ClickHouse capacity, and insertion latency.\n\nConcurrent inserts drive ClickHouse harder and increased memory\nconsumption during the saturation runs. Operators can reduce\n`max_in_flight` when ClickHouse resource consumption or insertion\nordering is more important than maximum throughput.\n\n## What issue does this PR close?\n\n- Related to #3512\n- Implements the bounded-concurrency follow-up identified by the\nClickHouse exporter benchmarks in #3512\n\n## How are these changes tested?\n\nAutomated tests cover:\n\n- defaulting `max_in_flight` to 10\n- accepting an explicitly configured concurrency limit\n- rejecting a zero concurrency limit\n- enforcing the configured bound\n- applying backpressure before admitting another request\n- draining all accepted writes during shutdown\n- preserving completed row counts\n\nThe full workspace check passed.\n\n## Are there any user-facing changes?\n\nYes. The ClickHouse exporter now allows up to ten concurrent inserts by\ndefault.\n\nThe new `max_in_flight` positive integer setting can be used to tune\nthis limit. Values greater than one improve throughput by overlapping\nsynchronous HTTP inserts, but inserts may complete out of order and can\nplace more load on ClickHouse.\n\nSet `max_in_flight: 1` to retain serialized insertion behavior.\n\n### Changelog\n\n- [x] Added a `.chloggen/*.yaml` entry\n- [ ] This PR is a chore (indicated in title)\n- [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-12T06:43:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/48b3bbc76d8522f8891f5ebb53c4a0602a368efc"
        },
        "date": 1786557538832,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.12769366800785065,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17901558116569,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52938317467688,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.903385416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1937877.6005118953,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1940352.1476579348,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002898,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22401991.93601241,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22296033.101438284,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.545322823515468,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.4125920236110687,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 88.19761929828654,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53397709687404,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.480598958333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.8828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 430169.7976572751,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 428394.95152804826,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002948,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13295648.359998997,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13227529.44216421,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.035959486857987,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fcd8b2ae31030836252a389c3c14f71e20185e38",
          "message": "feat(console): add actionable exporter metrics (#3741)\n\n# Change Summary\n\nRefactors the Console Exporter's internal instrumentation to use bounded\nenum-based attributes and measurement metric sets.\n\nThe exporter now:\n- Reports terminal export results through\n`exporter.pdata.exports.messages`, partitioned by `signal` and\n`outcome`.\n- Reports actionable failures through\n`exporter.console.failures.messages`, partitioned by the fixed output\n`format`, `signal`, and bounded `error.type`.\n- Classifies failures as OTLP view creation, OTAP view creation,\nunsupported signal, formatting, or stdout write failures.\n- Reports metrics during periodic collection and includes touched\nbuckets in terminal shutdown snapshots.\n- Preserves the existing best-effort ACK behavior, including when\nconsole formatting or output fails.\n- Avoids duplicating input-volume metrics already provided by\nengine-owned channel instrumentation.\n\n## What issue does this PR close?\n\n* Related to #3300\n\n## How are these changes tested?\n\n- Added unit tests verifying that successful and failed exports are\nisolated by signal and outcome.\n- Added unit tests verifying actionable failure classification and\ndeterministic attribute ordering.\n- Added terminal snapshot tests verifying that only touched buckets are\nemitted and that they are drained exactly once.\n- `cargo xtask check` \n\n## Are there any user-facing changes?\n\nYes. The Console Exporter now exposes bounded internal telemetry for\nterminal export outcomes and actionable failure reasons:\n\n- `exporter.pdata.exports.messages{signal,outcome}`\n- `exporter.console.failures.messages{format,signal,error.type}`\n\nConsole output and ACK behavior are unchanged.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-13T01:26:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fcd8b2ae31030836252a389c3c14f71e20185e38"
        },
        "date": 1786588591472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5780456066131592,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17802530011149,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46824641611778,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.628385416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.1640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 517993.184903651,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509819.01558267686,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005608,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15905206.716194833,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15858640.765918076,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.197751025462683,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.49741432070732117,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17021160458933,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54099743250605,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.21497395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.85546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1946948.9306569127,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1956633.3334805416,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005765,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22626902.682453107,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22537894.121698067,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.564201782356136,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b106a3fe9bfe7364ee90bb307d89b3de46c6098e",
          "message": "[otap-dataflow] Kafka Receiver enhancements + integration tests (#3682)\n\n# Change Summary\n\n- fixed bug with Idempotency mode not considering the generation and\nallowing offsets of an old generation to block the same offset but in a\nnew generation from being processed.\n- fixed bug with consumer not being unable to unsubscribe if broker is\nunresponsive\n- Add and reorganized test cases to be defined under various scenario\ntypes\n  - Offset guarantees  \n  - Consumer-group rebalancing\n  - Lifecycle (drain & shutdown)\n  - Failure recovery\n  - Routing & payload correctness\n  - Operational visibility\n  - Security & config validation\n\n## What issue does this PR close?\n\n * Completes part of #3505 \n\n## How are these changes tested?\n\nintegration and unit tests\n\n## Are there any user-facing changes?\n\nno\n\n### Changelog\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-13T18:43:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b106a3fe9bfe7364ee90bb307d89b3de46c6098e"
        },
        "date": 1786653535597,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5449854135513306,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.25133411635652,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47891038538926,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.585546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.53515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519138.1971397682,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511117.5879337694,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005417,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15946373.113378728,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15902175.73189221,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.19903030111548,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5939103960990906,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.26789224003096,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.70861358745773,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.082942708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.4140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1927932.436029696,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1916482.244273846,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008078,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22507171.974214666,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22433117.113780912,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.744002346728038,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4eb82c9408649b996a5049442495b49195efe0c8",
          "message": "feat(engine): add NUMA-aware core placement planning (#3471)\n\n# Change Summary\n\n**_Note:_** The implementation is based on the design proposal #3317,\nand so subject to the design approval.\n\nThis PR moves pipeline core placement into the controller and makes it\ntopology-aware.\n\nToday, `core_count` pipelines each pick their own cores, starting from\nthe lowest. Two pipelines can silently land on the same cores and\ncompete, and nothing in the engine knows which NUMA node a core belongs\nto.\n\nThis change resolves placement explicitly, before any pipeline launches:\n\n- The engine discovers the process-visible CPU set and the CPU-to-NUMA\nmapping.\n- The controller plans `core_count` placement globally, accounting for\nother pipelines' reserved cores.\n- `core_count` pipelines receive exclusive cores instead of silently\noverlapping.\n- Explicit `core_set` is still honored, but now fails if a requested\ncore is hidden by process affinity or cgroup limits.\n- Startup, live rollout, rollback, and full-config reconcile all share\none placement model.\n\nThe default policy is deterministic NUMA packing: it keeps a pipeline on\na single NUMA node when possible, using stable lowest-node then\nlowest-core ordering. When topology is incomplete, it falls back to\ndeterministic visible-core ordering. The policy sits behind a small\nstrategy interface, so balancing or hardware-aware strategies can be\nadded later without touching placement call sites.\n\nThis PR also adds listener-group metadata as groundwork for future\nsocket-placement work. It does **not** bind sockets, enable\n`SO_REUSEPORT`, or attach eBPF selectors - there is no production\nruntime consumer yet.\n\nThe per-record data path is unchanged. Topology discovery and placement\nplanning run only at startup and during live control operations.\n\n## Breaking Behavior Changes\n\nThese configs now fail loudly instead of doing something surprising:\n\n- `core_count` no longer clamps to the available core count. Requesting\nmore cores than are visible is now a validation error.\n- Multiple `core_count` pipelines no longer overlap on the same first\ncores. Each gets an exclusive set, and startup or live update fails when\nthere aren't enough unreserved cores.\n- `core_count: 0` means \"all unreserved visible cores\" and can now fail\nif none remain.\n- Explicit `core_set` fails if any requested core is hidden by process\naffinity or cgroup CPU limits.\n- Full-config reconcile rejects placement transitions that need another\nlive pipeline to vacate cores first. Stage the shrink or delete first,\nthen apply the growth.\n\nExplicit `core_set`-to-`core_set` overlap is still allowed, as\ndeliberate operator intent.\n\n  * Closes #1837\n\nRelated context, not closed by this PR: #2155 (placement abstraction /\nbalancing), #2974 (socket + eBPF placement).\n\n  ## How are these changes tested?\n\n  New and updated unit coverage:\n\n- Linux topology discovery from sysfs, affinity, and cgroup v2 cpuset\nlimits\n- complete, partial, and unknown topology states, including disjoint\naffinity/cgroup visibility\n  - cpulist parse errors, oversized ranges, and duplicate CPU mappings\n  - NUMA-packing placement and deterministic fallback ordering\n  - strategy injection via the placement interface\n  - startup reservation conflicts across pipelines\n  - `core_count: 0` and omitted-count behavior\n  - explicit `core_set` hidden-core rejection\n  - live rollout, rollback, and reconcile placement handling\n  - conservative vacate-before-claim reconcile rejection\n\n  ## Are there any user-facing changes?\n\n  Yes — see **Breaking Behavior Changes** above.\n\n  ### Changelog\n\n  * [x] Added a `.chloggen/*.yaml` entry\n  * [ ] This PR is a `chore` (indicated in title)\n  * [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-13T21:38:52Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4eb82c9408649b996a5049442495b49195efe0c8"
        },
        "date": 1786673453640,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.33207374811172485,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2447098376453,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.62903166308024,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.98229166666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.47265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1937377.0971211176,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1930943.5764734007,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005714,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22313326.47093962,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22236289.74802463,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.55565949352689,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5942554473876953,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19761296761219,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50415187911663,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.621223958333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 518080.61692618014,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509821.08866134036,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005364,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15854171.039139038,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15824920.503211955,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.097519093939468,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bf9ff5d0d28abf0aec68c40dd1d4691c43aa52dc",
          "message": "chore(release): Avoid edge case race conditions during Release prep and push (#3755)\n\n# Change Summary\n\nProtect the release window from changelog gaps.\n\nThe release process previously rendered the changelog at Prepare Release\ntime but created tags from the current `main` tip later. A pull request\nmerged during that window could therefore be included in the release\ntags without appearing in that release's changelog.\n\nThis change closes the gap at three points:\n\n1. **Before and after opening the release PR:** Prepare Release records\nits starting `main` commit and verifies that the merge queue is empty\nand `main` has not moved. If the first check fails, no PR is opened. If\nthe second check fails, the release PR remains open to freeze merges and\nPrepare Release must be rerun to update it from the latest `main`.\n2. **While the release PR is open:** The required `changelog` check\nrejects every other pull request and merge-group entry. The release PR\nis allowed only when no other PR remains queued and its branch contains\nthe latest `main`.\n3. **When creating the release:** Push Release resolves the uniquely\nmerged `otelbot/release-v<VERSION>` PR and creates all release tags at\nthat PR's merge commit rather than the current `main` tip.\n\n```mermaid\nsequenceDiagram\n    participant MQ as Merge queue\n    participant Prep as Prepare Release\n    participant Main as main\n    participant RP as Release PR\n    participant Push as Push Release\n\n    Prep->>Main: Record starting commit\n    Prep->>MQ: Require empty queue\n    Prep->>Main: Require unchanged commit\n    Prep->>RP: Open otelbot/release-vX.Y.Z\n    Prep->>MQ: Recheck empty queue\n    Prep->>Main: Recheck unchanged commit\n\n    Note over RP,MQ: Required changelog check freezes unrelated merges\n    RP->>MQ: Require no other queued PRs\n    RP->>Main: Merge prepared changelog and versions\n\n    Push->>RP: Resolve merged release PR\n    Push->>Main: Tag the release PR merge commit\n    Note over Main,Push: Later main commits remain pending for the next release\n```\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\nN/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [x] This PR is a `chore` (indicated in title)\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-14T16:51:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bf9ff5d0d28abf0aec68c40dd1d4691c43aa52dc"
        },
        "date": 1786730476505,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.1780356764793396,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.24720301212695,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61798686751642,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.755989583333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.3515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 517621.0116078375,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518542.5616598247,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003252,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15893528.964854311,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15858657.997868996,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.650384635699037,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.691778302192688,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21819407508497,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54837458297774,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.37135416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.05859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1906955.819598569,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1920147.7263163333,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00285,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22166528.726344705,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22097481.61548532,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.544178826735177,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786759108230,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3768092393875122,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22086266926433,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54085798358369,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.465755208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.15625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 531727.0319286535,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 524406.1648694605,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006007,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16037732.477028733,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16000717.453603186,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.582654345837025,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.2105906009674072,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.26352629992681,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57249013081508,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.19322916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.80078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1956138.9112679327,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1912896.688454071,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006536,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22452411.584080014,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22385004.674426775,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.737388495468192,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786816069287,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.61404949426651,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18531235177997,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52126949639171,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.610286458333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.2421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521104.8459215513,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 524304.6873694075,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002973,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16032008.395633241,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15993927.472139774,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.577656049711464,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5659370422363281,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21950561773187,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69227820710974,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.858984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.16796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1917856.5866348147,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1907002.726614351,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002985,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22369749.271250725,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22290104.40414907,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.730318451597322,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786845488436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.043259859085083,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19744227263318,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51483161724859,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.403645833333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.99609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530000.0343313318,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 524470.7566882184,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003498,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16071431.982082028,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16036762.33295999,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.64314220980674,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.375333696603775,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23744203794227,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61295687726606,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.575260416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.44921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1936874.3774313217,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1944144.1197619655,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005428,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22289256.094392788,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22287995.38933403,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.464816763235541,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786902480841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1689852476119995,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21635727026721,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53773132017034,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.841145833333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1928504.4997984353,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1905960.5682187236,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003021,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22381912.28790084,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22310030.319624666,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.74311402927846,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.7412351369857788,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.3112005986024,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.79385568287395,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.650390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 363126.89141852525,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 356803.9986057344,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002915,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11164052.714168971,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11111798.897678625,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.289034758002146,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "c1ly",
            "username": "c1ly",
            "email": "129437996+c1ly@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5abeaf8efbd37b6c8519fc8edcd519281e988bbd",
          "message": "[otap-dataflow] Kafka Exporter enhancement + integration tests (#3683)\n\n# Change Summary\n\n- Updated to immediately send a permanent nack for unretryable errors\nthat occur when attempting to send data\n- Add live reconfiguration support\n- Updated config to allow control of dynamic topics via allowlist and\nregex\n- Updated config to allow user to directly update config setting for\nautomatic topic creation\n- Added and reorganized test under various scenario types\n  - Security\n  - Shutdown & Live Reconfiguration\n  - Retry expectations\n  - Delivery semantics\n  - Kafka Integration\n  - Telemetry/Observability\n  - Configuration\n  \n## What issue does this PR close?\n\n* Completes part of #3509 \n\n## How are these changes tested?\n\nintegration tests and unit tests\n\n## Are there any user-facing changes?\n\nyes, allow_auto_create_topics is now exposed directly in the Kafka\nExporter config and overwrites any changes set in the producer_config\nhash map (if configured)\n\n### Changelog\n\n* [ x ] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-08-15T01:02:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5abeaf8efbd37b6c8519fc8edcd519281e988bbd"
        },
        "date": 1786931864577,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.4215278625488281,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23638288085697,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47454517278784,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.56875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.9609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 533007.2324470382,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525430.3863432255,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00597,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16092695.825369531,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16053879.271173056,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.627645913986676,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9834574460983276,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22078638272087,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59266140269392,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.331380208333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.84375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1931985.8719837787,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1912985.6134810545,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010973,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22442872.4258426,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22356012.464258004,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.731856354634768,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "renovate[bot]",
            "username": "renovate[bot]",
            "email": "29139614+renovate[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "57058ad9b922bd8c170c04c3ab6dd0258000c40f",
          "message": "chore(deps): update geneva-uploader digest to 7819998 (#3779)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `f101f2a` →\n`7819998` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4yOS41IiwidXBkYXRlZEluVmVyIjoiNDQuMjkuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-17T06:16:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/57058ad9b922bd8c170c04c3ab6dd0258000c40f"
        },
        "date": 1786988982636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9762901067733765,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22087533104269,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.61388687398718,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.37083333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1933153.3598579739,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1952026.5446822497,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008102,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22529695.159760937,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22458598.673051577,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.541695076399852,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.1348453313112259,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23816478626621,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5470580035623,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.223958333333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.7734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525162.0268746521,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525870.1833639716,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009335,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16098665.442952694,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16062134.7819959,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.61338321174656,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "55aebb067841cf02a78ea7bb4bc24e838c8945b0",
          "message": "feat(file-exporter): add multi-signal file exporter (#3776)\n\n# Change Summary\n\nAdds an experimental multi-signal file exporter for OTAP Dataflow. The\nexporter writes logs, metrics, and traces as bounded OTLP JSON lines and\nsupports multi-core path templates, configurable open and durability\nmodes, partial-tail recovery, and exporter telemetry.\n\n## What issue does this PR close?\n\n- Closes #3773\n\n## How are these changes tested?\n\n- Unit and integration tests covering all three OTel signals\n- Configuration and path-collision validation tests\n- File open mode, tail recovery, path lease, and frame-size tests\n- cargo xtask check\n\n## Are there any user-facing changes?\n\nYes. Users can configure the experimental `exporter:file` component to\ncapture logs, metrics, and traces as OTLP JSONL files.\n\n### Changelog\n\n- [x] Added a .chloggen/*.yaml entry\n- [ ] This PR is a chore (indicated in title)\n- [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-18T00:57:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/55aebb067841cf02a78ea7bb4bc24e838c8945b0"
        },
        "date": 1787019104373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.57273006439209,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16979942658901,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56762170110673,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.79453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1970345.4110359915,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1919653.7398787832,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005755,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22568594.349602424,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22483640.81727825,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.756596453185104,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7893999218940735,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19537744929377,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44354836223506,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.35859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.08984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 517732.35923400405,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 513645.38021205104,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007159,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16005189.703914678,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15964367.404213933,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.159999331264633,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "46f0898750a81543bad4dd245339f02ff8713b5f",
          "message": "chore(repo): Split build/test chains and avoid full CI repeat after merge (#3807)\n\n# Chore Summary\n\n- Split the required Linux and Windows Rust build/test chains so each\ntest matrix starts as soon as its matching build completes\n- Make Rust-CI caches restore-only to prevent ephemeral pull-request and\nmerge-queue refs from consuming the repository cache budget\n- Replace the full Rust-CI run on `main` with a `Post-Merge Actions`\nworkflow that warms the required Linux and Windows build caches\n\n## Expected impact\n\n| Area | Before | After | Potential gain |\n| --- | --- | --- | --- |\n| Required test chain | Linux tests wait for the slower Windows build |\nLinux and Windows tests start after their matching builds |\nApproximately 7-9 minutes removed from the Linux test path |\n| End-to-end required status | The delayed Linux test path can determine\nworkflow completion | Other required jobs may become the critical path |\nApproximately 2 minutes in a sampled PR run and 8-9 minutes in a sampled\nmerge-queue run |\n| Rust-CI cache writes | Large job-specific caches are saved under\nephemeral PR and merge-queue refs | Rust-CI restores caches but never\nsaves them | Reduces churn within GitHub's 10 GB repository cache limit\n|\n| Post-merge Rust-CI | Approximately 6.4 runner-hours across 57 jobs |\nApproximately 0.8 runner-hours across two cache-warming jobs |\nApproximately 85% less compute |\n| Shared cache priority | The full suite competes to publish many\njob-specific caches | `main` publishes the required Linux and Windows\nbuild caches | Prioritizes the merge-critical build configurations |\n\n## Tradeoff\n\nThe post-merge workflow intentionally does not warm Rust-CI's coverage,\nclippy, smoke-test, ARM, or macOS caches. Those configurations can still\nrestore an existing compatible cache, but only the required Linux and\nWindows build caches are guaranteed to be refreshed after each merge.\n\nThe current cache inventory supports prioritizing these builds:\n\n| Observation | Current state |\n| --- | --- |\n| Repository cache limit | 10 GB |\n| Recent Rust cache size | Commonly 1-2 GB per entry |\n| Dominant cache refs | `refs/pull/*/merge` entries from recent PR runs\n|\n| Shared `main` Rust caches | None were present in the inspected cache\ninventory |\n\nThis means a small number of ephemeral PR entries can consume the\nrepository budget and evict caches that would otherwise be reusable by\nevery PR and merge-queue run. The restore-only Rust-CI policy stops\nadding those ephemeral entries, while the post-merge workflow\nrepopulates the two critical shared caches.",
          "timestamp": "2026-08-18T20:06:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/46f0898750a81543bad4dd245339f02ff8713b5f"
        },
        "date": 1787090391179,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.029637379571795464,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23147465241871,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60989260700389,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.250390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1957701.4636788692,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1958281.6750610613,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005717,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22523311.7335793,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22451876.599413402,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.5015689624308,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5017956495285034,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.27349533426903,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56286201022147,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.956901041666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.6328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522682.23934378894,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 514832.6203396054,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008008,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16034823.303697752,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15998088.487001082,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.145701865434447,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2e9fff0f0b8808f44b663fdf27a09b2034a5096b",
          "message": "  feat(geneva-exporter): support account group routing (#3799)\n\n## Summary\n\n- add required `account_routing.default_group` configuration to the\nGeneva exporter\n- support optional destination event/table overrides through\n`account_routing.events`\n  - pass account routing into `geneva-uploader`\n- preserve the complete logical-group-to-primary-moniker map from\nagent-fed credentials\n- update `geneva-uploader` to the revision containing multi-moniker\nrouting\n  - update Geneva examples, documentation, tests, and changelog\n\n  This integrates the account-group routing added by\n  https://github.com/open-telemetry/opentelemetry-rust-contrib/pull/747.\n\n  ## Routing model\n\n  YAML config selects logical GCS account groups:\n\n  ```yaml\n  account_routing:\n    default_group: \"diagnostics\"\n    events:\n      AuditLogs: \"audit\"\n      SecurityEvents: \"security\"\n```\n\n  The events keys are final destination event/table names after\n  event_name_mapping has run. Events without an exact override use\n  default_group.\n\n  Physical monikers are not configured in YAML. The uploader resolves the chosen\n  logical group against the current primary-moniker mapping supplied by GCS or an\n  agent-fed credential snapshot.\n\n  ## Breaking configuration change\n\n  Every Geneva exporter configuration must now provide:\n\n```yaml\n  account_routing:\n    default_group: \"<logical-account-group>\"\n```\n\n  Existing configurations must add the logical GCS account group that should\n  receive events without an explicit override.\n\n  ## Validation\n\n  - cargo xtask check\n  - cargo test -p otap-df-contrib-nodes --features geneva-exporter geneva_exporter\n      - 87 passed\n\n  - python3 tools/sanitycheck.py\n  - make chlog-validate\n  - Markdown lint\n  - git diff --check\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-19T00:30:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2e9fff0f0b8808f44b663fdf27a09b2034a5096b"
        },
        "date": 1787107215301,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.49201464653015137,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20896813606275,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58361904761905,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.8703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1945863.1824282731,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1955437.1139438911,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002936,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22537623.026867487,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22463758.481961813,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.525618935099223,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.26123303174972534,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21638012246737,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50860501082586,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.960286458333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.77734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522624.27885368955,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 523989.5460398277,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002907,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16059787.587942267,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16016911.330859568,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.649061053446257,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2e9fff0f0b8808f44b663fdf27a09b2034a5096b",
          "message": "  feat(geneva-exporter): support account group routing (#3799)\n\n## Summary\n\n- add required `account_routing.default_group` configuration to the\nGeneva exporter\n- support optional destination event/table overrides through\n`account_routing.events`\n  - pass account routing into `geneva-uploader`\n- preserve the complete logical-group-to-primary-moniker map from\nagent-fed credentials\n- update `geneva-uploader` to the revision containing multi-moniker\nrouting\n  - update Geneva examples, documentation, tests, and changelog\n\n  This integrates the account-group routing added by\n  https://github.com/open-telemetry/opentelemetry-rust-contrib/pull/747.\n\n  ## Routing model\n\n  YAML config selects logical GCS account groups:\n\n  ```yaml\n  account_routing:\n    default_group: \"diagnostics\"\n    events:\n      AuditLogs: \"audit\"\n      SecurityEvents: \"security\"\n```\n\n  The events keys are final destination event/table names after\n  event_name_mapping has run. Events without an exact override use\n  default_group.\n\n  Physical monikers are not configured in YAML. The uploader resolves the chosen\n  logical group against the current primary-moniker mapping supplied by GCS or an\n  agent-fed credential snapshot.\n\n  ## Breaking configuration change\n\n  Every Geneva exporter configuration must now provide:\n\n```yaml\n  account_routing:\n    default_group: \"<logical-account-group>\"\n```\n\n  Existing configurations must add the logical GCS account group that should\n  receive events without an explicit override.\n\n  ## Validation\n\n  - cargo xtask check\n  - cargo test -p otap-df-contrib-nodes --features geneva-exporter geneva_exporter\n      - 87 passed\n\n  - python3 tools/sanitycheck.py\n  - make chlog-validate\n  - Markdown lint\n  - git diff --check\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-19T00:30:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2e9fff0f0b8808f44b663fdf27a09b2034a5096b"
        },
        "date": 1787161991955,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1642087697982788,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17481857670589,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47290597628091,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.234244791666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524785.8363459114,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518676.2339195497,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002595,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16151736.660051465,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16109050.886864748,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.14030604023533,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.22799156606197357,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19848858385254,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57404769278415,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.960286458333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.08984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1942434.9249931774,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1946863.512790012,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002875,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22290826.11926871,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22230317.020747747,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.449609062385768,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla",
            "email": "66651184+utpilla@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1f0f51b347e76a5fd58028e52db3225b2428787d",
          "message": "chore: [Geneva Exporter] Update the YAML to show events mapping example (#3824)\n\n# Change summary\n- Update the example YAML to add some comments and also show event\nmapping\n\nCo-authored-by: Utkarsh Umesan Pillai <utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-19T23:12:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f0f51b347e76a5fd58028e52db3225b2428787d"
        },
        "date": 1787191099373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.013705200515687466,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 79.18203636284528,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.13124816219144,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.846484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.5546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 373563.2425276012,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 373614.4401213143,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00282,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11490105.74015082,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11435156.740786228,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.753912339201694,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.532603144645691,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16988856420437,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5149234812181,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.980859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.3203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1902358.015100716,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1873202.4164678177,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005765,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21932173.12671858,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21870288.71828312,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.708383959953846,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6c815bd0bdf6c5e023daf7987e17ca5b4dda565c",
          "message": "chore(clippy): Resolve clippy 1.98 errors in main (#3835)\n\n# Chore Summary\n\nAddress clippy error on unrelated PR:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/32402148082/job/96532892549?pr=3834\n\n```text\nerror: draining all elements of a collection into a new collection of the same type\n    --> crates/core-nodes/src/processors/batch_processor/mod.rs:1485:22\n     |\n1485 |             pending: self.pending.drain(..).collect(),\n     |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use `mem::take` to avoid creating a new allocation: `std::mem::take(&mut self.pending)`\n     |\n     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#drain_collect\n     = note: `-D clippy::drain-collect` implied by `-D clippy::perf`\n     = help: to override `-D clippy::perf` add `#[allow(clippy::drain_collect)]`\n\nerror: draining all elements of a collection into a new collection of the same type\n    --> crates/core-nodes/src/processors/batch_processor/mod.rs:1486:22\n     |\n1486 |             context: self.context.drain(..).collect(),\n     |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: use `mem::take` to avoid creating a new allocation: `std::mem::take(&mut self.context)`\n     |\n     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#drain_collect\n\nerror: the `Err`-variant returned from this function is very large\n   --> crates/core-nodes/src/processors/fanout_processor/mod.rs:562:10\n    |\n562 |     ) -> Result<DeadlineVec, TypedError<OtapPdata>> {\n    |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the `Err`-variant is at least 152 bytes\n    |\n    = help: try reducing the size of `otap_df_engine::error::TypedError<otap_df_otap::pdata::OtapPdata>`, for example by boxing large elements or replacing it with `Box<otap_df_engine::error::TypedError<otap_df_otap::pdata::OtapPdata>>`\n    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#result_large_err\n    = note: `-D clippy::result-large-err` implied by `-D clippy::perf`\n    = help: to override `-D clippy::perf` add `#[allow(clippy::result_large_err)]`\n```\n\n## Related issue\n\n<!-- Link the related issue if one exists. -->",
          "timestamp": "2026-08-20T19:42:52Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6c815bd0bdf6c5e023daf7987e17ca5b4dda565c"
        },
        "date": 1787257981101,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.627602458000183,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20955532868541,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53438167466338,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.32252604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.24609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1941778.2921950347,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1910173.8611339713,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005763,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22447250.679286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22370155.798456363,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.751417573037163,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4164305925369263,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20447055718715,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49209792376821,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.949348958333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.67578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527120.8904551234,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519654.5888906032,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002934,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16171500.578836769,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16131375.719755074,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.119710909049942,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "26b710aa0c6e5900e94523992c018104db6cfe24",
          "message": "chore(engine): add local retained-work accounting (#3756)\n\n# Change Summary\n\nAdd a runtime-local retained-work account and non-`Send` ownership\nticket.\n\nThe account tracks known retained bytes and unknown-size items. Tickets\nrefund their charge exactly once on explicit completion. Dropping an\nunresolved ticket also refunds the charge and records abandonment.\n\nChecked arithmetic reports overflow and underflow as accounting\ncorruption.\n\nThis PR does not add runtime wiring, attribution, metrics export,\nconfiguration, enforcement, escrow, or production charge sites.\n\n## Background\n\nThe retained-work pilot needs a runtime-local accounting primitive\nbefore scope wiring, metrics, or processor integration can be added.\n\n## What issue does this PR close?\n\n* Part of #3272\n\n## How are these changes tested?\n\n- `cargo check -p otap-df-engine`\n- `cargo test -p otap-df-engine retained_work::tests`\n- `cargo test -p otap-df-engine --doc`\n- `cargo clippy -p otap-df-engine --all-targets -- -D warnings`\n- `cargo xtask check`\n- `python3 tools/sanitycheck.py`\n- `git diff --check`\n\n## Are there any user-facing changes?\n\nNo. This PR adds an internal accounting primitive without changing\nruntime behavior, configuration, or exported telemetry.\n\n  ### Changelog\n\n  * [ ] Added a `.chloggen/*.yaml` entry\n  * [x] This PR is a `chore` (indicated in title)\n  * [ ] This is a documentation-only PR.",
          "timestamp": "2026-08-21T00:33:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/26b710aa0c6e5900e94523992c018104db6cfe24"
        },
        "date": 1787292487030,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.4109634160995483,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17443126226217,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45573908305441,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.77265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526143.0599437957,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518719.37421315827,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00254,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16144895.32181561,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16111468.621081578,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.124527296297128,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8066998720169067,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22999037831568,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57706278547651,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.64453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.1015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1939919.9730933763,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1924270.640223419,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003069,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22530609.40141406,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22453041.293057036,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.70864894493122,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "845e34cdd9da76b9ff3b1fb312fefcbe63678ddb",
          "message": "chore(repo): Update CODEOWNERS to specify required approvals for core engine crates (#3851)\n\n# Chore Summary\n\nPer offline discussion among maintainers, wanted to clarify some\nadditional CODEOWNER policies as contribution trends increase to help\nensure proper visibility to core engine crates.\n\n## Related issue\n\nN/A\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-08-21T16:29:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/845e34cdd9da76b9ff3b1fb312fefcbe63678ddb"
        },
        "date": 1787334707542,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.43208232522010803,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20325331140738,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53135649616962,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.945703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.58984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1923491.6723997248,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1915180.6046643094,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00288,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22484115.587144475,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22408952.718331598,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.73994532546212,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.17634610831737518,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.98463327551089,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.10325233644859,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.234114583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 362904.79360558157,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 363544.7620871339,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002955,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11151608.994600212,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11101400.354840828,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.67465181062741,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Maksat Maratov",
            "username": "maksmara",
            "email": "mmaratov@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b6c89a1811bef396c531145227b3130066d61c20",
          "message": "feat(console-exporter): add compact histogram percentile estimates (#3850)\n\n# Change summary\n\nAdd approximate p50, p90, and p99 values to compact pretty output for\nexplicit\nand exponential histograms.\n\nThe estimator:\n\n- uses nearest-rank bucket selection;\n- uses arithmetic midpoints for explicit buckets and geometric midpoints\nfor\n  exponential buckets;\n- handles negative, zero, and positive exponential bucket populations;\n- supports the full OTLP `sint32` scale range;\n- marks estimates with `~=` to distinguish them from exact values;\n- omits estimates when bucket data is empty, internally inconsistent, or\ncannot\n  be represented safely;\n- produces equivalent output for OTLP and OTAP views.\n\nThe PR also corrects raw OTLP ZigZag decoding so exponential histogram\n`scale`\nand bucket `offset` values can use the full `sint32` range.\n\nRaw histogram output remains unchanged.\n\n## Related issue\n\n* Closes #3840\n\n## Validation\n\n- `cargo test -p otap-df-core-nodes console_exporter`\n- `cargo test -p otap-df-pdata decodes_full_sint32_range`\n- `cargo check -p otap-df-core-nodes`\n- `cargo clippy -p otap-df-core-nodes --all-targets -- -D warnings`\n- `cargo clippy -p otap-df-pdata --all-targets -- -D warnings`\n- `npx markdownlint-cli2\nrust/otap-dataflow/crates/core-nodes/src/exporters/console_exporter/README.md`\n- `chloggen validate --config rust/otap-dataflow/.chloggen/config.yaml`\n- `cargo xtask check`\n\n## User-facing changes\n\nCompact console histogram output now includes approximate `p50~=`,\n`p90~=`,\nand `p99~=` fields when sufficient valid bucket data is available.\nIndividual\npercentiles are omitted when they cannot be estimated safely.\n\nRaw OTLP exponential histogram scale and bucket offset values are now\ndecoded\ncorrectly across the full `sint32` range.\n\nRaw histogram output is unchanged.\n\nAdded\n\n`rust/otap-dataflow/.chloggen/console-compact-histogram-percentiles.yaml`.",
          "timestamp": "2026-08-21T23:38:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b6c89a1811bef396c531145227b3130066d61c20"
        },
        "date": 1787362471736,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.4417632818222046,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21430256699307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48282112752301,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.737760416666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.3203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 535022.852236426,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527309.0891637201,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00288,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16112460.659618212,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16079395.653720777,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.556007834364443,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.2335422933101654,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20770288694625,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.5584765443567,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.108072916666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.6953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1943804.6114871607,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1939265.0056410916,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001685,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22365893.62686169,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22302365.023973394,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.533180644111022,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Maksat Maratov",
            "username": "maksmara",
            "email": "mmaratov@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b6c89a1811bef396c531145227b3130066d61c20",
          "message": "feat(console-exporter): add compact histogram percentile estimates (#3850)\n\n# Change summary\n\nAdd approximate p50, p90, and p99 values to compact pretty output for\nexplicit\nand exponential histograms.\n\nThe estimator:\n\n- uses nearest-rank bucket selection;\n- uses arithmetic midpoints for explicit buckets and geometric midpoints\nfor\n  exponential buckets;\n- handles negative, zero, and positive exponential bucket populations;\n- supports the full OTLP `sint32` scale range;\n- marks estimates with `~=` to distinguish them from exact values;\n- omits estimates when bucket data is empty, internally inconsistent, or\ncannot\n  be represented safely;\n- produces equivalent output for OTLP and OTAP views.\n\nThe PR also corrects raw OTLP ZigZag decoding so exponential histogram\n`scale`\nand bucket `offset` values can use the full `sint32` range.\n\nRaw histogram output remains unchanged.\n\n## Related issue\n\n* Closes #3840\n\n## Validation\n\n- `cargo test -p otap-df-core-nodes console_exporter`\n- `cargo test -p otap-df-pdata decodes_full_sint32_range`\n- `cargo check -p otap-df-core-nodes`\n- `cargo clippy -p otap-df-core-nodes --all-targets -- -D warnings`\n- `cargo clippy -p otap-df-pdata --all-targets -- -D warnings`\n- `npx markdownlint-cli2\nrust/otap-dataflow/crates/core-nodes/src/exporters/console_exporter/README.md`\n- `chloggen validate --config rust/otap-dataflow/.chloggen/config.yaml`\n- `cargo xtask check`\n\n## User-facing changes\n\nCompact console histogram output now includes approximate `p50~=`,\n`p90~=`,\nand `p99~=` fields when sufficient valid bucket data is available.\nIndividual\npercentiles are omitted when they cannot be estimated safely.\n\nRaw OTLP exponential histogram scale and bucket offset values are now\ndecoded\ncorrectly across the full `sint32` range.\n\nRaw histogram output is unchanged.\n\nAdded\n\n`rust/otap-dataflow/.chloggen/console-compact-histogram-percentiles.yaml`.",
          "timestamp": "2026-08-21T23:38:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b6c89a1811bef396c531145227b3130066d61c20"
        },
        "date": 1787419458029,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.4752577543258667,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22256011732517,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56713336424046,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.10625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524031.2848956435,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 516300.4731112419,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003013,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16077180.587626806,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16039288.113241604,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.13919398668229,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4803158044815063,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18732003417021,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55390712074303,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.17356770833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.65234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1919336.1199530414,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1947748.3558053283,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007949,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22472164.050425395,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22398786.652210973,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.53750893099017,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot",
            "email": "107717825+opentelemetrybot@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "929d7e87d6402c4a5a92b97fb0f9d4f5535b983b",
          "message": "chore: use shared OSSF Scorecard workflow (#3861)\n\nDesign discussion: open-telemetry/sig-security#309\n\n## Changes\n\nMigrate OSSF Scorecard to the shared workflow. This limits code scanning\nalerts from Scorecard to `BinaryArtifactsID`, `DangerousWorkflowID`,\n`PinnedDependenciesID`, and `TokenPermissionsID`.",
          "timestamp": "2026-08-22T18:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/929d7e87d6402c4a5a92b97fb0f9d4f5535b983b"
        },
        "date": 1787448891244,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5106987953186035,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1796144479483,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4802479274812,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.042317708333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.65234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537141.5273379636,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 529026.9367437172,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004506,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16173989.43027361,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16137472.038343603,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.57309242101781,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7361494302749634,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22024313461404,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55133126934986,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.731119791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.87109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1965883.7290519378,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1951411.8867490743,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002865,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22496085.527386986,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22413506.985057242,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.528107254109232,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot",
            "email": "107717825+opentelemetrybot@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "929d7e87d6402c4a5a92b97fb0f9d4f5535b983b",
          "message": "chore: use shared OSSF Scorecard workflow (#3861)\n\nDesign discussion: open-telemetry/sig-security#309\n\n## Changes\n\nMigrate OSSF Scorecard to the shared workflow. This limits code scanning\nalerts from Scorecard to `BinaryArtifactsID`, `DangerousWorkflowID`,\n`PinnedDependenciesID`, and `TokenPermissionsID`.",
          "timestamp": "2026-08-22T18:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/929d7e87d6402c4a5a92b97fb0f9d4f5535b983b"
        },
        "date": 1787505886723,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.2028080224990845,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.2295704181311,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58558287442798,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.55065104166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.25390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1942380.231329499,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1919017.1261985425,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002983,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22483108.94531101,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22409469.175048016,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.715950127995312,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.104449987411499,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19837341846095,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50799471680523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.6328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534589.9315421822,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 517993.8545318786,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004542,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16123310.285046121,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16091386.776254263,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.126450910536533,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "OpenTelemetry Bot",
            "username": "opentelemetrybot",
            "email": "107717825+opentelemetrybot@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "929d7e87d6402c4a5a92b97fb0f9d4f5535b983b",
          "message": "chore: use shared OSSF Scorecard workflow (#3861)\n\nDesign discussion: open-telemetry/sig-security#309\n\n## Changes\n\nMigrate OSSF Scorecard to the shared workflow. This limits code scanning\nalerts from Scorecard to `BinaryArtifactsID`, `DangerousWorkflowID`,\n`PinnedDependenciesID`, and `TokenPermissionsID`.",
          "timestamp": "2026-08-22T18:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/929d7e87d6402c4a5a92b97fb0f9d4f5535b983b"
        },
        "date": 1787535289650,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.5385611057281494,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19032480577121,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42944621359224,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.313541666666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.84375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527425.640604467,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 519310.87487967935,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003211,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16164758.394287186,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16126067.51126693,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.127325030565643,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.31770452857017517,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19777867549571,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42697284716834,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.620052083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1963282.8206343635,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1957045.3821678474,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004119,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22571768.05206375,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22506547.456174597,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.53359460017257,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "renovate[bot]",
            "username": "renovate[bot]",
            "email": "29139614+renovate[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3e85c3460361446ebfce99e9f35fffd2dd5ab740",
          "message": "chore(deps): update geneva-uploader digest to 43bd04b (#3867)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `b4cbfda` →\n`43bd04b` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4zOS4wIiwidXBkYXRlZEluVmVyIjoiNDQuMzkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-24T15:08:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3e85c3460361446ebfce99e9f35fffd2dd5ab740"
        },
        "date": 1787592657458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.07094994187355042,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1989142395733,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47372597452721,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.84427083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.49609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1948229.489496872,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1949611.7571677805,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005744,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22478449.794497725,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22410528.41898183,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.529705702612494,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.1009645462036133,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.15546254254859,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.40049504950494,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.111458333333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.05859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 531679.4613389872,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525825.8589624679,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002709,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16111420.929952295,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16080072.45826347,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.640221767987047,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ebc01cc6fb0c71b9830895fbdaffca89607699a5",
          "message": "feat(engine): Add opt-in `size` metric to node produced/consumed (#3842)\n\n# Change summary\n\nClosing the loop on final piece to match the [Go Collector universal\ntelemetry RFC on auto-instrumented\nmetrics](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#auto-instrumented-metrics)\n🥳\n\nAdds logical payload size to the engine-owned node outcome metrics:\n\n- `node.producer.produced.size{signal,outcome}`\n- `node.consumer.consumed.size{signal,outcome}`\n\nSize follows the existing item-count policy model. `runtime_metrics:\ndetailed` enables both measurements for every node. At `runtime_metrics:\nnormal`, nodes can independently opt in with:\n\n```yaml\npolicies:\n  telemetry:\n    size: true\n```\n\nThe forward path measures the payload once at each produced or consumed\nboundary and stores the value on the context frame. Ack/Nack unwinding\nrecords the stored size with the same signal and outcome as the\ncorresponding message metric.\n\nOTLP payloads report their encoded protobuf length. OTAP payloads report\nlogical Arrow bytes. Cached OTAP sizing avoids repeating the Arrow array\nand buffer walk when the payload is unchanged.\n\nThe `trafficgen-universal-produced-consumed-metrics.yaml` demo includes\nboth item and size policies and prints only produced/consumed node\nmetrics through its internal observability pipeline.\n\n### Sample config run\n\nThe `full` pipeline uses `runtime_metrics: detailed`. Its log sampler\nkeeps one third of log records while metrics and traces pass through\nunchanged:\n\n| Signal | Receiver produced | Sampler consumed | Sampler produced |\nNoop consumed |\n| --- | ---: | ---: | ---: | ---: |\n| Logs messages | 4 | 4 | 4 | 4 |\n| Logs items | 30 | 30 | 10 | 10 |\n| Logs size (By) | 7,472 | 7,472 | 6,448 | 6,448 |\n| Metrics messages | 2 | 2 | 2 | 2 |\n| Metrics items | 18 | 18 | 18 | 18 |\n| Metrics size (By) | 3,710 | 3,710 | 3,710 | 3,710 |\n| Traces messages | 2 | 2 | 2 | 2 |\n| Traces items | 12 | 12 | 12 | 12 |\n| Traces size (By) | 2,290 | 2,290 | 2,290 | 2,290 |\n\nDropping two thirds of the log records reduces logical size from `7,472\nBy` to `6,448 By` rather than by two thirds. OTAP is a columnar,\nrelational representation: resource, scope, attribute, and dictionary\nbuffers remain largely unchanged, while sampling removes primarily the\nper-record offsets and dictionary keys. The synthetic records also\nrepeat values that are stored once in dictionaries. Logical Arrow size\ntherefore describes the resulting representation, not an average record\nsize multiplied by the item count.\n\nThe `partial` pipeline uses `runtime_metrics: normal` and opts only the\nsampler into `item_counts` and `size`:\n\n| Node boundary | Messages (logs / metrics / traces) | Items (logs /\nmetrics / traces) | Size in By (logs / metrics / traces) |\n| --- | ---: | ---: | ---: |\n| Receiver produced | 4 / 2 / 2 | Not present | Not present |\n| Sampler consumed | 4 / 2 / 2 | 30 / 18 / 12 | 7,472 / 3,710 / 2,290 |\n| Sampler produced | 4 / 2 / 2 | 10 / 18 / 12 | 6,448 / 3,710 / 2,290 |\n| Noop consumed | 4 / 2 / 2 | Not present | Not present |\n\n## Related issue\n\n* Closes #2884\n\n## Validation\n\nLocal engine runs\n\n## User-facing changes\n\nYes. Users can enable node-level logical payload size metrics globally\nwith `runtime_metrics: detailed` or per node with\n`policies.telemetry.size: true`.",
          "timestamp": "2026-08-24T22:34:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ebc01cc6fb0c71b9830895fbdaffca89607699a5"
        },
        "date": 1787623783820,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.535380482673645,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18465985493295,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46623762376237,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.165104166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524051.80934269616,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 516005.6200136694,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005548,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16085313.536188126,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16050405.968506845,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.172748730453772,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.368079662322998,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22086606812542,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.56762872315863,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.829427083333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1920331.5528450843,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1946603.2175176607,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005638,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22428894.41409943,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22360542.013837855,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.522067883305521,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ebc01cc6fb0c71b9830895fbdaffca89607699a5",
          "message": "feat(engine): Add opt-in `size` metric to node produced/consumed (#3842)\n\n# Change summary\n\nClosing the loop on final piece to match the [Go Collector universal\ntelemetry RFC on auto-instrumented\nmetrics](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#auto-instrumented-metrics)\n🥳\n\nAdds logical payload size to the engine-owned node outcome metrics:\n\n- `node.producer.produced.size{signal,outcome}`\n- `node.consumer.consumed.size{signal,outcome}`\n\nSize follows the existing item-count policy model. `runtime_metrics:\ndetailed` enables both measurements for every node. At `runtime_metrics:\nnormal`, nodes can independently opt in with:\n\n```yaml\npolicies:\n  telemetry:\n    size: true\n```\n\nThe forward path measures the payload once at each produced or consumed\nboundary and stores the value on the context frame. Ack/Nack unwinding\nrecords the stored size with the same signal and outcome as the\ncorresponding message metric.\n\nOTLP payloads report their encoded protobuf length. OTAP payloads report\nlogical Arrow bytes. Cached OTAP sizing avoids repeating the Arrow array\nand buffer walk when the payload is unchanged.\n\nThe `trafficgen-universal-produced-consumed-metrics.yaml` demo includes\nboth item and size policies and prints only produced/consumed node\nmetrics through its internal observability pipeline.\n\n### Sample config run\n\nThe `full` pipeline uses `runtime_metrics: detailed`. Its log sampler\nkeeps one third of log records while metrics and traces pass through\nunchanged:\n\n| Signal | Receiver produced | Sampler consumed | Sampler produced |\nNoop consumed |\n| --- | ---: | ---: | ---: | ---: |\n| Logs messages | 4 | 4 | 4 | 4 |\n| Logs items | 30 | 30 | 10 | 10 |\n| Logs size (By) | 7,472 | 7,472 | 6,448 | 6,448 |\n| Metrics messages | 2 | 2 | 2 | 2 |\n| Metrics items | 18 | 18 | 18 | 18 |\n| Metrics size (By) | 3,710 | 3,710 | 3,710 | 3,710 |\n| Traces messages | 2 | 2 | 2 | 2 |\n| Traces items | 12 | 12 | 12 | 12 |\n| Traces size (By) | 2,290 | 2,290 | 2,290 | 2,290 |\n\nDropping two thirds of the log records reduces logical size from `7,472\nBy` to `6,448 By` rather than by two thirds. OTAP is a columnar,\nrelational representation: resource, scope, attribute, and dictionary\nbuffers remain largely unchanged, while sampling removes primarily the\nper-record offsets and dictionary keys. The synthetic records also\nrepeat values that are stored once in dictionaries. Logical Arrow size\ntherefore describes the resulting representation, not an average record\nsize multiplied by the item count.\n\nThe `partial` pipeline uses `runtime_metrics: normal` and opts only the\nsampler into `item_counts` and `size`:\n\n| Node boundary | Messages (logs / metrics / traces) | Items (logs /\nmetrics / traces) | Size in By (logs / metrics / traces) |\n| --- | ---: | ---: | ---: |\n| Receiver produced | 4 / 2 / 2 | Not present | Not present |\n| Sampler consumed | 4 / 2 / 2 | 30 / 18 / 12 | 7,472 / 3,710 / 2,290 |\n| Sampler produced | 4 / 2 / 2 | 10 / 18 / 12 | 6,448 / 3,710 / 2,290 |\n| Noop consumed | 4 / 2 / 2 | Not present | Not present |\n\n## Related issue\n\n* Closes #2884\n\n## Validation\n\nLocal engine runs\n\n## User-facing changes\n\nYes. Users can enable node-level logical payload size metrics globally\nwith `runtime_metrics: detailed` or per node with\n`policies.telemetry.size: true`.",
          "timestamp": "2026-08-24T22:34:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ebc01cc6fb0c71b9830895fbdaffca89607699a5"
        },
        "date": 1787679252942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.0595383644104004,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21693062808228,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54116226882303,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.7171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1958537.2181550513,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1918200.3914148672,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.01305,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22439096.58790697,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22375098.29093588,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.697993957427911,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0979121923446655,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1738341617529,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42210705973622,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.027604166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.40625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521497.4554089513,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515771.87139528216,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002962,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16027984.12534954,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15992100.96106034,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.075723617866437,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a942658eab44e66650313ecdce8ca90a924feae7",
          "message": "fix(metrics): Simplify engine-owned node and flow metric names (#3853)\n\n# Change summary\n\nSimplify engine-owned node and flow metric names around the direction of\ndata through the pipeline:\n\n- `node.consumer.consumed.*` becomes `node.input.*`\n- `node.producer.produced.*` becomes `node.output.*`\n- `flow.consumed.*` becomes `flow.input.*`\n- `flow.produced.*` becomes `flow.output.*`\n\nAlso:\n\n- Add `messages` and logical payload `size` measurements to flow input\nand output metrics alongside `items`.\n- Report `flow.compute.duration` as a seconds-based exponential\nhistogram while retaining its processor-compute semantics.\n- Gate flow measurements with a compact interest bitmap so disabled item\nand size metrics do not inspect PData.\n- Consolidate the node-versus-flow example as\n`trafficgen-input-output-metrics.yaml`.\n- Align Rust flow metric types, fields, methods, tests, documentation,\nand configuration values with input/output terminology.\n\n## Related issue\n\n- Related to #3300\n\n### Validation\n\nThe `trafficgen-input-output-metrics.yaml` example produced the\nfollowing representative one-second delta interval. All node rows have\n`outcome=success`.\n\nThe `full` pipeline enables detailed node metrics and a one-processor\nflow around `sampler`:\n\n```text\nreceiver output\n      │\n      ├── sampler node input == flow input\n      │            30 logs in\n      │            20 logs dropped\n      │            10 logs out\n      └── sampler node output == flow output == noop node input\n```\n\nNode and flow metric sets are stacked in pipeline order. Every value\ncell is `traces / metrics / logs`.\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / 2 / 4` | `6 / 18 / 30` | `1145\n/ 3710 / 7472` |\n| Sampler input | `node.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 /\n3710 / 7472` |\n| Flow input | `flow.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 / 3710\n/ 7472` |\n| Flow output | `flow.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Sampler output | `node.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Flow decision | `flow.dropped` | — | `— / — / 20` | — |\n| Noop input | `node.input` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 / 3710\n/ 6448` |\n\nThe stacked rows show both boundary agreements and the sampler\ntransformation: `30 log items input - 20 dropped = 10 output`.\n\nDuration histograms use seconds. Each cell is `count / sum / min / max`.\n\n| Measurement | Traces | Metrics | Logs |\n| --- | ---: | ---: | ---: |\n| Receiver `node.output.duration` | `1 / 0.0002026 / 0.0002026 /\n0.0002026` | `2 / 0.0003946 / 0.0001809 / 0.0002137` | `4 / 0.0064401 /\n0.0009315 / 0.0022869` |\n| Sampler `node.input.duration` | `1 / 0.0001319 / 0.0001319 /\n0.0001319` | `2 / 0.0002355 / 0.0000921 / 0.0001434` | `4 / 0.005842 /\n0.0008909 / 0.0020428` |\n| Sampler `flow.compute.duration` | `1 / 0.000009 / 0.000009 / 0.000009`\n| `2 / 0.0000223 / 0.0000109 / 0.0000114` | `4 / 0.0053347 / 0.0008101 /\n0.0018651` |\n| Noop `node.input.duration` | `1 / 0.0000063 / 0.0000063 / 0.0000063` |\n`2 / 0.0000189 / 0.000009 / 0.0000099` | `4 / 0.0000316 / 0.0000062 /\n0.000009` |\n\nOver the course of implementation, I was confused about what\n`node.*.duration` actually represented, opened follow-up issue to\nimprove:\n- https://github.com/open-telemetry/otel-arrow/issues/3881\n\nThe `partial` pipeline uses normal runtime metrics and opts only the\nsampler into item and size measurements. Each signal cell remains\n`messages / items / size in bytes`; `—` means that instrument is\ndisabled.\n\n| Boundary | Scope | Traces | Metrics | Logs |\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / — / —` | `2 / — / —` | `4 / — /\n—` |\n| Sampler input | `node.input` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4 /\n30 / 7472` |\n| Sampler output | `node.output` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4\n/ 10 / 6448` |\n| Noop input | `node.input` | `1 / — / —` | `2 / — / —` | `4 / — / —` |\n\nThe partial pipeline emits no flow scopes or node duration histograms.\n\nThe `no_output` pipeline uses a deterministic filter that drops every\ngenerated log and ACKs without sending. A representative snapshot\ncontained 4 input messages with 40 items and 9792 logical bytes:\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `4` | `40` | `9792` |\n| Filter input | `node.input` | `4` | `40` | `9792` |\n| Flow input | `flow.input` | `4` | `40` | `9792` |\n| Flow decision | `flow.dropped` | — | `40` | — |\n| Flow output | `flow.output` | absent | absent | absent |\n| Filter output | `node.output` | absent | absent | absent |\n| Noop input | `node.input` | absent | absent | absent |\n\n`flow.compute.duration` still records the completed processor work:\n`count=4`, `sum=0.006460706 s`, `min=0.001050001 s`, and\n`max=0.002712202 s`. This demonstrates that an ACK without a send\nfinalizes flow compute and drop accounting without inventing an output\nmessage.\n\n## User-facing changes\n\nReplace:\n\n- `node.consumer.consumed.*` with `node.input.*`\n- `node.producer.produced.*` with `node.output.*`\n- `flow.consumed.*` with `flow.input.*`\n- `flow.produced.*` with `flow.output.*`\n- Flow metric configuration values based on consumed/produced\nterminology with `input_messages`, `input_items`, `input_size`,\n`output_messages`, `output_items`, and `output_size`.\n- Scope selectors targeting the previous names with the corresponding\ninput/output scopes.\n\nTreat `flow.compute.duration` values as seconds and as histogram\nobservations.",
          "timestamp": "2026-08-25T22:23:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a942658eab44e66650313ecdce8ca90a924feae7"
        },
        "date": 1787708115770,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.14527633786201477,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.82506301930093,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35402669978268,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.060026041666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.9140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 393519.08275060094,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 392947.39258940297,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004531,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12077965.246016568,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12024327.479737584,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.736850463433477,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.4916048049926758,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19788394514752,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53112264662587,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.010546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.1796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1959535.9649000305,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1949902.7913518883,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005978,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22484910.946504,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22411040.8357895,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.531298404324541,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a942658eab44e66650313ecdce8ca90a924feae7",
          "message": "fix(metrics): Simplify engine-owned node and flow metric names (#3853)\n\n# Change summary\n\nSimplify engine-owned node and flow metric names around the direction of\ndata through the pipeline:\n\n- `node.consumer.consumed.*` becomes `node.input.*`\n- `node.producer.produced.*` becomes `node.output.*`\n- `flow.consumed.*` becomes `flow.input.*`\n- `flow.produced.*` becomes `flow.output.*`\n\nAlso:\n\n- Add `messages` and logical payload `size` measurements to flow input\nand output metrics alongside `items`.\n- Report `flow.compute.duration` as a seconds-based exponential\nhistogram while retaining its processor-compute semantics.\n- Gate flow measurements with a compact interest bitmap so disabled item\nand size metrics do not inspect PData.\n- Consolidate the node-versus-flow example as\n`trafficgen-input-output-metrics.yaml`.\n- Align Rust flow metric types, fields, methods, tests, documentation,\nand configuration values with input/output terminology.\n\n## Related issue\n\n- Related to #3300\n\n### Validation\n\nThe `trafficgen-input-output-metrics.yaml` example produced the\nfollowing representative one-second delta interval. All node rows have\n`outcome=success`.\n\nThe `full` pipeline enables detailed node metrics and a one-processor\nflow around `sampler`:\n\n```text\nreceiver output\n      │\n      ├── sampler node input == flow input\n      │            30 logs in\n      │            20 logs dropped\n      │            10 logs out\n      └── sampler node output == flow output == noop node input\n```\n\nNode and flow metric sets are stacked in pipeline order. Every value\ncell is `traces / metrics / logs`.\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / 2 / 4` | `6 / 18 / 30` | `1145\n/ 3710 / 7472` |\n| Sampler input | `node.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 /\n3710 / 7472` |\n| Flow input | `flow.input` | `1 / 2 / 4` | `6 / 18 / 30` | `1145 / 3710\n/ 7472` |\n| Flow output | `flow.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Sampler output | `node.output` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 /\n3710 / 6448` |\n| Flow decision | `flow.dropped` | — | `— / — / 20` | — |\n| Noop input | `node.input` | `1 / 2 / 4` | `6 / 18 / 10` | `1145 / 3710\n/ 6448` |\n\nThe stacked rows show both boundary agreements and the sampler\ntransformation: `30 log items input - 20 dropped = 10 output`.\n\nDuration histograms use seconds. Each cell is `count / sum / min / max`.\n\n| Measurement | Traces | Metrics | Logs |\n| --- | ---: | ---: | ---: |\n| Receiver `node.output.duration` | `1 / 0.0002026 / 0.0002026 /\n0.0002026` | `2 / 0.0003946 / 0.0001809 / 0.0002137` | `4 / 0.0064401 /\n0.0009315 / 0.0022869` |\n| Sampler `node.input.duration` | `1 / 0.0001319 / 0.0001319 /\n0.0001319` | `2 / 0.0002355 / 0.0000921 / 0.0001434` | `4 / 0.005842 /\n0.0008909 / 0.0020428` |\n| Sampler `flow.compute.duration` | `1 / 0.000009 / 0.000009 / 0.000009`\n| `2 / 0.0000223 / 0.0000109 / 0.0000114` | `4 / 0.0053347 / 0.0008101 /\n0.0018651` |\n| Noop `node.input.duration` | `1 / 0.0000063 / 0.0000063 / 0.0000063` |\n`2 / 0.0000189 / 0.000009 / 0.0000099` | `4 / 0.0000316 / 0.0000062 /\n0.000009` |\n\nOver the course of implementation, I was confused about what\n`node.*.duration` actually represented, opened follow-up issue to\nimprove:\n- https://github.com/open-telemetry/otel-arrow/issues/3881\n\nThe `partial` pipeline uses normal runtime metrics and opts only the\nsampler into item and size measurements. Each signal cell remains\n`messages / items / size in bytes`; `—` means that instrument is\ndisabled.\n\n| Boundary | Scope | Traces | Metrics | Logs |\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `1 / — / —` | `2 / — / —` | `4 / — /\n—` |\n| Sampler input | `node.input` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4 /\n30 / 7472` |\n| Sampler output | `node.output` | `1 / 6 / 1145` | `2 / 18 / 3710` | `4\n/ 10 / 6448` |\n| Noop input | `node.input` | `1 / — / —` | `2 / — / —` | `4 / — / —` |\n\nThe partial pipeline emits no flow scopes or node duration histograms.\n\nThe `no_output` pipeline uses a deterministic filter that drops every\ngenerated log and ACKs without sending. A representative snapshot\ncontained 4 input messages with 40 items and 9792 logical bytes:\n\n| Pipeline boundary | Metric scope | `messages` | `items` | `size` (By)\n|\n| --- | --- | ---: | ---: | ---: |\n| Receiver output | `node.output` | `4` | `40` | `9792` |\n| Filter input | `node.input` | `4` | `40` | `9792` |\n| Flow input | `flow.input` | `4` | `40` | `9792` |\n| Flow decision | `flow.dropped` | — | `40` | — |\n| Flow output | `flow.output` | absent | absent | absent |\n| Filter output | `node.output` | absent | absent | absent |\n| Noop input | `node.input` | absent | absent | absent |\n\n`flow.compute.duration` still records the completed processor work:\n`count=4`, `sum=0.006460706 s`, `min=0.001050001 s`, and\n`max=0.002712202 s`. This demonstrates that an ACK without a send\nfinalizes flow compute and drop accounting without inventing an output\nmessage.\n\n## User-facing changes\n\nReplace:\n\n- `node.consumer.consumed.*` with `node.input.*`\n- `node.producer.produced.*` with `node.output.*`\n- `flow.consumed.*` with `flow.input.*`\n- `flow.produced.*` with `flow.output.*`\n- Flow metric configuration values based on consumed/produced\nterminology with `input_messages`, `input_items`, `input_size`,\n`output_messages`, `output_items`, and `output_size`.\n- Scope selectors targeting the previous names with the corresponding\ninput/output scopes.\n\nTreat `flow.compute.duration` values as seconds and as histogram\nobservations.",
          "timestamp": "2026-08-25T22:23:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a942658eab44e66650313ecdce8ca90a924feae7"
        },
        "date": 1787766623414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.15349814295768738,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19651583490013,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49722017220174,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.5875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528097.2207336515,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528907.8401547163,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003497,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16180864.979313608,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16151887.173613187,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.592976225461843,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.16176149249076843,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18723501749645,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42234055727555,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.04921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.59375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1957020.7138157857,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1960186.4198756316,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003044,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22605787.785341576,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22551328.50885486,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.532468318383641,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla",
            "email": "66651184+utpilla@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a7f8706e54b01c778a8d7bbbf98f92b53aa4bc09",
          "message": "Return client-error status codes for permanent NACKs at the OTLP receivers (#3885)\n\n# Change summary\n\n### What\nWhen the pipeline permanently rejects a request because of the request\nitself (for example, telemetry that fails the Resource Validator\nProcessor with a missing or disallowed resource attribute), the OTLP\nreceivers now return a non-retryable client error (`400 Bad Request` /\ngRPC `INVALID_ARGUMENT`) instead of a retryable `503` / `UNAVAILABLE`.\n\n### Why\nA permanent NACK means \"do not retry; fix your config.\" Previously the\nOTLP/HTTP receiver returned 503 for every NACK, so clients kept\nresending data that could never succeed. gRPC returned `INTERNAL`, which\nis non-retryable but blames the server. Neither could express \"the\nclient sent bad data.\"\n\n## Related issue\n\n* Closes #3826 \n\n## Validation\n\n- Unit tests\n\n## User-facing changes\n\n| Failure | Before | After |\n| --- | --- | --- |\n| Client-caused permanent rejection | `503` / `UNAVAILABLE` (HTTP),\n`INTERNAL` (gRPC) | `400` / `INVALID_ARGUMENT` |\n| Permanent server-side failure | `503` / `INTERNAL` | `500` /\n`INTERNAL` |\n| Transient failure | `503` / `UNAVAILABLE` | `503` / `UNAVAILABLE`\n(unchanged) |\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <utpilla@users.noreply.github.com>",
          "timestamp": "2026-08-27T00:58:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7f8706e54b01c778a8d7bbbf98f92b53aa4bc09"
        },
        "date": 1787799408488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.24164381623268127,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21249472736224,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50903685351503,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.852604166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.4921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526134.9151451155,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527406.2876878348,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004442,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16095779.126623858,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16057004.983382154,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.518747126031133,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0763815641403198,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21336129856773,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59762376237624,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.093359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.37890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1929365.5536794565,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1950132.887922256,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008087,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22485431.32760586,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22412315.67694783,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.530204668032994,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "51d89ded5ad4a169eaa4ba422368e0913f74a738",
          "message": "fix(azure_monitor_exporter): Return permanant NACK for an empty batch (#3903)\n\n# Change summary\n\nMinor follow-up from #3891\n\nTechnically there is no reason to return a retryable NACK from the\nexporter if there is an empty payload detected. While we are fixing this\nspecific issue upstream, it is best for this component to also handle it\ncorrectly.\n\n## Related issue\n\n* Related to #3891\n\n## Validation\n\nUnit test\n\n## User-facing changes\n\nAzure Monitor exporter now permanently rejects empty batches instead of\nallowing them to be retried.",
          "timestamp": "2026-08-27T17:31:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/51d89ded5ad4a169eaa4ba422368e0913f74a738"
        },
        "date": 1787863976600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3822407722473145,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.41410690769001,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45984368954576,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.453645833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.6171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 492007.7030195992,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 485206.97219568974,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002963,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14800525.833957424,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14762784.90049132,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.503530827228502,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.9857559204101562,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20962010567622,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55073684210527,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.04348958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.2421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1945543.3513992946,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1906909.6097969369,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008063,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22368140.99880471,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22294088.462129634,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.730047865869558,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "80d38834e747127ee02559e8c6dafd792193bdb2",
          "message": "chore(repo): Isolate test-only nodes in `dev-nodes` crate to minimize dependency chain (#3912)\n\n# Chore summary\n\nMoves the following nodes into a new `dev-nodes` crate:\n- `traffic_generator_receiver`\n- `delay_processor`\n- `error_exporter`\n- `perf_exporter`\n\nAs #3909 mentions, we do this to unblock publishing of `core-nodes`\ncrate on `crates.io` because there is currently a git Weaver dependency\nwith no published crate. We do not plan to publish `dev-nodes`\nexternally.\n\n## Related issue\n\n- Closes #3909",
          "timestamp": "2026-08-28T01:54:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/80d38834e747127ee02559e8c6dafd792193bdb2"
        },
        "date": 1787890702563,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.1134558692574501,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22824161381712,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4815265061168,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.996223958333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.2734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526441.3779697779,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527038.6565923895,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005496,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16126444.769994892,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16088494.153247349,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.59822001342693,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.4421733319759369,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18701297105916,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51051838178742,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.66236979166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.02734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1958639.5574424213,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1949978.975991901,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005209,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22504888.516245443,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22443168.072562933,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.54109290065439,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2edcaaf8e6a6e3c85f52abdbace9a7f39f774be5",
          "message": "feat(kafka): replay transiently nacked records (#3928)\n\n# Change summary\n\nAdds reliable transient-NACK replay to the Kafka receiver:\n\n- Manual-commit mode now replays transiently NACKed records by default.\n- Replay pauses only the affected partition and uses capped backoff.\n- Rebalance handling prevents partitions from remaining paused after\nreassignment, including retrying failed resume operations.\n- Replay logic and integration tests are isolated in dedicated\nsubmodules.\n- Receiver instrumentation and documentation are updated, including\nplanned future DLQ support.\n- End-to-end coverage verifies retry-processor exhaustion falls back to\nKafka replay.\n\n## Related issue\n\n- Related to #3505\n\n## Validation\n\n- End-to-end Kafka receiver → retry processor → exporter replay test\npassed.\n- Rebalance, reassignment, ingress-drain, and shutdown tests passed.\n\n## User-facing changes\n\nThis is a breaking behavioral change for manual-commit configurations:\ntransient NACKs now preserve the offset and replay the record by\ndefault.\n\nTo retain the previous offset-advancing behavior, configure:\n\n```yaml\ntransient_nack:\n  mode: commit_and_skip\n```\n\nA changelog entry is included.",
          "timestamp": "2026-08-28T20:40:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2edcaaf8e6a6e3c85f52abdbace9a7f39f774be5"
        },
        "date": 1787951992500,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.09075043350458145,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19187786313593,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55547609913297,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.356770833333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.03515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1955741.919542735,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1953967.0752178445,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003009,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22572458.996920224,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22515653.622916583,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.55211839708387,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.03131624683737755,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.9268955144498,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.4920264656101,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.97265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 354218.3175993847,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 354329.24547876394,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00295,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10896063.019081537,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10848249.939167093,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.75123817216655,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a3d3101072d2b7e0e0bdbca175ec981854aff015",
          "message": "fix(quiver): Fix semantics of loss metrics and introduce contrasting reclaimed metrics (#3898)\n\n# Change summary\n\n- report durable-buffer retention loss from only the bundles that remain\nunresolved when a segment is removed\n- persist optional logical byte counts per bundle so loss bytes reflect\nthe original OTAP or OTLP payload rather than the containing segment\nfile\n- separate physical storage reclamation into\n`processor.durable_buffer.reclaimed`\n- restore subscriber progress before startup expiry accounting and\nreport deferred file deletion only after removal is confirmed\n\n### Details\n\nQuiver stores multiple bundles in each physical segment. Previously, if\none unresolved bundle kept a segment until retention expiry, the entire\nsegment was reported as lost—including bundles that downstream exporters\nhad already ACKed.\n\n```text\nsegment selected by retention\n├── bundle 0: unresolved  -> logical loss\n└── bundle 1: ACKed       -> excluded from logical loss\n\nphysical segment removal  -> reclaimed storage\n```\n\nRetention now snapshots the union of unresolved bundle indices across\nsubscribers while force-completing the segment under the same\nsubscriber-state lock. Each stored bundle is counted once when any\nsubscriber still needs it, avoiding both ACK races and per-subscriber\ndouble counting.\n\nThe segment manifest now includes an optional nullable `byte_count` for\neach bundle:\n\n- OTAP records use their active-range Arrow logical size\n- OTLP pass-through records use encoded protobuf wire length\n- known zero remains distinct from an unavailable count\n- loss bytes are emitted only when the selected unresolved bundles all\nhave exact counts\n\nThe manifest change is additive. New readers treat a missing\n`byte_count` column as unavailable, while existing readers continue\nlocating known fields by name and ignore the additional column.\n\n## Related issue\n\n* Closes #3892 \n\n## Validation\n\nRegression coverage exercises:\n\n- a partially ACKed segment, verifying only its unresolved bundle\ncontributes bundles, items, and logical bytes to expiry loss\n- a fully ACKed later segment retained behind an incomplete predecessor,\nverifying resolved data is excluded from expiry loss\n- multiple subscribers with overlapping unresolved bundles, verifying\nunion semantics count each stored bundle once\n- startup expiry after subscriber progress restoration, verifying\npersisted ACKs remain excluded\n- immediate, deferred, retried, and abandoned physical deletion,\nverifying reclaimed counters advance only after confirmed removal\n- nullable manifest byte counts, including known zero, known nonzero,\nunavailable counts, and legacy manifests without the new column\n- canonical OTAP active-range and OTLP protobuf-wire byte measurements\n\nA standalone mixed-segment repro sends one log bundle and one metric\nbundle through the same durable buffer, ACKs one downstream, transiently\nNACKs the other, and waits for the segment to expire.\n\n### Single-item batches\n\n| Unresolved bundle | `reclaimed.segments` | `reclaimed.bytes` |\n`loss.bundles` | `loss.bytes` | `loss.items` |\n| --- | ---: | ---: | ---: | ---: | ---: |\n| 1 log | 1 `reason=expired` | 7,344 `reason=expired` | 1\n`reason=expired` | 359 `reason=expired` | 1 `signal=logs,\nreason=expired` |\n| 1 metric datapoint | 1 `reason=expired` | 7,344 `reason=expired` | 1\n`reason=expired` | 329 `reason=expired` | 1 `signal=metrics,\nreason=expired` |\n\n### 100-item batches\n\n| Unresolved bundle | `reclaimed.segments` | `reclaimed.bytes` |\n`loss.bundles` | `loss.bytes` | `loss.items` |\n| --- | ---: | ---: | ---: | ---: | ---: |\n| 100 logs | 1 `reason=expired` | 57,840 `reason=expired` | 1\n`reason=expired` | 24,248 `reason=expired` | 100 `signal=logs,\nreason=expired` |\n| 100 metric datapoints | 1 `reason=expired` | 57,840 `reason=expired` |\n1 `reason=expired` | 26,952 `reason=expired` | 100 `signal=metrics,\nreason=expired` |\n\nFor the 100-item run, the segment contained 51,200 logical payload bytes\nand 6,640 bytes of Arrow IPC and Quiver segment overhead:\n\n```text\n57,840 physical bytes reclaimed\n├── 24,248 logical log bytes\n├── 26,952 logical metric bytes\n└──  6,640 segment encoding overhead\n```\n\nIn every permutation, the physical segment size remained constant while\n`loss.bytes` changed to match only the unresolved bundle. The ACKed\nneighboring bundle was excluded from logical loss.\n\n## User-facing changes\n\n| Previous metric | Replacement | Meaning |\n| --- | --- | --- |\n| `processor.durable_buffer.loss.segments` |\n`processor.durable_buffer.reclaimed.segments` | Physical segment files\nremoved |\n| `processor.durable_buffer.loss.bytes` |\n`processor.durable_buffer.reclaimed.bytes` | Physical persisted bytes\nremoved |\n| N/A | `processor.durable_buffer.loss.bytes` | Logical bytes in\nunresolved bundles |\n| `processor.durable_buffer.loss.bundles` | unchanged | Unresolved\nbundles removed by retention |\n| `processor.durable_buffer.loss.items` | unchanged | Items in\nunresolved bundles, partitioned by signal |\n\nPhysical reclamation advances only after a segment file is confirmed\nremoved. Deferred or abandoned deletion attempts are not reported as\nreclaimed.",
          "timestamp": "2026-08-28T23:29:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a3d3101072d2b7e0e0bdbca175ec981854aff015"
        },
        "date": 1787968184376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.0744740217924118,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 80.91812992037933,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.32410439092386,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.674479166666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.63671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 366643.9571213074,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 366917.01164159883,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002669,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11228404.245456047,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11178876.211067524,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.602026859479192,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.08878808468580246,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19509870717414,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.50001086197533,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.54127604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.20703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1941303.2082823934,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1943026.8542556034,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003041,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22463104.53475748,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22381857.467525057,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.56088218006815,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788024540992,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.0051112174987793,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19337720474498,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.49099690402477,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.33997395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.37109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1949817.6423543692,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1910721.629658244,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00571,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22449001.70883615,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22381413.179622244,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.748965082292719,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5408799052238464,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19184819989067,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48996822936846,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.285677083333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530057.6876487623,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527190.7121807244,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004699,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16144036.897322156,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16113592.911698574,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.622764256491447,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788054945467,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5369604825973511,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19997915226926,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51739648634006,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.483333333333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.16015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1919650.1291086865,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1929957.8917257611,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002934,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22272743.997408386,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22194449.04850286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.540533652520358,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.33394935727119446,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16533036718616,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3284066318985,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.514973958333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.1015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526364.6715920559,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528122.4630252492,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002568,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16121811.69591358,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16086976.897218166,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.526653995292765,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788110975479,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.20579923689365387,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.17571885204617,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58083410565337,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.78958333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1944581.365389867,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1940579.4317530221,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002994,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22395511.78463929,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22339289.5492361,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.540631328040156,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4656656980514526,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16612831295933,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4958202247191,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.384244791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525063.9147485411,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 517368.2329198194,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010797,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16106319.020002265,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16068023.28977684,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.13124849027672,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3065676427079b08049c6167d0d7fd410ff25743",
          "message": "fix(otlp_http_exporter): enhance error handling for HTTP status responses (#3902)\n\n# Change summary\nThe OTLP/HTTP exporter `usesreqwest::Response::error_for_status()` to\ndetect non-2xx responses. This call discards the response body, so any\nexplanation the backend sent back (missing/invalid field, rejection\nreason, rate-limit detail, etc.) was silently lost before it ever\nreached logs or NACK reasons.\n\nI would like to change it to include an explicit status check and add\n`RpcStatus` or falls back to raw `UTF-8` text.\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\nNone\n\n## Validation\nValidated locally while debugging issues like `400 Bad Request`, `503\nServer Unavailable` etc.\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->",
          "timestamp": "2026-08-29T06:01:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3065676427079b08049c6167d0d7fd410ff25743"
        },
        "date": 1788141364849,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.17978911101818085,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18045005275708,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48457000931388,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.60390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.28515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526817.8144544926,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527764.9755261726,
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
            "value": 16147897.418580867,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16108305.491641628,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.59675834396114,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.3498027324676514,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19771546516098,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48321657774686,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.325390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.19921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1920885.2816821258,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1946813.444862277,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.01073,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22463312.71517346,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22398581.883428954,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.538502969791528,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "renovate[bot]",
            "username": "renovate[bot]",
            "email": "29139614+renovate[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "e8a40bb244fe579134cbe0ed091c8890bb5a6097",
          "message": "chore(deps): update geneva-uploader digest to 91057e4 (#3948)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `43bd04b` →\n`91057e4` |\n\n---\n\n> [!WARNING]\n> Some dependencies could not be looked up. Check the [Dependency\nDashboard](../issues/417) for more information.\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC40OS4wIiwidXBkYXRlZEluVmVyIjoiNDQuNDkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-31T14:29:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8a40bb244fe579134cbe0ed091c8890bb5a6097"
        },
        "date": 1788197445254,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.232326626777649,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.47832675573295,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45535982647765,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.156901041666668,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.87890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 399476.53329921496,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 394553.67738290114,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010694,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12126146.243430551,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12078461.5832359,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.733831512771662,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.8888888359069824,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19278948991305,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48565762164657,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.206380208333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.18359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1965749.688527338,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1928618.861077377,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010082,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22650621.729118943,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22586011.648119196,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.744477971383994,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "801f2bbdff14bf0ceea000d8314a434d371cbe7c",
          "message": "chore(repo): Rename rust/experimental to rust/contrib (#3953)\n\n# Chore Summary\n\nUse proper terminology now that we released\n`otel-arrow-contrib-data-engine` crates.\n\n## Related issue\n\nN/A",
          "timestamp": "2026-08-31T19:34:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/801f2bbdff14bf0ceea000d8314a434d371cbe7c"
        },
        "date": 1788227939115,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1558077335357666,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22022796123376,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55486568088565,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.76588541666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1903816.3579222362,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1925820.8143792595,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008208,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22282446.66418656,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22212288.490225073,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.570363399239069,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.5869696140289307,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 79.29623829787974,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.67702098010375,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.83125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.40625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 367223.10550962243,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 361395.38626005064,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005636,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11274372.98248575,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11224053.127577344,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.19678172751494,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "uros-stefanovic-db",
            "username": "uros-stefanovic-db",
            "email": "uros.stefanovic@databricks.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "408dd1982aa0e936e98ec135a173e71564721ed7",
          "message": "Avoid quadratic resource/scope lookups in OTAP trace view (#3936)\n\n`OtapTracesView` looks up the first spans-batch row for each resource\nand scope (to read the resource and scope name, version, and\ndropped-attribute counts) by scanning `resource_groups` and\n`scope_groups_map` on every call. Those accessors run once per resource\nand once per scope, so on a batch with G distinct resources and S\ndistinct scopes the repeated scans add up to O(G^2) and O(S^2).\n\nEach `OtapResourceSpansView` / `OtapScopeSpansView` is already handed\nthe `RowGroup` for its id when the view is built. The fix carries that\ngroup's first row onto the resource/scope view and reads it directly in\nthe accessors, so each lookup is O(1) with no extra maps and no derived\nstate to keep in sync — the grouping the view already does stays the\nonly pass over the rows.\n\nThis should not change any behavior. Resource and scope ids are assigned\nmonotonically per batch, so each id maps to exactly one group, and that\ngroup's first row is the same row the previous scan resolved. The\nexisting `views::otap::traces` unit tests already exercise\nmulti-resource and multi-scope batches.\n\nI only touched the traces view here. `metrics.rs` has the same pattern,\nso I can follow up with the same change there if that's useful.\n\n---------\n\nSigned-off-by: Uros Stefanovic <uros.stefanovic@databricks.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-01T15:52:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/408dd1982aa0e936e98ec135a173e71564721ed7"
        },
        "date": 1788285490317,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.17240166664123535,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18361949304548,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.62613861386139,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.3921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.12890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1940181.1658861602,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1943526.070436453,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002908,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22406464.178209025,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22339903.412982292,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.528769548832067,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.4520766735076904,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18349394493387,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45656671548602,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.54453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.95703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534148.2614066715,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 521050.5364664281,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004314,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16226155.644404484,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16193701.751994573,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.14123200878799,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "otelbot-arrow[bot]",
            "username": "otelbot-arrow[bot]",
            "email": "289780372+otelbot-arrow[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c5bda0a3cb6700ee1e455dea29ef229eb16c9fcf",
          "message": "chore(release) Prepare Release v0.54.0 (#3970)\n\n## Release v0.54.0\n\nThis PR prepares the repository for release v0.54.0.\n\n### Changes included:\n- Rendered pending chloggen entries into `go/CHANGELOG.md` and\n`rust/otap-dataflow/CHANGELOG.md`\n- Bumped `rust/otap-dataflow/Cargo.toml` (workspace + root package)\n\n## What's Changed (Go :hamster:)\n\nNo changes. This release maintains version parity across the repository.\n\n## What's Changed (Rust :crab:)\n\n### :stop_sign: Breaking changes :stop_sign:\n\n- `pipeline`: Migrated temporal reaggregation processor telemetry from\nthree flat counters to three dimensioned metric populations:\n`operations`, `failures{error.type}`, and `flushes{outcome,reason}`.\n([#3530](https://github.com/open-telemetry/otel-arrow/issues/3530))\n\n### :bulb: Enhancements :bulb:\n\n- `dependencies`: Upgrade various Rust dependencies.\n([#3947](https://github.com/open-telemetry/otel-arrow/issues/3947))\n- `engine`: Publish the telemetry, state, engine, admin, and controller\ncrates as versioned crates.io packages.\n([#1340](https://github.com/open-telemetry/otel-arrow/issues/1340))\n- `otap`: Avoid quadratic resource and scope scans in the OTAP trace\nview by resolving each resource's and scope's representative row from\nthe pre-computed row group\n([#3936](https://github.com/open-telemetry/otel-arrow/issues/3936))\n- `otap`: Avoid quadratic resource and scope scans in the OTAP metrics\nview by resolving each resource's and scope's representative row from\nthe pre-computed row group\n([#3964](https://github.com/open-telemetry/otel-arrow/issues/3964))\n\n### Checklist:\n- [ ] Verify both CHANGELOG.md files render the expected entries\n- [ ] Verify Rust crate versions updated\n- [ ] Confirm all tests pass\n- [ ] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot-arrow[bot] <289780372+otelbot-arrow[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-01T22:41:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c5bda0a3cb6700ee1e455dea29ef229eb16c9fcf"
        },
        "date": 1788313990528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5750324726104736,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19419397071523,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58226906385616,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.280208333333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1932037.7337120567,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1943147.5778495215,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003002,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22394340.60599027,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22332131.331641,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.524776018697484,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.419701099395752,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18790722568416,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.46969659442723,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.958333333333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.68359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537299.5735053715,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 529671.5253766488,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005914,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16198996.414411133,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16160509.035568004,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.58309846445313,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "renovate[bot]",
            "username": "renovate[bot]",
            "email": "29139614+renovate[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "11b92ef5b8a51aea55be14a84c9232760a6f5095",
          "message": "fix(deps): update module google.golang.org/grpc to v1.83.1 [security] (#3974)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [google.golang.org/grpc](https://redirect.github.com/grpc/grpc-go) |\n`v1.82.1` → `v1.83.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/google.golang.org%2fgrpc/v1.83.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/google.golang.org%2fgrpc/v1.82.1/v1.83.1?slim=true)\n|\n\n---\n\n> [!WARNING]\n> Some dependencies could not be looked up. Check the [Dependency\nDashboard](../issues/417) for more information.\n\n---\n\n### gRPC-Go: Heap Memory Exhaustion (OOM) via HTTP/2 DATA Frame\nFragmentation\n[CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304) /\n[GHSA-vp52-pcj8-j9qc](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n\n<details>\n<summary>More information</summary>\n\n#### Details\n##### Impact\nAn unauthenticated remote attacker can initiate a gRPC stream and\npurposefully fragment their payload into millions of tiny (e.g., 1-byte)\nHTTP/2 DATA frames. Even if the total payload volume falls within the\nconfigured connection and stream flow-control windows, each independent\nfragment incurs memory overhead due to internal tracking structures and\nqueue allocation.\n\nRepeated fragmentation massively inflates the heap space consumed by the\nstream. An attacker multiplexing multiple concurrent streams can exhaust\nthe memory bounds of the runtime, forcing a runtime panic or OutOfMemory\ncondition and leading to a remote Denial of Service (DoS).\n\n##### Patches\nThe change to fix this issue is merged in `master` and a patch release,\n1.83.1, has been published that contains this fix.\n\n##### Workarounds\nThis vulnerability is mitigated by implementing receive buffer\ncompaction. Consecutive small data buffers are automatically coalesced\ninto larger buffers from a shared pool once the overhead is perceived to\nbe excessive relative to actual payload data, drastically minimizing\nper-frame memory overheads.\n\nThis behavior is enabled by default. A temporary escape hatch is\nprovided via the environment variable\n`GRPC_GO_EXPERIMENTAL_ENABLE_RECEIVE_BUFFER_COMPACTION=false` to disable\nthe feature if unforeseen issues arise, but it will be removed in a\nfuture release.\n\n#### Severity\n- CVSS Score: 8.7 / 10 (High)\n- Vector String:\n`CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:N/VA:H/SC:N/SI:N/SA:N`\n\n#### References\n-\n[https://github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc](https://redirect.github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc)\n-\n[https://nvd.nist.gov/vuln/detail/CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304)\n-\n[https://github.com/grpc/grpc-go/pull/9331](https://redirect.github.com/grpc/grpc-go/pull/9331)\n-\n[https://github.com/grpc/grpc-go/pull/9333](https://redirect.github.com/grpc/grpc-go/pull/9333)\n-\n[https://github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176](https://redirect.github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176)\n-\n[https://github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77](https://redirect.github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77)\n-\n[https://github.com/grpc/grpc-go/releases/tag/v1.83.1](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.1)\n-\n[https://github.com/advisories/GHSA-vp52-pcj8-j9qc](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n\nThis data is provided by the [GitHub Advisory\nDatabase](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n([CC-BY\n4.0](https://redirect.github.com/github/advisory-database/blob/main/LICENSE.md)).\n</details>\n\n---\n\n### gRPC-Go: Heap Memory Exhaustion (OOM) via HTTP/2 DATA Frame\nFragmentation\n[CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304) /\n[GHSA-vp52-pcj8-j9qc](https://redirect.github.com/advisories/GHSA-vp52-pcj8-j9qc)\n\n<details>\n<summary>More information</summary>\n\n#### Details\n##### Impact\nAn unauthenticated remote attacker can initiate a gRPC stream and\npurposefully fragment their payload into millions of tiny (e.g., 1-byte)\nHTTP/2 DATA frames. Even if the total payload volume falls within the\nconfigured connection and stream flow-control windows, each independent\nfragment incurs memory overhead due to internal tracking structures and\nqueue allocation.\n\nRepeated fragmentation massively inflates the heap space consumed by the\nstream. An attacker multiplexing multiple concurrent streams can exhaust\nthe memory bounds of the runtime, forcing a runtime panic or OutOfMemory\ncondition and leading to a remote Denial of Service (DoS).\n\n##### Patches\nThe change to fix this issue is merged in `master` and a patch release,\n1.83.1, has been published that contains this fix.\n\n##### Workarounds\nThis vulnerability is mitigated by implementing receive buffer\ncompaction. Consecutive small data buffers are automatically coalesced\ninto larger buffers from a shared pool once the overhead is perceived to\nbe excessive relative to actual payload data, drastically minimizing\nper-frame memory overheads.\n\nThis behavior is enabled by default. A temporary escape hatch is\nprovided via the environment variable\n`GRPC_GO_EXPERIMENTAL_ENABLE_RECEIVE_BUFFER_COMPACTION=false` to disable\nthe feature if unforeseen issues arise, but it will be removed in a\nfuture release.\n\n#### Severity\n- CVSS Score: 8.7 / 10 (High)\n- Vector String:\n`CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:N/VI:N/VA:H/SC:N/SI:N/SA:N`\n\n#### References\n-\n[https://github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc](https://redirect.github.com/grpc/grpc-go/security/advisories/GHSA-vp52-pcj8-j9qc)\n-\n[https://nvd.nist.gov/vuln/detail/CVE-2026-84304](https://nvd.nist.gov/vuln/detail/CVE-2026-84304)\n-\n[https://github.com/grpc/grpc-go/pull/9331](https://redirect.github.com/grpc/grpc-go/pull/9331)\n-\n[https://github.com/grpc/grpc-go/pull/9333](https://redirect.github.com/grpc/grpc-go/pull/9333)\n-\n[https://github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176](https://redirect.github.com/grpc/grpc-go/commit/7354d9c8debb4bcf2225bf429857078de310c176)\n-\n[https://github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77](https://redirect.github.com/grpc/grpc-go/commit/8cfeca0e1ee5ea0980dcc320e20240fa1079ec77)\n-\n[https://github.com/grpc/grpc-go](https://redirect.github.com/grpc/grpc-go)\n-\n[https://github.com/grpc/grpc-go/releases/tag/v1.83.1](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.1)\n\nThis data is provided by\n[OSV](https://osv.dev/vulnerability/GHSA-vp52-pcj8-j9qc) and the [GitHub\nAdvisory Database](https://redirect.github.com/github/advisory-database)\n([CC-BY\n4.0](https://redirect.github.com/github/advisory-database/blob/main/LICENSE.md)).\n</details>\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc-go (google.golang.org/grpc)</summary>\n\n###\n[`v1.83.1`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.1):\nRelease 1.83.1\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.83.0...v1.83.1)\n\n### Security\n\n- xds/rbac: Fix a bug where nested `Principal` or `Permission` rules\nwith `:scheme` or `grpc-` prefixed header matchers were not rejected,\nwhich could cause DENY rules to fail open.\n([#&#8203;9258](https://redirect.github.com/grpc/grpc-go/issues/9258))\n  - Special Thanks: [@&#8203;nvxbug](https://redirect.github.com/nvxbug)\n- xds/rbac: Fix a bug where the `host` header matcher was not being\nreplaced with `:authority` in nested `Principal` or `Permission` rules.\n([#&#8203;9258](https://redirect.github.com/grpc/grpc-go/issues/9258))\n  - Special Thanks: [@&#8203;nvxbug](https://redirect.github.com/nvxbug)\n- xds/rbac: Fix a bug where a header matcher whose name was not\nlowercase, such as `X-Role`, matched no header, which could cause DENY\nrules to fail open.\n([#&#8203;9332](https://redirect.github.com/grpc/grpc-go/issues/9332))\n- Special Thanks: [@&#8203;alimony](https://redirect.github.com/alimony)\n- xds/rbac: Fix a bug where a `:scheme` or `grpc-` prefixed header\nmatcher was accepted when its name was not lowercase.\n([#&#8203;9332](https://redirect.github.com/grpc/grpc-go/issues/9332))\n- Special Thanks: [@&#8203;alimony](https://redirect.github.com/alimony)\n- xds/rbac: Fix a bug where a `Host` header matcher was not replaced\nwith `:authority`.\n([#&#8203;9332](https://redirect.github.com/grpc/grpc-go/issues/9332))\n- Special Thanks: [@&#8203;alimony](https://redirect.github.com/alimony)\n\n### Performance\n\n- transport: Restrict memory overhead of buffering small data frames.\n([#&#8203;9331](https://redirect.github.com/grpc/grpc-go/issues/9331))\n\n###\n[`v1.83.0`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.83.0):\nRelease 1.83.0\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.82.2...v1.83.0)\n\n### Security\n\n- server: Stop reading from connections when flooded by HTTP/2 frames to\nmitigate resource exhaustion. The default value for this limit is 100\nframes, excluding DATA and HEADERS, and may be changed by setting\nenvironment variable\n`GRPC_GO_EXPERIMENTAL_CONTROL_BUFFER_THROTTLE_LIMIT`.\n- xds/rbac: Support `Metadata` and `RequestedServerName` permissions\nmatcher fields. If present in a DENY rule, previously these would be\nignored and fail-open.\n- xds/rbac: Fix panic when parsing unsupported fields in\n`NotRule`/`NotId` permissions.\n- xds/rbac: Support the deprecated `source_ip` principal identifier by\ntreating it as equivalent to `direct_remote_ip`.\n- xds: Fix panic when parsing route header matchers configured with\nempty `exact_match`, `prefix_match`, or `suffix_match` strings.\n([#&#8203;9223](https://redirect.github.com/grpc/grpc-go/issues/9223))\n\n### New Features\n\n- xds/googlec2p: Enable DirectPath over Interconnect support for\non-premises clients via the `force-xds` target URI query parameter.\n([#&#8203;9133](https://redirect.github.com/grpc/grpc-go/issues/9133))\n- xds: Enable xDS configuration to control which fields get propagated\nfrom ORCA backend metric reports to LRS load reports.\n([#&#8203;9145](https://redirect.github.com/grpc/grpc-go/issues/9145))\n- authz: Add `OnPolicyUpdate` callback to `FileWatcherOptions` to notify\nwhen an authz policy is loaded or updated.\n([#&#8203;9142](https://redirect.github.com/grpc/grpc-go/issues/9142))\n- Special Thanks: [@&#8203;hnefatl](https://redirect.github.com/hnefatl)\n- xds: Add support for the GCP Authentication HTTP Filter, which\nautomatically fetches and attaches GCP Service Account Identity JWT\ntokens to outgoing RPCs.\n- This feature can be enabled by setting environment variable\n`GRPC_EXPERIMENTAL_XDS_GCP_AUTHENTICATION_FILTER=true`.\n([#&#8203;9119](https://redirect.github.com/grpc/grpc-go/issues/9119))\n- xds: Add support for xDS-based HTTP CONNECT proxies.\n- This feature can be enabled by setting environment variable\n`GRPC_EXPERIMENTAL_XDS_HTTP_CONNECT=true`.\n([#&#8203;9151](https://redirect.github.com/grpc/grpc-go/issues/9151))\n- xds: Add support for `contains_match` in route header matchers.\n([#&#8203;9223](https://redirect.github.com/grpc/grpc-go/issues/9223))\n\n### Bug Fixes\n\n- credentials/alts: Fix panic when processing malformed frames by\nvalidating that the message frame length exceeds the message type field\nsize.\n([#&#8203;9197](https://redirect.github.com/grpc/grpc-go/issues/9197))\n- grpc: Fix compilation on Plan 9 targets (`GOOS=plan9`), broken since\nv1.81.0.\n([#&#8203;9255](https://redirect.github.com/grpc/grpc-go/issues/9255))\n- Special Thanks:\n[@&#8203;Yusufihsangorgel](https://redirect.github.com/Yusufihsangorgel)\n\n###\n[`v1.82.2`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.82.2):\nRelease 1.82.2\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.82.1...v1.82.2)\n\n### Security\n\n- server: Reject requests missing both `:authority` and `Host` headers\nwith HTTP 400 and status `Internal`.\n([#&#8203;9365](https://redirect.github.com/grpc/grpc-go/pull/9365))\n- Special Thanks:\n[@&#8203;winklemad](https://redirect.github.com/winklemad)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC40OS4wIiwidXBkYXRlZEluVmVyIjoiNDQuNTcuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiYXJlYTpzZWN1cml0eSIsImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-02T16:29:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/11b92ef5b8a51aea55be14a84c9232760a6f5095"
        },
        "date": 1788370411588,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.19706544280052185,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1924110211124,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58644356650399,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.83984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.94921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1952758.9026374286,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1956607.115350506,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004999,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22594121.0371449,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22520519.034923077,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.547602408211317,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.11189594119787216,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.26672885838165,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 79.27468659970411,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.811979166666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.28125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 366036.34220711776,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 366445.9220011813,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00296,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11231672.67110461,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11183444.318974826,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.650286977592298,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "albertlockett",
            "username": "albertlockett",
            "email": "a.lockett@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "67653eee12a09b7eadce90ea8d40302916ffff8c",
          "message": "fix: inserting new attributes when no previous attributes did exist (#3982)\n\n# Change summary\n\nFixes two related issues related to inserting new attributes when none\ndid previously exist.\n\nThe first issue was that, when using the transform processor, if writing\na program like:\n```\nlogs | set attributes[\"x\"] = \"y\"\n```\nIf some input batch had no existing attribute record batch, a new one\nwould be created but we wouldn't set the schema metadata to indicate\nthat parent_id the column was not delta encoded. So when we decoded that\nto OTLP, we'd treat it as a delta encoded column and the net effect\nwould be that some attributes would end up missing from the result.\n\nThe second issue is somewhat similar - when inserting attributes using\nthe attribute processor we'd create a new attribute record batch, this\ntime with the correct parent_id encoding, but we'd also create a new ID\ncolumn for the root batch and this did not have indicator that said it\nwas not delta encoded.\n\nThis PR \n- corrects the behaviour to add the proper encoding metadata to the\nid/parent_id columns\n- updates tests so that our tests cover regressions agains these kind of\nerrors\n- adds a check in the upsert attributes helper function to ensure it is\npassed an attribute record batch with plain encoded parent_ids.\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #3985\n\n## Validation\n\nUnit tests\n\n<!--How did you confirm your change has the intended effect?-->\n\n## User-facing changes\n\nno\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-09-03T00:46:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67653eee12a09b7eadce90ea8d40302916ffff8c"
        },
        "date": 1788400422197,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.375842809677124,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1848890472049,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59869450889406,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.207942708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.77734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1956244.026944341,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1909766.7419794276,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004882,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22410399.19614801,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22331977.006831337,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.734626383178172,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.15284369885921478,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1899202451765,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54816744186046,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.858984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.4765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530363.327851491,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531173.9547704178,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002942,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16275885.752734428,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16243418.251164688,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.64134754078659,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "albertlockett",
            "username": "albertlockett",
            "email": "a.lockett@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "67653eee12a09b7eadce90ea8d40302916ffff8c",
          "message": "fix: inserting new attributes when no previous attributes did exist (#3982)\n\n# Change summary\n\nFixes two related issues related to inserting new attributes when none\ndid previously exist.\n\nThe first issue was that, when using the transform processor, if writing\na program like:\n```\nlogs | set attributes[\"x\"] = \"y\"\n```\nIf some input batch had no existing attribute record batch, a new one\nwould be created but we wouldn't set the schema metadata to indicate\nthat parent_id the column was not delta encoded. So when we decoded that\nto OTLP, we'd treat it as a delta encoded column and the net effect\nwould be that some attributes would end up missing from the result.\n\nThe second issue is somewhat similar - when inserting attributes using\nthe attribute processor we'd create a new attribute record batch, this\ntime with the correct parent_id encoding, but we'd also create a new ID\ncolumn for the root batch and this did not have indicator that said it\nwas not delta encoded.\n\nThis PR \n- corrects the behaviour to add the proper encoding metadata to the\nid/parent_id columns\n- updates tests so that our tests cover regressions agains these kind of\nerrors\n- adds a check in the upsert attributes helper function to ensure it is\npassed an attribute record batch with plain encoded parent_ids.\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #3985\n\n## Validation\n\nUnit tests\n\n<!--How did you confirm your change has the intended effect?-->\n\n## User-facing changes\n\nno\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-09-03T00:46:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67653eee12a09b7eadce90ea8d40302916ffff8c"
        },
        "date": 1788468306165,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.0802435651421547,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.04651873034904,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.26854929904732,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.158072916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.75390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 361540.40217100707,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 361830.51507072186,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004226,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11075355.887671398,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11021218.504723853,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.60923671820951,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6392216682434082,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21346047382514,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.55070287044221,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.81236979166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.4609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1940074.114896689,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1908271.9992309725,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003052,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22397561.205583457,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22330922.44713869,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.737090527246432,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "albertlockett",
            "username": "albertlockett",
            "email": "a.lockett@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "67653eee12a09b7eadce90ea8d40302916ffff8c",
          "message": "fix: inserting new attributes when no previous attributes did exist (#3982)\n\n# Change summary\n\nFixes two related issues related to inserting new attributes when none\ndid previously exist.\n\nThe first issue was that, when using the transform processor, if writing\na program like:\n```\nlogs | set attributes[\"x\"] = \"y\"\n```\nIf some input batch had no existing attribute record batch, a new one\nwould be created but we wouldn't set the schema metadata to indicate\nthat parent_id the column was not delta encoded. So when we decoded that\nto OTLP, we'd treat it as a delta encoded column and the net effect\nwould be that some attributes would end up missing from the result.\n\nThe second issue is somewhat similar - when inserting attributes using\nthe attribute processor we'd create a new attribute record batch, this\ntime with the correct parent_id encoding, but we'd also create a new ID\ncolumn for the root batch and this did not have indicator that said it\nwas not delta encoded.\n\nThis PR \n- corrects the behaviour to add the proper encoding metadata to the\nid/parent_id columns\n- updates tests so that our tests cover regressions agains these kind of\nerrors\n- adds a check in the upsert attributes helper function to ensure it is\npassed an attribute record batch with plain encoded parent_ids.\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #3985\n\n## Validation\n\nUnit tests\n\n<!--How did you confirm your change has the intended effect?-->\n\n## User-facing changes\n\nno\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-09-03T00:46:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67653eee12a09b7eadce90ea8d40302916ffff8c"
        },
        "date": 1788486621380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.355545997619629,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19332021998879,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.54574527424853,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.76119791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.4765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1902338.5635440587,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1876551.4895002518,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001534,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21995969.14332589,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 21930499.393542055,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.721484471062226,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.28808650374412537,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19508954580712,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44868865149404,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.010286458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.92578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527238.0386043591,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528756.9401673349,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001255,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16179479.304637976,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16144232.452682996,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.599086414861397,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "87c518a3f04a5d7f5791a1f201b8050c3156e7cf",
          "message": "feat(core-nodes): batch internal telemetry logs (#3886)\n\n## Summary\n\nBatch internal telemetry log records in the Internal Telemetry Receiver\ninstead\nof emitting one OTLP request per record.\n\n- Flush partial batches at a estimated-size threshold or latency\ndeadline.\n- Prevent a partial batch from exceeding a configurable estimated size.\n- Group by scope; preserve event order within scope.\n- Flush on channel close, ingress drain, and shutdown.\n- Preserve the existing independent internal-metrics export path.\n\nThe `logs` configuration mirrors the batch processor's format-specific\nstructure. Current output consumes `otlp`; `otap` reserves the same\nsurface for\nplanned OTAP log output.\n\n```yaml\nlogs:\n  otap: {}\n  otlp:\n    min_size: 65536\n    max_size: 2097152\n    sizer: bytes\n  max_batch_duration: 200ms\n```\n\nBoth format blocks use optional `min_size` and `max_size` plus `sizer`.\nCurrent\nOTLP output validates `bytes` sizing with the batch processor's\nconfiguration\nmodel. The reserved OTAP settings are parsed but remain inactive and\nunvalidated until OTAP output is implemented.\n\n## Benchmark\n\nA new `self_tracing` benchmark compares the previous one-request-per-log\nencoding path with one scope-grouped request per batch.\n\n| Records | Scopes | Per record | Grouped batch | Speedup |\n|--------:|-------:|-----------:|--------------:|--------:|\n| 1 | 1 | 579.86 ns | 617.13 ns | 0.94x |\n| 8 | 1 | 4.281 us | 1.189 us | 3.60x |\n| 8 | 8 | 4.291 us | 1.818 us | 2.36x |\n| 64 | 1 | 34.475 us | 5.179 us | 6.66x |\n| 64 | 8 | 34.207 us | 10.127 us | 3.38x |\n| 64 | 64 | 34.659 us | 10.444 us | 3.32x |\n| 512 | 1 | 285.26 us | 35.835 us | 7.96x |\n| 512 | 8 | 282.93 us | 90.356 us | 3.13x |\n| 512 | 512 | 289.01 us | 80.835 us | 3.58x |\n\n## Credit\n\nThis is a fresh implementation on current `main` of the problem\naddressed by\n#3374. The receiver-side batching design and original test strategy were\ncontributed by @AvinashDevX. Commit `0e1488f00` retains Avinash Sharma\nas a\nco-author.\n\nCloses #1902.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-09-04T16:44:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/87c518a3f04a5d7f5791a1f201b8050c3156e7cf"
        },
        "date": 1788543139810,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.30058661103248596,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16353115350367,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42116603097999,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.718098958333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528011.6312376488,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 529598.7635993579,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002557,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16195326.383675238,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16153368.88870287,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.58037045556062,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.19673386216163635,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.21681207741848,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58571671958492,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.475,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.96484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1960385.1395327374,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1964241.881076092,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005058,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22695072.488447074,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22631843.415237617,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.554112916080268,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6f96123c02aae9b2839d470e8b33aa7b543e4b5b",
          "message": "chore(otlp_receiver): adjust rate limit test parameters for deterministic behavior (#3984)\n\n# Change summary\n\nFix the flakiness in `test_otlp_grpc_transient_rate_limit_rejection` by\nkeeping its rate-limit bucket deterministically saturated for the\nduration of the test.\n\nThe test pre-consumes the bucket and expects the next request to be\nrejected by the gRPC pre-decode saturation check. The original policy\nallowed one request-sized burst per second. Because bucket capacity\nrefills continuously, even a small scheduling delay could restore the\nsingle unit checked by the saturation probe. The request would then\nproceed to the weighted admission check, where it was still rejected but\nreturned  grpc-retry-pushback-ms: 1000 . This conflicted with the\nexpected fast-rejection response, which intentionally has no\nrequest-specific retry guidance.\n\nThe test now configures a one-unit-per-hour refill rate while retaining\nthe request-sized burst. This preserves the behavior under test but\nprevents wall-clock scheduling from selecting a different rejection\npath.\n\nThe shutdown signal is also sent immediately after the RPC completes,\nbefore response assertions. Previously, an assertion failure skipped\nshutdown and left the receiver running until nextest terminated the test\nafter 120 seconds, obscuring the original assertion failure as a\ntimeout.\n\n## Related issue\n\nAddresses the OTLP receiver flake reported in #2720\n\n## Validation\n\nRan 50 iterations of `test_otlp_grpc_transient_rate_limit_rejection`\nsuccessfully\n\n## User-facing changes\n\nNone. This is a test-only change.",
          "timestamp": "2026-09-04T22:05:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f96123c02aae9b2839d470e8b33aa7b543e4b5b"
        },
        "date": 1788573071503,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3646137714385986,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.23459481199335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48971800433839,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.3125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527752.5021534788,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520550.7185867673,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002914,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16179175.23127973,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16134333.068720484,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.080881561751077,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.034238457679748535,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19808724908819,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.52672544470225,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.14609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.12890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1943904.589311195,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1944570.152261524,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00334,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22412515.563037973,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22336438.055021025,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.525691442385016,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6f96123c02aae9b2839d470e8b33aa7b543e4b5b",
          "message": "chore(otlp_receiver): adjust rate limit test parameters for deterministic behavior (#3984)\n\n# Change summary\n\nFix the flakiness in `test_otlp_grpc_transient_rate_limit_rejection` by\nkeeping its rate-limit bucket deterministically saturated for the\nduration of the test.\n\nThe test pre-consumes the bucket and expects the next request to be\nrejected by the gRPC pre-decode saturation check. The original policy\nallowed one request-sized burst per second. Because bucket capacity\nrefills continuously, even a small scheduling delay could restore the\nsingle unit checked by the saturation probe. The request would then\nproceed to the weighted admission check, where it was still rejected but\nreturned  grpc-retry-pushback-ms: 1000 . This conflicted with the\nexpected fast-rejection response, which intentionally has no\nrequest-specific retry guidance.\n\nThe test now configures a one-unit-per-hour refill rate while retaining\nthe request-sized burst. This preserves the behavior under test but\nprevents wall-clock scheduling from selecting a different rejection\npath.\n\nThe shutdown signal is also sent immediately after the RPC completes,\nbefore response assertions. Previously, an assertion failure skipped\nshutdown and left the receiver running until nextest terminated the test\nafter 120 seconds, obscuring the original assertion failure as a\ntimeout.\n\n## Related issue\n\nAddresses the OTLP receiver flake reported in #2720\n\n## Validation\n\nRan 50 iterations of `test_otlp_grpc_transient_rate_limit_rejection`\nsuccessfully\n\n## User-facing changes\n\nNone. This is a test-only change.",
          "timestamp": "2026-09-04T22:05:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f96123c02aae9b2839d470e8b33aa7b543e4b5b"
        },
        "date": 1788629381739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.16883742809295654,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1995934572056,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.4794548974061,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.761979166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.57421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530662.3513196863,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531558.3080151126,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0029,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16249603.066890355,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16216828.265532246,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.56974714132088,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.10291866213083267,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18636958164603,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59105474791214,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.531640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.48828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1940076.3458710045,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1938079.6451403091,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002983,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22347300.53427339,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22277715.009377815,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.530640957046703,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6f96123c02aae9b2839d470e8b33aa7b543e4b5b",
          "message": "chore(otlp_receiver): adjust rate limit test parameters for deterministic behavior (#3984)\n\n# Change summary\n\nFix the flakiness in `test_otlp_grpc_transient_rate_limit_rejection` by\nkeeping its rate-limit bucket deterministically saturated for the\nduration of the test.\n\nThe test pre-consumes the bucket and expects the next request to be\nrejected by the gRPC pre-decode saturation check. The original policy\nallowed one request-sized burst per second. Because bucket capacity\nrefills continuously, even a small scheduling delay could restore the\nsingle unit checked by the saturation probe. The request would then\nproceed to the weighted admission check, where it was still rejected but\nreturned  grpc-retry-pushback-ms: 1000 . This conflicted with the\nexpected fast-rejection response, which intentionally has no\nrequest-specific retry guidance.\n\nThe test now configures a one-unit-per-hour refill rate while retaining\nthe request-sized burst. This preserves the behavior under test but\nprevents wall-clock scheduling from selecting a different rejection\npath.\n\nThe shutdown signal is also sent immediately after the RPC completes,\nbefore response assertions. Previously, an assertion failure skipped\nshutdown and left the receiver running until nextest terminated the test\nafter 120 seconds, obscuring the original assertion failure as a\ntimeout.\n\n## Related issue\n\nAddresses the OTLP receiver flake reported in #2720\n\n## Validation\n\nRan 50 iterations of `test_otlp_grpc_transient_rate_limit_rejection`\nsuccessfully\n\n## User-facing changes\n\nNone. This is a test-only change.",
          "timestamp": "2026-09-04T22:05:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f96123c02aae9b2839d470e8b33aa7b543e4b5b"
        },
        "date": 1788659783270,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1245313882827759,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.22351384937467,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.58979038224413,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.8578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.1796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1966637.7027401256,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1944522.2433590414,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007978,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22439205.499347962,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22377717.90955478,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.539701114750752,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.13985338807106018,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1749705681287,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.44188118811881,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.252994791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.86328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530790.7856287891,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531533.1145631658,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005744,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16262681.981443677,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16222903.26711679,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.59580209750237,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6f96123c02aae9b2839d470e8b33aa7b543e4b5b",
          "message": "chore(otlp_receiver): adjust rate limit test parameters for deterministic behavior (#3984)\n\n# Change summary\n\nFix the flakiness in `test_otlp_grpc_transient_rate_limit_rejection` by\nkeeping its rate-limit bucket deterministically saturated for the\nduration of the test.\n\nThe test pre-consumes the bucket and expects the next request to be\nrejected by the gRPC pre-decode saturation check. The original policy\nallowed one request-sized burst per second. Because bucket capacity\nrefills continuously, even a small scheduling delay could restore the\nsingle unit checked by the saturation probe. The request would then\nproceed to the weighted admission check, where it was still rejected but\nreturned  grpc-retry-pushback-ms: 1000 . This conflicted with the\nexpected fast-rejection response, which intentionally has no\nrequest-specific retry guidance.\n\nThe test now configures a one-unit-per-hour refill rate while retaining\nthe request-sized burst. This preserves the behavior under test but\nprevents wall-clock scheduling from selecting a different rejection\npath.\n\nThe shutdown signal is also sent immediately after the RPC completes,\nbefore response assertions. Previously, an assertion failure skipped\nshutdown and left the receiver running until nextest terminated the test\nafter 120 seconds, obscuring the original assertion failure as a\ntimeout.\n\n## Related issue\n\nAddresses the OTLP receiver flake reported in #2720\n\n## Validation\n\nRan 50 iterations of `test_otlp_grpc_transient_rate_limit_rejection`\nsuccessfully\n\n## User-facing changes\n\nNone. This is a test-only change.",
          "timestamp": "2026-09-04T22:05:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f96123c02aae9b2839d470e8b33aa7b543e4b5b"
        },
        "date": 1788715793526,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.14632341265678406,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20446068146522,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.45990278527893,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.69453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530670.6896707463,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531447.185188748,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002922,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16249477.062790604,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16212406.717550876,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.575902019350174,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8859415650367737,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20993799492605,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7139337666357,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.73645833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.51171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1960967.9467105668,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1943594.9168448385,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00289,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22394676.41173041,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22328453.055316314,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.52229624477775,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6f96123c02aae9b2839d470e8b33aa7b543e4b5b",
          "message": "chore(otlp_receiver): adjust rate limit test parameters for deterministic behavior (#3984)\n\n# Change summary\n\nFix the flakiness in `test_otlp_grpc_transient_rate_limit_rejection` by\nkeeping its rate-limit bucket deterministically saturated for the\nduration of the test.\n\nThe test pre-consumes the bucket and expects the next request to be\nrejected by the gRPC pre-decode saturation check. The original policy\nallowed one request-sized burst per second. Because bucket capacity\nrefills continuously, even a small scheduling delay could restore the\nsingle unit checked by the saturation probe. The request would then\nproceed to the weighted admission check, where it was still rejected but\nreturned  grpc-retry-pushback-ms: 1000 . This conflicted with the\nexpected fast-rejection response, which intentionally has no\nrequest-specific retry guidance.\n\nThe test now configures a one-unit-per-hour refill rate while retaining\nthe request-sized burst. This preserves the behavior under test but\nprevents wall-clock scheduling from selecting a different rejection\npath.\n\nThe shutdown signal is also sent immediately after the RPC completes,\nbefore response assertions. Previously, an assertion failure skipped\nshutdown and left the receiver running until nextest terminated the test\nafter 120 seconds, obscuring the original assertion failure as a\ntimeout.\n\n## Related issue\n\nAddresses the OTLP receiver flake reported in #2720\n\n## Validation\n\nRan 50 iterations of `test_otlp_grpc_transient_rate_limit_rejection`\nsuccessfully\n\n## User-facing changes\n\nNone. This is a test-only change.",
          "timestamp": "2026-09-04T22:05:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f96123c02aae9b2839d470e8b33aa7b543e4b5b"
        },
        "date": 1788746130527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.02610873430967331,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1916532124062,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48548359706678,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.9390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.98046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1928251.0192875038,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1927747.577314666,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002943,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22235576.537176695,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22161051.675568234,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.534485530595578,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.9451919198036194,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18041505292585,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47372744243933,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.215885416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.9140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524513.6468989557,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 529471.3073269485,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002496,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16210548.50791845,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16173062.986158563,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.6164815422348,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6f96123c02aae9b2839d470e8b33aa7b543e4b5b",
          "message": "chore(otlp_receiver): adjust rate limit test parameters for deterministic behavior (#3984)\n\n# Change summary\n\nFix the flakiness in `test_otlp_grpc_transient_rate_limit_rejection` by\nkeeping its rate-limit bucket deterministically saturated for the\nduration of the test.\n\nThe test pre-consumes the bucket and expects the next request to be\nrejected by the gRPC pre-decode saturation check. The original policy\nallowed one request-sized burst per second. Because bucket capacity\nrefills continuously, even a small scheduling delay could restore the\nsingle unit checked by the saturation probe. The request would then\nproceed to the weighted admission check, where it was still rejected but\nreturned  grpc-retry-pushback-ms: 1000 . This conflicted with the\nexpected fast-rejection response, which intentionally has no\nrequest-specific retry guidance.\n\nThe test now configures a one-unit-per-hour refill rate while retaining\nthe request-sized burst. This preserves the behavior under test but\nprevents wall-clock scheduling from selecting a different rejection\npath.\n\nThe shutdown signal is also sent immediately after the RPC completes,\nbefore response assertions. Previously, an assertion failure skipped\nshutdown and left the receiver running until nextest terminated the test\nafter 120 seconds, obscuring the original assertion failure as a\ntimeout.\n\n## Related issue\n\nAddresses the OTLP receiver flake reported in #2720\n\n## Validation\n\nRan 50 iterations of `test_otlp_grpc_transient_rate_limit_rejection`\nsuccessfully\n\n## User-facing changes\n\nNone. This is a test-only change.",
          "timestamp": "2026-09-04T22:05:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f96123c02aae9b2839d470e8b33aa7b543e4b5b"
        },
        "date": 1788802253667,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.572048306465149,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.69274227003352,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.53801098986146,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.843489583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.49609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 365793.1562756736,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 360042.7110797552,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010658,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11205308.083840622,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11155218.53385113,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.122163396215697,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.3522931337356567,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.1595513203512,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.47846356385037,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.334635416666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.2109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1909863.05131004,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1935689.9982472344,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008022,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22314472.622983683,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22252125.27540799,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.527916475876518,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6f96123c02aae9b2839d470e8b33aa7b543e4b5b",
          "message": "chore(otlp_receiver): adjust rate limit test parameters for deterministic behavior (#3984)\n\n# Change summary\n\nFix the flakiness in `test_otlp_grpc_transient_rate_limit_rejection` by\nkeeping its rate-limit bucket deterministically saturated for the\nduration of the test.\n\nThe test pre-consumes the bucket and expects the next request to be\nrejected by the gRPC pre-decode saturation check. The original policy\nallowed one request-sized burst per second. Because bucket capacity\nrefills continuously, even a small scheduling delay could restore the\nsingle unit checked by the saturation probe. The request would then\nproceed to the weighted admission check, where it was still rejected but\nreturned  grpc-retry-pushback-ms: 1000 . This conflicted with the\nexpected fast-rejection response, which intentionally has no\nrequest-specific retry guidance.\n\nThe test now configures a one-unit-per-hour refill rate while retaining\nthe request-sized burst. This preserves the behavior under test but\nprevents wall-clock scheduling from selecting a different rejection\npath.\n\nThe shutdown signal is also sent immediately after the RPC completes,\nbefore response assertions. Previously, an assertion failure skipped\nshutdown and left the receiver running until nextest terminated the test\nafter 120 seconds, obscuring the original assertion failure as a\ntimeout.\n\n## Related issue\n\nAddresses the OTLP receiver flake reported in #2720\n\n## Validation\n\nRan 50 iterations of `test_otlp_grpc_transient_rate_limit_rejection`\nsuccessfully\n\n## User-facing changes\n\nNone. This is a test-only change.",
          "timestamp": "2026-09-04T22:05:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f96123c02aae9b2839d470e8b33aa7b543e4b5b"
        },
        "date": 1788832376084,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.1708478927612305,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19356963080493,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.51880444064902,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.095052083333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.58203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537392.1547527603,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 520352.2669623307,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004151,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16194396.328495013,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16165691.129243512,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.121986693809017,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.794152855873108,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20385327288577,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.60987871765161,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.5078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1935673.0744360215,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1900944.1395855728,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002992,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22348822.788066983,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22284073.92522047,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.75669622408751,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "24dff749d3194fef725195f3de5d47c0845c8a97",
          "message": "chore(deps): Raise the `otap-dataflow` Rust MSV to `1.88` (#4019)\n\n# Chore Summary\n\nRaise the `otap-dataflow` Rust MSV to `1.88`\n\nThis aligns the workspace's compatibility contract with Azure SDK\ndependencies that require Rust 1.88.\n\nThis was actually already an issue:\n- `otap` used a pinned Azure SDK Git revision and was not published to\ncrates.io.\n- CI built with Rust `1.98`, so Azure's Rust `1.88` minimum did not\nfail.\n- Preparing `otap` crate for publication requires released registry\ndependencies.\n- Reviewing that dependency contract exposed the advertised Rust `1.87`\nmismatch.\n\nThe overwhelming majority of changes in this PR are `clippy` fixes to\nmake use of `let` chains.\n\n### References\n\n- [Rust 1.88 release\nannouncement](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/)\ndocuments let-chain stabilization.\n- [Clippy MSRV\ndefinitions](https://github.com/rust-lang/rust/blob/1.98.0/src/tools/clippy/clippy_utils/src/msrvs.rs#L30)\nassign both `LET_CHAINS` and `AS_CHUNKS` to Rust 1.88.\n- [Clippy `collapsible_if`\nimplementation](https://github.com/rust-lang/rust/blob/1.98.0/src/tools/clippy/clippy_lints/src/collapsible_if.rs#L219-L220)\ngates let-chain suggestions on the configured MSRV.\n- [Clippy `collapsible_if`\ndocumentation](https://doc.rust-lang.org/stable/clippy/lints.html#collapsible_if)\ndescribes the lint.\n\n## Related issue\n\nRelated to #1340",
          "timestamp": "2026-09-08T18:44:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/24dff749d3194fef725195f3de5d47c0845c8a97"
        },
        "date": 1788897351736,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.08521062880754471,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18728291976919,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.57413798425682,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.240885416666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.30859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1922670.6532270135,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1921032.333450072,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002938,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22163978.242326405,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22090253.4249899,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.537535238941594,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.11213704943656921,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.53594706155165,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 80.80358236516119,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.449739583333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.0234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 357640.27232880564,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 358041.31956473377,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002907,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10988903.421505092,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10939651.144923363,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.691718583944894,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Drew Relmas",
            "username": "drewrelmas",
            "email": "drewrelmas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "31443bb2d97023b0693f20438c8db67bab53c114",
          "message": "chore(release): Isolate pdata-views from release automation (#4023)\n\n# Chore Summary\n\nAddress release problem identified in\nhttps://github.com/open-telemetry/otel-arrow/pull/4022. This is caused\nby `contrib` `geneva_exporter` usage of a library that has its own\ndependency on the `pdata-views` crate.\n\n- Keep `otel-arrow-dfe-pdata-views` at its current independent version\nduring normal releases\n- Add a Prepare Release opt-in for coordinated `pdata-views` version\nbumps\n- Allow crates.io publication tooling to validate and skip independently\nversioned crates outside the requested release\n- List only crates matching the requested version in generated release\nnotes\n- Document the independent release policy and external-consumer\ncompatibility requirement\n\nMoving forward, we should think about declaring `1.0.0` stability on\nthis crate so consumers can depend on `1` (meaning `>= 1.0.0 and <\n2.0.0`).\n\n## Related issue\n\nN/A",
          "timestamp": "2026-09-09T00:45:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/31443bb2d97023b0693f20438c8db67bab53c114"
        },
        "date": 1788934616710,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.0326592922210693,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19116341667227,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.42984667802386,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.434765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.80859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 537606.0765850135,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 521302.31531999377,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.012655,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16229273.317773385,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16194903.396727659,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.132171948653795,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.12541769444942474,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19857744084948,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.64252553389043,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.80833333333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.73828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1966238.2451885766,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1963772.234457077,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002983,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22625020.43032217,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22558737.46201589,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.521203952950938,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "otelbot-arrow[bot]",
            "username": "otelbot-arrow[bot]",
            "email": "289780372+otelbot-arrow[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "0dd9a5e12f59f55d3e7cde7604ba09a0232d6037",
          "message": "chore(release) Prepare Release v0.55.0 (#4026)\n\n## Release v0.55.0\n\nThis PR prepares the repository for release v0.55.0.\n\n### Changes included:\n- Rendered pending chloggen entries into `go/CHANGELOG.md` and\n`rust/otap-dataflow/CHANGELOG.md`\n- Bumped Rust workspace crates to 0.55.0\n- Include `otel-arrow-dfe-pdata-views` in this release: false\n\n## What's Changed (Go :hamster:)\n\n### :bulb: Enhancements :bulb:\n\n- `dependencies`: Upgrade various Go dependencies.\n([#3951](https://github.com/open-telemetry/otel-arrow/issues/3951),\n[#4014](https://github.com/open-telemetry/otel-arrow/issues/4014))\n\n## What's Changed (Rust :crab:)\n\n### :stop_sign: Breaking changes :stop_sign:\n\n- `all`: Raise the minimum supported Rust version for OTAP Dataflow from\n1.87 to 1.88.\n([#1340](https://github.com/open-telemetry/otel-arrow/issues/1340))\n\n### :rocket: New components :rocket:\n\n- `engine`: Add the `BasicAuthProvider` capability.\n ([#3901](https://github.com/open-telemetry/otel-arrow/issues/3901))\n\n### :bulb: Enhancements :bulb:\n\n- `all`: Publish otap, core-nodes, contrib-nodes, and contrib-extensions\nas versioned crates.io packages.\n([#1340](https://github.com/open-telemetry/otel-arrow/issues/1340))\n- `dependencies`: Upgrade various Rust dependencies.\n([#4012](https://github.com/open-telemetry/otel-arrow/issues/4012),\n[#4013](https://github.com/open-telemetry/otel-arrow/issues/4013),\n[#4015](https://github.com/open-telemetry/otel-arrow/issues/4015))\n- `engine`: Add TLS and mTLS support to the OpAMP controller extension\nWebSocket client.\n([#3884](https://github.com/open-telemetry/otel-arrow/issues/3884))\n\n- `observability`: Internal telemetry logs batching\n([#1902](https://github.com/open-telemetry/otel-arrow/issues/1902))\n\n- `pipeline`: Add Syslog decoding for Kafka log topics, including RFC\n3164, RFC 5424, and embedded CEF messages.\n([#3837](https://github.com/open-telemetry/otel-arrow/issues/3837))\n\n### :toolbox: Bug fixes :toolbox:\n\n- `engine`: Live pipeline reconciliation no longer fails when a runtime\nfinishes forced shutdown just after its graceful drain deadline.\n([#3266](https://github.com/open-telemetry/otel-arrow/issues/3266))\n- `observability`: Honor `RUST_LOG` when `engine.telemetry.logs.level`\nis omitted, while applying an explicit level before pipelines start.\n([#3996](https://github.com/open-telemetry/otel-arrow/issues/3996))\n- `pdata`: Return the input batch unchanged instead of panicking when\n`upsert_attributes` is called with no upserts\n([#3987](https://github.com/open-telemetry/otel-arrow/issues/3987))\n\n- `pipeline`: OTAP exporter shutdown now NACKs the batch already sent to\nthe server when a streaming request is still opening.\n([#2720](https://github.com/open-telemetry/otel-arrow/issues/2720),\n[#3870](https://github.com/open-telemetry/otel-arrow/issues/3870))\n\n- `pipeline`: Reject invalid traffic generator receiver settings during\nconfiguration loading and live reconfiguration by validating\nmax_batch_size to be strictly positive and preventing all-zero-weight\nconfig.\n([#3569](https://github.com/open-telemetry/otel-arrow/issues/3569))\n- `query-engine`: fix invalid ID column encoding when inserting new\nattributes when no prior attributes existed\n([#3985](https://github.com/open-telemetry/otel-arrow/issues/3985))\n\n### Checklist:\n- [ ] Verify both CHANGELOG.md files render the expected entries\n- [ ] Verify Rust crate versions updated\n- [ ] Confirm all tests pass\n- [ ] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\n---------\n\nCo-authored-by: otelbot-arrow[bot] <289780372+otelbot-arrow[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-09-09T15:24:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0dd9a5e12f59f55d3e7cde7604ba09a0232d6037"
        },
        "date": 1788978888736,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8597622513771057,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.20778939707048,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53982188492216,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.26354166666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.25,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1945246.8997589203,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1928522.401256804,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002995,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22223960.827884775,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22151927.475068815,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.523828197899896,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.4316654205322266,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.16894997775448,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.48425940256926,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.31875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.37109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 535766.2241888029,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528095.8442871285,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008501,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16147655.856080588,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16110361.526958674,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.577131084752526,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Laurent Quérel",
            "username": "lquerel",
            "email": "l.querel@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "acb03674161cbfd1811b21bfd45934e0d8929be5",
          "message": "chore(pdata): establish pluggable PData codec foundations (#3942)\n\n# Change summary\n\nThis PR establishes the codec and runtime foundations for a five-PR\nseries implementing pluggable PData codecs in the OTel Arrow Dataflow\nEngine.\n\nToday, `OtapPdata` supports native OTAP records and an OTLP-specific\nbyte representation. That representation allows transparent processors\nto forward OTLP without decoding and re-encoding it, but the\noptimization cannot be extended to other independently decodable\nformats.\n\nThe series generalizes that mechanism while preserving its performance\nand ownership properties:\n\n1. **Codec and runtime foundations**  \nDefine the codec model, add the built-in OTLP codec, inject\nruntime-owned codec access through effect handlers, and establish the\nperformance baseline. Existing payload storage and node behavior remain\nunchanged.\n\n2. **Encoded payload capabilities and node migration**  \nAdd generalized encoded payload storage and recoverable capability APIs,\nthen migrate core, contrib, and WASM consumers to\nrepresentation-independent inspection, views, native conversion, and\nstartup-resolved output encoding.\n\n3. **Codec-aware batching**  \nAllow codecs to batch compatible encoded payloads without materializing\nOTAP, with an OTAP fallback when native batching is unavailable.\n\n4. **OTLP activation and transitional cleanup**  \nMove OTLP HTTP and gRPC receivers onto the generalized codec path,\npreserve matching-format forwarding, and remove OTLP-specific storage\nand obsolete APIs.\n\n5. **Syslog extensibility proof**  \nAdd a decode-only syslog/CEF codec whose receiver preserves and batches\noriginal messages until native processing is required.\n\nParquet and other representations are left for later work.\n\n### Full-series reference performance\n\nThe table below evaluates the complete five-PR reference implementation,\nnot this foundation PR in isolation. \"Before\" is the series base\n(`9a2b51e7`); \"after\" is the cleaned PR 5 tip (`d8e3125d`). Rates are\nmedian successfully delivered log records per second. Changes are the\ngeometric mean of five paired after/before ratios, with 95% paired t\nintervals.\n\n| Path | 1 CPU before | 1 CPU after | 1 CPU change (95% CI) | 4 CPUs\nbefore | 4 CPUs after | 4 CPU change (95% CI) |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n| OTLP gRPC -> OTLP gRPC | 0.449 M/s | 0.450 M/s | +1.1% (-1.6% to\n+3.8%) | 1.803 M/s | 1.802 M/s | -0.3% (-1.9% to +1.3%) |\n| OTAP gRPC -> OTAP gRPC | 3.525 M/s | 3.560 M/s | +0.8% (-1.0% to\n+2.6%) | 9.338 M/s | 9.252 M/s | -1.0% (-4.5% to +2.7%) |\n| Syslog RFC 3164/TCP -> OTAP gRPC | 0.329 M/s | 0.325 M/s | -2.0%\n(-3.9% to -0.1%) | 1.339 M/s | 1.286 M/s | -4.6% (-7.3% to -1.9%) |\n\nThe controlled logs-only workload uses 1,000-record batches, 1,024-byte\nbodies, loopback transports, no processors, and no transport or Arrow\ncompression. Only the SUT binary changes; senders and backends use the\nbefore binary. The SUT is pinned to CPU 14 for one-core runs and CPUs\n10-13 for four-core runs, with generators and backends on separate\nphysical cores. All runs completed without failed or refused batches.\nThe OTLP and OTAP intervals include zero; the measurements therefore do\nnot establish a throughput change for those paths. The Syslog result\nshows a small measurable cost in the full codec path.\n\n### Design principles\n\n- Native OTAP remains the common mutable processing representation and\nthe intermediate representation for cross-codec conversion.\n- Codecs represent independently decodable byte formats and may support\ndecoding, encoding, or both.\n- Conversion remains lazy. Transparent nodes preserve the original\nrepresentation, while native consumers request OTAP explicitly.\n- Payloads contain codec identity, signal, bytes, counts, and delivery\ncontext, but never mutable codec state.\n- Codec instances and reusable buffers belong to the single-threaded\npipeline runtime, are created lazily, and are accessed through effect\nhandlers.\n- Codec failures preserve the original message and its Ack/Nack\nownership for rejection or retry.\n- Admission and matching-format forwarding must not instantiate codecs,\ncopy the underlying `Bytes`, or allocate a per-message codec wrapper.\n- Reusable buffers have bounded retained capacity, and signal-specific\nstate is allocated only when used.\n- Encoding identity includes incompatible format versions and intrinsic\ncompression. HTTP and gRPC compression remain transport concerns.\n- Output selection is resolved explicitly at startup. Automatic\nrepresentation negotiation and predicate/projection pushdown are outside\nthis series.\n\n### This PR\n\nThis first PR provides the foundations needed by later migrations\nwithout changing `PayloadData`, receiver admission, or node processing\nbehavior.\n\nIt introduces:\n\n- Legacy-path benchmarks for forwarding, conversion, batching,\ncompression, syslog parsing, allocation, and layout.\n- The new `otel-arrow-dfe-pdata-codec` crate, which centralizes codec\nidentity, contracts, registration, plans, and runtime lifecycle without\ndepending on the pipeline engine or delivery context.\n- `PdataEncoding`, validated link-time registration, codec capability\nmetadata, and startup resolution through `ResolvedCodec`.\n- Separate decoder and encoder contracts, prepared output, borrowed\nviews, and stateless item counting, allowing one-sided codecs without\ndummy operations.\n- A codec-level `EncodedPdata` envelope used by codec APIs and tests. It\nis not yet a `PayloadData` variant.\n- A built-in OTLP codec for logs, metrics, and traces, with lazy\nper-signal state and bounded retained buffers.\n- `EncodePolicy` and startup-resolved `EncodingPlan` for explicit output\nselection.\n- One lazy `CodecService` in each `PipelineRuntimeServices`, exposed\nuniformly through the `CodecEffectHandler` interface for receiver,\nprocessor, and exporter effect handlers.\n- Codec conformance tests and Criterion benchmarks comparing direct\ncodec behavior with the existing OTLP implementation.\n\nExisting OTLP receivers continue producing `OtlpBytes`, and\n`PayloadData` still contains only its existing storage variants.\nGeneralized payload storage, recoverable payload capabilities, node\nmigration, codec-aware batching, OTLP activation, and syslog integration\nremain in later PRs.\n\n## Related issue\n\n- Part of #3452\n\n## Validation\n\n- `cargo check -p otel-arrow-dfe-pdata-codec -p otel-arrow-dfe-engine -p\notel-arrow-dfe-otap -p otel-arrow-dfe-core-nodes -p\notel-arrow-dfe-contrib-nodes`\n- `cargo test -p otel-arrow-dfe-pdata-codec`\n- `cargo check -p benchmarks --all-targets`\n\n## User-facing changes\n\nNone. This PR adds internal codec and runtime infrastructure, tests, and\nbenchmarks without changing pipeline configuration or wire behavior.",
          "timestamp": "2026-09-10T00:07:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/acb03674161cbfd1811b21bfd45934e0d8929be5"
        },
        "date": 1789006650756,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.1673691272735596,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.18569669510369,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.53997381997381,
            "unit": "%",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.45533854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.64453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1955893.4582549373,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1913502.0286961307,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003072,
            "unit": "seconds",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 22484827.148920454,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 22417234.307131156,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.750615788080308,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough OTAP/OTAP-OTAP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.1466582864522934,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.19980622205512,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.43574809219147,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.182552083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 529462.4478603262,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 530238.9484123945,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002533,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16217079.318764206,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16179084.187466402,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.584473975969296,
            "unit": "bytes/log",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}