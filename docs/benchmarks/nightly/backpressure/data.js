window.BENCHMARK_DATA = {
  "lastUpdate": 1794620785327,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
        "date": 1787890674738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.6656497552058,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.9137890897804,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.334114583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.36692938917,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.36692938917,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003234,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107209.6623559003,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167395.6391802079,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.601302485728777,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90464460535345,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.31643394199786,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.625130208333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.08984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.58695969258,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.01051036437,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003248,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3199592.7782551306,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3037271.0267648282,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.53999304621214,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787951963739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73296490637223,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09496347558631,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.430208333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.0625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98317.49805258302,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98317.49805258302,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006409,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3113349.9416002026,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168675.9140548024,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.66628528255564,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92978992217931,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24960211788523,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.266536458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.91015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.28513800712,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.680385707,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002229,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193959.8362191063,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027943.0322248335,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.4821541541731,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787968157363,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92633845860223,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33352295563456,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.552734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.47512425187,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.86720551434,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002115,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192112.7936212122,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028491.490454688,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46330829420869,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8259838710318,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26948079877111,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.3796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.08671799542,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.08671799542,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003405,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107410.7842273703,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167485.9011900388,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.603438023295972,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788024516624,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82288921928904,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14228979211914,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.297395833333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.54801066348,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.54801066348,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00292,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107657.999807613,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1169385.5080186985,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.604839720307833,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86140817005477,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12126649076518,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.001692708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.46875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.16873781802,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.58815003872,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003099,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193392.1562708034,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031247.0015000873,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.477732565381515,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788054917167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90104884266609,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.29405621256336,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.797916666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.28125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.30846961233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.70332845213,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002215,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3189576.0010025543,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026819.550857729,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.43756355440601,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77597106575152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15050622150089,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.95859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.48046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.27158337798,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.27158337798,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005123,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3105580.305183465,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166798.619378379,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.585725748310352,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788110947761,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.686282992362976,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8263947259977,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26871723498027,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.364453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.9921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.12563843092,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96668.06887968557,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002771,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3112248.196830639,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167641.5166464723,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.195203989273715,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93254780247081,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24231803430692,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.885416666666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.29296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.73033464693,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.14038667087,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002762,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187360.201927705,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3018425.9945601,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.416203762227454,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788141341380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72882535975582,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.06656771799629,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.2875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.81640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.30957604993,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.30957604993,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003269,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3106921.083569807,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166448.8093552634,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.598385979824982,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90269743803351,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26645502645503,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.431901041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.83193376956,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.25140154007,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003101,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191724.976043043,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031719.4572232077,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.459897644361575,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788197419877,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90435085470268,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14474581477499,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.848307291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.39453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.42679434922,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.81968111006,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002144,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191781.562790823,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3034113.756728867,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.45995541476611,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6950531005859375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78422404770178,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1359177954227,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.915104166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98312.28401616825,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96645.83854236414,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007964,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110608.076270652,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167160.212123357,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.185639063053245,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788227911157,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678201586008072,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8557751944152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26386513030702,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.554166666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.54226715551,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98334.07515578314,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003127,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190183.0271153776,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027066.303432402,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.44229451552186,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.71760169324916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.05193053500507,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.219270833333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.28295321832,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.28295321832,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002675,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3113029.3803652856,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168341.7907103545,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.660195899466704,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788285465499,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90202100965006,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.50185750440984,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.62265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.05859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.4134619705,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.80657093765,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002152,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3186617.6097345944,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3024164.679947096,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.40744308223251,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77309185263026,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11185938945421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.832161458333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.44140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.72540375673,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96660.12553996794,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002405,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3114639.08800157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166270.098297769,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.222584758734875,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788313961740,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87523776212217,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.37503434436984,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.069010416666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.63671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98319.95690319665,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.95690319665,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008163,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191501.4742634376,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028626.5334919114,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46036282751537,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80743103238633,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1022112415071,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.824479166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.58898018645,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.58898018645,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003302,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3112289.3748356085,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168272.092706057,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.653859068568504,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788370375246,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79179582911712,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15803400309119,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.59375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.93029076829,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.93029076829,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004721,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110947.405965528,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167613.4735576857,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.64010060283588,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88266139040807,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25505084745762,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.6375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.38679722373,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.78035060334,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002168,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194230.5895082103,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029001.916592223,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48487465464587,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788400398453,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75122536392131,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.01134698944755,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.773697916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.19921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.7249937553,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96662.1364345391,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002812,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3114950.9941646396,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168282.7260499897,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.22514118828862,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83461884502695,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14332689484664,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.063541666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.36513212745,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.15296105653,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190253.686881331,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032112.0829881793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.003823992443266,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788468277021,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6581333875656128,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87466005399672,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.38110490378646,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.659375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.24609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.28013837885,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98338.20848531101,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002232,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3188110.766148745,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029519.1158150486,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.41985811267814,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81885299246541,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12034621329211,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.190234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.05722215214,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.05722215214,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003423,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3112994.067876832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1169423.4371040992,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.660231438700755,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788486596583,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.11437239342285,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.52935578557083,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.64296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.17578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.68366324138,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.68366324138,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002227,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192133.902829626,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031112.1236646,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46358356813206,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76285350321083,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.0838454404946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.024869791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.8671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.13347191148,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96657.62273509942,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005614,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3111601.4728540736,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167769.0493241628,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.19199256929535,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788543110208,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.60376353468703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.99637537468296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.633203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.11457519699,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96658.54201432758,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003388,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3108592.0786559633,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165560.4964607914,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.1605521237345,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87385232786532,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32371841434201,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.823697916666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.7736064995,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.19404639117,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003136,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3195347.614644753,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033396.3011182994,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.49675889640758,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788573042964,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88587209934693,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30552848052149,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.05859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99977.6749851758,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96645.08581900327,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.013398,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3185457.068098529,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3022789.1628821883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.960362558570715,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.00867843721061945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74271245543792,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26862266192612,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.054296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.89540301365,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98314.36251267955,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003115,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107535.145118269,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167304.9706477826,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.608150281374115,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788629353387,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7385114780461,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15416480037068,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.203515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.41732666454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.41732666454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002593,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3109825.2138432804,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168101.834307946,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.627565596249443,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92054951185182,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34063209952863,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.056380208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.59375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.9035930802,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.32186652886,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003058,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3196398.2452711645,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032125.802073092,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.507401576627785,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788659756747,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84407528159258,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.38025825407871,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.154296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.83762697825,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.83762697825,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004574,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3197227.064898002,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031562.4146146937,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.516652205165244,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.68868590523796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18389618414955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.58828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.33984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.50377203914,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96655.93026683142,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003354,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3109625.434984044,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167154.5545758852,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.17211221701052,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788715764895,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72056413869377,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21398082276524,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.863932291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.48046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98317.48556910137,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98317.48556910137,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008044,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3109122.5601480627,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167327.1631077768,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.623292053811223,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9024941054785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22765947503699,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.63203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.93692302714,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.35464097669,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003038,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190025.4646661934,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3024249.6340492466,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.442579521581905,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788746106316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.00867843721061945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79785182526724,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23164400494437,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.836067708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.90234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.85935317015,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98314.3264659646,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003137,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116992.627840376,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168383.5130412267,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.704358254637963,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83674067357933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23437760940158,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.494401041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.71694578804,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.1383300249,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00317,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3188308.203634183,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027210.0736311222,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.4251862974672,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788802225124,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75794615193496,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19729866933312,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.723828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98321.47524742229,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98321.47524742229,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005609,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3108652.594634751,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166051.1439925944,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.617228960529157,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.67520010471344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8741825326562,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24188708430007,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.737760416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99988.30136873986,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98313.29734421073,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00702,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187408.573245085,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029173.381665574,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.42093042699456,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788832353837,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82670039400367,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19373676481955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.473307291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.96875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.30957604993,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.30957604993,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003269,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3109313.6159041557,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167305.7499512944,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.622718802621723,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88867575849999,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20677798392084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.965104166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.56640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.96025400319,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.37758310314,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003024,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187018.235549757,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028622.486366764,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.411988419683006,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788897325575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78659144385148,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10014742014742,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.963020833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.6782772554,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96659.09616185472,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003044,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3109978.3423828855,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166842.1090203854,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.17470952940898,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92361605989927,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.167250811312,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.025390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.44140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.60190542223,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.9974308792,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002239,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3188725.6137462463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028024.900284616,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.42980765265227,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788934589352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75641720027916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13734111145226,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.904296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.53125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98321.58943236902,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96655.03142528441,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003912,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110174.394079009,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166009.9838934592,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.17809097173812,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91060169254487,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30057973255005,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.129947916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.40864598322,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.83516855015,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003355,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194055.396239668,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030467.238113567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48373556444652,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1788978854804,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76981708876492,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20225766871167,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.25546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.87890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98320.49650744154,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96653.95702551301,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004579,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3114489.3450591634,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167689.4973608502,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.223091955118356,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8335807946838,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25157310302284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.009244791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.9765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.40012959533,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.79346076873,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00216,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3195027.577474853,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035968.967844136,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.49297557763704,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1789006623619,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84987893809657,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24585214970615,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.238411458333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.37679794736,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.77051798157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002174,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3185327.9288514894,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3025553.8486899203,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.39433909050961,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75559814778703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24020725388601,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.80625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.59375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.11828038919,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96657.56260633042,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003996,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3108759.6070010965,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166904.5437084595,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.16261121400855,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "4b5d5fec907c2c40edae458c43c96d5f29492dbe",
          "message": "chore: Refactor TokenProviderExtension (#3979)\n\nRelates to #3901\n\n# Changes\n\n* Refactor reusable bits of `TokenProviderExtension` into\n`BackgroundProviderExtension`\n\n# Details\n\nI'm working on adding `ApiKey` and `BasicAuth` capabilities and there\nwill be flat file extensions implemented initially. There is code in\n`TokenProviderExtension` which handles the background refresh of\n`BearerToken`s. What this PR does is refactor the common things so\nrefresh logic can be reused for any generic struct (`BearerToken`,\n`ApiKey`, `BasicAuthCredentials`, etc.).\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-09-10T18:07:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4b5d5fec907c2c40edae458c43c96d5f29492dbe"
        },
        "date": 1789066560722,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667555570602417,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.65514666895919,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13650401482397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.785416666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.53515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99990.16044343602,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.5688844956,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002704,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3101241.2152332696,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166951.0860374165,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.54117827920195,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88157356238005,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2198624526698,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.147916666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.66891433846,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.06435646831,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002236,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191484.3121724883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030944.926054527,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.016585325736436,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "f7d3913ce77ffe216426f9267e7075cd20d746dd",
          "message": "chore(release) Prepare Release v0.56.0 (#4053)\n\n## Release v0.56.0\n\nThis PR prepares the repository for release v0.56.0.\n\n### Changes included:\n- Rendered pending chloggen entries into `go/CHANGELOG.md` and\n`rust/otap-dataflow/CHANGELOG.md`\n- Bumped Rust workspace crates to 0.56.0\n- Include `otel-arrow-dfe-pdata-views` in this release: true\n\n## What's Changed (Go :hamster:)\n\nNo changes. This release maintains version parity across the repository.\n\n## What's Changed (Rust :crab:)\n\n### :stop_sign: Breaking changes :stop_sign:\n\n- `engine`: Rename node completion duration and make node telemetry\nmeasurements independently selectable.\n([#3881](https://github.com/open-telemetry/otel-arrow/issues/3881))\n\n- `pipeline`: Add shared receiver and exporter boundary metrics, make\ncomponent duration opt-in, and migrate console exports to\nexporter.attempted.\n([#3822](https://github.com/open-telemetry/otel-arrow/issues/3822))\n\n### :bulb: Enhancements :bulb:\n\n- `engine`: Allow extensions to be declared in the engine observability\npipeline.\n([#4035](https://github.com/open-telemetry/otel-arrow/issues/4035))\n- `engine`: Publish the public admin API SDK as the\n`otel-arrow-dfe-admin-api` crate on crates.io.\n([#4038](https://github.com/open-telemetry/otel-arrow/issues/4038))\n- `observability`: Allow flow compute duration metrics to select basic,\nnormal, or detailed distribution aggregation.\n([#3670](https://github.com/open-telemetry/otel-arrow/issues/3670))\n- `pipeline`: Add event_ids filtering to the ETW receiver for\nmanifest-based providers\n([#3895](https://github.com/open-telemetry/otel-arrow/issues/3895))\n\n- `pipeline`: Add agent-fed bearer-token authentication to the OTLP/HTTP\nexporter.\n([#3275](https://github.com/open-telemetry/otel-arrow/issues/3275))\n\n### :toolbox: Bug fixes :toolbox:\n\n- `pipeline`: Preserve partition processor Nack payloads for interested\nupstreams and report transient only when all downstream Nacks are\ntransient.\n([#4016](https://github.com/open-telemetry/otel-arrow/issues/4016))\n\n### Checklist:\n- [x] Verify both CHANGELOG.md files render the expected entries\n- [x] Verify Rust crate versions updated\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\n---------\n\nCo-authored-by: otelbot-arrow[bot] <289780372+otelbot-arrow[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-11T14:29:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f7d3913ce77ffe216426f9267e7075cd20d746dd"
        },
        "date": 1789155216381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87084724264119,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24595902589873,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.117317708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.4375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.44149336903,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.44149336903,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002985,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3197939.1740807216,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028871.0402634884,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.52303327004711,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80100091619252,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15319162141195,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.16875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.9609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.30877702696,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96655.7385769987,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003473,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3106513.5763729014,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168621.0279255013,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.13998074101068,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "f7d3913ce77ffe216426f9267e7075cd20d746dd",
          "message": "chore(release) Prepare Release v0.56.0 (#4053)\n\n## Release v0.56.0\n\nThis PR prepares the repository for release v0.56.0.\n\n### Changes included:\n- Rendered pending chloggen entries into `go/CHANGELOG.md` and\n`rust/otap-dataflow/CHANGELOG.md`\n- Bumped Rust workspace crates to 0.56.0\n- Include `otel-arrow-dfe-pdata-views` in this release: true\n\n## What's Changed (Go :hamster:)\n\nNo changes. This release maintains version parity across the repository.\n\n## What's Changed (Rust :crab:)\n\n### :stop_sign: Breaking changes :stop_sign:\n\n- `engine`: Rename node completion duration and make node telemetry\nmeasurements independently selectable.\n([#3881](https://github.com/open-telemetry/otel-arrow/issues/3881))\n\n- `pipeline`: Add shared receiver and exporter boundary metrics, make\ncomponent duration opt-in, and migrate console exports to\nexporter.attempted.\n([#3822](https://github.com/open-telemetry/otel-arrow/issues/3822))\n\n### :bulb: Enhancements :bulb:\n\n- `engine`: Allow extensions to be declared in the engine observability\npipeline.\n([#4035](https://github.com/open-telemetry/otel-arrow/issues/4035))\n- `engine`: Publish the public admin API SDK as the\n`otel-arrow-dfe-admin-api` crate on crates.io.\n([#4038](https://github.com/open-telemetry/otel-arrow/issues/4038))\n- `observability`: Allow flow compute duration metrics to select basic,\nnormal, or detailed distribution aggregation.\n([#3670](https://github.com/open-telemetry/otel-arrow/issues/3670))\n- `pipeline`: Add event_ids filtering to the ETW receiver for\nmanifest-based providers\n([#3895](https://github.com/open-telemetry/otel-arrow/issues/3895))\n\n- `pipeline`: Add agent-fed bearer-token authentication to the OTLP/HTTP\nexporter.\n([#3275](https://github.com/open-telemetry/otel-arrow/issues/3275))\n\n### :toolbox: Bug fixes :toolbox:\n\n- `pipeline`: Preserve partition processor Nack payloads for interested\nupstreams and report transient only when all downstream Nacks are\ntransient.\n([#4016](https://github.com/open-telemetry/otel-arrow/issues/4016))\n\n### Checklist:\n- [x] Verify both CHANGELOG.md files render the expected entries\n- [x] Verify Rust crate versions updated\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\n---------\n\nCo-authored-by: otelbot-arrow[bot] <289780372+otelbot-arrow[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-11T14:29:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f7d3913ce77ffe216426f9267e7075cd20d746dd"
        },
        "date": 1789177680204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72712829876083,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.02069434779247,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.744270833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.99208226994,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.99208226994,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003056,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3108994.2407406797,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167364.0005841197,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.620215932193016,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81208770808799,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10706192767344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.106640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.605002477,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.605002477,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002275,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194080.123321588,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030795.7301403536,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48340236128404,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "f7d3913ce77ffe216426f9267e7075cd20d746dd",
          "message": "chore(release) Prepare Release v0.56.0 (#4053)\n\n## Release v0.56.0\n\nThis PR prepares the repository for release v0.56.0.\n\n### Changes included:\n- Rendered pending chloggen entries into `go/CHANGELOG.md` and\n`rust/otap-dataflow/CHANGELOG.md`\n- Bumped Rust workspace crates to 0.56.0\n- Include `otel-arrow-dfe-pdata-views` in this release: true\n\n## What's Changed (Go :hamster:)\n\nNo changes. This release maintains version parity across the repository.\n\n## What's Changed (Rust :crab:)\n\n### :stop_sign: Breaking changes :stop_sign:\n\n- `engine`: Rename node completion duration and make node telemetry\nmeasurements independently selectable.\n([#3881](https://github.com/open-telemetry/otel-arrow/issues/3881))\n\n- `pipeline`: Add shared receiver and exporter boundary metrics, make\ncomponent duration opt-in, and migrate console exports to\nexporter.attempted.\n([#3822](https://github.com/open-telemetry/otel-arrow/issues/3822))\n\n### :bulb: Enhancements :bulb:\n\n- `engine`: Allow extensions to be declared in the engine observability\npipeline.\n([#4035](https://github.com/open-telemetry/otel-arrow/issues/4035))\n- `engine`: Publish the public admin API SDK as the\n`otel-arrow-dfe-admin-api` crate on crates.io.\n([#4038](https://github.com/open-telemetry/otel-arrow/issues/4038))\n- `observability`: Allow flow compute duration metrics to select basic,\nnormal, or detailed distribution aggregation.\n([#3670](https://github.com/open-telemetry/otel-arrow/issues/3670))\n- `pipeline`: Add event_ids filtering to the ETW receiver for\nmanifest-based providers\n([#3895](https://github.com/open-telemetry/otel-arrow/issues/3895))\n\n- `pipeline`: Add agent-fed bearer-token authentication to the OTLP/HTTP\nexporter.\n([#3275](https://github.com/open-telemetry/otel-arrow/issues/3275))\n\n### :toolbox: Bug fixes :toolbox:\n\n- `pipeline`: Preserve partition processor Nack payloads for interested\nupstreams and report transient only when all downstream Nacks are\ntransient.\n([#4016](https://github.com/open-telemetry/otel-arrow/issues/4016))\n\n### Checklist:\n- [x] Verify both CHANGELOG.md files render the expected entries\n- [x] Verify Rust crate versions updated\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\n---------\n\nCo-authored-by: otelbot-arrow[bot] <289780372+otelbot-arrow[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-11T14:29:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f7d3913ce77ffe216426f9267e7075cd20d746dd"
        },
        "date": 1789234020988,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81648010666204,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15761941567476,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.267447916666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.75,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98317.19229546258,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96650.75401416236,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008223,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3101150.625608341,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168054.861086269,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.086150358991766,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90061311194813,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30949810532827,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.488541666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.92859053831,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.34644736268,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003043,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194700.2849356965,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032207.78054599,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.49012518120489,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "e109bdc62a1dd3849b62fdd7633949c7eb59429f",
          "message": "feat(context): compile transport header policies (#4008)\n\n###\n\nTransport header policies are re-implemented. This replaces #3934, which\nfell apart for not accomplishing anything real. This PR makes a first\nstep in optimizing the transport headers implementation, as such it is\nmore complete.\n\nThis PR compiles information about whether any consumer uses the\noriginal form of a transport header. When no consumer uses the transport\nheaders original name, the original name is omitted from the context.\n\nThis requires:\n\n1. New `ContextEntryName` is a config struct entry that checks a few\nrequirements for transport header names and normalizes them to ASCII\nlowercase. (This was existing behavior, now encapsulated in a type.)\n2. Transport headers are now a pair of `ContextEntryName` and value,\nwhere value is the optional original name, value type, and value bytes.\nNote we use `Box<[u8]>` and `Box<str>` to save one word per container,\nsince these are immutable after calculation.\n3. Transport header policies now include `ContextEntryName` fields,\nmeans less error checking and validation\n4. Kafka exporter/receiver work: declare context uses, accept\n`ContextEntryName`, etc. (see benchmarks)\n5. Controller/live_control: compile immutable engine-wide context\nruntime requirements and distribute per-generation compiled bindings via\nPipelineContext\n6. OTLP/grpc, partition processor, transform processor, OTAP, validation\nnodes etc. all add context declarations\n7. New `crates/engine/src/context_declaration.rs` separates immutable\nruntime requirements from per-generation compiled context bindings\n\nThe benchmarks change for three reasons:\n\n1. Avoiding the original header name when not required\n2. Re-using an intermediate Vec<_> when capturing headers\n3. Using a HashMap<_, _> instead of a linear scan to locate match rules\n(note case-insensitive hash function, since we hash and compare original\nvalue)\n\nNew benchmark results below.\n\nBenchmark methodology: Criterion point estimates with 100 samples, a\n3-second warmup, and a 5-second measurement period, pinned to CPU 1 on\nan otherwise idle Intel Core Ultra 7 165H system. gRPC receive cells\nwere measured individually with each upstream comparison adjacent;\nend-to-end and Kafka cases were grouped by header count.\n\n### gRPC receive, renamed / original-name consumer\n\n| Revision | 1 | 4 | 16 | 32 |\n|---|---:|---:|---:|---:|\n| upstream/main `f7d3913ce` | 213.94 ns | 516.79 ns | 1.962 us | 5.352\nus |\n| current `68392a415` | 86.78 ns | 349.76 ns | 1.116 us | 2.411 us |\n| Delta | -59.4% | -32.3% | -43.1% | -54.9% |\n\n### gRPC end-to-end, renamed / original-name consumer\n\n| Revision | 1 | 4 | 16 | 32 |\n|---|---:|---:|---:|---:|\n| upstream/main `f7d3913ce` | 325.53 ns | 930.05 ns | 5.832 us | 12.53\nus |\n| current `68392a415` | 183.94 ns | 664.91 ns | 3.303 us | 7.055 us |\n| Delta | -43.5% | -28.5% | -43.4% | -43.7% |\n\n### gRPC receive\n\nStored-name consumers are omitted here because receive-only capture\ncompiles identically to the no-consumer case; propagation differences\nappear in the end-to-end table.\n\n| Producer / original-name retention | 1 | 4 | 16 | 32 |\n|---|---:|---:|---:|---:|\n| Unrenamed / omit | 85.11 ns | 272.35 ns | 973.85 ns | 1.923 us |\n| Unrenamed / preserve | 74.74 ns | 317.37 ns | 1.022 us | 1.922 us |\n| Renamed / omit | 74.41 ns | 281.94 ns | 1.015 us | 1.851 us |\n| Renamed / preserve | 86.78 ns | 349.76 ns | 1.116 us | 2.411 us |\n\n### gRPC end-to-end\n\n| Producer / consumer | 1 | 4 | 16 | 32 |\n|---|---:|---:|---:|---:|\n| Unrenamed / none | 94.07 ns | 330.13 ns | 1.178 us | 3.188 us |\n| Unrenamed / stored-name | 152.77 ns | 636.36 ns | 2.905 us | 6.239 us\n|\n| Unrenamed / original-name | 172.70 ns | 628.94 ns | 3.210 us | 6.282\nus |\n| Renamed / none | 101.47 ns | 344.21 ns | 1.630 us | 2.638 us |\n| Renamed / stored-name | 166.93 ns | 633.69 ns | 3.064 us | 6.638 us |\n| Renamed / original-name | 183.94 ns | 664.91 ns | 3.303 us | 7.055 us\n|\n\n### Kafka receive, original names\n\n| Implementation | 1 | 4 | 16 | 32 |\n|---|---:|---:|---:|---:|\n| Legacy enum | 101.07 ns | 366.97 ns | 2.429 us | 6.680 us |\n| Current value | 98.67 ns | 448.05 ns | 2.150 us | 6.337 us |\n| Delta | -2.4% | +22.1% | -11.5% | -5.1% |\n\n### Kafka end-to-end, original names\n\n| Implementation | 1 | 4 | 16 | 32 |\n|---|---:|---:|---:|---:|\n| Legacy enum | 155.42 ns | 479.92 ns | 2.963 us | 8.702 us |\n| Current value | 161.52 ns | 512.91 ns | 2.639 us | 7.658 us |\n| Delta | +3.9% | +6.9% | -10.9% | -12.0% |\n\n### Transport header layout\n\n| Representation | Size | Delta |\n|---|---:|---:|\n| Legacy enum | 64 bytes | - |\n| Current value | 56 bytes | -12.5% |\n\n\nPart of #3919\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Maksat Maratov <mmaratov@microsoft.com>",
          "timestamp": "2026-09-12T23:08:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e109bdc62a1dd3849b62fdd7633949c7eb59429f"
        },
        "date": 1789264402779,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86859668530937,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20379424693452,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.667838541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.18359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99997.87837834707,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98331.2470720413,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001273,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194679.6820496786,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027202.3994420064,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48895724579931,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76954474168217,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27030752588472,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.979947916666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.1321932037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96659.54238410373,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002767,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3106558.918396774,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167258.0279365054,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.139185038265474,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "3281b2eebe891fa9410a477b7a26631410f6cdf2",
          "message": "feat(filter_processor): Align metrics with universal node telemetry (#4045)\n\n# Change summary\n\n- Rename `processor.filter.pdata` to `processor.filter`\n- Improve test coverage for dropped metric emission\n\n[#3053](https://github.com/open-telemetry/otel-arrow/issues/3053)\npredates universal node input and output metrics. Its proposed\nfilter-specific counters answered whether batches arrived and how many\nitems survived, while configured-path counters indicated only that a\nbatch arrived while an `include` or `exclude` rule was configured.\n\nThose requirements are now covered without duplicating throughput\ninstrumentation:\n\n| Requirement | Metric or source |\n| --- | --- |\n| Batches and items received | `node.input.messages` and\n`node.input.items` |\n| Items kept and forwarded | `node.output.items` |\n| Items filtered | `processor.filter.dropped.items` |\n| Include/exclude configuration | Node configuration |\n\nA configured-path counter would be equivalent to repeating node input\ncounts with a static configuration flag. It would not indicate that\nfiltering completed or that any rule matched, so this change does not\ncarry that counter forward from #3268.\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n- Closes #3053\n- Closes #3649\n\n## Validation\n\nUnit tests\n\n## User-facing changes\n\nMigration: Rename `processor.filter.pdata.dropped.items` to\n`processor.filter.dropped.items`.",
          "timestamp": "2026-09-13T15:26:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3281b2eebe891fa9410a477b7a26631410f6cdf2"
        },
        "date": 1789320705365,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74565334300443,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18152323513493,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.183463541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.79296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98321.48456295629,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98321.48456295629,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003976,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107169.2450174848,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167642.3513596833,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.602139235681815,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.666853427886963,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83543827644381,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.31677424324116,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.28671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.71484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99985.10555034991,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98318.50046474116,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002217,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3200024.450780566,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3039054.209079537,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.5475310918534,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "3281b2eebe891fa9410a477b7a26631410f6cdf2",
          "message": "feat(filter_processor): Align metrics with universal node telemetry (#4045)\n\n# Change summary\n\n- Rename `processor.filter.pdata` to `processor.filter`\n- Improve test coverage for dropped metric emission\n\n[#3053](https://github.com/open-telemetry/otel-arrow/issues/3053)\npredates universal node input and output metrics. Its proposed\nfilter-specific counters answered whether batches arrived and how many\nitems survived, while configured-path counters indicated only that a\nbatch arrived while an `include` or `exclude` rule was configured.\n\nThose requirements are now covered without duplicating throughput\ninstrumentation:\n\n| Requirement | Metric or source |\n| --- | --- |\n| Batches and items received | `node.input.messages` and\n`node.input.items` |\n| Items kept and forwarded | `node.output.items` |\n| Items filtered | `processor.filter.dropped.items` |\n| Include/exclude configuration | Node configuration |\n\nA configured-path counter would be equivalent to repeating node input\ncounts with a static configuration flag. It would not indicate that\nfiltering completed or that any rule matched, so this change does not\ncarry that counter forward from #3268.\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n- Closes #3053\n- Closes #3649\n\n## Validation\n\nUnit tests\n\n## User-facing changes\n\nMigration: Rename `processor.filter.pdata.dropped.items` to\n`processor.filter.dropped.items`.",
          "timestamp": "2026-09-13T15:26:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3281b2eebe891fa9410a477b7a26631410f6cdf2"
        },
        "date": 1789350792656,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82109393998292,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27188683620622,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.885416666666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.95029858401,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.95029858401,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002878,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118083.2572268164,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167928.7725408492,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.711702228742354,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8879443040562,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33076626877165,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.554817708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.83984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.18585280394,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.18585280394,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003141,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190923.7431663764,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026923.245969664,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.45177072567116,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "06a5118d62b1ed9ed7c268e40c40f171758291a6",
          "message": "chore(deps): update otel/opentelemetry-collector-contrib docker tag to v0.160.0 (#4064)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[otel/opentelemetry-collector-contrib](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases)\n| final | minor | `0.158.0` → `0.160.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector-releases\n(otel/opentelemetry-collector-contrib)</summary>\n\n###\n[`v0.160.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/blob/HEAD/CHANGELOG.md#v01600)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/compare/v0.159.0...v0.160.0)\n\n##### 🚀 New components 🚀\n\n- `spanpruningprocessor`: Added missing spanpruningprocessor to\notelcol-contrib\n([#&#8203;1608](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1608))\n\n##### 💡 Enhancements 💡\n\n- `otelcol-contrib`: Upgrade go.opentelemetry.io/obi to v0.11.0.\n([#&#8203;1604](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1604))\n\n###\n[`v0.159.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/blob/HEAD/CHANGELOG.md#v01590)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/compare/v0.158.0...v0.159.0)\n\n##### 🛑 Breaking changes 🛑\n\n- `contrib`: Remove deprecated kafkatopicsobserver extension from the\ncontrib distribution\n([#&#8203;1597](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1597))\n\n##### 🚀 New components 🚀\n\n- `extension/aws_iam_db_auth`: Add `aws_iam_db_auth` to the contrib\ndistribution\n([#&#8203;1591](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1591))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC43OS4xIiwidXBkYXRlZEluVmVyIjoiNDQuNzkuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-09-14T18:16:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/06a5118d62b1ed9ed7c268e40c40f171758291a6"
        },
        "date": 1789415956650,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8128532478464,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14544554455446,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.9640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.52142555274,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.52142555274,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002326,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194218.9692171956,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033750.6837684237,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48484201802612,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7511409753293,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 67.066249616682,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.164973958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.71187681289,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.71187681289,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003227,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3120721.977038294,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166797.6032639279,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.739584043900273,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "a474e8eea1a453412af50247f009141a6052e947",
          "message": "Implement resource and scope metadata in the OTAP logs view (#3993)\n\n# Change summary\n\n`OtapLogsView` never implemented the resource- and scope-level metadata\naccessors. `name()`, `version()`, `schema_url()` and\n`dropped_attributes_count()` on the resource and scope views just\nreturned `None` or `0` behind `// TODO`s, even though `OtapTracesView`\nand `OtapMetricsView` already read these straight from the batch. So a\nlogs batch traversed through the OTAP view silently dropped the\ninstrumentation-scope name and version, the resource and scope schema\nURLs, and the resource/scope dropped-attribute counts.\n\nThis fills them in the same way the traces and metrics views already do.\nThe resource and scope struct columns (plus the root `schema_url`\ncolumn) are cached once when the view is built, and each\n`OtapResourceLogsView` / `OtapScopeLogsView` already holds the\n`RowGroup` for its id, so the accessors just read the shared value from\nthat group's first row. Lookups stay O(1) with no extra maps and no\nderived state to keep in sync.\n\nTwo details worth a look.\n\n- The `ScopeLogs` `schema_url` comes from the root logs batch column,\nand the resource `schema_url` from the resource struct column, matching\nthe traces view. (The old stub comment hinted at a scope-id lookup,\nwhich would have been wrong.)\n- Resource and scope ids are assigned monotonically per batch, so each\nid maps to exactly one group and that group's first row is the\nrepresentative row.\n\nI mirrored the trace (#3936) and metrics (#3964) view changes rather\nthan introducing anything new. Only the previously stubbed\nresource/scope metadata is filled in; the log-record fields are\nunchanged.\n\n## Related issue\n\nNo dedicated tracking issue. This is the logs counterpart to the trace\n(#3936) and metrics (#3964) view work.\n\n## Validation\n\nMirrors the already-merged `OtapTracesView` / `OtapMetricsView`\naccessors. The existing `views::otap::logs` unit tests exercise\nmulti-resource and multi-scope traversal, and CI runs the full\n`otap-df-pdata` test suite.\n\n## User-facing changes\n\nConsumers of the OTAP logs view now get real instrumentation-scope\n`name` and `version`, resource and scope `schema_url`, and\nresource/scope `dropped_attributes_count` instead of `None` or `0`. A\nchangelog entry is included\n(`.chloggen/implement-otap-logs-scope-resource-metadata.yaml`).\n\n---------\n\nSigned-off-by: Uros Stefanovic <uros.stefanovic@databricks.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-15T01:08:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a474e8eea1a453412af50247f009141a6052e947"
        },
        "date": 1789451055832,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84504284657694,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26412033806311,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.940885416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.33021807882,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96662.15254414285,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002802,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193179.101362061,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031356.856805916,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.03442989130443,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77801360107995,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16848823666433,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.970833333333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.87890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.43123789514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96656.8420120745,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002788,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116337.615461369,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165791.2176095895,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.24125215131767,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "db40c0f38670e4ec06741b9c2eee0d9040f7f4f7",
          "message": "Re-export arrow from the OTAP pdata crate (#4078)\n\nSometimes code built on top of pdata needs to look at the raw Arrow type\nof a record batch column and downcast it. That only works if it\ndowncasts against the same Arrow version pdata used to build the batch,\nand if the caller is on a different (usually newer) Arrow version it\njust fails.\n\nRight now there's no way to get at pdata's Arrow types from outside the\ncrate. You end up adding your own `arrow` dependency and hoping it stays\non the same version as pdata, which tends to break quietly the next time\npdata bumps Arrow.\n\nThis re-exports the `arrow` crate as\n`otel_arrow_dfe_pdata::otap::arrow`, so callers can reach the exact\nArrow that built the batches through pdata. It's a one-line `pub use`.\n\n---------\n\nSigned-off-by: Uros Stefanovic <uros.stefanovic@databricks.com>",
          "timestamp": "2026-09-15T15:53:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db40c0f38670e4ec06741b9c2eee0d9040f7f4f7"
        },
        "date": 1789507086410,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76143088153418,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09563094042191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.267447916666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.13671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.75651074323,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.75651074323,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003403,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110398.3897797978,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1164793.4733743547,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.63296408008615,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89771815063528,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1545020778821,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.106901041666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.91359206196,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.33169886093,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003052,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193297.695816971,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031171.5549747325,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.47586571077726,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "0ebbefc60e0c8c1cabb8c4cf532330201b31ee5a",
          "message": "feat(geneva_exporter): Migrate to shared exporter and enum attribute metrics (#4075)\n\n# Change summary\n\n- Replace Geneva's flat per-signal metrics with bounded enum-attribute\nmetric sets\n- Adopt `exporter.attempted` for each encoded Geneva batch, including\noutcome, duration, record count, and compressed application-payload size\n- Retain Geneva-specific encoding, failure classification, and\nempty-payload diagnostics\n\n## Related issue\n\n- Part of #3530\n- Part of #3822\n\n## Validation\n\nUnit tests\n\n## User-facing changes\n\nReplace `otap.exporter.geneva` queries with `exporter.attempted` for\nbatch uploads and `exporter.geneva.{encoding,failures,skipped}` for\ncomponent diagnostics. Duration values now use seconds.",
          "timestamp": "2026-09-15T23:47:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ebbefc60e0c8c1cabb8c4cf532330201b31ee5a"
        },
        "date": 1789524487682,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81845333060133,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18593617185688,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.929557291666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.35546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.09534866967,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.5270928585,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003543,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3180027.241601531,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3023693.8803118854,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.34116971739133,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79617132553403,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30425367362722,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.940234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.78515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.5717632898,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96658.99145325953,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003109,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3117100.5312960516,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167570.7471181601,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.24842805031086,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "0ebbefc60e0c8c1cabb8c4cf532330201b31ee5a",
          "message": "feat(geneva_exporter): Migrate to shared exporter and enum attribute metrics (#4075)\n\n# Change summary\n\n- Replace Geneva's flat per-signal metrics with bounded enum-attribute\nmetric sets\n- Adopt `exporter.attempted` for each encoded Geneva batch, including\noutcome, duration, record count, and compressed application-payload size\n- Retain Geneva-specific encoding, failure classification, and\nempty-payload diagnostics\n\n## Related issue\n\n- Part of #3530\n- Part of #3822\n\n## Validation\n\nUnit tests\n\n## User-facing changes\n\nReplace `otap.exporter.geneva` queries with `exporter.attempted` for\nbatch uploads and `exporter.geneva.{encoding,failures,skipped}` for\ncomponent diagnostics. Duration values now use seconds.",
          "timestamp": "2026-09-15T23:47:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ebbefc60e0c8c1cabb8c4cf532330201b31ee5a"
        },
        "date": 1789580378648,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72809696290363,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.05778258518117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.326692708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.01171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.00894298682,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96656.47207176681,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004673,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116032.4415031755,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166489.491268312,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.23821824563948,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82013778674241,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24095925494761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.916536458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.70988352412,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.70988352412,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002211,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3184979.990133224,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3022044.17754171,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.3908205760596,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "47c3a6060c73c6a522db756ee196b4cc4d9365b8",
          "message": "chore(docs): Improve shared Receiver and Exporter boundary metric cardinality documentation (#4079)\n\n# Chore summary\n\nImplementing the shared receiver and exporter metrics in the first few\nnodes exposed work shapes more complex than one PData message mapping to\none external message or submission. Fan-out, aggregation, many-to-many\nbatching, retries, generated data, and asynchronous buffering require\nnode authors to make explicit decisions about metric cardinality,\ntiming, and ACK/NACK ownership.\n\nThis adds that guidance to the telemetry metrics guide, with concise\npointers from the OTAP crate and operator documentation. It\ndistinguishes external work from PData work, logical batches from\nnode-local attempts, and pushed ingress from receiver-initiated\ngeneration.\n\n### Cardinality guidance\n\n`R = receiver.received.messages`, `O = node.output.messages`, `I =\nnode.input.messages`, and `A = exporter.attempted.messages`.\n\n| Work shape | Receiver boundary | Exporter boundary |\n| --- | --- | --- |\n| 1:1 | `R=1`, `O=1`: one external message produces one PData message. |\n`I=1`, `A=1`: one PData message produces one node-local delivery\nattempt. |\n| 1:N | `R=1`, `O=N`: record one local outcome for the external message.\n| `I=1`, `A=N`: record each external submission independently when\nfan-out succeeds. |\n| N:1 | `R=N`, `O=1`: record each external message independently. |\n`I=N`, `A=1`: record the external batch rather than each contributing\nPData message. |\n| N:M | `R=N`, `O=M`: track external-message-to-PData ownership\nindependently. | `I=N`, `A=M`: track logical batches for ACK/NACK and\nrecord each node-local attempt independently. |\n\nReceivers without independently classifiable external messages can have\n`R=0`, `O=N`; they omit `receiver.received` rather than inventing\nexternal messages.\n\nCurrent node mappings:\n\n- **OTLP receiver (#4044) -- 1:1:** each HTTP or gRPC request produces\none PData message, so `R=1` and `O=1`.\n- **OTLP HTTP and gRPC exporters (#4044) -- 1:1:** each PData message\nproduces one request, so `I=1` and `A=1`; preparation failures still\nrecord one attempt.\n- **Syslog receiver (#3899) -- N:1:** each framed TCP message or UDP\ndatagram records a received outcome, while several accepted messages can\nform one PData batch, so `R=N` and `O=1`.\n- **Azure Monitor exporter (#4059) -- N:M:** PData messages are\nregrouped across logical gzip batches, so ACK/NACK follows explicit\nPData-to-batch ownership while each physical submission and retry\nrecords an attempt.\n- **Geneva exporter (#4075) -- 1:N:** one PData message can produce\nseveral encoded upload batches, so `I=1` and `A=N`; sibling attempts own\nindependent sizes, item counts, and outcomes.\n\n## Related issue\n\n- Related to #3822",
          "timestamp": "2026-09-16T22:48:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/47c3a6060c73c6a522db756ee196b4cc4d9365b8"
        },
        "date": 1789609739313,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77114984577283,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18744006187161,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.596484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.97733457449,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.97733457449,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003065,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3112644.932610507,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167653.2715488586,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.65735026532776,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93320270666429,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.35141928494042,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.316015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.90234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.61273017233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.61273017233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003084,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187805.4577378468,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3022784.5149767124,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.420906101911655,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "be2903c6142e3745040af7cfe62b8df2891935c3",
          "message": "chore(deps): Resolve 3x remaining Pinned-Dependencies alerts (#4097)\n\n# Chore Summary\n\n- Install comparison dashboard dependencies from a hash-locked\nrequirements file\n- Pin the protobuf mock generator to the go.uber.org/mock v0.6.0 release\ncommit",
          "timestamp": "2026-09-17T18:51:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/be2903c6142e3745040af7cfe62b8df2891935c3"
        },
        "date": 1789676663381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78995362112273,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.03110561311273,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.825520833333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.7652740379,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.18585280394,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003141,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3196327.4603785397,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032990.348941953,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.50672665885855,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73247231320045,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15654671814671,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.647786458333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.89006139543,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96657.2930584835,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002508,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3105822.6795053035,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165222.0466143398,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.1323159508108,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "MuaazTasawar",
            "username": "MuaazTasawar",
            "email": "muaaztasawar6@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "7ff859021a9eef6f4bb3d4cd8e037fbb95fac4ac",
          "message": "fix(pdata): make exemplar time_unix_nano non-optional in OTAP encoder (#4076)\n\nFixes #4056.\n\n## Problem\nConverting an OTLP metric batch to OTAP fails when none of the exemplars\nin a batch have a non-zero timestamp. The panic:\n\nInvalidSchemaForPayload { payload_type: ExpHistogramDpExemplars, source:\nMissingRequiredFields { names: [time_unix_nano] } }\n\n## Root cause\nThe exemplars OTAP schema declares `time_unix_nano` as `required: true`,\nbut `ExemplarsRecordBatchBuilder` (shared by\nNumber/Histogram/ExpHistogram data point exemplars) configured the\nunderlying Arrow builder with `optional: true`. When every appended\ntimestamp is the proto3 default (`0`), the adaptive builder elides the\ncolumn entirely from the finished RecordBatch to save space -- which\nthen fails schema validation since the required column is missing\naltogether, not just null.\n\n## Fix\nConfigure `time_unix_nano` on `ExemplarsRecordBatchBuilder` as\nnon-optional, matching the schema, and update `finish()` to use the same\nunconditional pattern already used for the `parent_id` field in the same\nbuilder.\n\n## Testing\n- Added `test_exemplar_with_no_timestamp` using the exact repro from the\nissue.\n- Verified it fails with the original error on the pre-fix code\n(reverted just the metrics.rs change and reran).\n- Full crate test suite passes (723 tests, 0 failures).\n- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`\nboth clean.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-17T22:20:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7ff859021a9eef6f4bb3d4cd8e037fbb95fac4ac"
        },
        "date": 1789696143386,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84683793361314,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30402563711277,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.173828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.76360754576,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.18421408666,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003142,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187234.106357128,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028662.161497921,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.41424757135421,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74439989427299,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16816206603264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.9421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.67578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.85607591296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.85607591296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003139,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110590.895357644,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165203.1946705433,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.636498567088246,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "MuaazTasawar",
            "username": "MuaazTasawar",
            "email": "muaaztasawar6@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "7ff859021a9eef6f4bb3d4cd8e037fbb95fac4ac",
          "message": "fix(pdata): make exemplar time_unix_nano non-optional in OTAP encoder (#4076)\n\nFixes #4056.\n\n## Problem\nConverting an OTLP metric batch to OTAP fails when none of the exemplars\nin a batch have a non-zero timestamp. The panic:\n\nInvalidSchemaForPayload { payload_type: ExpHistogramDpExemplars, source:\nMissingRequiredFields { names: [time_unix_nano] } }\n\n## Root cause\nThe exemplars OTAP schema declares `time_unix_nano` as `required: true`,\nbut `ExemplarsRecordBatchBuilder` (shared by\nNumber/Histogram/ExpHistogram data point exemplars) configured the\nunderlying Arrow builder with `optional: true`. When every appended\ntimestamp is the proto3 default (`0`), the adaptive builder elides the\ncolumn entirely from the finished RecordBatch to save space -- which\nthen fails schema validation since the required column is missing\naltogether, not just null.\n\n## Fix\nConfigure `time_unix_nano` on `ExemplarsRecordBatchBuilder` as\nnon-optional, matching the schema, and update `finish()` to use the same\nunconditional pattern already used for the `parent_id` field in the same\nbuilder.\n\n## Testing\n- Added `test_exemplar_with_no_timestamp` using the exact repro from the\nissue.\n- Verified it fails with the original error on the pre-fix code\n(reverted just the metrics.rs change and reran).\n- Full crate test suite passes (723 tests, 0 failures).\n- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`\nboth clean.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-09-17T22:20:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7ff859021a9eef6f4bb3d4cd8e037fbb95fac4ac"
        },
        "date": 1789752495238,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79501817227296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1161996131528,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.076432291666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.94168453096,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.94168453096,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00329,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118863.41025231,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166845.9000614963,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.71899418233192,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.67520010471344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86053376177578,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2946500657335,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.119791666666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.79027142686,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.67754479991,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003126,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192236.5715660425,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3037859.98671855,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46793166211801,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "149d90e4a8a244906b085108a54ff66f1c166701",
          "message": "perf(batching): Optimize concatenation kernel's same_values function to avoid to_data calls (#4427)\n\n# Change summary\n\nSimilar to #4426, we have performance problems for small batch sizes\nduring concatenation by calling to_data which allocates. This PR\nimplements a new version of same_values which avoids to_data calls in\nmost circumstances.\n\n## Validation\n```\n➜  otap-dataflow git:(concatenate-same-values) ✗ cargo bench -p otel-arrow-dfe-pdata --bench concatenate --features testing 8items\n   Compiling otel-arrow-dfe-pdata v0.61.0 (/home/jakedern/repos/otel-arrow-concat/rust/otap-dataflow/crates/pdata)\n    Finished `bench` profile [optimized] target(s) in 1m 07s\n     Running benches/concatenate.rs (target/release/deps/concatenate-2e7e784795eff9b8)\nconcatenate/8items/1r1s/contiguous/metrics\n                        time:   [43.021 µs 43.355 µs 43.714 µs]\n                        change: [−52.076% −51.529% −50.860%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high severe\nconcatenate/8items/1r1s/contiguous/logs\n                        time:   [30.592 µs 30.838 µs 31.111 µs]\n                        change: [−52.694% −52.388% −52.093%] (p = 0.00 < 0.05)\n                        Performance has improved.\nconcatenate/8items/1r1s/contiguous/traces\n                        time:   [55.116 µs 55.742 µs 56.404 µs]\n                        change: [−51.304% −50.800% −50.344%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high severe\n\nconcatenate/8items/1r1s/presplit/logs\n                        time:   [79.293 µs 79.690 µs 80.066 µs]\n                        change: [−30.679% −30.248% −29.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 3 outliers among 100 measurements (3.00%)\n  1 (1.00%) high mild\n  2 (2.00%) high severe\nconcatenate/8items/1r1s/presplit/traces\n                        time:   [127.10 µs 127.59 µs 128.08 µs]\n                        change: [−31.710% −31.418% −31.124%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  7 (7.00%) high mild\n  1 (1.00%) high severe\n\nconcatenate/8items/3r2s/contiguous/metrics\n                        time:   [43.882 µs 44.151 µs 44.428 µs]\n                        change: [−52.588% −52.373% −52.165%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nconcatenate/8items/3r2s/contiguous/logs\n                        time:   [31.844 µs 31.960 µs 32.095 µs]\n                        change: [−52.148% −51.933% −51.704%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  4 (4.00%) high mild\n  3 (3.00%) high severe\nconcatenate/8items/3r2s/contiguous/traces\n                        time:   [54.389 µs 54.663 µs 54.959 µs]\n                        change: [−51.235% −50.891% −50.548%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 2 outliers among 100 measurements (2.00%)\n  2 (2.00%) high mild\n\nconcatenate/8items/3r2s/presplit/metrics\n                        time:   [171.24 µs 172.25 µs 173.19 µs]\n                        change: [−23.384% −22.632% −21.844%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 2 outliers among 100 measurements (2.00%)\n  2 (2.00%) high severe\nconcatenate/8items/3r2s/presplit/logs\n                        time:   [67.037 µs 67.328 µs 67.687 µs]\n                        change: [−33.677% −33.282% −32.763%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) high mild\n  1 (1.00%) high severe\nconcatenate/8items/3r2s/presplit/traces\n                        time:   [106.93 µs 107.25 µs 107.61 µs]\n                        change: [−37.502% −36.250% −35.480%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 15 outliers among 100 measurements (15.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  8 (8.00%) high mild\n  5 (5.00%) high severe\n```\n## User-facing changes\n\nBetter concatenation performance\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-10-16T22:38:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/149d90e4a8a244906b085108a54ff66f1c166701"
        },
        "date": 1792200824485,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92237834256083,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22855210574853,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.535416666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.24609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99984.99725116247,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96652.16400945705,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009003,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3199914.345574533,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3038298.179971743,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.10752923505607,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677448146045208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7653903346897,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12007399414213,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.300651041666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.95703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98333.32622264697,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.79339900764,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003584,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3113038.479426678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166483.1132472113,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.660768070915637,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "a9289e51a7568f6b1fe1920db106917b1762c0e0",
          "message": "feat(pdata): add codec-aware batching (#4347)\n\n# Change summary\n\n**PR3 of the five-PR pluggable PData codec series**, following #4042.\n\nAdds batching as an optional codec capability. Encoded telemetry can be\nmerged and split without first materializing OTAP, and new formats can\nprovide batching support without adding format-specific branches to the\nbatch processor.\n\n## How it works\n\n- **Codec-native batching:** `PdataBatcher` produces independently\ndecodable batches. OTLP reuses the existing protobuf batching\nalgorithms.\n- **OTAP fallback:** `BatchPlan` resolves representations and sizing at\nstartup. Compatible item-based policies use native OTAP batching when\ncodec-native batching is unavailable; unsupported combinations are\nrejected.\n- **Explicit delivery ownership:** outputs identify their contributing\ninput ranges, including splits and merges. Inputs consumed without\noutput are accounted for separately. Ack/Nack tracking no longer depends\non additive byte or item counts; the processor retains delivery\ncontexts, buffering, flush deadlines, and backpressure.\n\nThe **purple batching path** extends the framework introduced by the\npreceding PRs:\n\n![PData forwarding, views, conversion, batching, and codec\nlifecycle](https://raw.githubusercontent.com/lquerel/otel-arrow/3a899d7b056a3787a7a82693b470f7b622245e2c/rust/otap-dataflow/crates/pdata-codec/assets/pdata-codec-framework.png)\n\nUnchanged forwarding retains the original bytes without creating a codec\ninstance. Encoded batching may create new buffers and reuse a batcher\ninstance, while avoiding OTAP materialization.\n\n## Remaining series\n\n- **PR4:** activate encoded OTLP, remove transitional storage, and\nexpose pipeline decode-policy configuration.\n- **PR5:** activate the syslog codec, enabling batching before syslog\nparsing and Arrow construction.\n\n## Related issue\n\n- Advances #3452.\n\n## Validation\n\n`cargo xtask check` passed \n\n### Full-engine benchmarks\n\n**No broad CPU or RSS regression is apparent.** CPU point estimates\ndiffer by less than 1% in every scenario.\n\n| Scenario | Pairs | Main CPU (us/log) | PR3 CPU (us/log) | CPU change\n(95% paired CI) | Main RSS (MiB) | PR3 RSS (MiB) |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n| OTAP → OTAP | 4 | 1.7237 | 1.7188 | -0.29% [-2.53, +1.96] | 56.94 |\n57.15 |\n| OTLP → OTLP | 4 | 4.6525 | 4.6675 | +0.32% [-0.10, +0.74] | 45.41 |\n45.48 |\n| Syslog → OTAP | 4 | 6.8262 | 6.8113 | -0.22% [-1.31, +0.87] | 65.97 |\n66.09 |\n| HTTP gzip, 128 KiB | 4 | 6.3348 | 6.3260 | -0.14% [-0.60, +0.33] |\n53.33 | 53.20 |\n| HTTP gzip, 240 KiB | 4 | 6.2564 | 6.2589 | +0.04% [-0.28, +0.36] |\n53.20 | 52.17 |\n| HTTP gzip, 272 KiB | 4 | 6.1621 | 6.1571 | -0.08% [-0.40, +0.23] |\n52.84 | 51.66 |\n| HTTP gzip, 512 KiB | 4 | 6.0203 | 6.0353 | +0.25% [-0.93, +1.43] |\n54.33 | 53.11 |\n| HTTP gzip, 1 MiB | 4 | 5.5714 | 5.5664 | -0.09% [-2.30, +2.12] | 56.15\n| 54.67 |\n| HTTP gzip, 4 MiB | 4 | 4.5040 | 4.5427 | +0.86% [+0.49, +1.23] | 63.45\n| 61.55 |\n| gRPC gzip, 128 KiB | 4 | 6.2685 | 6.2860 | +0.28% [-0.21, +0.77] |\n51.42 | 50.81 |\n| gRPC gzip, 240 KiB | 4 | 6.2314 | 6.2314 | +0.00% [-0.18, +0.18] |\n52.66 | 51.00 |\n| gRPC gzip, 272 KiB | 4 | 6.1483 | 6.1521 | +0.06% [-0.39, +0.51] |\n52.69 | 50.96 |\n| gRPC gzip, 512 KiB | 4 | 6.0066 | 6.0141 | +0.13% [-0.37, +0.62] |\n54.00 | 52.22 |\n| gRPC gzip, 1 MiB | 4 | 5.6389 | 5.6014 | -0.66% [-2.21, +0.89] | 56.19\n| 54.40 |\n| gRPC gzip, 4 MiB | 1 | 4.5053 | 4.5227 | +0.39% (CI unavailable: one\npair) | 65.22 | 63.38 |\n\n## User-facing changes\n\nAdds per-codec batching profiles and output encoding selection. Existing\nOTLP and OTAP configurations remain compatible. In preserve mode, native\nbatching retains the encoding; supported fallback paths emit OTAP.",
          "timestamp": "2026-10-17T07:28:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9289e51a7568f6b1fe1920db106917b1762c0e0"
        },
        "date": 1792257248733,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87563027284399,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26596231078159,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.629947916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.92578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.32186652886,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.32186652886,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003058,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191857.920851338,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032024.821279817,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46122643264445,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76816090384732,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2248600154679,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.404427083333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.96484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98332.1821342739,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98332.1821342739,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002655,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3117570.555021929,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166419.7922423438,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.70447850699423,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "362999dfe64598e82ee1738f3da463d854f3ebcb",
          "message": "perf(batch_processor): Update logical bytes implementation to avoid to_data calls (#4426)\n\n# Change summary\n\nThis PR improves performance of the batch processor for low batch sizes\nby eliminating to_data calls on arrow arrays for the purpose of sizing\nbatches.\n\n`to_data` is invoked _per column_ and allocates two vectors on every\ncall. This is cost-prohibitive for large numbers of small record batches\nand is a significant chunk of the profile on these benches.\n\n## Validation\n\nnum_bytes before:\n<img width=\"1910\" height=\"482\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/af676678-0316-4466-ad7c-458d49f10d31\"\n/>\n\nnum_bytes after:\n\n<img width=\"1909\" height=\"379\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/47829995-f75c-4f34-8ef4-48f931de75a0\"\n/>\n\nBenchmark results:\n\n```\nbatch_processor/logs/8  time:   [24.965 ms 25.482 ms 26.019 ms]\n                        thrpt:  [314.84 Kelem/s 321.48 Kelem/s 328.13 Kelem/s]\n                 change:\n                        time:   [−19.743% −17.417% −14.850%] (p = 0.00 < 0.05)\n                        thrpt:  [+17.440% +21.090% +24.600%]\n                        Performance has improved.\nbatch_processor/traces/8\n                        time:   [36.589 ms 37.072 ms 37.578 ms]\n                        thrpt:  [218.00 Kelem/s 220.98 Kelem/s 223.89 Kelem/s]\n                 change:\n                        time:   [−35.221% −33.530% −31.697%] (p = 0.00 < 0.05)\n                        thrpt:  [+46.407% +50.445% +54.371%]\n                        Performance has improved.\nbatch_processor/metrics/8\n                        time:   [31.795 ms 32.171 ms 32.549 ms]\n                        thrpt:  [251.68 Kelem/s 254.64 Kelem/s 257.65 Kelem/s]\n                 change:\n                        time:   [−22.287% −20.927% −19.540%] (p = 0.00 < 0.05)\n                        thrpt:  [+24.285% +26.465% +28.678%]\n                        Performance has improved.\n```\n\n## User-facing changes\n\nBetter performance.",
          "timestamp": "2026-10-18T00:27:52Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/362999dfe64598e82ee1738f3da463d854f3ebcb"
        },
        "date": 1792287478141,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77096463667034,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09362392766056,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.363411458333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.97265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.11785251586,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.11785251586,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003386,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118414.2775796833,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168732.2895967464,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.715337298219083,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.7034454345703125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.95075162051215,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.49502251203229,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.094010416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.70492099636,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96654.71208969304,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007421,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3188998.8428089367,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030106.6319823563,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.99372347050839,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "f3215c1c0d729d9de2442116c073fc79455a2bf6",
          "message": "[query-engine] Add ExecutionContext to columnar engine (#4430)\n\nRelates to #3554\n\n# Changes\n\n* Define `ExecutionContext`\n\n# Details\n\n`ExecutionContext` is the state which is passed around during query\nexecution.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-10-18T08:51:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f3215c1c0d729d9de2442116c073fc79455a2bf6"
        },
        "date": 1792345543999,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6751017570495605,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.70606880502834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11311254168929,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.442708333333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.02734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 100000.6799647325,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.5668472662,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003112,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3121769.196246067,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167548.2402222934,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.74931298484412,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89474935065202,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19590233382115,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.0265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.43359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.62341777585,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.62341777585,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004298,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3195957.2621383336,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032824.282942441,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.50446994369554,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "f3215c1c0d729d9de2442116c073fc79455a2bf6",
          "message": "[query-engine] Add ExecutionContext to columnar engine (#4430)\n\nRelates to #3554\n\n# Changes\n\n* Define `ExecutionContext`\n\n# Details\n\n`ExecutionContext` is the state which is passed around during query\nexecution.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-10-18T08:51:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f3215c1c0d729d9de2442116c073fc79455a2bf6"
        },
        "date": 1792373826786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677448146045208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.71454946690982,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27347039727933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.043359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.6328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98331.80217075002,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.2694793595,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004514,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3119323.8087473037,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167702.3409381956,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.7251839291423,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88399208129817,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.37580453104461,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.111067708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.8984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98318.59969719716,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96652.13756098335,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007364,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192256.142078409,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033143.687970347,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.02830359094989,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "f3215c1c0d729d9de2442116c073fc79455a2bf6",
          "message": "[query-engine] Add ExecutionContext to columnar engine (#4430)\n\nRelates to #3554\n\n# Changes\n\n* Define `ExecutionContext`\n\n# Details\n\n`ExecutionContext` is the state which is passed around during query\nexecution.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-10-18T08:51:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f3215c1c0d729d9de2442116c073fc79455a2bf6"
        },
        "date": 1792430145871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6864750385284424,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73636593726995,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08823711021921,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.161588541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.05859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98313.5236141642,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96655.49050893207,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003627,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3114754.0759837525,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166061.0856422326,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.22531963350715,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.7241379022598267,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.94560804865323,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.41169335499343,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.08671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96663.07724439832,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.68202447415,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002228,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3189221.392510674,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027235.948828175,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.43396426032253,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "c3ff42082802a6eb6fc586e153f6f3ab55145b9b",
          "message": "chore(deps): update otel/opentelemetry-collector-contrib docker tag to v0.163.1 (#4445)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[otel/opentelemetry-collector-contrib](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases)\n| final | minor | `0.161.0` → `0.163.1` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector-releases\n(otel/opentelemetry-collector-contrib)</summary>\n\n###\n[`v0.163.1`](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/blob/HEAD/CHANGELOG.md#v01631)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/compare/v0.162.0...v0.163.1)\n\n###\n[`v0.162.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/blob/HEAD/CHANGELOG.md#v01620)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/compare/v0.161.0...v0.162.0)\n\n##### 🛑 Breaking changes 🛑\n\n- `otelcol-contrib`: Remove `wavefrontreceiver` from the contrib\ndistribution after its deprecation period.\n([#&#8203;1673](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1673))\n\n##### 🚀 New components 🚀\n\n- `otelcol-contrib`: Add `sdnotifyextension` to the contrib\ndistribution.\n([#&#8203;49607](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/49607))\n\n- `otelcol-prometheus`: Create the new otelcol-prometheus distribution,\nwhich includes \"Prometheus exporters\" as metric receivers.\n([#&#8203;1618](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1618))\nThe new otelcol-prometheus distribution is a collaboration between the\nPrometheus and OpenTelemetry communities.\nExisting [Prometheus\nexporters](https://prometheus.io/docs/instrumenting/exporters/) are\nslowly being refactored to be usable as Collector receivers in custom\nCollector distributions built with OCB.\n\nAs more Prometheus exporters are refactored to be usable as Collector\nreceivers, we will add them to the otelcol-prometheus distribution.\n\nThe otelcol-prometheus distribution is experimental, and is not\nrecommended for production use.\n\n[Build a custom Collector\ndistribution](https://opentelemetry.io/docs/collector/extend/ocb/) if\nyou need components from other OpenTelemetry Collector distributions\nsuch as contrib.\n\n- `receiver/blackbox`: Add the [Prometheus\nblackbox_exporter](https://redirect.github.com/prometheus/blackbox_exporter)\nas a receiver for probe metrics in otelcol-prometheus distribution.\n([#&#8203;1664](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1664))\n\n- `receiver/postgres`: Add the [Prometheus\npostgres_exporter](https://redirect.github.com/prometheus-community/postgres_exporter)\nas a receiver for PostgreSQL metrics in otelcol-prometheus distribution.\n([#&#8203;1664](https://redirect.github.com/open-telemetry/opentelemetry-collector-releases/issues/1664))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4xNjQuMSIsInVwZGF0ZWRJblZlciI6IjQ0LjE2NC4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-10-20T00:55:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c3ff42082802a6eb6fc586e153f6f3ab55145b9b"
        },
        "date": 1792467591567,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84377943859812,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18761037140854,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.814322916666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.71316106046,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.10785324586,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002209,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3184363.3185152304,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3023075.0926756067,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.94290230508352,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677448146045208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75427107517888,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27010742715821,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.990755208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98331.9660450313,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.43333942066,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004414,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3119204.2550672293,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168663.4996209086,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.723915135260555,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "adgrieco",
            "username": "adgrieco",
            "email": "157547602+adgrieco@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "932c632341a048d5e4382d326df63338d44a1794",
          "message": "chore(windows_perf_counters): accept metric definitions as written (#4433)\n\n# Chore Summary\n\nThe first config PRs applied OTel SDK instrument-name limits and\ntrimming rules to metric definitions. Neither OTLP nor Go's receiver\nimposes these rules, so valid configs were rejected or silently changed.\nThis PR corrects that by relaxing the checks so that valid metric\ndefinitions are accepted and kept as written.\n\nFor example, valid configs rejected or changed before this PR:\n\n- (rejected) a definition named `% Processor Time`, because it doesn't\nstart with an ASCII letter.\n- (rejected) the unit `µs`, because it isn't ASCII.\n- (rejected) definitions `cpu.time` and `CPU.time` as duplicates, even\nthough lookup is case-sensitive.\n- (changed) a definition named `\"cpu.time \"` is published as\n`\"cpu.time\"`, because names, units and descriptions are trimmed.\n\nNative object, instance and counter names are unchanged here and will be\naddressed in a follow-up.\n\n- **Definition names:** accept any nonempty name as written, including\nUnicode, long, path-shaped, whitespace-only, padded and\ncontrol-character names. Go: same.\n- **`metric` references:** kept as written and looked up exactly and\ncase-sensitively. An omitted or empty reference uses the counter path.\nGo: same.\n- **Units:** kept as written. An omitted or empty unit defaults to\n`\"1\"`. Go: [intends the same\ndefault](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/c766745fdaed961d2559b1e2d5d9c468c85d3574/receiver/windowsperfcountersreceiver/config.go#L86-L88),\nbut leaves defined metrics with an empty unit.\n- **Descriptions:** kept as written. Go: same.\n- **`metrics: null`:** treated as no definitions. Go: same.\n- **Definition count:** no cap. Go: same. The receiver's existing 256\ncounter-path budget is unchanged.\n- **Case-only variants:** kept distinct, with one warning per group\nnaming every spelling. Go: kept distinct, no warning.\n- **Unused path-shaped definitions:** one warning each when no counter\nselects it, explicitly or by path. Go: silently ignored. Unused\ndefinitions emit nothing.\n- **Docs:** config documentation moved from the module header onto each\nfield.\n\n## Related issue\n\nPart of #4074.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCopilot-Session: 1699f46c-a707-4cd1-9a9e-a9a508d6dd81",
          "timestamp": "2026-10-20T16:06:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/932c632341a048d5e4382d326df63338d44a1794"
        },
        "date": 1792516560853,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91958365390474,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21814089717898,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.120442708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.4533516229,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96660.8863456632,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003588,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187763.073899935,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029298.9255682286,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.978831401362974,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6751017570495605,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82475347326518,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22121994588326,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.675390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.32421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 100000.75329481486,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.63894899508,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003068,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116949.5247756676,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167466.0519906206,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.70027225953281,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "adgrieco",
            "username": "adgrieco",
            "email": "157547602+adgrieco@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dc1d258f32c442ba860d53b1d0fb929da7514bfe",
          "message": "chore(windows_perf_counters): delegate raw-fraction formatting to PDH (#4456)\n\n# Chore Summary\n\nLet PDH check denominators for raw-fraction counters.\n\nPreviously, the receiver rejected raw-fraction samples with a zero or\nnegative base before asking PDH to format them. That check dropped valid\ndata: PDH defines a zero base as a value of 0. To match PDH (and the Go\nreceiver), this change removes the check and uses PDH's formatted result\nand status for any base.\n\n## Related issue\n\nPart of #4074.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCopilot-Session: 848be878-de17-4206-8418-20986d579f32",
          "timestamp": "2026-10-20T23:02:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dc1d258f32c442ba860d53b1d0fb929da7514bfe"
        },
        "date": 1792546816884,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.675146460533142,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74315461883407,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.04430426716141,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.959895833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.85546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99997.76678800452,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.65780225942,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00326,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115621.1670238334,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167106.049936838,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.68772322336711,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86504360873482,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27722084367245,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.511067708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.85546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.13013644695,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96661.55165955801,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003175,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3197827.156558504,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3034712.2106209993,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.08272111978143,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "d1a8dcf20f1b2d524fe5200d05058e60e61baf22",
          "message": "chore(deps): bump aiohttp from 3.14.3 to 3.14.4 in /tools/pipeline_perf_test/orchestrator (#4455)\n\nBumps [aiohttp](https://github.com/aio-libs/aiohttp) from 3.14.3 to\n3.14.4.\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=aiohttp&package-manager=pip&previous-version=3.14.3&new-version=3.14.4)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\n---------\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@microsoft.com>",
          "timestamp": "2026-10-21T16:21:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d1a8dcf20f1b2d524fe5200d05058e60e61baf22"
        },
        "date": 1792602912061,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73725085622031,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20905403313206,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.041796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.74467504154,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.74467504154,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004224,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116058.7623862238,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168026.7607721165,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.69182350290614,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.840257115624,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20252917987168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.275520833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.97445872534,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.97445872534,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00327,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3197379.7739395523,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3037153.691261796,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.51749862173456,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "adgrieco",
            "username": "adgrieco",
            "email": "157547602+adgrieco@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c78d87ab8939ca74ae62abe09ab6aa0903388201",
          "message": "chore(windows_perf_counters): preserve exact native path spelling (#4489)\n\n# Chore Summary\n\nThe receiver uses each counter path exactly as configured, without\ntrimming, rewriting, or respelling it when it looks up the metric\ndefinition.\n\nThis matters because some Windows counter names end in a space. For\nexample, `\\Hyper-V Virtual Switch Port(*)\\IPsec SAs Offloaded ` fails to\nmatch without that trailing space, so trimming breaks it.\n\nThe receiver still treats equivalent selectors as duplicates. `app` and\n`app#0` conflict, but `app` and `app#1` are distinct.\n\n## Related issue\n\nPart of #4074.\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: cd7fcffe-9884-47b1-bf3e-5ee928b4d950",
          "timestamp": "2026-10-21T23:54:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c78d87ab8939ca74ae62abe09ab6aa0903388201"
        },
        "date": 1792633422942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677448146045208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72884169506247,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11607384903014,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.281119791666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98333.38521915404,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.85239039532,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003548,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116917.820176246,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166651.4023794802,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.700203401278806,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9055709622518,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22626671820642,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.9484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.40234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.00234704585,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.00234704585,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002236,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187425.115627603,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030576.9202096644,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.416579775081146,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "06cdc5979281969f38e26fcded2de49f0609941c",
          "message": "[query-engine] Columnar Engine scalar scaffolding (#4463)\n\nRelates to #3554\n\n# Changes\n\n* Adds scaffolding for scalar expressions\n* Adds test helpers\n* Implements `TemporalScalarExpression::Now`\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-10-22T16:46:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/06cdc5979281969f38e26fcded2de49f0609941c"
        },
        "date": 1792690400862,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6641331911087036,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.71709611333772,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20344238318836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.337109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99985.50927402296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98321.61728834437,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003895,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3123601.3260423774,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166825.6313036021,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.769222396758398,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008678201586008072,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82775733033867,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22469196877174,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.047135416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.51171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.46361088393,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98316.93072908226,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003175,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3200243.1064048186,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3045511.7564731534,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.550274735724464,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "b23c96ab7755630646705478fff90f567251ef63",
          "message": "chore(filelog): add receiver settings and validation (#4487)\n\n## Summary\n\n  Add configuration support for the Filelog receiver.\n\nThe receiver will use these settings to decide which files to read, how\nto\n  handle log records, and how to save reading progress.\n\n  This PR:\n\n  - Defines supported settings and their default values.\n  - Checks invalid values and conflicting settings.\n  - Reports errors that identify the setting and explain the problem.\n  - Prepares file include/exclude rules once for reuse during scans.\n  - Chooses a stable checkpoint ID for saved reading progress.\n  - Checks size limits and resource calculations.\n  - Reuses existing framing, multiline, and checkpoint code.\n  - Updates the spec to explain that relative file patterns need a \nbase directory supplied by the code configuring Filelog.\n\nThis PR prepares configuration for the remaining Filelog work. It does\nnot\nstart reading files or connect the receiver to a pipeline. Some settings\nwill\n  take effect only after the related runtime components are connected.\n\n  ## Related issue\n\n  Part of #2844 (Task 4).",
          "timestamp": "2026-10-23T00:48:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b23c96ab7755630646705478fff90f567251ef63"
        },
        "date": 1792719827534,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678954094648361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76356343522515,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13259590201818,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.199609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98317.17814755469,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.7110508297,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003024,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3119799.671546337,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167160.9308767521,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.729235804189095,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87564686252016,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.36560759106213,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.362760416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.96134904496,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.96134904496,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003278,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3196689.2581737316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3040311.219280327,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.51048037929021,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "4431586643ed3713edbe2a902fc8e7d3079cc449",
          "message": "perf(syslog): accelerate RFC 3164 timestamp conversion (#4479)\n\n# Change summary\n\nAccelerates syslog receipt-year capture and RFC 3164 timestamp\nconversion, including RFC 3164-wrapped CEF, following #4439:\n\n- Parses common 15-byte timestamps directly, avoiding temporary strings\nand generic timestamp parsing.\n- Keeps the legacy parser for previously accepted forms such as\nmixed-case months, whitespace padding, and leap seconds.\n- Avoids unnecessary timezone conversion when capturing receipt years,\nwhile preserving year-boundary, local-timezone, and DST behavior. No\ntimestamp or offset cache is introduced.\n\nRFC 5424 and raw CEF parsing are unchanged. PR5's batch limits remain\n256 logs / 256 KiB; larger batches and cooperative codecs are separate\nwork.\n\nCodec configuration follows in PR6.\n\n## Related issue\n\n- Advances #3452.\n\n## Validation\n\n`cargo xtask check` and Clippy passed. \n\n### Full-engine benchmarks\n\nCompares upstream main against **PR5.1 only**, using identical release\nsettings and pipeline configuration.\nTCP syslog -> SUT receiver / batch / zstd OTAP exporter -> DFE backend,\nat 102,400 logs/s on one otherwise idle pinned CPU.\n\n| Input | Main CPU (us/log) | PR5.1 CPU (us/log) | CPU change (95%\npaired CI) | Mean RSS, main -> PR5.1 (MiB) |\n| --- | ---: | ---: | ---: | ---: |\n| RFC 3164 | 6.5519 | 6.3275 | -3.42% [-3.56%, -3.29%] | 43.29 -> 43.32\n|\n| RFC 5424 | 6.3525 | 6.3475 | -0.08% [-0.33%, +0.17%] | 43.08 -> 43.42\n|\n\n**Important Note**: The earlier ~35% CPU reduction was measured with 8K\nbatches and included both PR5 and PR5.1. PR5 now limits synchronous\nsyslog batches to 256 logs / 256 KiB to protect pipeline responsiveness,\nwhich reduces the opportunity to amortize Arrow construction and export\ncosts. PR5.1 therefore keeps the timestamp optimization without relaxing\nthose limits. I intend to recover some of the larger-batch gains through\ncooperative codec execution: processing bounded chunks and yielding to\nI/O while retaining a larger logical batch, rather than monopolizing the\npipeline thread.\n\n## User-facing changes\n\nLower CPU usage for syslog ingestion and RFC 3164 timestamp conversion.\nNo configuration or protocol changes. A changelog entry is included.",
          "timestamp": "2026-10-23T04:30:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4431586643ed3713edbe2a902fc8e7d3079cc449"
        },
        "date": 1792775767899,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81040125798526,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18997303751637,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.1140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.71875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.25467897917,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.25467897917,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003099,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3117322.77874787,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166563.0406116913,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.703224967485347,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.913388664468,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22040765904879,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.364713541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.00390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.9564628428,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.9564628428,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002264,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192790.3321261504,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3025288.092219402,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.471159964487335,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "e810613034e00e37795747cc74d63b236e5cb3f8",
          "message": "chore(dependencies): update ctutils to version 0.4.4 to fix cryptographic timing side channel (#4518)\n\n# Change summary\n\nUpdate the locked `ctutils` dependency from `0.4.3` to `0.4.4` to\nresolve\nRUSTSEC-2026-0385: non-constant-time mask generation and `u128`\ncomparisons.\n\nNo unrelated dependencies are updated, and no advisory exceptions are\nadded.\n\n## Related issue\n\n* Closes #4517\n\n## Validation\n\nPassed from `rust/otap-dataflow`:\n\n* `cargo deny --locked --all-features check` - advisories, bans,\nlicenses, and sources.\n* `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0\nCARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=4 cargo xtask check` - structure\nchecks, formatting, Clippy, and workspace tests.\n\n## User-facing changes\n\nFix a potential cryptographic timing side channel in a transitive\ndependency.\nNo configuration changes are required.\n\nChangelog entry: `rust/otap-dataflow/.chloggen/ctutils-timing-fix.yaml`.",
          "timestamp": "2026-10-24T00:03:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e810613034e00e37795747cc74d63b236e5cb3f8"
        },
        "date": 1792805580565,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.017352920025587082,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84132530665859,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22674869270993,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.714583333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.20703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98345.1500080754,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.08425243566,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003203,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115059.773236045,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166903.3163179671,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.68026507298583,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678954094648361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8770320328304,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19574409115405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.19453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98317.16012365042,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.69302536114,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003035,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3196021.287014407,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035135.9447194785,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.504436924640416,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "15a01decec7ec2ebf5e68fc85542f0a69ef3c548",
          "message": "ProtoBuffer, BoundedBuf pdata/otlp/common.rs, telemetry/self_tracing/encoder.rs cleanup (#4492)\n\n# Change summary\n\nPart of #4482.\n\nThe addition of truncate support in this code left a mess. This PR adds\nnew types `Truncation`, `TruncateError` and distinguishes callers that\ntruncate from those that do not w/o use of `Result<bool, _>`. The\nresulting code makes it clear where truncation is happening, thus makes\nit clear when rollback is needed for incomplete fields.\n\nCorrects the same issues pointed out by reviewers in #4450. \n\nRemoves some dead code. \n\nRenames things for legibility.\n\n## Related issue\n\n#4450, #4453.\n\n## Validation\n\nSome new tests. No perf regressions, see commit message\nhttps://github.com/open-telemetry/otel-arrow/pull/4492/commits/eaf3c36dbc8799f7d530428bb9506768c7798089.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: 8dc081e5-0d89-492c-84a3-d8f71f60c585\nCopilot-Session: 3822bd71-36ab-4d7d-81b2-e3424f1ce583",
          "timestamp": "2026-10-24T01:04:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/15a01decec7ec2ebf5e68fc85542f0a69ef3c548"
        },
        "date": 1792861993360,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.017358379438519478,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.71037323540133,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11899930507296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.177604166666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98314.7704977959,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98331.83634928372,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002866,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115221.4822606174,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167414.0988479895,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.6807007569254,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.0027119379956275225,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87544392370339,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.31235121347967,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.076302083333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.65002301322,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.31658075101,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002451,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3189242.6935592587,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026649.0386692197,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.43430143176228,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "15a01decec7ec2ebf5e68fc85542f0a69ef3c548",
          "message": "ProtoBuffer, BoundedBuf pdata/otlp/common.rs, telemetry/self_tracing/encoder.rs cleanup (#4492)\n\n# Change summary\n\nPart of #4482.\n\nThe addition of truncate support in this code left a mess. This PR adds\nnew types `Truncation`, `TruncateError` and distinguishes callers that\ntruncate from those that do not w/o use of `Result<bool, _>`. The\nresulting code makes it clear where truncation is happening, thus makes\nit clear when rollback is needed for incomplete fields.\n\nCorrects the same issues pointed out by reviewers in #4450. \n\nRemoves some dead code. \n\nRenames things for legibility.\n\n## Related issue\n\n#4450, #4453.\n\n## Validation\n\nSome new tests. No perf regressions, see commit message\nhttps://github.com/open-telemetry/otel-arrow/pull/4492/commits/eaf3c36dbc8799f7d530428bb9506768c7798089.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: 8dc081e5-0d89-492c-84a3-d8f71f60c585\nCopilot-Session: 3822bd71-36ab-4d7d-81b2-e3424f1ce583",
          "timestamp": "2026-10-24T01:04:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/15a01decec7ec2ebf5e68fc85542f0a69ef3c548"
        },
        "date": 1792892275747,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667555570602417,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72342329842172,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14193390452876,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.003125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.95703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99989.99713380051,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.40829683254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002802,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118403.05552154,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166864.5954496781,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.715774600767155,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.7241379022598267,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90288253635755,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22072799877131,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.587890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96663.12718515957,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.73282628301,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002197,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3189858.2663075915,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033109.207903254,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.44042442323162,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "15a01decec7ec2ebf5e68fc85542f0a69ef3c548",
          "message": "ProtoBuffer, BoundedBuf pdata/otlp/common.rs, telemetry/self_tracing/encoder.rs cleanup (#4492)\n\n# Change summary\n\nPart of #4482.\n\nThe addition of truncate support in this code left a mess. This PR adds\nnew types `Truncation`, `TruncateError` and distinguishes callers that\ntruncate from those that do not w/o use of `Result<bool, _>`. The\nresulting code makes it clear where truncation is happening, thus makes\nit clear when rollback is needed for incomplete fields.\n\nCorrects the same issues pointed out by reviewers in #4450. \n\nRemoves some dead code. \n\nRenames things for legibility.\n\n## Related issue\n\n#4450, #4453.\n\n## Validation\n\nSome new tests. No perf regressions, see commit message\nhttps://github.com/open-telemetry/otel-arrow/pull/4492/commits/eaf3c36dbc8799f7d530428bb9506768c7798089.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: 8dc081e5-0d89-492c-84a3-d8f71f60c585\nCopilot-Session: 3822bd71-36ab-4d7d-81b2-e3424f1ce583",
          "timestamp": "2026-10-24T01:04:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/15a01decec7ec2ebf5e68fc85542f0a69ef3c548"
        },
        "date": 1792948550893,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6669422388076782,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74042318479583,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14391599752933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.646484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99978.32778061874,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98311.7467762157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003084,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3119747.6117909057,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167280.644297461,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.73321311127042,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.7062286138534546,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86127998437239,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.29766401973633,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.881640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96676.07334747828,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.58815003872,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003099,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191778.393963126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3037856.4672591076,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46132013055107,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "adgrieco",
            "username": "adgrieco",
            "email": "157547602+adgrieco@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "cf6acdc8ed9f0d2ed799cb5728d5895b994109e2",
          "message": "chore(windows_perf_counters): add worker lifecycle, recovery, and diagnostics  (#4510)\n\n# Chore Summary\n\nAdds the background worker, recovery, diagnostics, and Windows-native\ntests for the performance-counter receiver.\n\nThe receiver still runs native counter calls on a dedicated worker\nthread, and it is not registered or enabled. This PR makes collection\nsurvive failures: failed collections are retried, queries are rebuilt\nwhen needed, and counter state carries over. Operators get clear logs\nfor failures, limit hits, membership changes, and recoveries.\n\n## Commits\n\n1. **Worker and lifecycle** (f96ac30f3): Starts, stops, and cancels the\nworker, and owns the native query. Retries and rebuilds keep counter\nstate and do not emit partial data.\n2. **Diagnostics** (dfc1b1aec): Logs failures, limit hits, membership\nchanges, and recoveries. Ordinary counter warnings are logged once per\nepisode.\n3. **Live tests and native CI** (919dc8df1): Adds Windows-native tests\nfor collection, lifecycle, recovery, and projection. CI runs them\nserially and fails if a command fails or runs no tests.\n\n## Related issue\n\nPart of #4074.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: b57921b8-61e5-4fac-afae-a18d40bd60ae",
          "timestamp": "2026-10-25T18:17:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf6acdc8ed9f0d2ed799cb5728d5895b994109e2"
        },
        "date": 1792978543535,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80453932998597,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26053755183536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.011328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.47883224771,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.9097432012,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003513,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3119180.302506676,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167980.1283676566,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.723195176615512,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91216361969437,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3664222737819,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.542838541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.09375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.49146829905,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.49146829905,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003158,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3189712.251683655,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035648.2528662113,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.4403387570358,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "adgrieco",
            "username": "adgrieco",
            "email": "157547602+adgrieco@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "cf6acdc8ed9f0d2ed799cb5728d5895b994109e2",
          "message": "chore(windows_perf_counters): add worker lifecycle, recovery, and diagnostics  (#4510)\n\n# Chore Summary\n\nAdds the background worker, recovery, diagnostics, and Windows-native\ntests for the performance-counter receiver.\n\nThe receiver still runs native counter calls on a dedicated worker\nthread, and it is not registered or enabled. This PR makes collection\nsurvive failures: failed collections are retried, queries are rebuilt\nwhen needed, and counter state carries over. Operators get clear logs\nfor failures, limit hits, membership changes, and recoveries.\n\n## Commits\n\n1. **Worker and lifecycle** (f96ac30f3): Starts, stops, and cancels the\nworker, and owns the native query. Retries and rebuilds keep counter\nstate and do not emit partial data.\n2. **Diagnostics** (dfc1b1aec): Logs failures, limit hits, membership\nchanges, and recoveries. Ordinary counter warnings are logged once per\nepisode.\n3. **Live tests and native CI** (919dc8df1): Adds Windows-native tests\nfor collection, lifecycle, recovery, and projection. CI runs them\nserially and fails if a command fails or runs no tests.\n\n## Related issue\n\nPart of #4074.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: b57921b8-61e5-4fac-afae-a18d40bd60ae",
          "timestamp": "2026-10-25T18:17:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf6acdc8ed9f0d2ed799cb5728d5895b994109e2"
        },
        "date": 1793035323244,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.65827476978302,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79723552927102,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17513705505367,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.120572916666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.69921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99985.79913828551,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.75978814934,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003401,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116843.58860545,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165952.7295480815,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.698511135825736,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87247935575702,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.35406189405104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.197135416666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.1900449249,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.1900449249,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001918,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194356.1785562974,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031767.734915113,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48601652347938,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "adgrieco",
            "username": "adgrieco",
            "email": "157547602+adgrieco@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "cf6acdc8ed9f0d2ed799cb5728d5895b994109e2",
          "message": "chore(windows_perf_counters): add worker lifecycle, recovery, and diagnostics  (#4510)\n\n# Chore Summary\n\nAdds the background worker, recovery, diagnostics, and Windows-native\ntests for the performance-counter receiver.\n\nThe receiver still runs native counter calls on a dedicated worker\nthread, and it is not registered or enabled. This PR makes collection\nsurvive failures: failed collections are retried, queries are rebuilt\nwhen needed, and counter state carries over. Operators get clear logs\nfor failures, limit hits, membership changes, and recoveries.\n\n## Commits\n\n1. **Worker and lifecycle** (f96ac30f3): Starts, stops, and cancels the\nworker, and owns the native query. Retries and rebuilds keep counter\nstate and do not emit partial data.\n2. **Diagnostics** (dfc1b1aec): Logs failures, limit hits, membership\nchanges, and recoveries. Ordinary counter warnings are logged once per\nepisode.\n3. **Live tests and native CI** (919dc8df1): Adds Windows-native tests\nfor collection, lifecycle, recovery, and projection. CI runs them\nserially and fails if a command fails or runs no tests.\n\n## Related issue\n\nPart of #4074.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: b57921b8-61e5-4fac-afae-a18d40bd60ae",
          "timestamp": "2026-10-25T18:17:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf6acdc8ed9f0d2ed799cb5728d5895b994109e2"
        },
        "date": 1793064945225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6583632230758667,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78671555681815,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23180081960875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.680989583333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99977.11039911982,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.12669881544,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005415,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3124839.0019222167,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168099.994781737,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.78261551788036,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88443450527883,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1596877173997,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.587109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.77456517346,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.77456517346,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002375,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194279.0025279806,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031828.721629821,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48636006473224,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "9b44e32c660dad4214f152900ab31d3173afd8ec",
          "message": "chore(ci): extract test archives into the workspace instead of /tmp (#4530)\n\nGitHub started moving `ubuntu-latest` to Ubuntu 26.04 on October 19. Our\naffected jobs are already using the new image, where `/tmp` is a\nseparate 7.9 GB tmpfs. Nextest extracts there by default and hits the\nquota, even though the workspace disk has 89 GB free.\n\nThis change extracts the Linux test archive into the workspace instead.\nIt also removes unused CodeQL, Homebrew, and Miniconda installations\nfrom the required Linux jobs and logs disk usage.\n\n- Runner migration:\nhttps://github.com/actions/runner-images/issues/14748\n- Report of the same `/tmp` quota failure:\nhttps://github.com/actions/runner-images/issues/14777\n\nValidation: disk measurements confirmed the separate `/tmp` limit. CI\nvalidation of the extraction change is pending. No local tests run.",
          "timestamp": "2026-10-27T15:28:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b44e32c660dad4214f152900ab31d3173afd8ec"
        },
        "date": 1793121340289,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90126657014902,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33678336811191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.130989583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.0078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.64269407756,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.64269407756,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002252,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194621.9363838295,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029101.3837248986,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48890007993737,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008679425343871117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77936316585969,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17943212757682,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.168880208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98311.8450830078,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98320.37798628282,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003024,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116531.4428712726,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166292.9252097192,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.69771624867101,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Nev",
            "username": "MSNev",
            "email": "54870357+MSNev@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2b6bcc0478697d427fa1e75112b52aa4c34a00b5",
          "message": "chore(azure-data-explorer): Additional Skeleton for Azure Data Explorer exporter (#4464)\n\n# Change Summary\n\nThis is **PR 3 of the unified multi-signal Azure Data Explorer\nexporter**, building on the component foundation merged in #4311 and the\nbounded signal transformations added in #4382.\n\n`urn:microsoft:exporter:azure_data_explorer`\n\nThis phase adds the standalone, signal-agnostic HTTP transport client\nand telemetry contracts needed for ADX managed streaming ingestion.\n\nThe client implements bounded request attempts, retry handling,\noperation deadlines, response classification, and transport metrics for\nlogs, metrics, and traces. It accepts already transformed and\ngzip-compressed payloads produced by later runtime integration.\n\nThe exporter remains explicitly non-operational after this phase. The\nclient and metrics are not yet wired into the exporter loop, and\npipeline messages are not sent, acknowledged, or negatively\nacknowledged.\n\n## Signals Supported\n\nThe transport and telemetry contracts support batches for:\n\n- Logs\n- Metrics\n- Traces\n\nSignal transformation continues to use the bounded implementation\nintroduced in #4382.\n\n## Includes\n\n- A standalone HTTP client for the ADX managed streaming ingestion\nendpoint\n- Signal-agnostic client reuse across log, metric, and trace destination\ntables\n- Percent-encoding of database, table, and mapping names\n- JSON streaming-ingestion endpoint construction using\n`streamFormat=JSON`\n- Authorization, content type, gzip content encoding, and ADX\nclient-request headers\n- Stable operation IDs across retries with per-attempt request\nidentifiers\n- A total operation deadline covering:\n  - HTTP submission\n  - Response handling\n  - Retry backoff\n  - All physical attempts\n- Configurable retries after the initial submission\n- Exponential retry backoff with jitter:\n  - 3-second initial delay\n  - 30-second maximum delay\n- `Retry-After` support for both:\n  - Delta-seconds values\n  - HTTP-date values\n- Retry handling for:\n  - Network failures\n  - HTTP 408\n  - HTTP 429\n  - HTTP 5xx\n  - Retryable response-body failures\n- Error classification for:\n  - Retryable failures\n  - Backend refusals\n  - Permanent refusals\n  - Unauthorized responses\n- Bounded ADX error-response retention\n- UTF-8-safe response diagnostics\n- Optional response-body debug logging with:\n  - 512-byte chunks\n  - A maximum of 128 events per response\n  - No response content in warning-level events\n- Shared exporter-attempt telemetry for:\n  - Attempt count\n  - Duration\n  - Item count\n  - Payload size\n  - Signal\n  - Outcome\n- ADX-specific metrics for:\n  - Logical batch outcomes by signal\n  - HTTP attempts by bounded response category\n- Exactly one HTTP response category per physical attempt, including\nresponse-body timeouts\n- Pipeline-local `Rc<RefCell<_>>` metric ownership without shared\ncross-core state\n- Focused tests covering endpoint construction, headers, retries,\ndeadlines, response bounds, metric outcomes, and diagnostic-work limits\n\n## Resource, Runtime, and Correctness Requirements\n\n- The complete operation, including retries and backoff, must remain\nbounded by the configured timeout.\n- Retry delays must never extend beyond the operation deadline.\n- Error-response retention must remain bounded independently of\nserver-provided content length.\n- Optional diagnostic logging must have bounded bytes, allocations, and\nevent count.\n- One physical HTTP attempt must produce exactly one bounded\nHTTP-response metric category.\n- Retried attempts must be measured independently while retaining the\nlogical operation identity.\n- Sensitive response content must never appear in warning-level\ndiagnostics.\n- Response content may be emitted only when `log_response_body` is\nexplicitly enabled.\n- All client and metric state must remain local to one pipeline\ninstance.\n- The implementation must not introduce shared cross-core\nsynchronization or unbounded queues.\n- Rust source must remain ASCII-only.\n- Every test must include the required `Scenario` and `Guarantees`\ndocumentation.\n\n## Explicit Exclusions from This PR\n\nThese capabilities remain deferred so Phase 3 stays focused on the\nstandalone transport and telemetry contracts:\n\n- Wiring the client into the exporter runtime\n- Constructing or managing the runtime HTTP client pool\n- Enforcing `max_in_flight` through runtime scheduling\n- Azure Identity token acquisition\n- Token refresh and replacement-token handling\n- Retaining HTTP 401 requests while awaiting a replacement token\n- Gzip compression of transformed payloads\n- Cross-message batching and request coalescing\n- Runtime backpressure and retained-message ownership\n- Pipeline ACK/NACK behavior\n- Recording logical batch metrics from the exporter loop\n- Periodic runtime metric reporting\n- Shutdown draining and pending-request cancellation\n- ADX queued ingestion\n- Provisioning ADX databases, tables, or mappings\n- Activating the exporter for production use\n\n## Related Issue\n\nThis is Phase 3 of #4159.\n\nIt builds on:\n\n- #4311 — component skeleton, configuration, and design\n- #4382 — bounded logs, metrics, and traces transformation\n\nLater phases will connect transformation, compression, authentication,\ntransport, batching, telemetry reporting, and pipeline completion\nhandling into the operational exporter runtime.\n\n## Validation\n\nThe following validation completed successfully:\n\n- `cargo fmt --all` - passed\n- `cargo fmt --all -- --check` - passed\n- `cargo build -p otel-arrow-dfe --bin df_engine --features\nazure-data-explorer` - passed\n- `cargo test -p otel-arrow-dfe-contrib-nodes --features\nazure-data-explorer azure_data_explorer_exporter` - passed\n- `cargo xtask check` - passed, including workspace structure, component\ninventory, formatting, Clippy, unit tests, integration tests, and\ndoctests\n- `npx markdownlint-cli2\nrust/otap-dataflow/crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/README.md`\n- passed\n- `git diff --check` - passed\n\n## User-Facing Changes\n\nNone. The exporter remains non-operational and continues to return its\nexplicit not-implemented error.\n\nNo changelog entry is required because this is an internal\nimplementation phase and the PR title includes `chore`.\n\n## Additional Context\n\nPhase 1 established the exporter component, configuration model,\nschemas, and complete intended design.\n\nPhase 2 added bounded transformation of logs, metrics, and traces into\nADX-compatible JSON rows.\n\nPhase 3 adds the standalone managed-streaming transport client, retry\nand timeout behavior, bounded response diagnostics, error\nclassification, and exporter telemetry contracts while deliberately\nleaving runtime integration disabled.\n\nRelevant design documentation:\n\n[Azure Data Explorer exporter\ndesign](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/design.md)",
          "timestamp": "2026-10-27T22:51:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2b6bcc0478697d427fa1e75112b52aa4c34a00b5"
        },
        "date": 1793151536772,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008679189719259739,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.69231031466381,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.97363445867288,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.213411458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98314.12329406563,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.6561636375,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003261,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3117955.909916659,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167126.1067998933,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.711469477873678,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677213452756405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83524977536733,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3121657774685,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.305338541666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98336.77937728021,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.24648538182,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003104,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193815.1319924053,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031477.9352344475,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48115619012102,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "1f717601c69b998390be4aba5478703fe1644655",
          "message": "perf(syslog): optimize the batch codec and add cooperative decoding (#4509)\n\n# Change summary\n\nOptimizes the `syslog-batch` codec and receiver path to reduce CPU,\ncopying, and Arrow construction costs while keeping large-batch decoding\nresponsive.\n\n- **Cooperative decoding:** owned incremental jobs yield between chunks\nwithout holding codec state across yields, then produce one complete\nOTAP batch.\n- **Cheaper ingestion:** borrowed TCP frames, amortized receipt-clock\nreads, and bounded, learned buffer reservations.\n- **Faster Arrow construction:** direct observed-timestamp appends, bulk\nparent-ID appends, and shared opt-in known-key dictionary caching.\nSyslog vocabulary remains in the receiver.\n- Small batches remain synchronous; cooperative decoding starts above\n256 messages or 256 KiB. Supports batches up to 8,192 messages / 2 MiB;\nthe receiver default stays 100.\n\nPreserves decode-failure recovery and cancellation cleanup. Arrow\nfinalization, encoding, and compression remain synchronous.\n\n## Related issue\n\n- Advances #3452.\n- Closes #4493.\n\n## Validation\n\n`cargo xtask check` \n\n### Full-engine benchmarks\n\nPre-PR5 main (`06cdc5979`) -> PR5.2 (`c2ded8be9`). This comparison\nincludes PR5, PR5.1, PR5.2, and intervening upstream changes, not async\ndecoding alone.\n\nTCP syslog -> DFE receiver / batch / zstd OTAP exporter -> DFE backend.\nOne idle SUT CPU held at verified 3 GHz; three interleaved pairs per\ncase/pass,\n168 accepted runs. Values are before -> after; CPU intervals are paired\n95% CIs.\n\n| Scenario | CPU us/log | CPU change [95% paired CI] | Mean RSS MiB |\nHTTP p99 ms |\n| --- | ---: | ---: | ---: | ---: |\n| RFC 3164, 100 -> 8K, fixed | 6.831 -> 3.053 | -55.3% [-55.7%, -54.9%]\n| 60.5 -> 53.8 | 32.33 -> 4.68 |\n| RFC 5424, 100 -> 8K, fixed | 4.919 -> 2.625 | -46.6% [-48.2%, -45.0%]\n| 61.0 -> 54.9 | 19.65 -> 4.91 |\n| RFC 3164, 8K -> 8K, fixed | 5.093 -> 2.931 | -42.4% [-44.7%, -40.2%] |\n51.5 -> 53.5 | 8.90 -> 4.56 |\n| RFC 5424, 8K -> 8K, fixed | 2.995 -> 2.505 | -16.3% [-16.9%, -15.8%] |\n52.1 -> 55.4 | 6.97 -> 4.83 |\n| RFC 3164, 8K -> 8K, high | 5.017 -> 2.940 | -41.4% [-42.7%, -40.1%] |\n50.8 -> 183.5 | 16.97 -> 6.87 |\n| RFC 5424, 8K -> 8K, high | 2.927 -> 2.470 | -15.6% [-17.7%, -13.5%] |\n52.1 -> 189.6 | 10.74 -> 7.54 |\n\nFixed load: 102,400 offered logs/s.\nHigh load: 1M with TCP backpressure (finite overload tests, not\nmaximum-throughput claims). CPU/RSS and latency use separate passes.\nHTTP p99 is the mean of per-run p99s for a concurrent OTLP/noop Ack\nprobe targeting 100 requests/s, not syslog delivery latency.\nThe 100 -> 8K cases disable encoded fragment budgets to match output\nsizes.\n\n**Tradeoff:** overload RSS increases substantially with the same\n100-slot queues.\n\nThe chart sweeps matching receiver/output batch targets. Contributor\nnotes summarize prior profiling and isolated comparisons, not per-change\nattribution of this campaign's total gains.\n\n![Syslog batch size: CPU, RSS, I/O latency, and main CPU\ncontributors](https://raw.githubusercontent.com/lquerel/otel-arrow/7e67152cd374958f7f86460d980db43a3bb1899b/rust/otap-dataflow/crates/pdata-codec/assets/syslog-batch-size-benchmarks.png)\n\n## User-facing changes\n\nMore efficient syslog processing and support for larger batches with\ncooperative decoding. Existing configuration and the default batch size\nare unchanged.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-10-28T05:28:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f717601c69b998390be4aba5478703fe1644655"
        },
        "date": 1793207580330,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.94690952962742,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.39550061804698,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.316796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.69302536114,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.69302536114,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003035,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190257.192406417,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3023185.8033835567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.44581445852158,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7902689967099,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1910538426139,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.473307291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.15625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.69630271856,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.69630271856,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003033,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3119421.7901635715,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166772.6540349915,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.725397403337016,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "639df8b5d35e04eb28bbfd512fde6fca26533135",
          "message": "chore(context): separate node bindings from compiler state (step 3A) (#4567)\n\n# Chore Summary\n\n<!-- Briefly describe the maintenance work in this PR. -->\n\nSeparates compiler bookkeeping from shared `NodeBindings` and adds\ncompiler/current-node lookups. Existing policy accessors borrow the same\nstored policies; focused tests cover ownership and lookup keys.\n\nNo runtime behavior or receiver/exporter plumbing changes. Claims and\narrival/sink adapters are deferred.\n\n## Related issue\n\n<!-- Link the related issue if one exists. -->\n\nStep 3A of open-telemetry/otel-arrow#4541; follows\nopen-telemetry/otel-arrow#4551.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: 107fe8c3-a55e-4925-9d29-5abe807f960c",
          "timestamp": "2026-10-28T23:12:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/639df8b5d35e04eb28bbfd512fde6fca26533135"
        },
        "date": 1793237252262,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677683770656586,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76005430730774,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1066501854141,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.3734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.24609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98331.26441641503,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.73154030164,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003215,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115583.6567392484,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1141045.8864392603,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.687317957213157,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678954094648361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.95220722837965,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.44655636307407,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.567057291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.25390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98316.77998466317,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.3128533818,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003267,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191785.280365447,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028080.085241349,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.461481054475676,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "sapatrjv",
            "username": "sapatrjv",
            "email": "sapatrjv@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "e62bacd642fee6f0908162b1e1a79dfef826cf55",
          "message": "Add private commit bytes metrics to the OTel-Arrow Dataflow Engine da… (#4573)\n\n…shboard.\n\n# Change summary\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\nAdd private commit size metrics to the OTel-Arrow Dataflow Engine\ndashboard\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #4572 \n\n## Validation\n\nManually built and ran df-engine with this PR, then verified in the\nbrowser at http://127.0.0.1:9091/dashboard that the private commit field\nis added.\n\n<img width=\"1609\" height=\"412\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/5eba0860-ccbc-4687-95a6-f2e72837b9c2\"\n/>\n\n\n## User-facing changes\n\nNone\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-10-29T16:55:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e62bacd642fee6f0908162b1e1a79dfef826cf55"
        },
        "date": 1793297303913,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678954094648361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75631532918909,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25051329622758,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.276302083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.89453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98316.41787224103,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.95070953209,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003488,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118742.044396657,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139579.2864679152,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.71872471728899,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008677965961396694,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92613567719582,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18278106964217,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.826302083333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.77051798157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98338.30354213499,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002174,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3183073.6022616224,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3023422.0844467925,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.368603968216426,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "1c442d89def5d78783a34edec2f588a5819f251e",
          "message": "chore(deps): update module github.com/axiomhq/hyperloglog to v0.3.0 (#4599)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/axiomhq/hyperloglog](https://redirect.github.com/axiomhq/hyperloglog)\n| `v0.2.6` → `v0.3.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2faxiomhq%2fhyperloglog/v0.3.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2faxiomhq%2fhyperloglog/v0.2.6/v0.3.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>axiomhq/hyperloglog (github.com/axiomhq/hyperloglog)</summary>\n\n###\n[`v0.3.0`](https://redirect.github.com/axiomhq/hyperloglog/releases/tag/v0.3.0)\n\n[Compare\nSource](https://redirect.github.com/axiomhq/hyperloglog/compare/v0.2.6...v0.3.0)\n\n#### Upgrade notes\n\nThe binary format does not change. v0.3.0 decodes sketches that v0.1.0\nto v0.2.6 encoded, and v0.2.6 decodes sketches that v0.3.0 encodes.\n\n##### Code changes for callers\n\n- Match errors with `errors.Is`. Do not compare errors with `==` or by\ntheir text.\n- When you append an encoded sketch to a larger buffer, frame it.\n\n##### Behavior changes\n\n- `UnmarshalBinary`, `MarshalBinary`, `AppendBinary`, and `Merge` wrap\nthe errors that they return. `err == ErrorTooShort` is always false.\n- `Merge` wraps `ErrorPrecisionMismatch` when the precisions differ. The\nerror message includes both precisions.\n- `UnmarshalBinary` rejects trailing bytes with `ErrorInvalidData`. It\nalso rejects unknown versions, invalid precisions, out-of-range\nregisters, and malformed sparse lists.\n- If `UnmarshalBinary` returns an error, the receiver does not change.\nThe sketch that it decodes does not share memory with the input buffer.\n- `Insert` and `InsertHash` on a zero-value `Sketch` give it precision\n14 and the sparse representation, the same as `New`. In v0.2.6, they\npanicked.\n- `Merge` into a zero-value `Sketch` copies the other sketch, and\n`Estimate` on a zero-value `Sketch` returns 0. In v0.2.6, both panicked.\n- `Merge` with a zero-value argument does nothing. In v0.2.6, it\nreturned an error.\n- `MarshalBinary` and `AppendBinary` on a zero-value `Sketch` return an\nerror that wraps `ErrorInvalidPrecision`. In v0.2.6, they returned an\n8-byte header with precision 0.\n- `Estimate` converts a sparse sketch to dense when its sparse list\ngrows past the register count. In v0.2.6, calls to `Estimate` between\nsmall batches of inserts let the sparse list grow without limit.\n- A sketch that v0.2.6 saved with such an oversized sparse list can give\na different estimate after v0.3.0 decodes it. In tests, the difference\nwas 1.1% or less.\n\n##### Additions\n\n- `Sketch.Reset` clears a sketch and keeps its representation and\nallocated storage.\n- `ErrorInvalidVersion`, `ErrorInvalidPrecision`, `ErrorInvalidData`,\nand `ErrorPrecisionMismatch` are sentinel errors to match with\n`errors.Is`.\n\n#### What's Changed\n\n- fix ut logic error in TestHLL_Merge_Complex. by\n[@&#8203;CodeZHXS](https://redirect.github.com/CodeZHXS) in\n[#&#8203;59](https://redirect.github.com/axiomhq/hyperloglog/pull/59)\n- Add runnable examples to replace demo directory by\n[@&#8203;lukasmalkmus](https://redirect.github.com/lukasmalkmus) in\n[#&#8203;56](https://redirect.github.com/axiomhq/hyperloglog/pull/56)\n- allow compiler to inline hash function by\n[@&#8203;mgartner](https://redirect.github.com/mgartner) in\n[#&#8203;64](https://redirect.github.com/axiomhq/hyperloglog/pull/64)\n- Feature: Adding support for Reset by\n[@&#8203;MovieStoreGuy](https://redirect.github.com/MovieStoreGuy) in\n[#&#8203;62](https://redirect.github.com/axiomhq/hyperloglog/pull/62)\n- Harden UnmarshalBinary validation by\n[@&#8203;seiflotfy](https://redirect.github.com/seiflotfy) in\n[#&#8203;65](https://redirect.github.com/axiomhq/hyperloglog/pull/65)\n- Harden sketch API and decoding by\n[@&#8203;seiflotfy](https://redirect.github.com/seiflotfy) in\n[#&#8203;66](https://redirect.github.com/axiomhq/hyperloglog/pull/66)\n- docs: fix project name reference, spelling, and period in\nContributing.md by\n[@&#8203;amir-rezaei](https://redirect.github.com/amir-rezaei) in\n[#&#8203;67](https://redirect.github.com/axiomhq/hyperloglog/pull/67)\n- Add CI and release automation by\n[@&#8203;seiflotfy](https://redirect.github.com/seiflotfy) in\n[#&#8203;68](https://redirect.github.com/axiomhq/hyperloglog/pull/68)\n- Fix unbounded sparse growth, a uint32 underflow in Estimate, and\nbenchmark measurement by\n[@&#8203;seiflotfy](https://redirect.github.com/seiflotfy) in\n[#&#8203;69](https://redirect.github.com/axiomhq/hyperloglog/pull/69)\n- docs: fix H1 heading level and fork clone URL placeholder in\nContributing.md by\n[@&#8203;amir-rezaei](https://redirect.github.com/amir-rezaei) in\n[#&#8203;70](https://redirect.github.com/axiomhq/hyperloglog/pull/70)\n- docs: fix verb parallelism, missing trailing period, and article in\nREADME.md by\n[@&#8203;amir-rezaei](https://redirect.github.com/amir-rezaei) in\n[#&#8203;71](https://redirect.github.com/axiomhq/hyperloglog/pull/71)\n- .github/workflows: publish releases with goreleaser by\n[@&#8203;lukasmalkmus](https://redirect.github.com/lukasmalkmus) in\n[#&#8203;74](https://redirect.github.com/axiomhq/hyperloglog/pull/74)\n\n#### New Contributors\n\n- [@&#8203;CodeZHXS](https://redirect.github.com/CodeZHXS) made their\nfirst contribution in\n[#&#8203;59](https://redirect.github.com/axiomhq/hyperloglog/pull/59)\n- [@&#8203;lukasmalkmus](https://redirect.github.com/lukasmalkmus) made\ntheir first contribution in\n[#&#8203;56](https://redirect.github.com/axiomhq/hyperloglog/pull/56)\n- [@&#8203;MovieStoreGuy](https://redirect.github.com/MovieStoreGuy)\nmade their first contribution in\n[#&#8203;62](https://redirect.github.com/axiomhq/hyperloglog/pull/62)\n- [@&#8203;amir-rezaei](https://redirect.github.com/amir-rezaei) made\ntheir first contribution in\n[#&#8203;67](https://redirect.github.com/axiomhq/hyperloglog/pull/67)\n\n**Full Changelog**:\n<https://github.com/axiomhq/hyperloglog/compare/v0.2.6...v0.3.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Friday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4xODguMiIsInVwZGF0ZWRJblZlciI6IjQ0LjE4OC4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-10-30T01:01:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1c442d89def5d78783a34edec2f588a5819f251e"
        },
        "date": 1793326463468,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678719401359558,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84473659489738,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32613861386139,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.827994791666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.2734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98319.57103748032,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.10391700668,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003191,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192677.655008231,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027037.535412687,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46963510760864,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677213452756405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77584215395571,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16670377241806,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.474739583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98336.69251769535,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.15963333395,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003157,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116417.44643156,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1141738.8315476335,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.69404835860542,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "66beb81c61088782dca5d7b112845a136ae4e0e1",
          "message": "chore(deps): update go patches and stable minors (#4598)\n\n> [!NOTE]\n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/HdrHistogram/hdrhistogram-go](https://redirect.github.com/HdrHistogram/hdrhistogram-go)\n| `v1.2.0` → `v1.4.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fHdrHistogram%2fhdrhistogram-go/v1.4.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fHdrHistogram%2fhdrhistogram-go/v1.2.0/v1.4.0?slim=true)\n|\n|\n[github.com/apache/arrow-go/v18](https://redirect.github.com/apache/arrow-go)\n| `v18.6.0` → `v18.8.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fapache%2farrow-go%2fv18/v18.8.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fapache%2farrow-go%2fv18/v18.6.0/v18.8.0?slim=true)\n|\n|\n[github.com/dustin/go-humanize](https://redirect.github.com/dustin/go-humanize)\n| `v1.0.1` → `v1.1.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fdustin%2fgo-humanize/v1.1.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fdustin%2fgo-humanize/v1.0.1/v1.1.0?slim=true)\n|\n|\n[github.com/fxamacker/cbor/v2](https://redirect.github.com/fxamacker/cbor)\n| `v2.9.6` → `v2.10.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2ffxamacker%2fcbor%2fv2/v2.10.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2ffxamacker%2fcbor%2fv2/v2.9.6/v2.10.0?slim=true)\n|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.7` → `v1.20.3` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.20.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.7/v1.20.3?slim=true)\n|\n|\n[github.com/stretchr/testify](https://redirect.github.com/stretchr/testify)\n| `v1.11.1` → `v1.12.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fstretchr%2ftestify/v1.12.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fstretchr%2ftestify/v1.11.1/v1.12.1?slim=true)\n|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.60.0` → `v1.70.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.70.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.60.0/v1.70.0?slim=true)\n|\n|\n[go.opentelemetry.io/otel](https://redirect.github.com/open-telemetry/opentelemetry-go)\n| `v1.44.0` → `v1.47.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fotel/v1.47.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fotel/v1.44.0/v1.47.0?slim=true)\n|\n|\n[go.opentelemetry.io/otel/metric](https://redirect.github.com/open-telemetry/opentelemetry-go)\n| `v1.44.0` → `v1.47.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fotel%2fmetric/v1.47.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fotel%2fmetric/v1.44.0/v1.47.0?slim=true)\n|\n| [google.golang.org/grpc](https://redirect.github.com/grpc/grpc-go) |\n`v1.83.2` → `v1.85.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/google.golang.org%2fgrpc/v1.85.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/google.golang.org%2fgrpc/v1.83.2/v1.85.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>HdrHistogram/hdrhistogram-go\n(github.com/HdrHistogram/hdrhistogram-go)</summary>\n\n###\n[`v1.4.0`](https://redirect.github.com/HdrHistogram/hdrhistogram-go/releases/tag/v1.4.0):\nVersion 1.4.0\n\n[Compare\nSource](https://redirect.github.com/HdrHistogram/hdrhistogram-go/compare/v1.3.0...v1.4.0)\n\n#### Version 1.4.0\n\n- **New: `PackedHistogram`**, a sparse histogram for keeping many\nlightly populated histograms in memory: about 0.3 KB instead of 184 KB\neach at a typical latency configuration (see below).\n- **Faster:** asking for several percentiles in one call is 3.4 to 5.7x\nfaster; single queries are unchanged and recording is within 4%.\n- **Fixed:** a number of silent correctness and decoding problems,\nincluding hangs and panics on malformed input.\n- **Behaviour changes:** some of those fixes change results for inputs\nthat were already wrong, such as hostile streams, overflowing totals and\nnegative values. Read [Behaviour changes](#behaviour-changes) before\nupgrading.\n- **Compatibility:** nothing exported was removed or changed signature.\n`Snapshot` gained a field, so unkeyed literals such as `Snapshot{1,\n1000, 3, counts}` no longer compile; use field names.\n\n![hdrhistogram-go 1.4.0 against v1.3.0: asking for several percentiles\nat once is 4 to 6x faster, single queries are unchanged, recording is\nwithin\n4%](https://raw.githubusercontent.com/redis-performance/hdr-agent-workspace/e8d2f5f6ecd2c7bb6aea659a76ea2a3e1ecc33f9/experiments/GO-BENCH-1.3.0-VS-TIP-186f8b9-2026-10-08/charts/speedup.png)\n\n##### Highlight: `PackedHistogram`\n\n`New` allocates the full counts array up front: about 184 KB at a\ntypical latency configuration (`New(1, 3600000000, 3)`), however few\nvalues are recorded. Services that keep **many sparsely populated\nhistograms** (per endpoint, per tenant, per connection, or a ring of\nper-second slices) pay that for every one. `PackedHistogram` stores only\nthe populated buckets, each count in 1, 2, 4 or 8 bytes as needed:\n\n| Populated buckets | `Histogram` | `PackedHistogram` |\n| ----------------- | ----------: | ----------------: |\n| 10                |      184 KB |          \\~0.3 KB |\n| 100               |      184 KB |          \\~0.8 KB |\n| 1,600             |      184 KB |           \\~10 KB |\n\nFor one thousand histograms with ten populated buckets each, that is\n179.7 MB dense against 273 KB packed\n([#&#8203;75](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/75)).\n\n- **Same model as `Histogram`:** the same constructor arguments and\nbucket layout, and the same queries: `RecordValue(s)`,\n`RecordCorrectedValue`, `ValueAtPercentile`, `ValueAtPercentilesSlice`,\n`Min`, `Max`, `Mean`, `StdDev`, `CountAtValue`, `TotalCount`, plus\n`Clone`, geometry getters, and interval start/end times and tags.\n- **Wire-compatible:** `Encode` writes the standard V2 compressed\nformat, byte-for-byte the same as the dense encoder on equivalent data.\n`DecodePacked` reads V2 streams from Go, Java and C writers, including\nshifted Java histograms and streams from hdrhistogram-go v1.2.0 and\nearlier. `HistogramLogWriter.OutputIntervalPackedHistogram` writes\npacked intervals to logs directly.\n- **Built for rolling windows:** record into a dense histogram, keep\ncompleted slices packed, and aggregate on demand with `MergeFrom` (dense\ninto packed), `MergeInto` (packed into dense), `Merge` (packed into\npacked), `Reset` (keeps storage), `Compact` (shrinks it) and\n`ForEachBucket`.\n- **The trade-off is recording speed:** recording is a binary search,\nand a new bucket costs O(populated). A last-hit cache skips the search\nwhen consecutive records land in the same bucket\n([#&#8203;117](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/117)).\nKeep the dense `Histogram` for hot recording paths and for histograms\nthat fill most buckets.\n- **Hardened before release:** two dedicated fuzz targets (hostile\ndecoding, and a differential against the dense histogram) run in CI and\nnightly, alongside multi-agent adversarial reviews of every packed PR.\n\n```go\nh := hdrhistogram.NewPacked(1, 3600000000, 3)\nh.RecordValue(1234)\np99 := h.ValueAtPercentile(99)\nencoded, err := h.Encode() // standard V2, same bytes as Histogram.Encode\n```\n\nGuides: the [README\nsection](https://redirect.github.com/HdrHistogram/hdrhistogram-go#packed-histograms)\n(memory, costs and a rolling-window example) and the [C/Java\ncompatibility\nguide](https://redirect.github.com/HdrHistogram/hdrhistogram-go/blob/master/PACKED_COMPATIBILITY.md).\n\n##### Update Urgency: High\n\n**High**: contains fixes for decoders that hung or panicked on malformed\ninput\n([#&#8203;80](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/80)),\nfor totals that silently wrapped past `MaxInt64`\n([#&#8203;114](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/114),\n[#&#8203;109](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/109)),\nfor negative values recorded as huge positive ones\n([#&#8203;103](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/103)),\nand for wrong percentiles at very large counts\n([#&#8203;108](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/108)).\nAnyone decoding untrusted streams or logs, or recording very large\nweighted counts, should update.\n\n##### API Additions\n\n- `PackedHistogram`, `NewPacked` and `DecodePacked`\n([#&#8203;75](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/75));\nsee [Highlight](#highlight-packedhistogram)\n- `PackedHistogram` rolling-window support: `Reset`, `ForEachBucket`,\n`MergeInto` (packed into dense) and `MergeFrom` (dense into packed)\n([#&#8203;81](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/81)),\nand packed-to-packed `Merge` and `Compact`\n([#&#8203;82](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/82))\n- `PackedHistogram` parity with the dense type: `Mean`, `StdDev`,\n`RecordCorrectedValue`, geometry getters, `Clone`, interval start/end\ntime and tag, `EncodeV2`, and\n`HistogramLogWriter.OutputIntervalPackedHistogram` with\n`...WithLogOptions`\n([#&#8203;105](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/105))\n- `Histogram.Clone`, a deep copy of geometry, counts, total, tag and\ntimes; use `w.Merge().Clone()` to keep a windowed result\n([#&#8203;112](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/112))\n- `Snapshot.Validate`, which rejects negative counts, counts outside the\ngeometry, sums past `MaxInt64` and unrepresentable geometry before\n`Import`; `Snapshot.IntegerToDoubleConversionRatio` carries the V2\nconversion ratio\n([#&#8203;112](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/112),\n[#&#8203;105](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/105))\n- `Histogram.EncodeV2`\n([#&#8203;105](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/105))\n\n##### Performance\n\nFleet microbenchmarks, one pinned core, interleaved with v1.3.0 (7\nrepetitions, medians; four percentiles on a 1M-sample lognormal\nhistogram):\n\n| Benchmark | Intel Xeon (Sapphire Rapids) | AMD EPYC (Zen 5) | AWS\nGraviton (Neoverse V2) |\n| ------------------------------------------------- |\n---------------------------: | --------------------------: |\n---------------------------: |\n| `ValueAtPercentiles` (four percentiles, one call) | 8.09 to 1.70 us\n(**4.76x**) | 4.73 to 1.12 us (**4.23x**) | 11.34 to 2.00 us (**5.66x**)\n|\n| `ValueAtPercentilesSlice` (sorted input) | 5.12 to 1.53 us (**3.35x**)\n| 3.59 to 0.98 us (**3.66x**) | 7.35 to 1.78 us (**4.13x**) |\n| `ValueAtPercentile` (one percentile) | unchanged | unchanged |\nunchanged |\n| `RecordValue` | 3.0 to 3.2 ns | 2.2 to 2.2 ns | 3.4 to 3.5 ns |\n\n- `ValueAtPercentiles` resolves all percentiles in one blocked\nskip-scan: sum eight counts at once and skip blocks that cannot reach\nthe next target\n([#&#8203;115](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/115))\n- `ValueAtPercentilesSlice` uses the same scan, and allocates once\ninstead of four times with Go 1.27\n([#&#8203;116](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/116));\nits figures compare the code before and after the change in the same\nsession\n- `RecordValue` is about 0.1 to 0.2 ns slower on Intel and Arm (0.96x),\nin line with the cost\n[#&#8203;114](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/114)\nmeasured for its new check that stops the total wrapping; there is no\nmeasurable change on AMD\n- `PackedHistogram` skips its binary search when a record lands in the\nsame bucket as the previous one\n([#&#8203;117](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/117)).\nMeasured in August on earlier code: bursty writes 30 to 59% faster,\nrandom (low-locality) writes 2.4 to 9.4% slower; not yet re-measured on\nthis release\n\n##### Correctness & Security Fixes\n\n- Decoders and `New` no longer loop forever on a header whose lowest\nvalue is too large for its precision, and dense `Decode` no longer\npanics on a `MinInt64` zero-run or allocates tens of MB before rejecting\na small stream\n([#&#8203;80](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/80))\n- Java zero-digit streams decode with their exact geometry; boundary\nmaxima such as `New(1, 2048, 3).RecordValue(2048)` now record, and real\nJava/C streams of that shape decode; negative values are rejected\ninstead of recorded as huge positive ones; merge `dropped` totals\nsaturate instead of wrapping\n([#&#8203;103](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/103),\nfixes\n[#&#8203;84](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/84),\n[#&#8203;85](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/85),\n[#&#8203;86](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/86),\n[#&#8203;90](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/90))\n- Percentile ranks are bounded for very large counts, so P100 reaches\nthe real maximum; a windowed histogram with `(1<<53)+3` observations\nreturned 0 before\n([#&#8203;108](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/108))\n- Dense `RecordValues`, `Merge` and `PackedHistogram.MergeInto` no\nlonger let the total wrap past `MaxInt64`\n([#&#8203;114](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/114),\nfixes\n[#&#8203;113](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/113))\n- Dense `Decode` rejects streams whose counts sum past `MaxInt64`, where\nit used to return an empty histogram without an error\n([#&#8203;109](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/109),\nfixes\n[#&#8203;107](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/107))\n- `DecodePacked` reads streams written by hdrhistogram-go v1.2.0 and\nearlier and shifted Java streams, which it used to reject\n([#&#8203;109](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/109),\nfixes\n[#&#8203;106](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/106))\n- `Import` stores negative counts as 0, so the total and the counts\nagree; `Histogram` ownership and `WindowedHistogram.Merge` lifetime are\ndocumented\n([#&#8203;112](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/112),\nfixes\n[#&#8203;110](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/110),\n[#&#8203;111](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/111))\n\n##### Behaviour changes\n\n| Area | Before | Now | PR |\n|\n--------------------------------------------------------------------------------\n| ------------------------------------- |\n--------------------------------------------------------- |\n------------------------------------------------------------------ |\n| `New` and `Import` with a geometry that cannot be represented | hung\nforever | panic (`New` has no error return) |\n[#&#8203;80](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/80)\n|\n| Decoding a hostile header, `MinInt64` zero-run or oversized payload |\nhang, panic or large allocation | an error; some error messages for\ninvalid streams changed |\n[#&#8203;80](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/80)\n|\n| Recording a negative value, dense or packed | recorded as a huge\npositive value | error, histogram unchanged |\n[#&#8203;103](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/103)\n|\n| Recording a value equal to the highest trackable value at some\ngeometries | error | recorded |\n[#&#8203;103](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/103)\n|\n| Decoding a zero-digit (Java) stream | decoded with one digit, wrong\nvalues | exact geometry |\n[#&#8203;103](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/103)\n|\n| `RecordValue`, `RecordValues`, `RecordCorrectedValue` past a total of\n`MaxInt64` | succeeded and corrupted the histogram | error; `Merge`\ncounts these as dropped |\n[#&#8203;114](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/114)\n|\n| Dense `Decode` of counts summing past `MaxInt64` | empty histogram, no\nerror | error; `nil` histogram on any payload error |\n[#&#8203;109](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/109)\n|\n| `DecodePacked` of pre-v1.3.0 Go and shifted Java streams | rejected |\ndecoded |\n[#&#8203;109](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/109)\n|\n| `Import` of negative counts | stored, total excluded them | stored as\n0 |\n[#&#8203;112](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/112)\n|\n| Unkeyed `Snapshot{...}` literals | compiled | compile error (new\nfield); use field names |\n[#&#8203;105](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/105)\n|\n| P100 with very large counts | could be 0 | the real maximum |\n[#&#8203;108](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/108)\n|\n\n##### Maintenance & Testing\n\n- Nightly native fuzzing of every `Fuzz*` target with a persisted corpus\n([#&#8203;76](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/76)),\nClusterFuzzLite registration and corpus handling\n([#&#8203;78](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/78),\n[#&#8203;79](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/79));\nten fuzz targets now run in CI, including two for `PackedHistogram`\n- New regression tests for each fix above, each shown to fail on the\nprevious code\n\n**Full Changelog**:\n<https://github.com/HdrHistogram/hdrhistogram-go/compare/v1.3.0...v1.4.0>\n\n###\n[`v1.3.0`](https://redirect.github.com/HdrHistogram/hdrhistogram-go/releases/tag/v1.3.0):\nVersion 1.3.0\n\n[Compare\nSource](https://redirect.github.com/HdrHistogram/hdrhistogram-go/compare/v1.2.0...v1.3.0)\n\n#### Version 1.3.0\n\nThis release is backward compatible with no breaking API changes. It\nadds a new\nbatch percentile API, delivers large read-path performance gains, and\nfixes\nseveral silent correctness and interop bugs. A native fuzzing suite was\nadded to\nharden decoding against malformed input.\n\n##### Update Urgency: High\n\n**High** — Contains correctness fixes (Mean/StdDev overflow,\n0th-percentile ==\nmin, empty-histogram and negative-clamp contracts) and hardening of\nDecode/log\nreader against malformed input. Users doing percentile queries or\ndecoding\nuntrusted logs should update.\n\n##### API Additions\n\n- `ValueAtPercentilesSlice` — ordered `[]int64` batch percentile lookup,\nno map alloc\n([#&#8203;63](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/63))\n\n##### Performance\n\n- Blocked prefix-sum skip-scan: +50% read, write-path bounds-check\nelision +5%\n([#&#8203;64](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/64))\n- Flat `counts[]` scan in `ValueAtPercentiles` batch: +303%\n([#&#8203;58](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/58))\n- Flat `counts[]` scan in `ValueAtPercentile`: +133%\n([#&#8203;57](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/57))\n- Range over `counts[]` in percentile scans to elide bounds checks: +72%\nread\n([#&#8203;62](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/62))\n- Single unsigned bounds check in `RecordValues`: +7%\n([#&#8203;59](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/59))\n\n##### Correctness & Security Fixes\n\n- Mean/StdDev int64 overflow, wire offset, base-time casing\n([#&#8203;66](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/66))\n- Percentile: reach first recorded entry (0th percentile == min) across\nall APIs\n([#&#8203;68](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/68))\n- Percentile edge-case contracts: empty histogram\n([#&#8203;60](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/60)),\nnegative clamp, map phantom key\n([#&#8203;67](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/67))\n- Reject negative count in `RecordValues`\n([#&#8203;74](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/74))\n- Reset: clear tag and start/end time, not just counts\n([#&#8203;69](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/69))\n- log: decode final interval line lacking a trailing newline\n([#&#8203;72](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/72))\n- Harden Decode/log-reader against malformed input + native fuzzing\n([#&#8203;65](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/65))\n- Make it harder for Mean() and StdDev() to overflow\n([#&#8203;21](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/21))\n\n##### Maintenance & Testing\n\n- Remove dead benchmark fill loop that panics for b.N > 1e6\n([#&#8203;70](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/70))\n- Pin golden values for logV2 reader fixtures\n([#&#8203;73](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/73))\n- Extended coverage: zigzag byte-lengths, bucket overflow guard,\ncorrected-value + merge edges\n([#&#8203;71](https://redirect.github.com/HdrHistogram/hdrhistogram-go/issues/71))\n\n**Full Changelog**:\n<https://github.com/HdrHistogram/hdrhistogram-go/compare/v1.2.0...v1.3.0>\n\n</details>\n\n<details>\n<summary>apache/arrow-go (github.com/apache/arrow-go/v18)</summary>\n\n###\n[`v18.8.0`](https://redirect.github.com/apache/arrow-go/releases/tag/v18.8.0):\nApache Arrow Go 18.8.0\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-go/compare/v18.7.0...v18.8.0)\n\n#### What's Changed\n\n- feat(parquet/schema): Wire up logical type functionality for writing\ngeo types by [@&#8203;C-Loftus](https://redirect.github.com/C-Loftus) in\n[#&#8203;960](https://redirect.github.com/apache/arrow-go/pull/960)\n- chore: Bump actions/setup-go from 6.5.0 to 7.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;961](https://redirect.github.com/apache/arrow-go/pull/961)\n- chore: Bump actions/checkout from 7.0.0 to 7.0.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;963](https://redirect.github.com/apache/arrow-go/pull/963)\n- chore(dev/release): filter out non numeric go releases (rc's and\nbeta's) when finding latest go by\n[@&#8203;raulcd](https://redirect.github.com/raulcd) in\n[#&#8203;968](https://redirect.github.com/apache/arrow-go/pull/968)\n- ci: fix release workflow discussions by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;980](https://redirect.github.com/apache/arrow-go/pull/980)\n- chore: Bump modernc.org/sqlite from 1.53.0 to 1.54.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;964](https://redirect.github.com/apache/arrow-go/pull/964)\n- chore: Bump google.golang.org/grpc from 1.82.0 to 1.82.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;965](https://redirect.github.com/apache/arrow-go/pull/965)\n- chore: Bump github.com/klauspost/compress from 1.19.0 to 1.19.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;966](https://redirect.github.com/apache/arrow-go/pull/966)\n- chore: Bump actions/setup-python from 6.3.0 to 7.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;962](https://redirect.github.com/apache/arrow-go/pull/962)\n- ci: fix lint on release workflow by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;982](https://redirect.github.com/apache/arrow-go/pull/982)\n- fix: remove superfluous struct field index compare by\n[@&#8203;Willem-J-an](https://redirect.github.com/Willem-J-an) in\n[#&#8203;970](https://redirect.github.com/apache/arrow-go/pull/970)\n- fix(ipc): reject truncated message frames by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;991](https://redirect.github.com/apache/arrow-go/pull/991)\n- fix(internal/utils): handle reader reset and short peeks by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;987](https://redirect.github.com/apache/arrow-go/pull/987)\n- fix(arrow/ipc): propagate CLI stream errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;986](https://redirect.github.com/apache/arrow-go/pull/986)\n- fix(arrow/flight): avoid blocking after stream cancellation by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;989](https://redirect.github.com/apache/arrow-go/pull/989)\n- fix(parquet/variant): handle empty metadata keys by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;985](https://redirect.github.com/apache/arrow-go/pull/985)\n- fix(flight/flightsql/example): propagate row iteration errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;990](https://redirect.github.com/apache/arrow-go/pull/990)\n- fix(parquet): reject truncated scalar bit reads by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;944](https://redirect.github.com/apache/arrow-go/pull/944)\n- fix: Fix arm64 assembly stack frame manipulation by\n[@&#8203;nsrip-dd](https://redirect.github.com/nsrip-dd) in\n[#&#8203;984](https://redirect.github.com/apache/arrow-go/pull/984)\n- fix(arrow/array): support zero-column tables by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;988](https://redirect.github.com/apache/arrow-go/pull/988)\n- docs(parquet): Add thrift generation contribution info by\n[@&#8203;C-Loftus](https://redirect.github.com/C-Loftus) in\n[#&#8203;971](https://redirect.github.com/apache/arrow-go/pull/971)\n- fix(array): reject mismatched known null counts during full validation\nby [@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;993](https://redirect.github.com/apache/arrow-go/pull/993)\n- fix(arrow/array): `Resize(0)` on non-empty builder causes memory leak\nby [@&#8203;serramatutu](https://redirect.github.com/serramatutu) in\n[#&#8203;995](https://redirect.github.com/apache/arrow-go/pull/995)\n- feat(parquet/pqarrow): Read support for extension types / geo by\n[@&#8203;C-Loftus](https://redirect.github.com/C-Loftus) in\n[#&#8203;969](https://redirect.github.com/apache/arrow-go/pull/969)\n- fix(array): validate run-end encoded invariants by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;992](https://redirect.github.com/apache/arrow-go/pull/992)\n- fix(array): validate binary_view and string_view payloads by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;994](https://redirect.github.com/apache/arrow-go/pull/994)\n- fix(parquet): wrap DELTA_BINARY_PACKED deltas at the physical type\nwidth by [@&#8203;tom-eon](https://redirect.github.com/tom-eon) in\n[#&#8203;1027](https://redirect.github.com/apache/arrow-go/pull/1027)\n- fix(arrow/compute): use is_null/is_not_null in expressions by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;997](https://redirect.github.com/apache/arrow-go/pull/997)\n- fix(arrow/ipc): reject NaN minSpaceSavings values by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;998](https://redirect.github.com/apache/arrow-go/pull/998)\n- fix(arrow/tensor): default nil dimension names to empty strings by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;999](https://redirect.github.com/apache/arrow-go/pull/999)\n- fix(arrow/scalar): allow null extension scalars to keep null storage\nby [@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1000](https://redirect.github.com/apache/arrow-go/pull/1000)\n- fix(arrow/scalar): handle nil child values in run-end scalars by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1001](https://redirect.github.com/apache/arrow-go/pull/1001)\n- fix(arrow/array): ignore zero-count union appends by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1026](https://redirect.github.com/apache/arrow-go/pull/1026)\n- fix(arrow/csv): preserve decimal precision by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1022](https://redirect.github.com/apache/arrow-go/pull/1022)\n- fix(parquet/metadata): compare bloom filter caps in bits by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;996](https://redirect.github.com/apache/arrow-go/pull/996)\n- fix(parquet/pqarrow): reject timestamp coercion overflow by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1008](https://redirect.github.com/apache/arrow-go/pull/1008)\n- fix(parquet/file): reject records starting with nonzero repetition\nlevels by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1009](https://redirect.github.com/apache/arrow-go/pull/1009)\n- fix(parquet/variant): reject truncated scalar values by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1017](https://redirect.github.com/apache/arrow-go/pull/1017)\n- fix(parquet/encoding): reject invalid delta byte prefixes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1014](https://redirect.github.com/apache/arrow-go/pull/1014)\n- fix(parquet/metadata): preserve infinite floating-point statistics by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1007](https://redirect.github.com/apache/arrow-go/pull/1007)\n- fix(parquet/encoding): bound lazy dictionary growth by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1011](https://redirect.github.com/apache/arrow-go/pull/1011)\n- fix(parquet/encoding): respect declared RLE payload lengths by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1015](https://redirect.github.com/apache/arrow-go/pull/1015)\n- fix(parquet/encoding): validate delta binary packed headers by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1012](https://redirect.github.com/apache/arrow-go/pull/1012)\n- fix(parquet/encoding): reject invalid delta length payloads by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1013](https://redirect.github.com/apache/arrow-go/pull/1013)\n- fix(parquet/pqarrow): preserve INT96 timestamps across units by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1006](https://redirect.github.com/apache/arrow-go/pull/1006)\n- fix(parquet/pqarrow): Change interface name for arrow type on read for\nextension types by\n[@&#8203;C-Loftus](https://redirect.github.com/C-Loftus) in\n[#&#8203;1028](https://redirect.github.com/apache/arrow-go/pull/1028)\n- chore: Bump docker/login-action from 4.4.0 to 4.5.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1030](https://redirect.github.com/apache/arrow-go/pull/1030)\n- fix(arrow/csv): respect timestamp timezones by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1025](https://redirect.github.com/apache/arrow-go/pull/1025)\n- fix(parquet/compress): avoid closing caller streams for uncompressed\nwrappers by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1003](https://redirect.github.com/apache/arrow-go/pull/1003)\n- fix(parquet/compress): allocate uncompressed output for short\ndestinations by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1004](https://redirect.github.com/apache/arrow-go/pull/1004)\n- fix(internal/utils): keep byteReader position after failed seeks by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1005](https://redirect.github.com/apache/arrow-go/pull/1005)\n- fix(parquet/file): count repeated skips in logical rows by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1002](https://redirect.github.com/apache/arrow-go/pull/1002)\n- fix(parquet/metadata): reject malformed fixed-width statistics by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1010](https://redirect.github.com/apache/arrow-go/pull/1010)\n- fix(arrow/array): skip empty table chunks by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1023](https://redirect.github.com/apache/arrow-go/pull/1023)\n- fix(arrow/scalar): reject negative array lengths by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1024](https://redirect.github.com/apache/arrow-go/pull/1024)\n- fix(arrow/array): validate fixed-size list child lengths by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1019](https://redirect.github.com/apache/arrow-go/pull/1019)\n- fix(parquet/pqarrow): calculate large decimal widths by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1034](https://redirect.github.com/apache/arrow-go/pull/1034)\n- fix(parquet/schema): reject invalid fixed lengths by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1035](https://redirect.github.com/apache/arrow-go/pull/1035)\n- fix(parquet/encoding): reject truncated plain boolean data by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1031](https://redirect.github.com/apache/arrow-go/pull/1031)\n- fix(parquet/metadata): return page index decode errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1032](https://redirect.github.com/apache/arrow-go/pull/1032)\n- fix(parquet/metadata): validate adaptive bloom options by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1033](https://redirect.github.com/apache/arrow-go/pull/1033)\n- fix(tmpl): report generated output write errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1037](https://redirect.github.com/apache/arrow-go/pull/1037)\n- fix(arrow/array): propagate embedded pointer nullability by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1038](https://redirect.github.com/apache/arrow-go/pull/1038)\n- fix(flight/flightsql/driver): convert Date32 values by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1039](https://redirect.github.com/apache/arrow-go/pull/1039)\n- fix(arrow/scalar): support same-width time casts by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1040](https://redirect.github.com/apache/arrow-go/pull/1040)\n- fix(arrow/scalar): use duration type for units by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1041](https://redirect.github.com/apache/arrow-go/pull/1041)\n- fix(arrow/array): validate union child upper bound by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1043](https://redirect.github.com/apache/arrow-go/pull/1043)\n- chore: Bump google.golang.org/grpc from 1.82.1 to 1.83.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1092](https://redirect.github.com/apache/arrow-go/pull/1092)\n- chore: Bump docker/login-action from 4.5.1 to 4.6.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1093](https://redirect.github.com/apache/arrow-go/pull/1093)\n- refactor(arrow/avro): migrate from hamba/avro to twmb/avro by\n[@&#8203;prochac](https://redirect.github.com/prochac) in\n[#&#8203;830](https://redirect.github.com/apache/arrow-go/pull/830)\n- fix(arrow/array): validate struct child lengths by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1021](https://redirect.github.com/apache/arrow-go/pull/1021)\n- fix(arrow/cdata): validate imported schema topology by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1046](https://redirect.github.com/apache/arrow-go/pull/1046)\n- fix(parquet/metadata): validate row-group ordinals in\nPageIndexReader.WillNeed by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1057](https://redirect.github.com/apache/arrow-go/pull/1057)\n- fix(arrow/flight): limit descriptors to next payload by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1042](https://redirect.github.com/apache/arrow-go/pull/1042)\n- fix(parquet): propagate nonce generation failures by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1045](https://redirect.github.com/apache/arrow-go/pull/1045)\n- fix(parquet/metadata): reject negative row-group indices in Subset by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1055](https://redirect.github.com/apache/arrow-go/pull/1055)\n- fix(parquet/metadata): use default writer properties when props is nil\nby [@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1058](https://redirect.github.com/apache/arrow-go/pull/1058)\n- fix(parquet/metadata): return schema initialization errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1056](https://redirect.github.com/apache/arrow-go/pull/1056)\n- fix(parquet/pqarrow): propagate level builder errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1049](https://redirect.github.com/apache/arrow-go/pull/1049)\n- fix(arrow/flight/session): clone option values by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1044](https://redirect.github.com/apache/arrow-go/pull/1044)\n- fix(parquet/variant): validate metadata offset tables by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1063](https://redirect.github.com/apache/arrow-go/pull/1063)\n- fix(arrow/csv): release Reader record builders by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1059](https://redirect.github.com/apache/arrow-go/pull/1059)\n- fix(parquet/pqarrow): release partially constructed field readers by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1061](https://redirect.github.com/apache/arrow-go/pull/1061)\n- fix(arrow/array): make dictionary suffix null indexes relative by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1064](https://redirect.github.com/apache/arrow-go/pull/1064)\n- fix(parquet/pqarrow): prefer stored schema extension type on read by\n[@&#8203;twuebi](https://redirect.github.com/twuebi) in\n[#&#8203;1051](https://redirect.github.com/apache/arrow-go/pull/1051)\n- fix(arrow/flight/flightsql): avoid double-retaining parameter readers\nby [@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1066](https://redirect.github.com/apache/arrow-go/pull/1066)\n- fix(flightsql): isolate example SQLite databases by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1067](https://redirect.github.com/apache/arrow-go/pull/1067)\n- fix(arrow): return an owned slice from Schema.FieldsByName by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1069](https://redirect.github.com/apache/arrow-go/pull/1069)\n- fix(arrow/ipc): return errors for malformed file record batches by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1062](https://redirect.github.com/apache/arrow-go/pull/1062)\n- fix(flight): return Unauthenticated for invalid unary auth tokens by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1070](https://redirect.github.com/apache/arrow-go/pull/1070)\n- fix(arrow/array): always release JSON reader builder by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1065](https://redirect.github.com/apache/arrow-go/pull/1065)\n- fix(flight): accept case-insensitive authorization schemes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1071](https://redirect.github.com/apache/arrow-go/pull/1071)\n- fix(parquet): reject short page writes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1050](https://redirect.github.com/apache/arrow-go/pull/1050)\n- fix(parquet/file): close file on OpenParquetFile errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1053](https://redirect.github.com/apache/arrow-go/pull/1053)\n- fix(parquet): reject truncated footer signatures by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1074](https://redirect.github.com/apache/arrow-go/pull/1074)\n- fix(arrow/ipc): reject non-dictionary initial messages by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1054](https://redirect.github.com/apache/arrow-go/pull/1054)\n- feat(flightsql): allow configuring the driver memory allocator by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1076](https://redirect.github.com/apache/arrow-go/pull/1076)\n- fix(arrow): avoid retains on invalid chunk construction by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1078](https://redirect.github.com/apache/arrow-go/pull/1078)\n- fix(parquet): row-align V1 pages with offset indexes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1075](https://redirect.github.com/apache/arrow-go/pull/1075)\n- fix(arrow/array): clean up partial table construction by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1077](https://redirect.github.com/apache/arrow-go/pull/1077)\n- fix(parquet_reader): propagate batch read errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1081](https://redirect.github.com/apache/arrow-go/pull/1081)\n- fix(arrow): isolate schemas created by AddField by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1079](https://redirect.github.com/apache/arrow-go/pull/1079)\n- fix(arrow/util): avoid protobuf map schema goroutines by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1082](https://redirect.github.com/apache/arrow-go/pull/1082)\n- fix(array): format date arrays in String by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1068](https://redirect.github.com/apache/arrow-go/pull/1068)\n- fix(parquet): report initialized row-group columns correctly by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1072](https://redirect.github.com/apache/arrow-go/pull/1072)\n- fix(parquet): serialize column chunk file offsets by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1073](https://redirect.github.com/apache/arrow-go/pull/1073)\n- fix(arrow/util): release protobuf record builders by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1084](https://redirect.github.com/apache/arrow-go/pull/1084)\n- fix(arrow/util): release protobuf enum dictionaries by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1083](https://redirect.github.com/apache/arrow-go/pull/1083)\n- fix(arrow/avro): stop reader workers before reuse by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1048](https://redirect.github.com/apache/arrow-go/pull/1048)\n- fix(parquet): restore Arrow duration units from schema metadata by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1095](https://redirect.github.com/apache/arrow-go/pull/1095)\n- fix(compute): round negative HalfToOdd ties correctly by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1094](https://redirect.github.com/apache/arrow-go/pull/1094)\n- perf(arrow/bitutil): add SetBitSwap/ClearBitSwap, use in\nbuilder.SetNull by\n[@&#8203;tom-eon](https://redirect.github.com/tom-eon) in\n[#&#8203;1090](https://redirect.github.com/apache/arrow-go/pull/1090)\n- chore: Bump modernc.org/sqlite from 1.54.0 to 1.55.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1091](https://redirect.github.com/apache/arrow-go/pull/1091)\n- fix(array): compare sparse union slices at relative offsets by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1096](https://redirect.github.com/apache/arrow-go/pull/1096)\n- fix(arrow/ipc): make writer failures terminal by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1047](https://redirect.github.com/apache/arrow-go/pull/1047)\n- feat(parquet): add WithDictionaryCostFallback writer property by\n[@&#8203;tom-eon](https://redirect.github.com/tom-eon) in\n[#&#8203;1036](https://redirect.github.com/apache/arrow-go/pull/1036)\n- fix(parquet/pqarrow): release extension arrays after chunking by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1060](https://redirect.github.com/apache/arrow-go/pull/1060)\n- fix(parquet/encoding): read ahead in the streaming value buffer by\n[@&#8203;joechenrh](https://redirect.github.com/joechenrh) in\n[#&#8203;937](https://redirect.github.com/apache/arrow-go/pull/937)\n- fix(parquet/variant): preserve metadata keys when cloning by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1080](https://redirect.github.com/apache/arrow-go/pull/1080)\n- test(arrow/util): cover zero-field protobuf records by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1085](https://redirect.github.com/apache/arrow-go/pull/1085)\n- fix(arrow/scalar): preserve metadata in FromScalar by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1100](https://redirect.github.com/apache/arrow-go/pull/1100)\n- fix(arrow/scalar): preserve binary cast target types by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1101](https://redirect.github.com/apache/arrow-go/pull/1101)\n- fix(arrow/scalar): support boolean identity casts by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1102](https://redirect.github.com/apache/arrow-go/pull/1102)\n- fix(arrow/util): handle invalid protobuf Any values by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1086](https://redirect.github.com/apache/arrow-go/pull/1086)\n- fix(arrow/array): avoid copying initialized extension refcounts by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1089](https://redirect.github.com/apache/arrow-go/pull/1089)\n- fix(parquet): preserve dictionary index types by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1098](https://redirect.github.com/apache/arrow-go/pull/1098)\n- fix(parquet_reader): propagate statistics decoding errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1087](https://redirect.github.com/apache/arrow-go/pull/1087)\n- fix(arrow/array): avoid panics cleaning partial tables by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1103](https://redirect.github.com/apache/arrow-go/pull/1103)\n- fix(arrow/ipc): release reader state on schema errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1112](https://redirect.github.com/apache/arrow-go/pull/1112)\n- fix(arrow/scalar): format extension values from storage scalars by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1111](https://redirect.github.com/apache/arrow-go/pull/1111)\n- fix(arrow/array): isolate table columns from caller slices by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1105](https://redirect.github.com/apache/arrow-go/pull/1105)\n- fix(arrow/scalar): preserve exact decimal string casts by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1107](https://redirect.github.com/apache/arrow-go/pull/1107)\n- fix(arrow/tensor): validate typed constructor data by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1104](https://redirect.github.com/apache/arrow-go/pull/1104)\n- fix(arrow/scalar): format interval scalars in String by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1115](https://redirect.github.com/apache/arrow-go/pull/1115)\n- fix(arrow/array): preserve rows for empty-schema JSON input by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1117](https://redirect.github.com/apache/arrow-go/pull/1117)\n- fix(arrow/util): return protobuf record conversion errors by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1088](https://redirect.github.com/apache/arrow-go/pull/1088)\n- fix(arrow/scalar): validate FromScalar destinations by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1106](https://redirect.github.com/apache/arrow-go/pull/1106)\n- fix(arrow/scalar): use wide offsets for large binary scalars by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1108](https://redirect.github.com/apache/arrow-go/pull/1108)\n- fix(arrow/scalar): apply timestamp zones when casting to dates by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1110](https://redirect.github.com/apache/arrow-go/pull/1110)\n- fix(arrow/ipc): validate file block framing and lengths by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1097](https://redirect.github.com/apache/arrow-go/pull/1097)\n- fix(arrow/scalar): release partial children after failed struct\nextraction by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1118](https://redirect.github.com/apache/arrow-go/pull/1118)\n- fix(arrow/ipc): return errors for invalid record indexes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1120](https://redirect.github.com/apache/arrow-go/pull/1120)\n- fix(parquet): read null fixed-size-list parents by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1099](https://redirect.github.com/apache/arrow-go/pull/1099)\n- fix(arrow/scalar): release partial children after failed struct\nconversion by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1114](https://redirect.github.com/apache/arrow-go/pull/1114)\n- chore: Bump github.com/klauspost/compress from 1.19.1 to 1.19.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1166](https://redirect.github.com/apache/arrow-go/pull/1166)\n- chore: Bump github.com/pierrec/lz4/v4 from 4.1.27 to 4.1.28 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1168](https://redirect.github.com/apache/arrow-go/pull/1168)\n- chore: Bump modernc.org/sqlite from 1.55.0 to 1.56.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1165](https://redirect.github.com/apache/arrow-go/pull/1165)\n- chore: Bump github.com/twmb/avro from 1.5.0 to 1.7.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1167](https://redirect.github.com/apache/arrow-go/pull/1167)\n- fix(parquet/metadata): return error for missing column metadata by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1137](https://redirect.github.com/apache/arrow-go/pull/1137)\n- fix(arrow/array): widen dictionary index bounds checks by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1129](https://redirect.github.com/apache/arrow-go/pull/1129)\n- fix(compute/exprs): avoid empty field reference panic by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1130](https://redirect.github.com/apache/arrow-go/pull/1130)\n- fix(compute): handle odd temporal multiples by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1119](https://redirect.github.com/apache/arrow-go/pull/1119)\n- fix(arrow/csv): preserve first row during headerless inference by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1145](https://redirect.github.com/apache/arrow-go/pull/1145)\n- fix(arrow/array): bound table reader batches by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1146](https://redirect.github.com/apache/arrow-go/pull/1146)\n- fix(arrow/extensions): compare JSON storage types semantically by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1150](https://redirect.github.com/apache/arrow-go/pull/1150)\n- fix(arrow/ipc): preserve minSpaceSavings in stream writer by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1143](https://redirect.github.com/apache/arrow-go/pull/1143)\n- fix(parquet/compress): honor LZ4 raw destination contract by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1147](https://redirect.github.com/apache/arrow-go/pull/1147)\n- fix(arrow): reject duplicate union type codes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1131](https://redirect.github.com/apache/arrow-go/pull/1131)\n- fix(parquet/metadata): handle invalid row-group column indexes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1125](https://redirect.github.com/apache/arrow-go/pull/1125)\n- fix(compute/exec): handle missing validity buffers by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1136](https://redirect.github.com/apache/arrow-go/pull/1136)\n- fix(compute/exec): report ListView buffer counts by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1133](https://redirect.github.com/apache/arrow-go/pull/1133)\n- fix(arrow): include KeysSorted in map equality by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1134](https://redirect.github.com/apache/arrow-go/pull/1134)\n- fix(compute): reject non-array-like filters by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1135](https://redirect.github.com/apache/arrow-go/pull/1135)\n- fix(memory): handle zero-length checked reallocations by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1132](https://redirect.github.com/apache/arrow-go/pull/1132)\n- fix(parquet/pqarrow): validate reader indexes by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1124](https://redirect.github.com/apache/arrow-go/pull/1124)\n- fix(arrow/csv): validate custom converter row counts by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1148](https://redirect.github.com/apache/arrow-go/pull/1148)\n- fix(parquet/encoding): honor BufferWriter offsets by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n[#&#8203;1144](https://redirect.github.com/apache/arrow-go/pull/1144)\n- fix(flightsql): stop row streaming after close by\n[@&#8203;fallintoplace](https://redirect.github.com/fallintoplace) in\n\n> [!IMPORTANT]\n> ✂ PR body was truncated to here.\n\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Friday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4xODguMiIsInVwZGF0ZWRJblZlciI6IjQ0LjE4OC4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-10-30T14:29:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66beb81c61088782dca5d7b112845a136ae4e0e1"
        },
        "date": 1793385246934,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.96093227812896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.49412492269636,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.203645833333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.98046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.24156922408,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.24156922408,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003107,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192555.5243987204,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033147.5600002683,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46834758202331,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7801988415037,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1648960024743,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.735026041666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.4375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.49474564305,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.49474564305,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003156,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115130.1715180865,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140760.374929058,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.681815378367293,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a",
          "message": "feat(clickhouse_exporter): TLS Config Support (#4506)\n\n# Change summary\n\nAdd TLS and mutual TLS (mTLS) support to the ClickHouse exporter. All\ncode paths now construct a provider-neutral hyper HTTP client rather\nthan relying on clickhouse::Client::default(), which hard-codes a\nspecific rustls crypto provider. This keeps the exporter aligned with\nthe workspace's crypto-* feature selection (ring, aws-lc, openssl, or\nsymcrypt). When a tls configuration block is present, a custom hyper\nHTTPS client is built with the configured CA trust anchors and optional\nclient certificate (mTLS). Without a tls block, the endpoint scheme\ndecides: https:// endpoints get a provider-neutral HTTPS connector with\nsystem CA trust anchors, while http:// endpoints use a plaintext\nconnector. The implementation reuses the shared TlsClientConfig type and\nbuild_rustls_client_config utility used by other exporters (e.g. OTLP\ngRPC) for a consistent configuration surface.\n\nKey changes:\n\n- Optional tls field on ClickHouse exporter Config/ConfigPatch\n(backward-compatible via serde defaults)\n- build_client is now async and returns Result, constructing\nprovider-neutral hyper clients for all code paths instead of ever\ncalling Client::default()\n- New build_plaintext_client helper for explicit plaintext (no TLS)\nconnections with TCP keepalive and pool idle timeout tuning\n- Removed rustls-tls-ring and rustls-tls-native-roots features from the\nclickhouse dependency; only lz4 is retained (with default-features =\nfalse)\n- hyper-rustls and rustls removed as direct dependencies of the\nclickhouse feature; TLS is handled via the shared\nbuild_rustls_client_config loader and the workspace crypto provider\n- New validation rejects http:// endpoints that carry TLS material\n(custom CA, client certs) without insecure: true, preventing\ncertificates from being silently ignored over plaintext\n- tls block with insecure: true and no custom CA on a non-https endpoint\nnow succeeds (returns a plaintext client), consistent with the shared\nTLS loader's contract and other components\n- New TlsConfigError variant on ClickhouseExporterError\n- Documentation and changelog entry\n\n## Related issue\n\n* Closes #4505\n\n## Validation\n\n- 13 new unit tests across config.rs (deserialization), writer.rs\n(build_client with various TLS configurations), and tls_utils.rs\n(build_rustls_client_config_from_material)\n- Existing integration tests in logs_otlp.rs and transform_batch.rs\nupdated for the new async/fallible build_client signature; no tests\nremoved or weakened\n- Validated end-to-end against a live ClickHouse instance with mTLS\nenforced\n\n## User-facing changes\n\nThe ClickHouse exporter now accepts an optional tls configuration block\nfor custom CA certificates and client certificate authentication (mTLS),\nusing the same TlsClientConfig fields as other exporters. The exporter\nno longer bundles its own rustls crypto provider, deferring to the\nworkspace-level crypto-* feature selection. See the\n.chloggen/clickhouse-exporter-mtls-support.yaml entry.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-10-31T00:38:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a"
        },
        "date": 1793410629382,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87828115498091,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28089088237569,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.32805203735,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.32805203735,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002444,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190443.813137797,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028518.2302322905,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.446512920838494,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678954094648361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7282299478023,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.97341967314215,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.641536458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98317.85322945607,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.38619132114,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002612,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3123831.6255819793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140828.9412890107,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.770023760496006,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a",
          "message": "feat(clickhouse_exporter): TLS Config Support (#4506)\n\n# Change summary\n\nAdd TLS and mutual TLS (mTLS) support to the ClickHouse exporter. All\ncode paths now construct a provider-neutral hyper HTTP client rather\nthan relying on clickhouse::Client::default(), which hard-codes a\nspecific rustls crypto provider. This keeps the exporter aligned with\nthe workspace's crypto-* feature selection (ring, aws-lc, openssl, or\nsymcrypt). When a tls configuration block is present, a custom hyper\nHTTPS client is built with the configured CA trust anchors and optional\nclient certificate (mTLS). Without a tls block, the endpoint scheme\ndecides: https:// endpoints get a provider-neutral HTTPS connector with\nsystem CA trust anchors, while http:// endpoints use a plaintext\nconnector. The implementation reuses the shared TlsClientConfig type and\nbuild_rustls_client_config utility used by other exporters (e.g. OTLP\ngRPC) for a consistent configuration surface.\n\nKey changes:\n\n- Optional tls field on ClickHouse exporter Config/ConfigPatch\n(backward-compatible via serde defaults)\n- build_client is now async and returns Result, constructing\nprovider-neutral hyper clients for all code paths instead of ever\ncalling Client::default()\n- New build_plaintext_client helper for explicit plaintext (no TLS)\nconnections with TCP keepalive and pool idle timeout tuning\n- Removed rustls-tls-ring and rustls-tls-native-roots features from the\nclickhouse dependency; only lz4 is retained (with default-features =\nfalse)\n- hyper-rustls and rustls removed as direct dependencies of the\nclickhouse feature; TLS is handled via the shared\nbuild_rustls_client_config loader and the workspace crypto provider\n- New validation rejects http:// endpoints that carry TLS material\n(custom CA, client certs) without insecure: true, preventing\ncertificates from being silently ignored over plaintext\n- tls block with insecure: true and no custom CA on a non-https endpoint\nnow succeeds (returns a plaintext client), consistent with the shared\nTLS loader's contract and other components\n- New TlsConfigError variant on ClickhouseExporterError\n- Documentation and changelog entry\n\n## Related issue\n\n* Closes #4505\n\n## Validation\n\n- 13 new unit tests across config.rs (deserialization), writer.rs\n(build_client with various TLS configurations), and tls_utils.rs\n(build_rustls_client_config_from_material)\n- Existing integration tests in logs_otlp.rs and transform_batch.rs\nupdated for the new async/fallible build_client signature; no tests\nremoved or weakened\n- Validated end-to-end against a live ClickHouse instance with mTLS\nenforced\n\n## User-facing changes\n\nThe ClickHouse exporter now accepts an optional tls configuration block\nfor custom CA certificates and client certificate authentication (mTLS),\nusing the same TlsClientConfig fields as other exporters. The exporter\nno longer bundles its own rustls crypto provider, deferring to the\nworkspace-level crypto-* feature selection. See the\n.chloggen/clickhouse-exporter-mtls-support.yaml entry.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-10-31T00:38:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a"
        },
        "date": 1793467439619,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.68626955842147,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08816818673674,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.318489583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.04296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.06745944911,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.06745944911,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00301,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3113955.0261029988,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139592.5438213462,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.670645623289484,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678719401359558,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92717072589228,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.47649686944423,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.848307291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98319.60217036527,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.13505259356,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003172,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187856.4413586813,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029089.3966673585,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.42059294273777,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a",
          "message": "feat(clickhouse_exporter): TLS Config Support (#4506)\n\n# Change summary\n\nAdd TLS and mutual TLS (mTLS) support to the ClickHouse exporter. All\ncode paths now construct a provider-neutral hyper HTTP client rather\nthan relying on clickhouse::Client::default(), which hard-codes a\nspecific rustls crypto provider. This keeps the exporter aligned with\nthe workspace's crypto-* feature selection (ring, aws-lc, openssl, or\nsymcrypt). When a tls configuration block is present, a custom hyper\nHTTPS client is built with the configured CA trust anchors and optional\nclient certificate (mTLS). Without a tls block, the endpoint scheme\ndecides: https:// endpoints get a provider-neutral HTTPS connector with\nsystem CA trust anchors, while http:// endpoints use a plaintext\nconnector. The implementation reuses the shared TlsClientConfig type and\nbuild_rustls_client_config utility used by other exporters (e.g. OTLP\ngRPC) for a consistent configuration surface.\n\nKey changes:\n\n- Optional tls field on ClickHouse exporter Config/ConfigPatch\n(backward-compatible via serde defaults)\n- build_client is now async and returns Result, constructing\nprovider-neutral hyper clients for all code paths instead of ever\ncalling Client::default()\n- New build_plaintext_client helper for explicit plaintext (no TLS)\nconnections with TCP keepalive and pool idle timeout tuning\n- Removed rustls-tls-ring and rustls-tls-native-roots features from the\nclickhouse dependency; only lz4 is retained (with default-features =\nfalse)\n- hyper-rustls and rustls removed as direct dependencies of the\nclickhouse feature; TLS is handled via the shared\nbuild_rustls_client_config loader and the workspace crypto provider\n- New validation rejects http:// endpoints that carry TLS material\n(custom CA, client certs) without insecure: true, preventing\ncertificates from being silently ignored over plaintext\n- tls block with insecure: true and no custom CA on a non-https endpoint\nnow succeeds (returns a plaintext client), consistent with the shared\nTLS loader's contract and other components\n- New TlsConfigError variant on ClickhouseExporterError\n- Documentation and changelog entry\n\n## Related issue\n\n* Closes #4505\n\n## Validation\n\n- 13 new unit tests across config.rs (deserialization), writer.rs\n(build_client with various TLS configurations), and tls_utils.rs\n(build_rustls_client_config_from_material)\n- Existing integration tests in logs_otlp.rs and transform_batch.rs\nupdated for the new async/fallible build_client signature; no tests\nremoved or weakened\n- Validated end-to-end against a live ClickHouse instance with mTLS\nenforced\n\n## User-facing changes\n\nThe ClickHouse exporter now accepts an optional tls configuration block\nfor custom CA certificates and client certificate authentication (mTLS),\nusing the same TlsClientConfig fields as other exporters. The exporter\nno longer bundles its own rustls crypto provider, deferring to the\nworkspace-level crypto-* feature selection. See the\n.chloggen/clickhouse-exporter-mtls-support.yaml entry.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-10-31T00:38:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a"
        },
        "date": 1793497290160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88622948183857,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19399443929565,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.073567708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.62109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.77907204932,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.77907204932,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002779,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192267.368653635,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033515.3597603017,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.465239564446705,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.017358379438519478,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80018384904953,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34323070957736,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.0828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.62109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98313.168069699,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.23364303127,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003844,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118750.375571179,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140706.712863919,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.717105309575423,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a",
          "message": "feat(clickhouse_exporter): TLS Config Support (#4506)\n\n# Change summary\n\nAdd TLS and mutual TLS (mTLS) support to the ClickHouse exporter. All\ncode paths now construct a provider-neutral hyper HTTP client rather\nthan relying on clickhouse::Client::default(), which hard-codes a\nspecific rustls crypto provider. This keeps the exporter aligned with\nthe workspace's crypto-* feature selection (ring, aws-lc, openssl, or\nsymcrypt). When a tls configuration block is present, a custom hyper\nHTTPS client is built with the configured CA trust anchors and optional\nclient certificate (mTLS). Without a tls block, the endpoint scheme\ndecides: https:// endpoints get a provider-neutral HTTPS connector with\nsystem CA trust anchors, while http:// endpoints use a plaintext\nconnector. The implementation reuses the shared TlsClientConfig type and\nbuild_rustls_client_config utility used by other exporters (e.g. OTLP\ngRPC) for a consistent configuration surface.\n\nKey changes:\n\n- Optional tls field on ClickHouse exporter Config/ConfigPatch\n(backward-compatible via serde defaults)\n- build_client is now async and returns Result, constructing\nprovider-neutral hyper clients for all code paths instead of ever\ncalling Client::default()\n- New build_plaintext_client helper for explicit plaintext (no TLS)\nconnections with TCP keepalive and pool idle timeout tuning\n- Removed rustls-tls-ring and rustls-tls-native-roots features from the\nclickhouse dependency; only lz4 is retained (with default-features =\nfalse)\n- hyper-rustls and rustls removed as direct dependencies of the\nclickhouse feature; TLS is handled via the shared\nbuild_rustls_client_config loader and the workspace crypto provider\n- New validation rejects http:// endpoints that carry TLS material\n(custom CA, client certs) without insecure: true, preventing\ncertificates from being silently ignored over plaintext\n- tls block with insecure: true and no custom CA on a non-https endpoint\nnow succeeds (returns a plaintext client), consistent with the shared\nTLS loader's contract and other components\n- New TlsConfigError variant on ClickhouseExporterError\n- Documentation and changelog entry\n\n## Related issue\n\n* Closes #4505\n\n## Validation\n\n- 13 new unit tests across config.rs (deserialization), writer.rs\n(build_client with various TLS configurations), and tls_utils.rs\n(build_rustls_client_config_from_material)\n- Existing integration tests in logs_otlp.rs and transform_batch.rs\nupdated for the new async/fallible build_client signature; no tests\nremoved or weakened\n- Validated end-to-end against a live ClickHouse instance with mTLS\nenforced\n\n## User-facing changes\n\nThe ClickHouse exporter now accepts an optional tls configuration block\nfor custom CA certificates and client certificate authentication (mTLS),\nusing the same TlsClientConfig fields as other exporters. The exporter\nno longer bundles its own rustls crypto provider, deferring to the\nworkspace-level crypto-* feature selection. See the\n.chloggen/clickhouse-exporter-mtls-support.yaml entry.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-10-31T00:38:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a"
        },
        "date": 1793557498637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678719401359558,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.61022776048836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 66.03301096863896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.604947916666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98312.79273091446,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98321.32502217063,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007328,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3180757.1696988316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028446.1090931883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.350633689910076,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008679425343871117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76869102552804,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15702581542743,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.60546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.62109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98310.12638092085,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98318.65913502239,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004073,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107899.3025497715,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1138575.4639770698,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.61047282267804,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a",
          "message": "feat(clickhouse_exporter): TLS Config Support (#4506)\n\n# Change summary\n\nAdd TLS and mutual TLS (mTLS) support to the ClickHouse exporter. All\ncode paths now construct a provider-neutral hyper HTTP client rather\nthan relying on clickhouse::Client::default(), which hard-codes a\nspecific rustls crypto provider. This keeps the exporter aligned with\nthe workspace's crypto-* feature selection (ring, aws-lc, openssl, or\nsymcrypt). When a tls configuration block is present, a custom hyper\nHTTPS client is built with the configured CA trust anchors and optional\nclient certificate (mTLS). Without a tls block, the endpoint scheme\ndecides: https:// endpoints get a provider-neutral HTTPS connector with\nsystem CA trust anchors, while http:// endpoints use a plaintext\nconnector. The implementation reuses the shared TlsClientConfig type and\nbuild_rustls_client_config utility used by other exporters (e.g. OTLP\ngRPC) for a consistent configuration surface.\n\nKey changes:\n\n- Optional tls field on ClickHouse exporter Config/ConfigPatch\n(backward-compatible via serde defaults)\n- build_client is now async and returns Result, constructing\nprovider-neutral hyper clients for all code paths instead of ever\ncalling Client::default()\n- New build_plaintext_client helper for explicit plaintext (no TLS)\nconnections with TCP keepalive and pool idle timeout tuning\n- Removed rustls-tls-ring and rustls-tls-native-roots features from the\nclickhouse dependency; only lz4 is retained (with default-features =\nfalse)\n- hyper-rustls and rustls removed as direct dependencies of the\nclickhouse feature; TLS is handled via the shared\nbuild_rustls_client_config loader and the workspace crypto provider\n- New validation rejects http:// endpoints that carry TLS material\n(custom CA, client certs) without insecure: true, preventing\ncertificates from being silently ignored over plaintext\n- tls block with insecure: true and no custom CA on a non-https endpoint\nnow succeeds (returns a plaintext client), consistent with the shared\nTLS loader's contract and other components\n- New TlsConfigError variant on ClickhouseExporterError\n- Documentation and changelog entry\n\n## Related issue\n\n* Closes #4505\n\n## Validation\n\n- 13 new unit tests across config.rs (deserialization), writer.rs\n(build_client with various TLS configurations), and tls_utils.rs\n(build_rustls_client_config_from_material)\n- Existing integration tests in logs_otlp.rs and transform_batch.rs\nupdated for the new async/fallible build_client signature; no tests\nremoved or weakened\n- Validated end-to-end against a live ClickHouse instance with mTLS\nenforced\n\n## User-facing changes\n\nThe ClickHouse exporter now accepts an optional tls configuration block\nfor custom CA certificates and client certificate authentication (mTLS),\nusing the same TlsClientConfig fields as other exporters. The exporter\nno longer bundles its own rustls crypto provider, deferring to the\nworkspace-level crypto-* feature selection. See the\n.chloggen/clickhouse-exporter-mtls-support.yaml entry.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-10-31T00:38:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a"
        },
        "date": 1793583519447,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.3335111141204834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76777174972501,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30909990720693,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.2703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99988.93896694618,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96655.79656711998,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003437,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3122624.8744769297,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140151.0104374597,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.30664880309075,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.0027119379956275225,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93282132395203,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.44009864364982,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.682552083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.05078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.76964902958,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.43621001154,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002378,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190843.2815646883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3034371.2234591544,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.45053978291608,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "cf12f51cd25be9cd9e325d84e2d932892dd102a6",
          "message": "chore(release): Prepare Go v0.60.2 and Rust v0.63.0 (#4619)\n\n## Go v0.60.2 and Rust v0.63.0\n\nThis PR contains the automatically calculated release plan.\n\n```json\n{\n  \"schema_version\": 1,\n  \"base_sha\": \"9dcda8ad4c61e731ae9b1ba7444f8884160c8b1a\",\n  \"branch\": \"otelbot/release-go-v0.60.2-rust-v0.63.0\",\n  \"summary\": \"Go v0.60.2 and Rust v0.63.0\",\n  \"go\": {\n    \"target\": \"go\",\n    \"has_changes\": true,\n    \"impact\": \"patch\",\n    \"entry_count\": 1,\n    \"last_tag\": \"go/v0.60.1\",\n    \"last_version\": \"0.60.1\",\n    \"next_version\": \"0.60.2\"\n  },\n  \"rust\": {\n    \"target\": \"rust\",\n    \"has_changes\": true,\n    \"impact\": \"minor\",\n    \"entry_count\": 32,\n    \"last_tag\": \"rust/otap-dataflow/v0.62.0\",\n    \"last_version\": \"0.62.0\",\n    \"next_version\": \"0.63.0\"\n  }\n}\n```\n\n## What's Changed (Go :hamster:)\n\n### :bulb: Enhancements :bulb:\n\n- `dependencies`: Upgrade various Go dependencies.\n([#4442](https://github.com/open-telemetry/otel-arrow/issues/4442),\n[#4478](https://github.com/open-telemetry/otel-arrow/issues/4478),\n[#4550](https://github.com/open-telemetry/otel-arrow/issues/4550),\n[#4598](https://github.com/open-telemetry/otel-arrow/issues/4598),\n[#4599](https://github.com/open-telemetry/otel-arrow/issues/4599))\n\n## What's Changed (Rust :crab:)\n\n### :stop_sign: Breaking changes :stop_sign:\n\n- `observability`: Migrate clickhouse exporter metrics to enum\nattributes\n([#3530](https://github.com/open-telemetry/otel-arrow/issues/3530))\n\n- `observability`: `receiver.processing.duration` no longer includes the\n`signal` dimension.\n([#3822](https://github.com/open-telemetry/otel-arrow/issues/3822))\n\n- `pipeline`: Migrated ETW receiver telemetry from flat counters to\ndimensioned metric populations with enum attributes.\n([#3530](https://github.com/open-telemetry/otel-arrow/issues/3530))\n- `query-engine`: OTAP query-engine now validates fields in expressions\nare valid for signal type\n([#4070](https://github.com/open-telemetry/otel-arrow/issues/4070))\n\n### :rocket: New components :rocket:\n\n- `pipeline`: Add an opt-in Windows performance-counter receiver with\nexperimental stability.\n([#4074](https://github.com/open-telemetry/otel-arrow/issues/4074))\n\n- `pipeline`: Add an Azure Key Vault SASL credential extension that\nretrieves both username and password at startup.\n([#4336](https://github.com/open-telemetry/otel-arrow/issues/4336))\n- `pipeline`: Add opt-in Azure Key Vault Basic Auth credentials for\nOracle, PostgreSQL, and SQL Server receivers.\n([#4533](https://github.com/open-telemetry/otel-arrow/issues/4533))\n- `pipeline`: Add an opt-in SQL Server query-polling receiver with\nverified native TLS by default and acknowledged file checkpoints.\n([#3918](https://github.com/open-telemetry/otel-arrow/issues/3918),\n[#4361](https://github.com/open-telemetry/otel-arrow/issues/4361))\n\n### :bulb: Enhancements :bulb:\n\n- `dependencies`: Upgrade weaver to v0.28.0. This allows the traffic\ngenerator to parse semantic conventions registries that use `unref`.\n([#4405](https://github.com/open-telemetry/otel-arrow/issues/4405))\n\n- `dependencies`: Upgrade various Rust dependencies.\n([#4444](https://github.com/open-telemetry/otel-arrow/issues/4444),\n[#4529](https://github.com/open-telemetry/otel-arrow/issues/4529),\n[#4531](https://github.com/open-telemetry/otel-arrow/issues/4531),\n[#4600](https://github.com/open-telemetry/otel-arrow/issues/4600),\n[#4601](https://github.com/open-telemetry/otel-arrow/issues/4601))\n- `engine`: The admin dashboard displays Windows private committed\nmemory alongside RSS, with a separate MiB value and history chart.\n([#4572](https://github.com/open-telemetry/otel-arrow/issues/4572))\n- `engine`: Windows engine memory telemetry now reports private\ncommitted memory as memory.commit alongside the existing memory.rss\nmetric.\n([#4540](https://github.com/open-telemetry/otel-arrow/issues/4540))\n- `engine`: Propagate materialized constant and UUID v7 context members\nas outbound transport headers.\n([#3919](https://github.com/open-telemetry/otel-arrow/issues/3919),\n[#4202](https://github.com/open-telemetry/otel-arrow/issues/4202))\n- `observability`: `receiver.processing.duration` now includes\nreceiver-local processing failures before signal classification.\n([#3822](https://github.com/open-telemetry/otel-arrow/issues/3822))\n\n- `pdata`: Decode large syslog batches cooperatively and support up to\n8,192 messages or 2 MiB per batch while keeping small batches\nsynchronous.\n([#3452](https://github.com/open-telemetry/otel-arrow/issues/3452))\n- `pdata`: Support encoded telemetry through pluggable codecs,\npreserving OTLP pass-through and lazy conversion to OTAP.\n([#3452](https://github.com/open-telemetry/otel-arrow/issues/3452))\n- `pipeline`: Enable verified TLS by default for the PostgreSQL\nreceiver, with optional private CA files and an explicit plaintext\nopt-out.\n([#4166](https://github.com/open-telemetry/otel-arrow/issues/4166))\n\n- `pipeline`: Add scalar and snapshot watermark modes to the PostgreSQL\nreceiver.\n([#4166](https://github.com/open-telemetry/otel-arrow/issues/4166))\n\n- `pipeline`: Add TLS and mTLS support to the ClickHouse exporter via a\nnew `tls` config block.\n([#4505](https://github.com/open-telemetry/otel-arrow/issues/4505))\n\n- `pipeline`: Add an experimental Rust SDK for authoring WASM processor\nplugins with typed configuration, host services, and existing data\nkernels.\n([#4454](https://github.com/open-telemetry/otel-arrow/issues/4454))\n- `pipeline`: Syslog receiver preserves original messages in encoded\npdata until native processing is required.\n([#3452](https://github.com/open-telemetry/otel-arrow/issues/3452))\n- `pipeline`: Reduce CPU usage when receiving syslog messages and\nconverting RFC 3164 timestamps, including wrapped CEF.\n([#3452](https://github.com/open-telemetry/otel-arrow/issues/3452))\n- `query-engine`: OTAP query-engine now supports metric data point\nfields as target (LHS) of assignment operations.\n([#4070](https://github.com/open-telemetry/otel-arrow/issues/4070))\n- `query-engine`: OTAP query-engine now supports if/else conditional\nbranches inside `apply data_points` pipelines.\n([#3722](https://github.com/open-telemetry/otel-arrow/issues/3722))\n\n### :toolbox: Bug fixes :toolbox:\n\n- `dependencies`: Fix a potential cryptographic timing side channel by\nupdating ctutils to 0.4.4 (RUSTSEC-2026-0385).\n([#4517](https://github.com/open-telemetry/otel-arrow/issues/4517))\n- `engine`: Reject full-config reconciliation while any pipeline has an\nin-flight rollout.\n([#3865](https://github.com/open-telemetry/otel-arrow/issues/3865))\n- `engine`: Reject full-config changes that cannot be applied to the\nrunning process.\n([#3847](https://github.com/open-telemetry/otel-arrow/issues/3847))\n- `engine`: Prevent senders of local MSPC channel becoming blocked under\ncondition of channel at capacity and multiple senders.\n([#4568](https://github.com/open-telemetry/otel-arrow/issues/4568))\n- `engine`: Prevent competing pipeline inputs from repeatedly losing\navailable queue capacity while reducing sender scheduling overhead under\ncontention.\n([#4568](https://github.com/open-telemetry/otel-arrow/issues/4568))\n- `pipeline`: SQL Server polling recovers from transient connection\ndeadlines, unreachable networks, and DNS failures without restarting the\nreceiver.\n([#4460](https://github.com/open-telemetry/otel-arrow/issues/4460))\n- `pipeline`: Backpressure admission policy now pauses pdata intake as\nsoon as any route is blocked, preventing unexpected NACKs.\n([#4535](https://github.com/open-telemetry/otel-arrow/issues/4535))\n\n- `query-engine`: `coalesce` now falls back to later arguments when an\nearlier argument is absent from every record in a batch, instead of\nreturning null for every record.\n([#4300](https://github.com/open-telemetry/otel-arrow/issues/4300))\n\n\nAfter this PR merges, Push Release starts automatically and waits for\napproval from the protected `release` environment.\n\nCo-authored-by: otelbot-arrow[bot] <289780372+otelbot-arrow[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-02T16:09:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf12f51cd25be9cd9e325d84e2d932892dd102a6"
        },
        "date": 1793640349189,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75678199561888,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30105197827774,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.233723958333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.01953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98319.77662729095,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.77662729095,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003391,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3112647.371880504,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1138710.2872203735,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.658405649962756,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88315905978496,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.36499033779083,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.884895833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.46875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96646.71395257117,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96646.71395257117,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.012387,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3186022.259670946,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026073.7302124705,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.965655316894356,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "b99ff8adaeaa2b76907cc8dd0e0b64695a74a6cf",
          "message": "feat(geneva_exporter): consume Azure Identity Auth tokens (#4474)\n\n# Change summary\n\nAdd `externaltoken` authentication to the Geneva exporter, allowing it\nto consume Entra tokens from a bound Azure Identity Auth extension.\n\n- Adapt the shared `bearer_token_provider` capability to Geneva's GCS\ntoken-provider interface.\n- Validate that external-token authentication has a non-empty resource\nand a shared capability binding.\n- Reject empty, expired, near-expiry, and unexpected-resource tokens\nbefore passing them to Geneva.\n- Keep Entra token acquisition, caching, refresh, readiness, and\ntelemetry in Azure Identity Auth while preserving Geneva's existing\ningestion-credential and routing caches.\n- Run managed identity SDK calls on a bounded blocking worker so Azure\nArc filesystem and agent-detection work does not block the\nthread-per-core runtime.\n- Document configuration, lifecycle, timeout, and cancellation behavior.\n- Add unit and integration coverage for capability resolution,\nvalidation, token adaptation, and uploads.\n- Add a changelog entry for the new authentication option.\n\n## Related issue\n\n* Closes #4457\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-11-03T00:15:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b99ff8adaeaa2b76907cc8dd0e0b64695a74a6cf"
        },
        "date": 1793669770233,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91842342419027,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3521960240407,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.085286458333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.04328460462,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.04328460462,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003228,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192984.275953476,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033604.798799232,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.47277347634768,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75190306790242,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3101630224832,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.984114583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.43359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.0129784204,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.0129784204,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00345,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3122842.840237824,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140697.412315247,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.76041116743306,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "57cf7fd2089d820bfd01a74efb7ed377d67ad32a",
          "message": "chore(MSRV): Lower the MSRV from 1.98.1 to 1.96.1 (#4624)\n\n# Chore Summary\n\nLower the Rust toolchain and OTAP Dataflow MSRV from `1.98.1` to\n`1.96.1`.\n\n`cfg_select!` stabilized in `1.95.0`, but the current locked\ndependencies require at least `1.96.0`. No dependency downgrades are\nneeded.\n\n* Related to open-telemetry/otel-arrow#4458, the earlier toolchain\nupgrade\n* The jump to `1.98` was not actually required by the dependency set we\nhave",
          "timestamp": "2026-11-03T17:20:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/57cf7fd2089d820bfd01a74efb7ed377d67ad32a"
        },
        "date": 1793729124418,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.85878909033899,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25764796785658,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.660286458333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.84827819688,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96661.27457856643,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003347,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192071.3982268,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029407.541352384,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.02327030285825,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.01735536754131317,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82001571587215,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08986263096624,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.992838541666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.93359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.7961034703,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98312.73060607463,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004111,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3114574.851513277,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140941.5164386472,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.680280186631606,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "65541beabc7b2e2e5df20aafd8e768894d2dccf5",
          "message": "chore(filelog): add file identity and local ownership (#4566)\n\n## Summary\n\nImplements Task 12 in #2844: checks whether a file is the same one\ntracked before and whether its saved position is safe to reuse. Prevents\ntwo local readers from reading the same file and gives replacement files\ntheir own identity and starting position.\n\nFiles admitted together are saved as one checkpoint transaction: all\nsucceed or none do. A replacement that reuses the old file’s system\nidentifier can use its tracking slot even at capacity. If a checkpoint\nwrite has an uncertain result, ownership stays held until the result is\nresolved or the component shuts down.\n\nThe checkpoint format is unchanged.\n\n## Bounded admission work\n\nChecking a new file against saved history now happens in small steps: at\nmost 64 saved records or 64 KiB of file-prefix comparisons per call. The\nworker can handle reads, checkpoints, and control commands between\nsteps.\n\nPaused checks keep no file handles or shared probe slots open. Before\nusing the result, Filelog reopens the file and checks it again. If probe\nslots are busy, it keeps the completed work for later. Changes to saved\nidentity information or checkpoint-write recovery require a fresh scan.\n\nThe scan uses the existing checkpoint table and avoids unnecessary\ncopying, allocation, and hashing. File-matching rules are unchanged.\n\n## Specification amendment\n\nA read error does not prove the file changed. Keep its saved identity\nand position, and resume only after the checks succeed.\n\nIf the checked bytes differ or the file ends before the required bytes\ncan be read, follow the configured recovery mismatch policy. A\ncheckpoint sync failure still stops the receiver.\n\n## Validation\n\nLocal Linux ARM64 Docker/Colima with native Linux storage and Rust\n1.98.1:\n\n- 67 identity and 197 checkpoint tests passed, along with lint and\nformat checks.\n\nThe [measurement\nguide](https://github.com/open-telemetry/otel-arrow/blob/e32a53b5a5805f128ceebe4e0e56ac3aee194e1f/rust/otap-dataflow/crates/core-nodes/src/receivers/filelog_receiver/docs/identity-performance.md)\nincludes commands and timing/memory results. In one unoptimized local\nrun, admitting 10,000 files took 6.89 seconds; the longest scan step\ntook 0.879 ms. Small steps let other work run, but total comparison work\nstill grows quadratically. These results cover this component, not the\ncomplete receiver.\n\n## Remaining integration\n\nThis component still needs to be connected to the receiver’s\nfile-opening checks, workers, scheduling, and Ack/Nack handling. Memory\nuse must also be checked across the complete receiver.\n\nThe worker must handle other work between scan steps. Checkpoint\ncompaction is still needed to keep running without reaching storage\nlimits.",
          "timestamp": "2026-11-04T02:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/65541beabc7b2e2e5df20aafd8e768894d2dccf5"
        },
        "date": 1793760655215,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677448146045208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77488430987067,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22681192125125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.318229166666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.44140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98334.04893473664,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.51604838434,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003143,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3128838.173705162,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1141352.7700498889,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.82122300955444,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678954094648361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.09818221475052,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.74568884070317,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.160026041666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98316.77179204494,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.30466005254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003272,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3162233.4036056665,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2995998.276082426,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.160931659847826,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "65541beabc7b2e2e5df20aafd8e768894d2dccf5",
          "message": "chore(filelog): add file identity and local ownership (#4566)\n\n## Summary\n\nImplements Task 12 in #2844: checks whether a file is the same one\ntracked before and whether its saved position is safe to reuse. Prevents\ntwo local readers from reading the same file and gives replacement files\ntheir own identity and starting position.\n\nFiles admitted together are saved as one checkpoint transaction: all\nsucceed or none do. A replacement that reuses the old file’s system\nidentifier can use its tracking slot even at capacity. If a checkpoint\nwrite has an uncertain result, ownership stays held until the result is\nresolved or the component shuts down.\n\nThe checkpoint format is unchanged.\n\n## Bounded admission work\n\nChecking a new file against saved history now happens in small steps: at\nmost 64 saved records or 64 KiB of file-prefix comparisons per call. The\nworker can handle reads, checkpoints, and control commands between\nsteps.\n\nPaused checks keep no file handles or shared probe slots open. Before\nusing the result, Filelog reopens the file and checks it again. If probe\nslots are busy, it keeps the completed work for later. Changes to saved\nidentity information or checkpoint-write recovery require a fresh scan.\n\nThe scan uses the existing checkpoint table and avoids unnecessary\ncopying, allocation, and hashing. File-matching rules are unchanged.\n\n## Specification amendment\n\nA read error does not prove the file changed. Keep its saved identity\nand position, and resume only after the checks succeed.\n\nIf the checked bytes differ or the file ends before the required bytes\ncan be read, follow the configured recovery mismatch policy. A\ncheckpoint sync failure still stops the receiver.\n\n## Validation\n\nLocal Linux ARM64 Docker/Colima with native Linux storage and Rust\n1.98.1:\n\n- 67 identity and 197 checkpoint tests passed, along with lint and\nformat checks.\n\nThe [measurement\nguide](https://github.com/open-telemetry/otel-arrow/blob/e32a53b5a5805f128ceebe4e0e56ac3aee194e1f/rust/otap-dataflow/crates/core-nodes/src/receivers/filelog_receiver/docs/identity-performance.md)\nincludes commands and timing/memory results. In one unoptimized local\nrun, admitting 10,000 files took 6.89 seconds; the longest scan step\ntook 0.879 ms. Small steps let other work run, but total comparison work\nstill grows quadratically. These results cover this component, not the\ncomplete receiver.\n\n## Remaining integration\n\nThis component still needs to be connected to the receiver’s\nfile-opening checks, workers, scheduling, and Ack/Nack handling. Memory\nuse must also be checked across the complete receiver.\n\nThe worker must handle other work between scan steps. Checkpoint\ncompaction is still needed to keep running without reaching storage\nlimits.",
          "timestamp": "2026-11-04T02:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/65541beabc7b2e2e5df20aafd8e768894d2dccf5"
        },
        "date": 1793813175833,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6583632230758667,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80321534892411,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21119023712056,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.286458333333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.86328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99980.75270249207,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.70859956554,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003229,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116035.120432161,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139772.8585318734,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.691917002843123,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677448146045208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89809173465541,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27801411837716,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.176302083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.77734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98334.0669617046,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.53407378802,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003132,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190126.8334794138,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028378.2020922005,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.44454112077739,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "63dcfba3f1c12d2bff082b76a48b5de753e26935",
          "message": "chore(metrics): Add pull-based receiver lifecycle helper (#4570)\n\n# Chore Summary\n\nPR 3/3 behind:\n- #4554\n- #4555\n\nAdd reusable, policy-gated collection instrumentation in\n`otap::metrics`, with automatic outcome and duration accounting and\ncancellation-safe, exactly-once recording.\n\nSample instrumentation:\n\n```rust\n// Node-specific: accept bounded work before starting the attempt.\nlet output = self.metrics.boundary.processing()\n    .run(async |attempt| {\n        // Node-specific: collect, validate, and encode while servicing controls.\n        self.collect_with_controls(attempt.cancellation_handle()).await\n    })\n    .await?; // Ok records success; Err or a dropped future records failure.\n\n// Node-specific: handoff cannot change the collection outcome.\neffect_handler.send_message(output).await?;\n```\n\nThe APIs deliberately look similar, but account for different units of\nwork:\n\n```text\nINGRESS: one external message\n  ReceiverMetrics::processing()\n    .run(|processing| { ... })\n      |\n      +--> ReceiverMetrics::record(completed)\n             |\n             +--> receiver.received (input signal, outcome)\n\nPULL: one accepted collection attempt\n  ReceiverPullMetrics::processing()\n    .run(async |attempt| { ... })\n      |\n      +--> Ok                   --> record success\n      +--> Err / dropped future --> record failure\n      +--> cancellation_handle().cancel()\n             |\n             +--> record failure BEFORE cleanup\n                  late completion cannot count again\n\n  BOTH:\n    May produce zero or more output messages.\n    Reuse ReceiverProcessingMetricSet for duration.\n\n  WHY SEPARATE:\n    Ingress counts external messages with input signal attribution.\n    Pull counts collection attempts without requiring a signal.\n    Pull accounting can finish before cancelled work returns.\n```\n\n## Related issue\n\n#3822",
          "timestamp": "2026-11-04T23:44:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63dcfba3f1c12d2bff082b76a48b5de753e26935"
        },
        "date": 1793842744478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.65827476978302,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88348288900366,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3855700475679,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.346484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.87890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99986.02909311783,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.98592969852,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003263,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193939.996393926,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3025720.021799332,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48251213726166,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.64896734873187,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.04964731249038,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.550130208333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.24609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.77545714751,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.20142422084,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003335,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118800.102199532,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140574.7035102802,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.71923430640708,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "cbe4458526756dc7f6e7c1a0a9fac7440a59e0db",
          "message": "chore(context): Add Edit and Merge context bindings (#4615)\n\n# Change summary\n\nFollows #4567, #4571, and #4589.\n\nNew declarations and bindings to replace existing context `Edit` and\ncontext `Merge` operations.\n\n## Related issue\n\nPart 3D of https://github.com/open-telemetry/otel-arrow/issues/4541\n\n## Validation\n\n✅\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCopilot-Session: de1bba42-b074-4ac0-8091-fb9b603d726e\nCopilot-Session: 72c4458a-d88c-45db-9904-6f8d6a3cae37\nCopilot-Session: f401029c-6abf-42a1-a759-a5c087300b8c\nCopilot-Session: 29cbf765-3f4b-487c-b78e-fc8107663b11\nCopilot-Session: 045a2e8d-a251-416c-ad41-dcc4be0331c2",
          "timestamp": "2026-11-05T06:14:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbe4458526756dc7f6e7c1a0a9fac7440a59e0db"
        },
        "date": 1793899525748,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8896373560738,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3483090288551,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.538541666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.9453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.47554024297,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.47554024297,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002354,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3188025.2218094612,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3025703.9088613284,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.4218674440576,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.011389151215553284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.709888254834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26927374301677,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.265104166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.98828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.06855354468,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98314.87004910494,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008013,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3120261.5065610358,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140481.3403888096,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.737432038536706,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "022818814d1d1fe69f24ba58b0b8c4e4197d4149",
          "message": "feat(pdata): add codec bindings and named configurations (#4480)\n\n# Change summary\n\nAdd optional codec configuration, with independent settings for each\nencoding and reusable named configurations.\n\nTo understand what this PR enables, start with the [codec user\nguide](https://github.com/lquerel/otel-arrow/blob/pdata-codec-series-06-codec-configuration/rust/otap-dataflow/docs/codecs.md)\nand [developer\nguide](https://github.com/lquerel/otel-arrow/blob/pdata-codec-series-06-codec-configuration/rust/otap-dataflow/crates/pdata-codec/README.md).\n\n- `codecs.bindings` selects an implementation and its settings for an\nencoding. Bindings can use inline settings or reference\n`codecs.configurations`.\n- Configuration inherits from root to group to pipeline. Omitting\n`codecs` preserves discovery of linked implementations and their\ndefaults; it is not an allowlist.\n- Startup preparation validates settings and selects factories. Mutable\ncodec instances remain lazy and local to each pipeline runtime.\n- Selected settings participate in pipeline redeployment decisions and\nlive configuration reconciliation.\n- User and developer guides explain configuration, validation\nboundaries, and extension contracts. Prepared factories accommodate\nfuture hosted codecs; WASM loading and execution are not implemented\nhere.\n\nFor example, bind `otlp-bytes` to a named configuration of the\n`otel.otlp` implementation:\n\n```yaml\ncodecs:\n  configurations:\n    checked_otlp:\n      implementation: otel.otlp\n      decode:\n        validation: strict\n  bindings:\n    otlp-bytes:\n      configuration: checked_otlp\n```\n\n## Related issue\n\nPart of #3452.\n\n## Validation\n\n`cargo xtask check` passed \n\n## User-facing changes\n\nAdds optional codec bindings and named configurations. Existing\nconfigurations keep the built-in `best_effort` default. Strict OTLP\ndecoding adds CPU and allocation cost when decoding is required; it does\nnot validate opaque\npass-through traffic.",
          "timestamp": "2026-11-06T00:48:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/022818814d1d1fe69f24ba58b0b8c4e4197d4149"
        },
        "date": 1793929102899,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.675289273262024,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7776058830504,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1685797995374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.08359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.59375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99988.18909848291,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98313.09761649274,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003887,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3120041.80151743,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140749.6964027665,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.73576946673299,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678954094648361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86386254898117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.38986954843689,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.934635416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98318.55289936526,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.08592195435,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002185,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3195610.623993792,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3036608.1491489564,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.49979996895525,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "d2c51006f5e004113de8dac0f391781c0cbda943",
          "message": "chore(perf-test): Remove push-based metrics machinery (#4586)\n\n# Change summary\n\nThis is the fifth and final pr in the series attempting to resolve\n#2731. This PR removes the push-based metrics machinery that's no longer\nneeded.\n\n## Related issue\n\n* Closes #2731 \n* Closes #4229\n\n## Validation\n\nSuites still run.\n\n## User-facing changes\n\nNone.",
          "timestamp": "2026-11-06T19:48:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2c51006f5e004113de8dac0f391781c0cbda943"
        },
        "date": 1793996946053,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677213452756405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86738867472889,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19838595410442,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.593229166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.09765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.35798908494,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.82582792833,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008243,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3199332.5341544403,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032563.3494140203,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.540054940228046,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008678719401359558,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75002280691488,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15850545370155,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.771354166666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.4296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98317.73750918893,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.27022958851,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00431,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3117438.5132533824,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140172.1426851428,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.705041856812723,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Asish",
            "username": "NopeCharon",
            "email": "115794254+NopeCharon@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "16023d6933cfc2b0a64335ec29764d368efec7ff",
          "message": "fix(retry_processor): NACK upstream when a retry cannot be scheduled (#4511)\n\n# Change summary\n\nWhen the retry processor gets a transient NACK, it subscribes a new\nretry frame onto the returned PData and passes it to the processor-local\ndelayed-resume scheduler (`requeue_later`). If the scheduler refuses the\nmessage, the message still carries that new frame. The scheduler refuses\nwhen its queue is full (it is bounded by the pdata channel capacity) or\nwhen shutdown has been latched. The `cannot requeue` NACK sent on that\npath then unwound back to the retry processor itself and never reached\nthe upstream subscriber. The termination was also not counted.\n\nOn a scheduling failure the processor now:\n\n- removes only the frame it just subscribed\n(`refused.context_mut().pop_frame()`), so the NACK unwinds to the\nupstream subscriber;\n- terminates the original NACK through the existing `terminate_nack`\npath. The payload is preserved, the reason becomes `cannot requeue:\n<original reason>` (the same prefix style as `final retry: ...` and\n`retry lost payload: ...`), and the configured `exhaustion_action` still\napplies;\n- counts the message in `processor.retry.messages.terminated` with a new\n`reason=\"scheduling_failure\"` value. `processor.retry.retries.scheduled`\nstill counts only retries the scheduler accepted.\n\nThe README now lists the new `reason` value and states that pending\nretries are bounded by `policies.channel_capacity.pdata`.\n\n## Related issue\n\n* Closes #3630\n\n## Validation\n\nNew test `test_retry_processor_scheduling_failure_nacks_upstream`, with\n`Scenario:`/`Guarantees:` docs:\n\n- builds the processor with a pdata channel capacity of 1, so the\ndelayed-resume queue has one slot;\n- sends two subscribed messages and NACKs both transiently. The first\nretry takes the only slot and the second is refused;\n- checks that the refused message is NACKed exactly once, to the\nupstream subscriber (node `4444`), with its own call data, the full\npayload, a transient flag and a `cannot requeue: simulated downstream\nfailure` reason;\n- checks the metrics: `retries_scheduled = 1` and `messages_terminated =\n[scheduling_failure: 1]`.\n\n`retry_metric_snapshots_preserve_enum_attribute_values` now also covers\nthe new enum value.\n\nI ran these locally on Linux aarch64 with the pinned 1.98.1 toolchain:\n\n- Without the fix (the `pop_frame` call removed), the new test fails:\nthe NACK goes back to the retry processor's own node (`left: 0, right:\n4444`).\n- `cargo test -p otel-arrow-dfe-core-nodes --lib`: 1511 passed, 0\nfailed.\n- `cargo clippy -p otel-arrow-dfe-core-nodes --all-targets -- -D\nwarnings`: clean.\n- `cargo fmt --all -- --check`: clean.\n- `python3 tools/sanitycheck.py`, `npx markdownlint-cli2` on the README,\nand `make chlog-validate`: pass.\n\nI didn't run the full-workspace `cargo xtask check` locally. The change\nis limited to the `core-nodes` crate.\n\n## User-facing changes\n\n- When the retry processor can't schedule a retry, the NACK now reaches\nthe upstream sender (reason `cannot requeue: ...`) instead of looping\nback into the retry processor. The configured `exhaustion_action`\napplies to it.\n- `processor.retry.messages.terminated` has a new `reason` value,\n`scheduling_failure`.\n\nChangelog entry:\n`rust/otap-dataflow/.chloggen/retry-processor-scheduling-failure.yaml`.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-11-06T23:35:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/16023d6933cfc2b0a64335ec29764d368efec7ff"
        },
        "date": 1794016030792,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016063692048192024,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75868194626364,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23163653222068,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.6265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.62163342931,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98318.08892881409,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004421,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3113071.694623304,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1137551.877072124,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.663264904154946,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81895744004731,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20104506454356,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.831510416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96675.96217586378,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.47508158235,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003168,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187578.3137405263,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028314.15439381,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.41864136527932,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Asish",
            "username": "NopeCharon",
            "email": "115794254+NopeCharon@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "16023d6933cfc2b0a64335ec29764d368efec7ff",
          "message": "fix(retry_processor): NACK upstream when a retry cannot be scheduled (#4511)\n\n# Change summary\n\nWhen the retry processor gets a transient NACK, it subscribes a new\nretry frame onto the returned PData and passes it to the processor-local\ndelayed-resume scheduler (`requeue_later`). If the scheduler refuses the\nmessage, the message still carries that new frame. The scheduler refuses\nwhen its queue is full (it is bounded by the pdata channel capacity) or\nwhen shutdown has been latched. The `cannot requeue` NACK sent on that\npath then unwound back to the retry processor itself and never reached\nthe upstream subscriber. The termination was also not counted.\n\nOn a scheduling failure the processor now:\n\n- removes only the frame it just subscribed\n(`refused.context_mut().pop_frame()`), so the NACK unwinds to the\nupstream subscriber;\n- terminates the original NACK through the existing `terminate_nack`\npath. The payload is preserved, the reason becomes `cannot requeue:\n<original reason>` (the same prefix style as `final retry: ...` and\n`retry lost payload: ...`), and the configured `exhaustion_action` still\napplies;\n- counts the message in `processor.retry.messages.terminated` with a new\n`reason=\"scheduling_failure\"` value. `processor.retry.retries.scheduled`\nstill counts only retries the scheduler accepted.\n\nThe README now lists the new `reason` value and states that pending\nretries are bounded by `policies.channel_capacity.pdata`.\n\n## Related issue\n\n* Closes #3630\n\n## Validation\n\nNew test `test_retry_processor_scheduling_failure_nacks_upstream`, with\n`Scenario:`/`Guarantees:` docs:\n\n- builds the processor with a pdata channel capacity of 1, so the\ndelayed-resume queue has one slot;\n- sends two subscribed messages and NACKs both transiently. The first\nretry takes the only slot and the second is refused;\n- checks that the refused message is NACKed exactly once, to the\nupstream subscriber (node `4444`), with its own call data, the full\npayload, a transient flag and a `cannot requeue: simulated downstream\nfailure` reason;\n- checks the metrics: `retries_scheduled = 1` and `messages_terminated =\n[scheduling_failure: 1]`.\n\n`retry_metric_snapshots_preserve_enum_attribute_values` now also covers\nthe new enum value.\n\nI ran these locally on Linux aarch64 with the pinned 1.98.1 toolchain:\n\n- Without the fix (the `pop_frame` call removed), the new test fails:\nthe NACK goes back to the retry processor's own node (`left: 0, right:\n4444`).\n- `cargo test -p otel-arrow-dfe-core-nodes --lib`: 1511 passed, 0\nfailed.\n- `cargo clippy -p otel-arrow-dfe-core-nodes --all-targets -- -D\nwarnings`: clean.\n- `cargo fmt --all -- --check`: clean.\n- `python3 tools/sanitycheck.py`, `npx markdownlint-cli2` on the README,\nand `make chlog-validate`: pass.\n\nI didn't run the full-workspace `cargo xtask check` locally. The change\nis limited to the `core-nodes` crate.\n\n## User-facing changes\n\n- When the retry processor can't schedule a retry, the NACK now reaches\nthe upstream sender (reason `cannot requeue: ...`) instead of looping\nback into the retry processor. The configured `exhaustion_action`\napplies to it.\n- `processor.retry.messages.terminated` has a new `reason` value,\n`scheduling_failure`.\n\nChangelog entry:\n`rust/otap-dataflow/.chloggen/retry-processor-scheduling-failure.yaml`.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-11-06T23:35:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/16023d6933cfc2b0a64335ec29764d368efec7ff"
        },
        "date": 1794072921693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84337076671264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16387750402947,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.640104166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.57714348646,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.57714348646,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002292,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3183824.9077237896,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3022317.7147882697,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.37911725256201,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.00805388018488884,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81872866188284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32054566473988,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.307942708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.94921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99982.64319840442,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.65618242524,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005295,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3120916.516969709,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1141378.9012251967,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.74093496121015,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "64ee1434abfbc578646f38e251a8b4a13c1d9845",
          "message": "chore(deps): update azure sdk crates (#4666)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [azure_core](https://redirect.github.com/azure/azure-sdk-for-rust) |\nworkspace.dependencies | patch | `1.2.0-beta.1` → `1.2.0-beta.2` |\n| [azure_identity](https://redirect.github.com/azure/azure-sdk-for-rust)\n| workspace.dependencies | patch | `1.1.0-beta.1` → `1.1.0-beta.2` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>azure/azure-sdk-for-rust (azure_core)</summary>\n\n###\n[`v1.2.0-beta.2`](https://redirect.github.com/Azure/azure-sdk-for-rust/releases/tag/azure_core%401.2.0-beta.2)\n\n[Compare\nSource](https://redirect.github.com/azure/azure-sdk-for-rust/compare/azure_core@1.2.0-beta.1...azure_core@1.2.0-beta.2)\n\n#### 1.2.0-beta.2 (2026-11-05)\n\n##### Features Added\n\n- Added `cloud::resource_manager::Audience` behind the\n`unstable_resource_manager` feature as the shared audience marker for\nAzure Resource Manager clients.\n- Added `CloudConfigurationExt::audience`,\n`CloudConfigurationExt::default_scope`, and\n`CloudConfigurationExt::endpoint` behind the `unstable_resource_manager`\nfeature to get authentication audiences, default credential scopes\nmatching Azure CLI, and management endpoints for known and custom\nclouds. `cloud::resource_manager::CloudConfigurationExt` is implemented\nfor `CloudConfiguration` when the `unstable_resource_manager` feature is\nenabled.\n- Added `CustomConfiguration::endpoint` behind the\n`unstable_resource_manager` feature to configure Azure Resource Manager\nendpoints.\n- Added `unstable_resource_manager` feature to support Azure Resource\nManager-specific functionality.\n- Added `http::poller::resource_manager::{new_poller, PollerOptions,\nFinalStateVia}` behind the `unstable_resource_manager` feature for\nResource Manager polling with optional final-state selection, method-\nand response-based fallbacks, and a callback for inline\noperation-location results.\n- `Error` now captures a `std::backtrace::Backtrace` at construction\ntime and includes it in `Debug` output (e.g., `{:?}`) when\n`RUST_BACKTRACE=1` is set.\n- Added bearer-token authorization options and\n`ClientOptions::allow_insecure_connections` to opt in to authenticated\nHTTP connections to `localhost` or `127.0.0.1` when the `test` feature\nis enabled.\n- Added experimental `http::fake` response extension traits and\npager/poller state propagation behind the `test` and `unstable_fake`\nfeatures for testing client libraries without network requests.\n- Added `http::fake::get_api_name()` to borrow operation names from\nrequest context without requiring a tracer provider or listener.\n- Added `http::fake::ResponseExt::fake_with_error()` to create JSON\nservice error responses.\n\n##### Bugs Fixed\n\n- Awaiting a `Poller` after fully consuming its stream no longer panics.\n- `check_success` would fail when error message contained escape\nsequences like CRLF\n([#&#8203;5364](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5364))\n- Avoided unnecessary error construction and backtrace capture in\n`Headers::get_optional_str()` when a header is absent\n([#&#8203;5446](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5446)).\n- Update TLS dependencies to distrust compromised emSign CAs\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Friday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4yMjAuMCIsInVwZGF0ZWRJblZlciI6IjQ0LjIyMC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-11-07T20:32:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/64ee1434abfbc578646f38e251a8b4a13c1d9845"
        },
        "date": 1794102627346,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008024795912206173,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80351441202727,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11985171455052,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.4765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.30078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.66025179884,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.66025179884,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003055,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3120099.3304739282,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139511.7579794165,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.732299813535676,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88532806569316,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28811253671356,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.635026041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98336.389329781,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.85647172785,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003342,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3185297.8485040287,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3021827.508536247,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.39466375858499,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "64ee1434abfbc578646f38e251a8b4a13c1d9845",
          "message": "chore(deps): update azure sdk crates (#4666)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [azure_core](https://redirect.github.com/azure/azure-sdk-for-rust) |\nworkspace.dependencies | patch | `1.2.0-beta.1` → `1.2.0-beta.2` |\n| [azure_identity](https://redirect.github.com/azure/azure-sdk-for-rust)\n| workspace.dependencies | patch | `1.1.0-beta.1` → `1.1.0-beta.2` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>azure/azure-sdk-for-rust (azure_core)</summary>\n\n###\n[`v1.2.0-beta.2`](https://redirect.github.com/Azure/azure-sdk-for-rust/releases/tag/azure_core%401.2.0-beta.2)\n\n[Compare\nSource](https://redirect.github.com/azure/azure-sdk-for-rust/compare/azure_core@1.2.0-beta.1...azure_core@1.2.0-beta.2)\n\n#### 1.2.0-beta.2 (2026-11-05)\n\n##### Features Added\n\n- Added `cloud::resource_manager::Audience` behind the\n`unstable_resource_manager` feature as the shared audience marker for\nAzure Resource Manager clients.\n- Added `CloudConfigurationExt::audience`,\n`CloudConfigurationExt::default_scope`, and\n`CloudConfigurationExt::endpoint` behind the `unstable_resource_manager`\nfeature to get authentication audiences, default credential scopes\nmatching Azure CLI, and management endpoints for known and custom\nclouds. `cloud::resource_manager::CloudConfigurationExt` is implemented\nfor `CloudConfiguration` when the `unstable_resource_manager` feature is\nenabled.\n- Added `CustomConfiguration::endpoint` behind the\n`unstable_resource_manager` feature to configure Azure Resource Manager\nendpoints.\n- Added `unstable_resource_manager` feature to support Azure Resource\nManager-specific functionality.\n- Added `http::poller::resource_manager::{new_poller, PollerOptions,\nFinalStateVia}` behind the `unstable_resource_manager` feature for\nResource Manager polling with optional final-state selection, method-\nand response-based fallbacks, and a callback for inline\noperation-location results.\n- `Error` now captures a `std::backtrace::Backtrace` at construction\ntime and includes it in `Debug` output (e.g., `{:?}`) when\n`RUST_BACKTRACE=1` is set.\n- Added bearer-token authorization options and\n`ClientOptions::allow_insecure_connections` to opt in to authenticated\nHTTP connections to `localhost` or `127.0.0.1` when the `test` feature\nis enabled.\n- Added experimental `http::fake` response extension traits and\npager/poller state propagation behind the `test` and `unstable_fake`\nfeatures for testing client libraries without network requests.\n- Added `http::fake::get_api_name()` to borrow operation names from\nrequest context without requiring a tracer provider or listener.\n- Added `http::fake::ResponseExt::fake_with_error()` to create JSON\nservice error responses.\n\n##### Bugs Fixed\n\n- Awaiting a `Poller` after fully consuming its stream no longer panics.\n- `check_success` would fail when error message contained escape\nsequences like CRLF\n([#&#8203;5364](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5364))\n- Avoided unnecessary error construction and backtrace capture in\n`Headers::get_optional_str()` when a header is absent\n([#&#8203;5446](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5446)).\n- Update TLS dependencies to distrust compromised emSign CAs\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Friday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4yMjAuMCIsInVwZGF0ZWRJblZlciI6IjQ0LjIyMC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-11-07T20:32:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/64ee1434abfbc578646f38e251a8b4a13c1d9845"
        },
        "date": 1794162240284,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008051934652030468,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75881954675624,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09630956978451,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.825,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.46484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 100001.1266132652,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.0060139816,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002844,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3117967.635789768,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139852.089571753,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.710508360793217,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7480980865972,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12082901554405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.959765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96651.83807694659,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.92897269291,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003908,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191211.9777230625,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033864.6387201473,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.45511693555808,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "64ee1434abfbc578646f38e251a8b4a13c1d9845",
          "message": "chore(deps): update azure sdk crates (#4666)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [azure_core](https://redirect.github.com/azure/azure-sdk-for-rust) |\nworkspace.dependencies | patch | `1.2.0-beta.1` → `1.2.0-beta.2` |\n| [azure_identity](https://redirect.github.com/azure/azure-sdk-for-rust)\n| workspace.dependencies | patch | `1.1.0-beta.1` → `1.1.0-beta.2` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>azure/azure-sdk-for-rust (azure_core)</summary>\n\n###\n[`v1.2.0-beta.2`](https://redirect.github.com/Azure/azure-sdk-for-rust/releases/tag/azure_core%401.2.0-beta.2)\n\n[Compare\nSource](https://redirect.github.com/azure/azure-sdk-for-rust/compare/azure_core@1.2.0-beta.1...azure_core@1.2.0-beta.2)\n\n#### 1.2.0-beta.2 (2026-11-05)\n\n##### Features Added\n\n- Added `cloud::resource_manager::Audience` behind the\n`unstable_resource_manager` feature as the shared audience marker for\nAzure Resource Manager clients.\n- Added `CloudConfigurationExt::audience`,\n`CloudConfigurationExt::default_scope`, and\n`CloudConfigurationExt::endpoint` behind the `unstable_resource_manager`\nfeature to get authentication audiences, default credential scopes\nmatching Azure CLI, and management endpoints for known and custom\nclouds. `cloud::resource_manager::CloudConfigurationExt` is implemented\nfor `CloudConfiguration` when the `unstable_resource_manager` feature is\nenabled.\n- Added `CustomConfiguration::endpoint` behind the\n`unstable_resource_manager` feature to configure Azure Resource Manager\nendpoints.\n- Added `unstable_resource_manager` feature to support Azure Resource\nManager-specific functionality.\n- Added `http::poller::resource_manager::{new_poller, PollerOptions,\nFinalStateVia}` behind the `unstable_resource_manager` feature for\nResource Manager polling with optional final-state selection, method-\nand response-based fallbacks, and a callback for inline\noperation-location results.\n- `Error` now captures a `std::backtrace::Backtrace` at construction\ntime and includes it in `Debug` output (e.g., `{:?}`) when\n`RUST_BACKTRACE=1` is set.\n- Added bearer-token authorization options and\n`ClientOptions::allow_insecure_connections` to opt in to authenticated\nHTTP connections to `localhost` or `127.0.0.1` when the `test` feature\nis enabled.\n- Added experimental `http::fake` response extension traits and\npager/poller state propagation behind the `test` and `unstable_fake`\nfeatures for testing client libraries without network requests.\n- Added `http::fake::get_api_name()` to borrow operation names from\nrequest context without requiring a tracer provider or listener.\n- Added `http::fake::ResponseExt::fake_with_error()` to create JSON\nservice error responses.\n\n##### Bugs Fixed\n\n- Awaiting a `Poller` after fully consuming its stream no longer panics.\n- `check_success` would fail when error message contained escape\nsequences like CRLF\n([#&#8203;5364](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5364))\n- Avoided unnecessary error construction and backtrace capture in\n`Headers::get_optional_str()` when a header is absent\n([#&#8203;5446](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5446)).\n- Update TLS dependencies to distrust compromised emSign CAs\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Friday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4yMjAuMCIsInVwZGF0ZWRJblZlciI6IjQ0LjIyMC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-11-07T20:32:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/64ee1434abfbc578646f38e251a8b4a13c1d9845"
        },
        "date": 1794189108460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016055552288889885,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72357552124028,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.153591160221,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.770963541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.53125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98331.09726250633,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.56440089803,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003317,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118917.257851508,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139047.995165695,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.72127656409073,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90458090354596,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19061259173426,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.156640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.71875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.12522030083,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.12522030083,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003178,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194753.07972188,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033737.3699893807,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.49073520484748,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "64ee1434abfbc578646f38e251a8b4a13c1d9845",
          "message": "chore(deps): update azure sdk crates (#4666)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [azure_core](https://redirect.github.com/azure/azure-sdk-for-rust) |\nworkspace.dependencies | patch | `1.2.0-beta.1` → `1.2.0-beta.2` |\n| [azure_identity](https://redirect.github.com/azure/azure-sdk-for-rust)\n| workspace.dependencies | patch | `1.1.0-beta.1` → `1.1.0-beta.2` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>azure/azure-sdk-for-rust (azure_core)</summary>\n\n###\n[`v1.2.0-beta.2`](https://redirect.github.com/Azure/azure-sdk-for-rust/releases/tag/azure_core%401.2.0-beta.2)\n\n[Compare\nSource](https://redirect.github.com/azure/azure-sdk-for-rust/compare/azure_core@1.2.0-beta.1...azure_core@1.2.0-beta.2)\n\n#### 1.2.0-beta.2 (2026-11-05)\n\n##### Features Added\n\n- Added `cloud::resource_manager::Audience` behind the\n`unstable_resource_manager` feature as the shared audience marker for\nAzure Resource Manager clients.\n- Added `CloudConfigurationExt::audience`,\n`CloudConfigurationExt::default_scope`, and\n`CloudConfigurationExt::endpoint` behind the `unstable_resource_manager`\nfeature to get authentication audiences, default credential scopes\nmatching Azure CLI, and management endpoints for known and custom\nclouds. `cloud::resource_manager::CloudConfigurationExt` is implemented\nfor `CloudConfiguration` when the `unstable_resource_manager` feature is\nenabled.\n- Added `CustomConfiguration::endpoint` behind the\n`unstable_resource_manager` feature to configure Azure Resource Manager\nendpoints.\n- Added `unstable_resource_manager` feature to support Azure Resource\nManager-specific functionality.\n- Added `http::poller::resource_manager::{new_poller, PollerOptions,\nFinalStateVia}` behind the `unstable_resource_manager` feature for\nResource Manager polling with optional final-state selection, method-\nand response-based fallbacks, and a callback for inline\noperation-location results.\n- `Error` now captures a `std::backtrace::Backtrace` at construction\ntime and includes it in `Debug` output (e.g., `{:?}`) when\n`RUST_BACKTRACE=1` is set.\n- Added bearer-token authorization options and\n`ClientOptions::allow_insecure_connections` to opt in to authenticated\nHTTP connections to `localhost` or `127.0.0.1` when the `test` feature\nis enabled.\n- Added experimental `http::fake` response extension traits and\npager/poller state propagation behind the `test` and `unstable_fake`\nfeatures for testing client libraries without network requests.\n- Added `http::fake::get_api_name()` to borrow operation names from\nrequest context without requiring a tracer provider or listener.\n- Added `http::fake::ResponseExt::fake_with_error()` to create JSON\nservice error responses.\n\n##### Bugs Fixed\n\n- Awaiting a `Poller` after fully consuming its stream no longer panics.\n- `check_success` would fail when error message contained escape\nsequences like CRLF\n([#&#8203;5364](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5364))\n- Avoided unnecessary error construction and backtrace capture in\n`Headers::get_optional_str()` when a header is absent\n([#&#8203;5446](https://redirect.github.com/Azure/azure-sdk-for-rust/pull/5446)).\n- Update TLS dependencies to distrust compromised emSign CAs\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Friday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4yMjAuMCIsInVwZGF0ZWRJblZlciI6IjQ0LjIyMC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-11-07T20:32:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/64ee1434abfbc578646f38e251a8b4a13c1d9845"
        },
        "date": 1794245728074,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86098553980678,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33546304163127,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.74140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.16015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.0662572572,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.0662572572,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002197,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187741.594817556,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3023620.181678134,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.41977734266305,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.01607450470328331,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78453091273757,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1386741363212,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.136588541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.73828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98343.02598048972,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.96013070841,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002872,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3113659.313187135,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139055.0834326919,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.66670642267851,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "2aabf4b62be2d173dbc405cf99b5959797942cbf",
          "message": "chore(ci): Use self-repository syntax per zizmor (#4698)\n\n# Chore Summary\n\nAddresses\nhttps://github.com/open-telemetry/otel-arrow/security/code-scanning/128",
          "timestamp": "2026-11-09T23:33:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2aabf4b62be2d173dbc405cf99b5959797942cbf"
        },
        "date": 1794276652696,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016069339588284492,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81110043472637,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28702318392581,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.569140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.04296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.98348262138,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.98348262138,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003468,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3129384.6222355426,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1141259.014719939,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.826952941097126,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8313155356438,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20306499146889,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.705859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.08984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98319.87006579233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.87006579233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008216,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194946.973572155,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030566.192611612,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.49543527096002,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "ab63c84dd85422901645e43cd730cabb33a84495",
          "message": "chore(docs): Add stacked PRs to contributing guidelines (#4699)\n\n# Change summary\n\nAdd some instructions to the contributing guidelines about stacked PRs.",
          "timestamp": "2026-11-10T16:25:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ab63c84dd85422901645e43cd730cabb33a84495"
        },
        "date": 1794332173371,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86267746020363,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18011589276057,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.136328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98338.10851195303,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.57550472279,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002293,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191558.2347167856,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028407.2207194995,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.45776480102362,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008044364862143993,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73005980448437,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2720356429559,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.408463541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.82421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.425371788,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.8405066618,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002945,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118446.442500883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139715.5969500146,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.71543132946421,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "1f0c53c6959f2f03afb620fd9a1150c508d66c49",
          "message": "chore(filelog): add bounded reader scheduling (#4627)\n\nAdds the logic that decides which log file to read next and how much\nwork it can do. Implements Task 15 of #2844 using the identity and\nownership support from #4566.\n\nFor example, if one file receives logs continuously while others receive\nonly a few lines, each file still gets a turn. When the open-file limit\nis reached, the scheduler can close a file, keep its unfinished record,\nand safely resume later.\n\n- Limits work per turn, open files, and memory used by unfinished\nrecords.\n  - Shares file-opening capacity between readers and discovery.\n- Preserves unfinished data and file ownership while waiting for\nresources or delivery.\n- Avoids repeated waits during discovery and unnecessary reads of\nunchanged files.\n  - Pauses safely when the batch sink cannot accept more data.\n\nThis is internal infrastructure; the Filelog receiver is not enabled\nyet. Batch construction follows in #4648. Worker hosting,\ndelivery/checkpoint handling, rotation controls, idle partial flushing,\nand remaining resource checks still need integration.",
          "timestamp": "2026-11-10T23:55:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f0c53c6959f2f03afb620fd9a1150c508d66c49"
        },
        "date": 1794364095612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008074894547462463,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77340829381045,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.99817327685791,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.090755208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.0703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.893774312,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96658.74176444802,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003264,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3118610.894516948,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1139326.5121397837,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.264137082570656,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89811443834593,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2703082747431,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.077864583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.93923432741,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.27270489812,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003088,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193594.6403100123,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031793.4778540796,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.47890512522882,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "bala6396",
            "username": "bala6396",
            "email": "balamuruganm96@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9ad67d5e3f7aace758462740ef43878e2c832260",
          "message": "chore(scraper): replace fs2 file locking (#4693)\n\n# Change summary\n\n- Replace the shared scraper's `fs2` checkpoint lock with\n`std::fs::File` locking.\n- Use the same standard-library locking API in the Filelog checkpoint\nstore.\n- Remove the unused `fs2` dependency and lockfile entry.\n\n## Related issue\n\n- Follow-up tracked in\nhttps://github.com/athomas9195/otel-arrow/issues/16\n- Addresses\nhttps://github.com/open-telemetry/otel-arrow/pull/4110#discussion_r4057427167\n\n## Validation\n\n- `cargo fmt --all --check`\n- `cargo check -p otel-arrow-dfe-scraper -p\notel-arrow-dfe-filelog-checkpoint`\n- `cargo test -p otel-arrow-dfe-scraper partition::tests`\n- `cargo test -p otel-arrow-dfe-filelog-checkpoint`\n- `cargo check -p otel-arrow-dfe-filelog-checkpoint --target\naarch64-unknown-linux-gnu`\n- `cargo clippy -p otel-arrow-dfe-scraper -p\notel-arrow-dfe-filelog-checkpoint --all-targets -- -D warnings`\n\nThe full local `cargo xtask check` reached workspace compilation but\ncould not complete because the Windows environment does not have OpenSSL\ndevelopment files. The affected targeted checks above pass.\n\n## User-facing changes\n\nNone.\n\nCo-authored-by: Balamurugan Murugesan <25451490+bala6396@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCopilot-Session: 597026ae-7075-4ee1-9534-96b418a7f44c",
          "timestamp": "2026-11-11T15:59:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9ad67d5e3f7aace758462740ef43878e2c832260"
        },
        "date": 1794418536732,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87059399038696,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16052148422433,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.287890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.95482412205,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.95482412205,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002265,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191798.6927935863,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032817.451537008,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46107538368064,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008135993033647537,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.6943759759869,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.00228757662761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.707291666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98336.03042326105,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.497596351,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003561,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3119775.7229301496,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1138544.030131745,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.728415745281072,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "5480e403d797571e31c24fa0a5676c0217764867",
          "message": "feat(query-engine): Adds support for function calls that access batch metadata (#4713)\n\n# Change summary\n\nBegins the implementation of what is described in RFC 0010\n(https://github.com/open-telemetry/otel-arrow/pull/4632). Basically we\nwant to add a collection of functions that can be called in the\ntransform processor (e.g. evaluated by the query-engine crate) to\ninspect / retrieve values from OTAP batch metadata.\n\nThis PR adds support for one single function\n`has_transport_header(header_name)`, and importantly it also adds all\nthe machinery needed to support invoking this function.\n\nI'll describe all the changes here:\n\n**Transform Processor resolves context accesses**\n\nIn https://github.com/open-telemetry/otel-arrow/issues/4008 we added\nthis trait called `ConfigNodeContextDeclaration` that DFE pipeline\ncomponents' configuration must implement to tell the engine about all\nthe context entry accesses it is going to do. E.g. which transport\nheaders it will read, etc.\n\nThe transform processor must now parse the program and look for these\nspecial metadata access functions, inspect their arguments to figure out\nwhich metadata accesses shall occur when the query evaluates.\n\nThis means that the trait method\n`ConfigNodeContextDeclaration::context_declarations` becomes fallible\nbecause the parsing could fail.\n\nTo visit the parsed AST, the query-engine module now implements a\n`parser::visit` crate, exposing a trait `ExprAstVisitor` that can be\nused to visit the expressions in the parser result. Currently this only\nsupports visiting `ScalarExpression` as that's all that's needed.\nThere's an implementation of this trait called\n`MetadataAccessDeclarationVisitor` that inspects the definition of the\nfunctions that are invoked to determine the context entry accesses, so\nthis is what is used by transform processor's `Config` in its\n`ConfigNodeContextDeclaration` impl.\n\n**`BatchEval` expression variant**\n\nWhen the query-engine has its own expression tree where the leaves are\nan enum called `LeafEval`. Previously there was a variant called\n`BatchPredicate` that operated on the batch itself (as opposed to the\ndata within) to evaluate a predicate such as `is Logs`.\n\nI've changed this `BatchPredicate` to a be called `BatchEval` which now\nreturns a datafusion `ScalarValue` instead of a `bool`. Our special\nmetadata functions are implemented as implementations of this trait.\n\n**Function definitions**\n\nThe query-engine's expression planner previously had a type called\n`expr::planner::DataFusionFunctionDef` representing well-known functions\nthat could be called in the queries it evaluates.\n\nI moved this type into a new module at `expr::planner::func` (to reduce\ncode in the expr planner, which was getting to be a long file) and now\nthe top-level type for function definitions is an enum in this same\nmodule called `FunctionDefinition` which has a datafusion variant but\nalso the `BatchEval` variant.\n\nThis `BatchEval` function definition variant is used both by the planner\n_and_ by the `MetadataAccessDeclarationVisitor` mentioned above.\nInternally, it contains some of the types defined in\n`expr::batch::metadata`, which is the module that contains the actual\nimplementation of the functions that inspect the batch metadata as well\nas some factories for creating them. One thing to note is that these\nfunction instances take `NodeBindings` (the pdata context type) as a\nconstructor arg.\n\n**Planner & Pipeline Changes**\n\nThe `NodeBindings` mentioned above now gets passed around by the\nplanner. We do this because we need it so we can get access to a\n`ValueRead` which can be used to execute the read operation on the batch\nmetadata to get the values (the function implementations may keep a\nclone of this to use at runtime).\n\nWhat this means however is that we needed another mechanism to visit the\nexpression AST (e.g. a mechanism other than the planner) for when the\n`NodeBindings` aren't available, which is why that `visit` module used\nby the transform processor `Config` type was implemented.\n\nAnyway, the way the `Metadata` gets passed around is though the\npipeline's `ExecutionState`. This is a mutable type that gets passed to\neach of query-engine's `PipelineStage` implementations. It's good that\nthis type is mutable because we'll eventually want to possibly modify\nthe `Metadata`. However, the `ExecutionState` doesn't get passed into\nexpression evaluation, there's a separate type for that called\n`EvalContext`, so this state has been added to the eval context as well\nso it's available in the function calls.\n\nThat basically sums up the changes. I also added some tests.\n\n**What's not yet supported:**\n- Calling these functions in partition processor partitioning expression\n- Some types of expressions (including `has_transport_header(\"foo\") as\nString`, see https://github.com/open-telemetry/otel-arrow/issues/4708)\n- Calling these functions in the OPL Playground app)\n\n> Chore PR? Open **Preview**, then [use the chore\ntemplate](?template=chore.md).\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## Related issue\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Relates to #https://github.com/open-telemetry/otel-arrow/issues/3310\n\n## Validation\n\nUnit tests & manual pipeline configuration\n\n<!--How did you confirm your change has the intended effect?-->\n\n## User-facing changes\n\n<!--\nDescribe the impact, or write `None`.\nUser-facing changes require a `.chloggen/*.yaml` entry. If no entry is\nneeded,\ninclude `chore` in the PR title. Documentation-only changes are exempt.\n-->\n\nYes, new function available in queries configured to execute by\ntransform processor",
          "timestamp": "2026-11-12T00:31:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5480e403d797571e31c24fa0a5676c0217764867"
        },
        "date": 1794448148846,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83246328037183,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20349061703607,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.80390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96643.74055947879,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96652.27285755995,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00728,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3190475.789202312,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032602.16511596,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.00983717066059,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.008028218522667885,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82210608774011,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21823908152975,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.383203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.9921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98334.32261759817,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.78970749718,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002976,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116981.8724478255,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140071.8844788573,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.700552639549873,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Ben Du",
            "username": "bendu",
            "email": "5668844+bendu@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "d51645841a7b4901f66093149da8c60789fdbed6",
          "message": "chore(sql-server): fix CI database-readiness race (#4711)\n\n# Chore summary\n\nAddress the recent SQL Server CI login failure after certificate\nrotation. Wait for a successful read from the fixture database before\nproceeding, preserving existing timeout and TLS behavior. Add\nrecovery-delay and timeout regressions.\n\n## Related issue\nRelated to #4696 and its [failed SQL Server CI\njob](https://github.com/open-telemetry/otel-arrow/actions/runs/42715906346/job/129497422789).\n\nCo-authored-by: Copilot App <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-11-12T16:50:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d51645841a7b4901f66093149da8c60789fdbed6"
        },
        "date": 1794505024892,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016112059354782104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87638931541869,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.31573151871328,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.902213541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.93391171438,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.93391171438,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002888,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115222.9255056316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1138339.3177395188,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.68261720557621,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89020720848086,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20738916256158,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.01171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96664.08251352746,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.70462582968,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001604,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192690.087775666,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026771.4564894033,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.46890276973571,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "e95c85add3ddbee5aa70e4f7189f7b670e8545f4",
          "message": "chore: Update unit tests and validation framework to point to vendored semconv registry (#4704)\n\n# Change summary\n\nPointing unit tests to the vendored semantic conventions registry rather\nthan pulling it from the internet all the time.\n\n## Related issue\n\n* Part of #4418 \n\n## Validation\n\ncargo test\n\n## User-facing changes\n\nNone.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-11-12T22:07:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e95c85add3ddbee5aa70e4f7189f7b670e8545f4"
        },
        "date": 1794534572687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92059017686512,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3928540573897,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.9515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.48046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.40461876281,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.40461876281,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003211,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3189980.3114096834,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033703.4785932745,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.44309366209269,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.007799285463988781,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.70405563180955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12905660377358,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.661067708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.97222293004,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98318.92680170499,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005537,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3123670.254558099,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140420.9205389281,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.77079282871027,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "cec34692557dece6b91c3f89cb8b9acb46162175",
          "message": "perf(pdata): Update OTLP proto field encoding order (#4730)\n\n# Change summary\n\nField order appears to matter when encoding logs data. Placing\ntimestamps next to each other works well and also encoding attribute\nvalues before keys also seems to move the needle.\n\nSee the attached issues for more info, but we get a very nice uplift\nfrom doing this.\n\n## Related issue\n\n* Closes #4720\n\n## Validation\n\nTested before/after:\n\n| Variant | In memory zstd-3 | In memory gzip-6 |\n|---|---|---|\n| Baseline (forward order) | 191,809 B | 228,959 B |\n| KeyValue + LogRecord in Go order | -20.5% | -10.4% |\n\n## User-facing changes\n\nBetter compression performance",
          "timestamp": "2026-11-13T16:52:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cec34692557dece6b91c3f89cb8b9acb46162175"
        },
        "date": 1794591789828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.015580722130835056,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72149077589354,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09575757575757,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.724739583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.17578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99981.3808850286,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.32636455348,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002852,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3111726.1331478423,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1138586.3640767217,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.647893213157708,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92147097279137,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34229957643434,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.197916666666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.71971613377,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.11429721625,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002205,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3199015.0839136587,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029654.518885584,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.0944756660482,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
          "id": "21b8264f12aa2d9b45007553ab03402d97aa4ff3",
          "message": "chore(deps): update weaver crates to 4f989fe (#4736)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| weaver_common | workspace.dependencies | digest | `039f9d2` →\n`4f989fe` |\n| weaver_forge | workspace.dependencies | digest | `039f9d2` → `4f989fe`\n|\n| weaver_resolved_schema | workspace.dependencies | digest | `039f9d2` →\n`4f989fe` |\n| weaver_resolver | workspace.dependencies | digest | `039f9d2` →\n`4f989fe` |\n| weaver_semconv | workspace.dependencies | digest | `039f9d2` →\n`4f989fe` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Friday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0NC4yMjguMCIsInVwZGF0ZWRJblZlciI6IjQ0LjIyOC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-14T00:08:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/21b8264f12aa2d9b45007553ab03402d97aa4ff3"
        },
        "date": 1794620784003,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.007792599964886904,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83915008835045,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10264823348695,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.24921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98334.84048873834,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.30753369934,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00266,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3097496.1157905767,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1138333.6847982085,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.50221129506946,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87472847084969,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.38256220058723,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.540364583333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96663.15940503291,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.15940503291,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002177,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2973630.4065034487,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029107.901773115,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.76280999717274,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}