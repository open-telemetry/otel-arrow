window.BENCHMARK_DATA = {
  "lastUpdate": 1777690088063,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "2abee0d4642c6bf2c2be774c5001320451913e74",
          "message": "Pipeline/group/engine policy precedence: prevent against misuse of unresolved policies (#2392)\n\n# Change Summary\n\nLike #2154 but for the other three policy fields. Make all fields Option\ntypes. Adds a ResolvedPolicies type which strips the Options after\nresolving. There was existing resolve code, but it was not used\nconsistently: this was observed for the `telemetry` policy.\n\n## What issue does this PR close?\n\nFixes #2389.\n\n## How are these changes tested?\n\nOne new test. The `configs/internal-telemetry.yaml` configuration is\nmodified to show the problem. Before the fix, no duration metrics. After\nthe fix, duration metrics, as set by the top-level policy.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-25T01:06:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2abee0d4642c6bf2c2be774c5001320451913e74"
        },
        "date": 1774403985702,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.893350643490734,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.17164086687306,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 325.48658854166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 508.4609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.79480721516,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.8702322105705,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002178,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33199.27155113208,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2483590.9295375566,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65101623535156,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.97025686557099,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.95580999535819,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.788020833333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.0234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.66859430552,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3572.116105181324,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002249,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66365.38857899942,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2481679.117410802,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "a44fe949ace10a2c36794537e4652ca89f2dce94",
          "message": "chore(deps): update dependency duckdb to v1.5.1 (#2418)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.5.0` → `==1.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.5.0/1.5.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-25T15:50:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a44fe949ace10a2c36794537e4652ca89f2dce94"
        },
        "date": 1774460516832,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6664810180664,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.562605917419692,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.736256948733788,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.213932291666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.53125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.77392009972,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3500.0591226203146,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002415,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99731.9707390546,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546834.6887896468,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65809631347656,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.21030803271751,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.193652019460963,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.204427083333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.96875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.77392009972,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3508.8587684345707,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002415,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 103637.95449467006,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2515103.5890726503,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.442325203605034,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.76289693593315,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.044401041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.36328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.46296795124,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.5574697852408,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007428,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 93815.88088011304,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 533891.0634136437,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.992900302323385,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.825140027850843,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.6421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.2734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.82816576087,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8601442236027,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002384,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 101156.59473338844,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2507824.456505492,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "a44fe949ace10a2c36794537e4652ca89f2dce94",
          "message": "chore(deps): update dependency duckdb to v1.5.1 (#2418)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.5.0` → `==1.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.5.0/1.5.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-25T15:50:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a44fe949ace10a2c36794537e4652ca89f2dce94"
        },
        "date": 1774460521490,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6816635131836,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.76988800431596,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.30250792914056,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.958723958333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.87109375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.56568727581,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3484.102854722769,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002534,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65930.67286571077,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2454433.3438847666,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.1109503043693,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 56.66130684463759,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 323.35377604166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 489.80078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.90515969878,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1630256420003,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00234,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33727.166756121085,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2502051.4920666376,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "238a1f78174fa44e3280a2662aa02277f2befa22",
          "message": "Add data sanitization step for transform processor results (#2434)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a \"sanitization\" step which can be performed on the result produced\nby the transform processor. This passes over all the the columns in all\nthe RecordBatch's and removes any values from dictionary columns that\nhave no keys pointing to them.\n\nThe procedure has some performance overhead, so there is an option to\nskip if if, for example the transformation isn't removing sensitive\ndata, or if something further along in the pipeline would remove the\nhidden arrow data (for example, serializing to OTLP in the OTLP\nexporter).\n\n**Why is this necessary?**\nSome of the arrow compute kernels will perform transformations ignoring\ncertain buffers for best performance if the result would still be a\nsemantically correct arrow array. For example when filtering, the arrow\ncompute kernels only filter dictionary key columns without touching the\ndictionaries.\n\nI'm imagining that someday someone will try to use transform processor\nto try to redact sensitive data, but the values from the rows they\ndeleted will still be present in the arrow buffers. If they then\ntransmitted the data using OTAP exporter (which does a simple arrow IPC\nserialization), the \"redacted\" data has escaped.\n\n**Why blindly do this for all transforms on all columns? Can't the\nquery-engine be smarter about this?**\nMaybe - but it's not as simple as it appears.\n\nFor example, consider when we're filtering. If we did something like:\n`logs | where event_name != \"sensitive_event_name\"`, it might be easy to\nthink that `event_name` is the sensitive column so it's the only one\nthat needs sanitizing. But _maybe_ the user actually knows a-priori that\nany log w/ this event name actually has sensitive data in some other\ncolumn.\n\nWhen it comes to the security of, I feel that it's better to be err on\nthe side of caution.\n\nIn the future we could maybe consider a better system where we let the\nuser provide hints about what fields they consider sensitive.\n\n**What's the performance impact?**\n\nThis sanitizing step adds significant overhead. When adding this\nsanitization step into the query engine's benchmarks, I saw anywhere\nbetween 5%-45% performance overhead on top of just executing the\ntransform pipeline w/ no sanitization. The actual overhead depends on\nthe complexity of the transform and size of output.\n\nThis poor performance is another reason why there's an option to skip\nthis step.\n\n**Should sanitize be on by default?**\n\nMy feeling is yes. If someone forgets to configure this or misconfigures\nit, I feel that it's best to fail on the side of worse\nperformance/better security.\n\n**Does this have to happen in the transform processor?**\n\nI feel that this is a reasonable place to do this, but open to\nsuggestions if anyone feels differently.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2313\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThere's a new user facing config field called `skip_sanitize_result`\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-26T01:03:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/238a1f78174fa44e3280a2662aa02277f2befa22"
        },
        "date": 1774490522573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.224905286036176,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.170101883297313,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.06015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.52734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.40775974204,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.5443363265904,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007768,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 103102.84918315253,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2517995.501386202,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.023727130502273,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.844966556469597,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.126692708333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.57421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.31805748779,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.809519641495,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003247,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 101119.45431621572,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2503093.75784454,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.785007377708688,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.560229155376636,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.223307291666668,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.27171293052,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.841489804909,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002702,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 95472.37354172725,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546256.0485613052,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.572228746047312,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.763470997679816,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.954427083333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.19140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.97515428576,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.865071838913,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0023,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 98380.73592505125,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540130.0586717031,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "238a1f78174fa44e3280a2662aa02277f2befa22",
          "message": "Add data sanitization step for transform processor results (#2434)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a \"sanitization\" step which can be performed on the result produced\nby the transform processor. This passes over all the the columns in all\nthe RecordBatch's and removes any values from dictionary columns that\nhave no keys pointing to them.\n\nThe procedure has some performance overhead, so there is an option to\nskip if if, for example the transformation isn't removing sensitive\ndata, or if something further along in the pipeline would remove the\nhidden arrow data (for example, serializing to OTLP in the OTLP\nexporter).\n\n**Why is this necessary?**\nSome of the arrow compute kernels will perform transformations ignoring\ncertain buffers for best performance if the result would still be a\nsemantically correct arrow array. For example when filtering, the arrow\ncompute kernels only filter dictionary key columns without touching the\ndictionaries.\n\nI'm imagining that someday someone will try to use transform processor\nto try to redact sensitive data, but the values from the rows they\ndeleted will still be present in the arrow buffers. If they then\ntransmitted the data using OTAP exporter (which does a simple arrow IPC\nserialization), the \"redacted\" data has escaped.\n\n**Why blindly do this for all transforms on all columns? Can't the\nquery-engine be smarter about this?**\nMaybe - but it's not as simple as it appears.\n\nFor example, consider when we're filtering. If we did something like:\n`logs | where event_name != \"sensitive_event_name\"`, it might be easy to\nthink that `event_name` is the sensitive column so it's the only one\nthat needs sanitizing. But _maybe_ the user actually knows a-priori that\nany log w/ this event name actually has sensitive data in some other\ncolumn.\n\nWhen it comes to the security of, I feel that it's better to be err on\nthe side of caution.\n\nIn the future we could maybe consider a better system where we let the\nuser provide hints about what fields they consider sensitive.\n\n**What's the performance impact?**\n\nThis sanitizing step adds significant overhead. When adding this\nsanitization step into the query engine's benchmarks, I saw anywhere\nbetween 5%-45% performance overhead on top of just executing the\ntransform pipeline w/ no sanitization. The actual overhead depends on\nthe complexity of the transform and size of output.\n\nThis poor performance is another reason why there's an option to skip\nthis step.\n\n**Should sanitize be on by default?**\n\nMy feeling is yes. If someone forgets to configure this or misconfigures\nit, I feel that it's best to fail on the side of worse\nperformance/better security.\n\n**Does this have to happen in the transform processor?**\n\nI feel that this is a reasonable place to do this, but open to\nsuggestions if anyone feels differently.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2313\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThere's a new user facing config field called `skip_sanitize_result`\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-26T01:03:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/238a1f78174fa44e3280a2662aa02277f2befa22"
        },
        "date": 1774490527734,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.67904663085938,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.31663282495606,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.12066269257567,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.320703125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.33984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.3994515807,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3486.8472179777323,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002629,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66551.26896208813,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2456339.922104338,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 51.82519177778613,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.54678623888674,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 327.2846354166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 500.14453125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.50353524758,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3477.951674250459,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004284,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33783.75512717771,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2521392.9382565627,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "ff9034401ece7eb6e46fee56e1abd225bf8c9078",
          "message": "chore(deps): update pipeline perf python dependencies (#2544)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [grpcio](https://redirect.github.com/grpc/grpc) | `==1.78.1` →\n`==1.80.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.80.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.78.1/1.80.0?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.0` → `==2.33.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.33.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.0/2.33.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.80.0`](https://redirect.github.com/grpc/grpc/releases/tag/v1.80.0)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.78.1...v1.80.0)\n\nThis is release 1.80.0\n([glimmering](https://redirect.github.com/grpc/grpc/blob/master/doc/g_stands_for.md))\nof gRPC Core.\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes, with\nhighlights listed below.\n\n## Core\n\n- \\[ssl] Implement TLS private key signer in Python.\n([#&#8203;41701](https://redirect.github.com/grpc/grpc/pull/41701))\n- \\[TLS Credentials]: Private Key Offload Implementation.\n([#&#8203;41606](https://redirect.github.com/grpc/grpc/pull/41606))\n- Fix max sockaddr struct size on OpenBSD.\n([#&#8203;40454](https://redirect.github.com/grpc/grpc/pull/40454))\n- \\[core] Enable EventEngine for Python by default, and EventEngine fork\nsupport in Python and Ruby.\n([#&#8203;41432](https://redirect.github.com/grpc/grpc/pull/41432))\n- \\[TLS Credentials]: Create InMemoryCertificateProvider to update\ncertificates independently.\n([#&#8203;41484](https://redirect.github.com/grpc/grpc/pull/41484))\n- \\[Ruby] Build/test ruby 4.0 and build native gems with Ruby 4.0\nsupport.\n([#&#8203;41324](https://redirect.github.com/grpc/grpc/pull/41324))\n- \\[EventEngine] Remove an incorrect std::move in DNSServiceResolver\nconstructor.\n([#&#8203;41502](https://redirect.github.com/grpc/grpc/pull/41502))\n- \\[RR and WRR] enable change to connect from a random index.\n([#&#8203;41472](https://redirect.github.com/grpc/grpc/pull/41472))\n- \\[xds] Implement gRFC A101.\n([#&#8203;41051](https://redirect.github.com/grpc/grpc/pull/41051))\n\n## C++\n\n- \\[C++] Add SNI override option to C++ channel credentials options API.\n([#&#8203;41460](https://redirect.github.com/grpc/grpc/pull/41460))\n\n## C\\#\n\n- \\[C# tools] Option to append Async to server side method names\n[#&#8203;39010](https://redirect.github.com/grpc/grpc/issues/39010).\n([#&#8203;39797](https://redirect.github.com/grpc/grpc/pull/39797))\n\n## Objective-C\n\n- \\[Fix]\\[Compiler] Plugins fall back to the edition 2023 for older\nprotobuf.\n([#&#8203;41357](https://redirect.github.com/grpc/grpc/pull/41357))\n\n## PHP\n\n- \\[PHP] Disable php infinite recursion check for callback from Core to\nPHP. ([#&#8203;41835](https://redirect.github.com/grpc/grpc/pull/41835))\n- \\[PHP] Fix runtime error with PHp8.5 alpha because\nzend\\_exception\\_get\\_defaul….\n([#&#8203;40337](https://redirect.github.com/grpc/grpc/pull/40337))\n\n## Python\n\n- \\[Python] Fix `GRPC_TRACE` not working when absl log initialized in\ncython.\n([#&#8203;41814](https://redirect.github.com/grpc/grpc/pull/41814))\n- Revert \"\\[Python] Align GRPC\\_ENABLE\\_FORK\\_SUPPORT env defaults in\ncore and python\n([#&#8203;41455](https://redirect.github.com/grpc/grpc/issues/41455))\".\n([#&#8203;41769](https://redirect.github.com/grpc/grpc/pull/41769))\n- \\[Python] Fix AsyncIO Server maximum\\_concurrent\\_rpcs enforcement\npreventing negative active\\_rpcs count.\n([#&#8203;41532](https://redirect.github.com/grpc/grpc/pull/41532))\n- \\[Python] Docs: correct `grpc.Compression` references.\n([#&#8203;41705](https://redirect.github.com/grpc/grpc/pull/41705))\n- \\[Python] \\[Typeguard] Part 4 - Add Typeguard to AIO stack in tests .\n([#&#8203;40226](https://redirect.github.com/grpc/grpc/pull/40226))\n- \\[Python] Fix multi-thread exception for Asyncio gRPC clients - Fixes\n[#&#8203;25364](https://redirect.github.com/grpc/grpc/issues/25364).\n([#&#8203;41483](https://redirect.github.com/grpc/grpc/pull/41483))\n- \\[Python] Resolve absl::InitializeLog warning.\n([#&#8203;39779](https://redirect.github.com/grpc/grpc/pull/39779))\n- \\[Python] Remove IF usage in Cython.\n([#&#8203;41400](https://redirect.github.com/grpc/grpc/pull/41400))\n- \\[Python] Add language features to exported proto files.\n([#&#8203;41501](https://redirect.github.com/grpc/grpc/pull/41501))\n- \\[Python] Fix crash when iterating on AIO Metadata keys(), values(),\nitems() or list(metadata.values()) etc. .\n([#&#8203;41481](https://redirect.github.com/grpc/grpc/pull/41481))\n- \\[Python] Modernize and revamp Public API Docs.\n([#&#8203;41287](https://redirect.github.com/grpc/grpc/pull/41287))\n\n## Ruby\n\n- \\[Ruby] Added support to push native-debug packages off rubygems to\npublic gcs bucket.\n([#&#8203;41270](https://redirect.github.com/grpc/grpc/pull/41270))\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.33.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2331-2026-03-30)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.0...v2.33.1)\n\n**Bugfixes**\n\n- Fixed test cleanup for CVE-2026-25645 to avoid leaving unnecessary\nfiles in the tmp directory.\n([#&#8203;7305](https://redirect.github.com/psf/requests/issues/7305))\n- Fixed Content-Type header parsing for malformed values.\n([#&#8203;7309](https://redirect.github.com/psf/requests/issues/7309))\n- Improved error consistency for malformed header values.\n([#&#8203;7308](https://redirect.github.com/psf/requests/issues/7308))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ1cGRhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-06T15:32:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff9034401ece7eb6e46fee56e1abd225bf8c9078"
        },
        "date": 1775494200380,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.55714416503906,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.192585522526315,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.083738491295936,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.390364583333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.94921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.7406727867,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6764.725577632401,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002434,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113957.24111015302,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2497662.0385082862,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "ff9034401ece7eb6e46fee56e1abd225bf8c9078",
          "message": "chore(deps): update pipeline perf python dependencies (#2544)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [grpcio](https://redirect.github.com/grpc/grpc) | `==1.78.1` →\n`==1.80.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.80.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.78.1/1.80.0?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.0` → `==2.33.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.33.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.0/2.33.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.80.0`](https://redirect.github.com/grpc/grpc/releases/tag/v1.80.0)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.78.1...v1.80.0)\n\nThis is release 1.80.0\n([glimmering](https://redirect.github.com/grpc/grpc/blob/master/doc/g_stands_for.md))\nof gRPC Core.\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes, with\nhighlights listed below.\n\n## Core\n\n- \\[ssl] Implement TLS private key signer in Python.\n([#&#8203;41701](https://redirect.github.com/grpc/grpc/pull/41701))\n- \\[TLS Credentials]: Private Key Offload Implementation.\n([#&#8203;41606](https://redirect.github.com/grpc/grpc/pull/41606))\n- Fix max sockaddr struct size on OpenBSD.\n([#&#8203;40454](https://redirect.github.com/grpc/grpc/pull/40454))\n- \\[core] Enable EventEngine for Python by default, and EventEngine fork\nsupport in Python and Ruby.\n([#&#8203;41432](https://redirect.github.com/grpc/grpc/pull/41432))\n- \\[TLS Credentials]: Create InMemoryCertificateProvider to update\ncertificates independently.\n([#&#8203;41484](https://redirect.github.com/grpc/grpc/pull/41484))\n- \\[Ruby] Build/test ruby 4.0 and build native gems with Ruby 4.0\nsupport.\n([#&#8203;41324](https://redirect.github.com/grpc/grpc/pull/41324))\n- \\[EventEngine] Remove an incorrect std::move in DNSServiceResolver\nconstructor.\n([#&#8203;41502](https://redirect.github.com/grpc/grpc/pull/41502))\n- \\[RR and WRR] enable change to connect from a random index.\n([#&#8203;41472](https://redirect.github.com/grpc/grpc/pull/41472))\n- \\[xds] Implement gRFC A101.\n([#&#8203;41051](https://redirect.github.com/grpc/grpc/pull/41051))\n\n## C++\n\n- \\[C++] Add SNI override option to C++ channel credentials options API.\n([#&#8203;41460](https://redirect.github.com/grpc/grpc/pull/41460))\n\n## C\\#\n\n- \\[C# tools] Option to append Async to server side method names\n[#&#8203;39010](https://redirect.github.com/grpc/grpc/issues/39010).\n([#&#8203;39797](https://redirect.github.com/grpc/grpc/pull/39797))\n\n## Objective-C\n\n- \\[Fix]\\[Compiler] Plugins fall back to the edition 2023 for older\nprotobuf.\n([#&#8203;41357](https://redirect.github.com/grpc/grpc/pull/41357))\n\n## PHP\n\n- \\[PHP] Disable php infinite recursion check for callback from Core to\nPHP. ([#&#8203;41835](https://redirect.github.com/grpc/grpc/pull/41835))\n- \\[PHP] Fix runtime error with PHp8.5 alpha because\nzend\\_exception\\_get\\_defaul….\n([#&#8203;40337](https://redirect.github.com/grpc/grpc/pull/40337))\n\n## Python\n\n- \\[Python] Fix `GRPC_TRACE` not working when absl log initialized in\ncython.\n([#&#8203;41814](https://redirect.github.com/grpc/grpc/pull/41814))\n- Revert \"\\[Python] Align GRPC\\_ENABLE\\_FORK\\_SUPPORT env defaults in\ncore and python\n([#&#8203;41455](https://redirect.github.com/grpc/grpc/issues/41455))\".\n([#&#8203;41769](https://redirect.github.com/grpc/grpc/pull/41769))\n- \\[Python] Fix AsyncIO Server maximum\\_concurrent\\_rpcs enforcement\npreventing negative active\\_rpcs count.\n([#&#8203;41532](https://redirect.github.com/grpc/grpc/pull/41532))\n- \\[Python] Docs: correct `grpc.Compression` references.\n([#&#8203;41705](https://redirect.github.com/grpc/grpc/pull/41705))\n- \\[Python] \\[Typeguard] Part 4 - Add Typeguard to AIO stack in tests .\n([#&#8203;40226](https://redirect.github.com/grpc/grpc/pull/40226))\n- \\[Python] Fix multi-thread exception for Asyncio gRPC clients - Fixes\n[#&#8203;25364](https://redirect.github.com/grpc/grpc/issues/25364).\n([#&#8203;41483](https://redirect.github.com/grpc/grpc/pull/41483))\n- \\[Python] Resolve absl::InitializeLog warning.\n([#&#8203;39779](https://redirect.github.com/grpc/grpc/pull/39779))\n- \\[Python] Remove IF usage in Cython.\n([#&#8203;41400](https://redirect.github.com/grpc/grpc/pull/41400))\n- \\[Python] Add language features to exported proto files.\n([#&#8203;41501](https://redirect.github.com/grpc/grpc/pull/41501))\n- \\[Python] Fix crash when iterating on AIO Metadata keys(), values(),\nitems() or list(metadata.values()) etc. .\n([#&#8203;41481](https://redirect.github.com/grpc/grpc/pull/41481))\n- \\[Python] Modernize and revamp Public API Docs.\n([#&#8203;41287](https://redirect.github.com/grpc/grpc/pull/41287))\n\n## Ruby\n\n- \\[Ruby] Added support to push native-debug packages off rubygems to\npublic gcs bucket.\n([#&#8203;41270](https://redirect.github.com/grpc/grpc/pull/41270))\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.33.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2331-2026-03-30)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.0...v2.33.1)\n\n**Bugfixes**\n\n- Fixed test cleanup for CVE-2026-25645 to avoid leaving unnecessary\nfiles in the tmp directory.\n([#&#8203;7305](https://redirect.github.com/psf/requests/issues/7305))\n- Fixed Content-Type header parsing for malformed values.\n([#&#8203;7309](https://redirect.github.com/psf/requests/issues/7309))\n- Improved error consistency for malformed header values.\n([#&#8203;7308](https://redirect.github.com/psf/requests/issues/7308))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ1cGRhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-06T15:32:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff9034401ece7eb6e46fee56e1abd225bf8c9078"
        },
        "date": 1775494204800,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.50947570800781,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.921672094986754,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.95748195296127,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.994140625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.0390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.9711168844,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6814.204357239267,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007446,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65721.38670242712,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2502920.4928802373,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "962ac30ac5ba928f035dbc513a34ed75787d83c4",
          "message": "[otap-dataflow] add transport header infrastructure (#2539)\n\n# Change Summary\n\nUpdate the OtapPdata Context with the transport_headers field, so that\nOtapPdata can carry the headers through the pipeline.\n\nDefined the TransportHeadersPolicy with HeaderCapturePolicy and\nHeaderPropagationPolicy, update the Policies to have a transport_headers\nfield. Receiver and Exporter nodes can also define HeaderCapturePolicy\nand HeaderPropagationPolicy respectively, these definitions will\noverride any top level HeaderCapturePolicy and HeaderPropagationPolicy\nrules.\n\nExposed the policy to the Receiver and Exporter nodes via the\nEffectHandler with helper functions that apply the policies on a\niterator of key value pairs (for receiver nodes) and transport_headers\n(for exporter nodes)\n\n## What issue does this PR close?\n\n* Closes #2508\n\n## How are these changes tested?\n\nunit tests and integration tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T00:18:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/962ac30ac5ba928f035dbc513a34ed75787d83c4"
        },
        "date": 1775524972506,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.50737762451172,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.27225859682353,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.64521072796935,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.42890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.2734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.75467165277,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6816.974367003094,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002426,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114393.88097623017,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2495812.2464023028,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "962ac30ac5ba928f035dbc513a34ed75787d83c4",
          "message": "[otap-dataflow] add transport header infrastructure (#2539)\n\n# Change Summary\n\nUpdate the OtapPdata Context with the transport_headers field, so that\nOtapPdata can carry the headers through the pipeline.\n\nDefined the TransportHeadersPolicy with HeaderCapturePolicy and\nHeaderPropagationPolicy, update the Policies to have a transport_headers\nfield. Receiver and Exporter nodes can also define HeaderCapturePolicy\nand HeaderPropagationPolicy respectively, these definitions will\noverride any top level HeaderCapturePolicy and HeaderPropagationPolicy\nrules.\n\nExposed the policy to the Receiver and Exporter nodes via the\nEffectHandler with helper functions that apply the policies on a\niterator of key value pairs (for receiver nodes) and transport_headers\n(for exporter nodes)\n\n## What issue does this PR close?\n\n* Closes #2508\n\n## How are these changes tested?\n\nunit tests and integration tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T00:18:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/962ac30ac5ba928f035dbc513a34ed75787d83c4"
        },
        "date": 1775524976063,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.52519226074219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.92096667262692,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.05217821782178,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.842057291666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.75,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.84391451172,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6798.2809013809865,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002375,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65696.60377458927,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2507293.5026939,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "2bc828b31d6c8aadb0c03b256d8c2c762f63af18",
          "message": "fix(query_engine): remove cross-workspace path dep that broke external git consumers (#2567)\n\n## Summary\n\nFixes #2560\n\n- Remove the `otap-df-pdata` dev-dependency from\n`engine-recordset-otlp-bridge` that created a circular cross-workspace\nreference between `query_engine` and `otap-dataflow`\n- Inline the benchmark fixture using `opentelemetry-proto` types already\navailable as dev-dependencies\n\n## Root Cause\n\nPR #2453 added `data_engine_recordset_otlp_bridge` as a workspace\ndependency in `otap-dataflow`, which meant `otap-dataflow` now\nreferenced crates in the `query_engine` workspace. Meanwhile,\n`engine-recordset-otlp-bridge` (a `query_engine` member) had a\ndev-dependency pointing back into `otap-dataflow`:\n\n```toml\n# rust/experimental/query_engine/engine-recordset-otlp-bridge/Cargo.toml\n[dev-dependencies]\notap-df-pdata = { path = \"../../../otap-dataflow/crates/pdata\", features = [\"testing\"] }\n```\n\nThis bidirectional cross-workspace reference causes cargo's package\ndiscovery to fail when resolving any `otap-dataflow` crate as an\nexternal git dependency:\n\n```\nerror: no matching package named `otap-df-pdata` found\nlocation searched: Git repository .../otel-arrow.git?rev=...\n```\n\n## Fix\n\n**`engine-recordset-otlp-bridge/Cargo.toml`** — Removed the\n`otap-df-pdata` dev-dependency.\n\n**`engine-recordset-otlp-bridge/benches/extend.rs`** — Replaced the\nimported `logs_with_varying_attributes_and_properties` fixture with a\nself-contained `generate_logs_batch` function that builds equivalent\n`ExportLogsServiceRequest` protobuf bytes using `opentelemetry-proto`\ntypes (already a dev-dependency). The benchmark logic is otherwise\nunchanged.\n\n## Validation\n\n- `cargo check --workspace` in `query_engine` — passes\n- `cargo bench -p data_engine_recordset_otlp_bridge --no-run` —\nbenchmark compiles\n- `cargo xtask check` in `otap-dataflow` — structure checks, fmt, clippy\n(0 warnings), all tests pass\n- Tested as external git dep consumer from a downstream project — cargo\nresolves all `otap-df-*` packages successfully",
          "timestamp": "2026-04-07T16:29:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2bc828b31d6c8aadb0c03b256d8c2c762f63af18"
        },
        "date": 1775585052109,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.60625457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.679029865319453,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.468820798514393,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.345703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.23046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.72725660667,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6819.748123969289,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002216,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 112465.5701155112,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2494424.097658014,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "2bc828b31d6c8aadb0c03b256d8c2c762f63af18",
          "message": "fix(query_engine): remove cross-workspace path dep that broke external git consumers (#2567)\n\n## Summary\n\nFixes #2560\n\n- Remove the `otap-df-pdata` dev-dependency from\n`engine-recordset-otlp-bridge` that created a circular cross-workspace\nreference between `query_engine` and `otap-dataflow`\n- Inline the benchmark fixture using `opentelemetry-proto` types already\navailable as dev-dependencies\n\n## Root Cause\n\nPR #2453 added `data_engine_recordset_otlp_bridge` as a workspace\ndependency in `otap-dataflow`, which meant `otap-dataflow` now\nreferenced crates in the `query_engine` workspace. Meanwhile,\n`engine-recordset-otlp-bridge` (a `query_engine` member) had a\ndev-dependency pointing back into `otap-dataflow`:\n\n```toml\n# rust/experimental/query_engine/engine-recordset-otlp-bridge/Cargo.toml\n[dev-dependencies]\notap-df-pdata = { path = \"../../../otap-dataflow/crates/pdata\", features = [\"testing\"] }\n```\n\nThis bidirectional cross-workspace reference causes cargo's package\ndiscovery to fail when resolving any `otap-dataflow` crate as an\nexternal git dependency:\n\n```\nerror: no matching package named `otap-df-pdata` found\nlocation searched: Git repository .../otel-arrow.git?rev=...\n```\n\n## Fix\n\n**`engine-recordset-otlp-bridge/Cargo.toml`** — Removed the\n`otap-df-pdata` dev-dependency.\n\n**`engine-recordset-otlp-bridge/benches/extend.rs`** — Replaced the\nimported `logs_with_varying_attributes_and_properties` fixture with a\nself-contained `generate_logs_batch` function that builds equivalent\n`ExportLogsServiceRequest` protobuf bytes using `opentelemetry-proto`\ntypes (already a dev-dependency). The benchmark logic is otherwise\nunchanged.\n\n## Validation\n\n- `cargo check --workspace` in `query_engine` — passes\n- `cargo bench -p data_engine_recordset_otlp_bridge --no-run` —\nbenchmark compiles\n- `cargo xtask check` in `otap-dataflow` — structure checks, fmt, clippy\n(0 warnings), all tests pass\n- Tested as external git dep consumer from a downstream project — cargo\nresolves all `otap-df-*` packages successfully",
          "timestamp": "2026-04-07T16:29:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2bc828b31d6c8aadb0c03b256d8c2c762f63af18"
        },
        "date": 1775585056400,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.517333984375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.77036830954329,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.023263494041586,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.709505208333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.0783964719,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6806.545775515285,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002241,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66427.35745008734,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2539055.833643343,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "c225a4bfb556577abf133dc52bcf01eb1e901e70",
          "message": "[otap-dataflow] update transport headers struct to wrap headers in arc (#2584)\n\n# Change Summary\n\nUpdated the TransportHeaders struct to have a Arc<Vec<TransportHeader>>\ninstead of Vec<TransportHeader>\n\n## What issue does this PR close?\n\n* Closes #2583\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T23:42:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c225a4bfb556577abf133dc52bcf01eb1e901e70"
        },
        "date": 1775612561776,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.5047607421875,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.32936082170857,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.20864856501895,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.230729166666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.73046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.65794555275,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6819.133401796856,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007625,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114477.20249505606,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2460165.5642946726,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "c225a4bfb556577abf133dc52bcf01eb1e901e70",
          "message": "[otap-dataflow] update transport headers struct to wrap headers in arc (#2584)\n\n# Change Summary\n\nUpdated the TransportHeaders struct to have a Arc<Vec<TransportHeader>>\ninstead of Vec<TransportHeader>\n\n## What issue does this PR close?\n\n* Closes #2583\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T23:42:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c225a4bfb556577abf133dc52bcf01eb1e901e70"
        },
        "date": 1775612565431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.517333984375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.91398891517035,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.61443110525908,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.350130208333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.55078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.61643301393,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6806.515827964183,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002505,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66187.7197927476,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2458421.85646186,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "68e472cef0583ced74c6db564ada17c5f8ba7005",
          "message": "fix(otlp-receiver): make request_bytes metric consistent across HTTP and gRPC (#2596)\n\nThe `request_bytes` counter was recording compressed wire size on the\nHTTP path but decompressed size on the gRPC path (where tonic\ndecompresses before handing bytes to the decoder). Move the HTTP\nrecording to after decompression so both protocols report decompressed\npayload bytes for successfully received requests.\n\n**Changes:**\n\n- **`otlp_http.rs`**: Move `request_bytes.add()` from before\n`decode_content_encoding()` to after it, so it records decompressed\npayload size — matching the gRPC path.\n- **`otlp_metrics.rs`**: Update doc comment to clarify the metric\nmeasures decompressed bytes for successfully received requests.",
          "timestamp": "2026-04-08T17:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/68e472cef0583ced74c6db564ada17c5f8ba7005"
        },
        "date": 1775673347315,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.51000213623047,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.966380857752313,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.004662235275934,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.3171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.0703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.80315884293,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6813.643525008906,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007542,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114682.61123621493,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2531950.6577971145,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "68e472cef0583ced74c6db564ada17c5f8ba7005",
          "message": "fix(otlp-receiver): make request_bytes metric consistent across HTTP and gRPC (#2596)\n\nThe `request_bytes` counter was recording compressed wire size on the\nHTTP path but decompressed size on the gRPC path (where tonic\ndecompresses before handing bytes to the decoder). Move the HTTP\nrecording to after decompression so both protocols report decompressed\npayload bytes for successfully received requests.\n\n**Changes:**\n\n- **`otlp_http.rs`**: Move `request_bytes.add()` from before\n`decode_content_encoding()` to after it, so it records decompressed\npayload size — matching the gRPC path.\n- **`otlp_metrics.rs`**: Update doc comment to clarify the metric\nmeasures decompressed bytes for successfully received requests.",
          "timestamp": "2026-04-08T17:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/68e472cef0583ced74c6db564ada17c5f8ba7005"
        },
        "date": 1775673352209,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.52257537841797,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 56.617121570757014,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.36473191489362,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.376692708333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.53764775778,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6800.8814057494765,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003693,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66245.38488796275,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2575484.4503298383,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f",
          "message": "fix: handle DrainIngress in fake_data_generator to unblock graceful shutdown (#2515)\n\n# Change Summary\n\nThe \"Ack nack redesign\" PR (3dca2837) introduced a two-phase\nDrainIngress/ReceiverDrained shutdown protocol but missed updating the\nfake_data_generator receiver. Without the DrainIngress handler, the\nmessage falls into the _ => {} catch-all, notify_receiver_drained() is\nnever called, the pipeline controller never removes the receiver from\nits pending set, and after the deadline expires it emits\nDrainDeadlineReached. This was causing pipeline-perf-test-basic to fail\nconsistently.\n\n## What issue does this PR close?\n\npipeline-perf-test-basic unit test is failing.\n\n* Closes #2511\n\n## How are these changes tested?\n\nfake_data_generator and runtime_control_metrics tests were executed.\n\n## Are there any user-facing changes?\n\nNo, fake_data_generator is an internal test/load-generation receiver,\nnot a user-facing component.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <josh.macdonald@gmail.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-09T00:04:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f"
        },
        "date": 1775699583871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.60625457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.852748287749208,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.20632098765432,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.313671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.12109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.28640210508,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6819.719936834594,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002464,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114644.8198523107,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2496946.1549107786,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f",
          "message": "fix: handle DrainIngress in fake_data_generator to unblock graceful shutdown (#2515)\n\n# Change Summary\n\nThe \"Ack nack redesign\" PR (3dca2837) introduced a two-phase\nDrainIngress/ReceiverDrained shutdown protocol but missed updating the\nfake_data_generator receiver. Without the DrainIngress handler, the\nmessage falls into the _ => {} catch-all, notify_receiver_drained() is\nnever called, the pipeline controller never removes the receiver from\nits pending set, and after the deadline expires it emits\nDrainDeadlineReached. This was causing pipeline-perf-test-basic to fail\nconsistently.\n\n## What issue does this PR close?\n\npipeline-perf-test-basic unit test is failing.\n\n* Closes #2511\n\n## How are these changes tested?\n\nfake_data_generator and runtime_control_metrics tests were executed.\n\n## Are there any user-facing changes?\n\nNo, fake_data_generator is an internal test/load-generation receiver,\nnot a user-facing component.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <josh.macdonald@gmail.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-09T00:04:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f"
        },
        "date": 1775699589442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 93.58355712890625,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.895040963267164,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.054314877822456,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.125651041666668,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.8828125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.87399312858,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 6843.89248109785,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002696,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66905.72829284907,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2529868.491501637,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775764201778,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65338134765625,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.801216465031956,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.215374107420056,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.128515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.17463937064,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3513.8219797525376,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002186,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 111158.22411401513,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543178.7538094448,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.048724962229002,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.16081469525786,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.983723958333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.13671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.00960734459,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.5645125509805,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007424,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106181.43181924615,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546049.4119540951,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.95418448462892,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.99656052914936,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.230859375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.02676691233,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.699183043156,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005128,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 112948.03475109162,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2499762.386878317,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.071864601585624,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.288164897708047,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.37890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.01171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.60593389167,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.852694164749,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002511,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116152.99035642341,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2542481.3374767,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775764207886,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69214630126953,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 56.004098608558074,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.598814814814816,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.592317708333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.36328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.87541202756,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3473.11356452214,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002357,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66471.5222526699,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2556717.6716721808,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.907682719685724,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 62.68381203883495,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1665.05859375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2741.1796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.99090308069,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.165893798955,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002291,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25324.632116965473,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541389.0761829873,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.70366668701172,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.595465330819515,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.8736975700356,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 321.5864583333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 476.74609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.28921135738,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3460.9947167037103,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002692,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33424.60806424451,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2475109.720117812,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775790497974,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.011842501977373,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.985929927457942,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.372526041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.5,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.03464975806,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.867066353794,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002266,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 105732.9560043919,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545292.3355334247,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.59354400634766,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.901941657466015,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.455619309922728,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.52109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103328.35979494854,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.830578821473,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002888,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110991.9135173327,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540975.9742630598,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.01515882840407,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.153563041976824,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.16015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.23105579703,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.5384125371957,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007869,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113427.06097436581,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2474861.1146314708,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.087485167094172,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.87771251931994,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.117447916666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.86328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.77916967738,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.858501688232,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002412,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115485.3290729448,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2507478.447895084,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775790501969,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.19272994475184,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.08122928499183,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1726.7248697916666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2836.15625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.93140765796,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1639036487336,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002325,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25606.486291492234,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539108.4684566926,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.52564037158079,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.80841584158416,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 323.97942708333335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 517.02734375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.4729057677,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3477.9837853413446,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00373,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33482.0362101837,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2499845.579594688,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65547943115234,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.9949139658246,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.99704885035225,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.949869791666668,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.1171875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.49044368544,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.599176815356,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002577,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66066.1071046815,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2486533.7406948037,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775842831573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.884261825566247,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.853069796170475,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.625520833333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.11328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.99965241324,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.865893109473,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002286,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 111141.48569454598,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545828.358395229,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67118835449219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.048519869238984,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.875939965960082,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.500390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.71087671848,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3495.0406470652406,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003594,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113539.82523689808,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2491046.9005386056,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.786954506096365,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.893249211356466,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.83828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.31640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.58504507894,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.863201901474,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002296,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114512.75394757609,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2486586.532360347,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.997839945691942,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.923479333744602,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.56171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.91015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.13264244767,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8703514420554,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00221,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106873.78552161163,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542686.6426131583,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775842837008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.62248229980469,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.82878250797758,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.68743768385198,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1690.2248697916666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3272.55859375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.60593389167,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3546.251589370985,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002511,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25625.451525555563,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 532636.7763319114,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.09056719452983,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 68.13353532230639,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 341.83828125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 530.84765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.0512098941,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.06755966201,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003971,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33720.27290532031,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2446605.6167396805,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68952178955078,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 56.12591358756001,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.11767087030983,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.022395833333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.87109375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104997.01108508445,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3475.9010526833667,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001708,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66377.78356352645,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2473408.0405971524,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775873937241,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.86521463159934,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.403937408463731,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.387239583333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.94140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.0731468643,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8683569234513,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002244,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110862.55970075476,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 551147.0584248693,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.04768312692405,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.86966744006187,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.359505208333335,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.9296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.05914791331,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.8699518811395,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002252,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 105998.01617960999,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543655.7981018184,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.494064525322724,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.57262023855329,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.188541666666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.76171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.16589000891,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.871466026965,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002191,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116846.9609103938,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2510283.02550861,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.46850484755456,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.638588089904996,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.650911458333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.4453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.8298625984,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.793153489013,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003526,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113197.77550487882,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2520297.0978257125,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775873940979,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.67904663085938,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.14742787131988,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.922081645372394,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.691666666666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.30859375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.02590041967,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3486.8680220453657,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002271,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66474.5773457633,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2481116.1059007165,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67118835449219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 49.12340051344037,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.38838569880823,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 315.2739583333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 467.97265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.20730878942,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3494.8241556766307,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007311,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32969.76727891471,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2506371.648601256,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.69120661654578,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 69.31152959381045,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1690.523046875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2991.6015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.95240603475,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1646060544367,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002313,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25864.33638434014,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 532760.8826857415,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775930350907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.898928845804045,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.572595849726142,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.501302083333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.71875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.41695005013,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8463587064425,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002619,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110212.54583741367,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545383.6047127647,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.042197584927422,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.459140649149923,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.060807291666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.3125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104989.66026829123,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.658788853611,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005909,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 105840.23277785638,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543818.3716230806,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.315599439500804,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.368694981885454,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.652864583333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.76514588705,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7239363192616,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004706,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 112803.72800737985,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2572045.9135159873,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.321558647763467,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.471176696641304,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.775130208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.84765625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.55168846014,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.850875651235,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002542,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115972.9935761938,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2528467.533710416,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775930354726,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.67904663085938,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.132331866595116,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.06578570879163,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.660286458333335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.2734375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.02415055217,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3486.8679639330994,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002272,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66474.44956129983,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2506028.57731736,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.07195463262191,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18900800985828,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 317.40703125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 473.61328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.78524577386,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3477.9610061128633,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004123,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33889.51682063433,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2485208.8327248683,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.30916242922456,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.62664190078167,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1694.8166666666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3255.95703125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.8876610666,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3478.0637758354464,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00235,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25688.892528921595,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542693.995662532,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775959750723,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.96841056719448,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.414695953757224,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.372005208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.51171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.29303796598,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.580670252877,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007262,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110055.81453913303,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540496.3334870294,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.058115213157965,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.037762863534677,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.563151041666668,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.67242836807,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8549233129106,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002473,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114970.21653496275,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2492907.3390399385,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.26187450252454,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.580849230769232,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.721875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.45365816442,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.780541683226,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003741,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113658.76931532008,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2504714.1804871312,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.10685802379819,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.519907954283962,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.16953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.07664660264,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8684742480123,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002242,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106907.61523860529,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544064.8036586846,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775959755352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.66071319580078,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.3011374089756,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.71440074326417,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.635807291666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.14453125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.29446088658,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3506.0928686046054,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002689,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66093.78287761868,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2461972.5635717073,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.00409991364968,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.075799767171134,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1707.0072916666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3063.5625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.10289464757,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1696399701964,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002227,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25406.56697082888,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538676.8681399229,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.19708682664178,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.89399361519894,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 350.43346354166664,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 526.31640625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.01973623052,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8330425860136,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002846,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34419.89909859521,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2598137.864744817,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776014936080,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.921299326195307,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.03150927422458,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.810026041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.24196561826,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8404925616787,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002719,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 111066.72627468657,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547992.3113807245,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.089324946436193,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.41250191982798,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.468098958333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.90625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.98215374959,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.865306487605,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002296,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 107740.23599943964,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540961.3647581547,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.101812091331638,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.941791737583166,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.061979166666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.04510271606,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7333215386716,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004546,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113572.93083653101,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2552596.259073859,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.085193418153032,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.042290721807976,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.2265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.74767221928,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.857445773446,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00243,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115101.5738541608,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2506600.731648299,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776014939867,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 46.95891832388001,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.32860138230954,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 332.607421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 521.77734375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104988.53875118634,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3477.820337946441,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00655,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33847.00809494886,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2520980.5544095477,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.54029653432032,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.806306695464365,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1627.840234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3047.42578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.42479604333,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8130979245,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003186,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25968.068325381977,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541241.7373738514,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65809631347656,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.204333674864905,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.1352870184125,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.5984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.5078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.04689883426,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3508.867891123899,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002259,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66184.02929733897,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2489688.0555425235,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776046219053,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.198725292394812,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.45357131866441,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.467838541666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.90625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.00140227993,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.86595177167,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002285,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115722.51075310848,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2535574.3994536647,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.203528355875655,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.400046143197724,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.136458333333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.0078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.95415589985,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.8664871446954,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002312,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 107110.23399620743,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544917.327408968,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.19958026993979,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.567347443225708,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.885807291666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.0703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.88416134087,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8620214087605,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002352,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 111151.51954160338,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542182.432477527,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.998340527620396,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.74730620454897,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.437239583333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.40037772547,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.678184091368,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005486,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113907.54397391061,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2487300.4170338935,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776046222947,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.83088590925214,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.26724468656487,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1690.2307291666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2964.5,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.03814949383,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.167474213973,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002264,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25431.788558437565,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539997.2507450238,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68428039550781,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.13401222611487,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.551451203568405,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.757421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.5,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.81241701478,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3481.361151712733,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002393,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66416.5259217586,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2496237.1167375413,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.5741177418395,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 56.78796657381615,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 330.874609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 485.24609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.93665725137,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3478.0653988690638,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002322,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33934.90292475001,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2467652.1266550464,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "c2e39e05477b8394ce45d84f73bd89f244a9419b",
          "message": "[query-engine] Slice expression bug fix (#2636)\n\n## Changes\n\n* Switch `Slice` expression (arrays and strings) to use `Slice(source,\nstart, [length])` instead of `Slice(source, start_inclusive,\n[end_exclusive])` to match KQL\n[substring](https://learn.microsoft.com/kusto/query/substring-function?view=azure-data-explorer)\nbehavior.\n\n/cc @albertlockett @drewrelmas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-13T13:06:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c2e39e05477b8394ce45d84f73bd89f244a9419b"
        },
        "date": 1776109018486,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.89289908282701,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.672514692236314,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.191145833333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.57421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.96815482285,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.864837190252,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002304,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110541.20017937355,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546989.4402753302,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.067724565422832,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.56988929889299,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.124609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.4921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.94799308573,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.830637482493,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002887,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106390.41708991505,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546178.7856820236,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.18171561394495,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.9559655596556,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.987369791666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.73828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104982.41544541289,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.4104987414607,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.01005,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113327.9553668872,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2516265.68881254,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.142130412967024,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.627885537509666,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.458333333333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.2734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.08189621057,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8686502348687,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002239,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115067.70079998922,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2485071.0756502203,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "c2e39e05477b8394ce45d84f73bd89f244a9419b",
          "message": "[query-engine] Slice expression bug fix (#2636)\n\n## Changes\n\n* Switch `Slice` expression (arrays and strings) to use `Slice(source,\nstart, [length])` instead of `Slice(source, start_inclusive,\n[end_exclusive])` to match KQL\n[substring](https://learn.microsoft.com/kusto/query/substring-function?view=azure-data-explorer)\nbehavior.\n\n/cc @albertlockett @drewrelmas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-13T13:06:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c2e39e05477b8394ce45d84f73bd89f244a9419b"
        },
        "date": 1776109023573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65809631347656,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.47175619621246,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.27387086808774,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.93828125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.2421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.92615806506,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3508.8638560823842,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002328,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66604.84490280299,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2442010.9075375064,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67118835449219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.35477618172342,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.22401978973407,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1723.9950520833333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3382.41796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.10114477748,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3495.1202145360335,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002228,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26116.32415931508,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542402.755929735,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69123077392578,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.02738356899355,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.36480916621507,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 325.512109375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 524.63671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.65890178122,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3529.1843047968896,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002817,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33556.42630722381,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2476177.7447943226,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163",
          "message": "Fix typo in Prometheus test config filename (#2648)\n\n# Change Summary\nFix typo in test config filename\nfake-debug-noop-prometh'~~u~~'eus-telemetry.yaml ->\nfake-debug-noop-prometheus-telemetry.yaml\n\n## What issue does this PR close?\nminor nit\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nN/A",
          "timestamp": "2026-04-13T23:38:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163"
        },
        "date": 1776133887866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.682624717002552,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.030267609966163,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.418229166666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.58984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.12172020819,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.5530167668703,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00762,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 109343.97982103733,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535518.9998823901,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.728615616196105,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.55839245691321,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.614453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.33203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.0039717519,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7801310678124,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003748,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 111661.89669772108,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2455312.9324683105,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.871172023877754,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.518552204176334,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.784635416666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.1388588721,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8505823427795,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002547,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115313.97084390417,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2448606.895021111,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.913452250597649,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.075300841893874,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.1640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.68378833903,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8355650151875,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002803,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 105337.7979979339,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 536948.3059979359,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163",
          "message": "Fix typo in Prometheus test config filename (#2648)\n\n# Change Summary\nFix typo in test config filename\nfake-debug-noop-prometh'~~u~~'eus-telemetry.yaml ->\nfake-debug-noop-prometheus-telemetry.yaml\n\n## What issue does this PR close?\nminor nit\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nN/A",
          "timestamp": "2026-04-13T23:38:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163"
        },
        "date": 1776133892244,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.68952178955078,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.863672662400255,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.89058869153805,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.54140625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.7421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.49219353516,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3475.8507701402687,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002576,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65567.62847315696,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2453472.0960709504,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65926361083984,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.470131449742276,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.725914469105255,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1779.3796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3076.78125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.85880260741,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3563.322789376419,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002142,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25509.825860318193,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535380.0521789895,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.70722198486328,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.69222171714497,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 56.22790014684288,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 329.60768229166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 501.59765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.28382720953,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1227548716374,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003028,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33757.84318489039,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2463172.301017596,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e",
          "message": "fix perf test (#2686)\n\nCloses #2667\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T02:51:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e"
        },
        "date": 1776361900000,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.74517879765753,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.710338642338023,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.166536458333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.84375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.33152866221,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.5819404458525,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00724,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110944.693915116,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543676.439603197,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6664810180664,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.03008547023763,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.971316257834868,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.702734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.40234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.19913759122,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3500.073297346636,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002172,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106493.64218091125,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549223.6555870821,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.05780540985202,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.809428880978178,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.048958333333335,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.52734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.17183863079,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.81583740411,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003091,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114861.77006496856,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2487625.426391597,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.98078727663038,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.24524833668575,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.656119791666665,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.28838142146,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.789516586908,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003588,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 112698.84604733778,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2509890.988557982,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e",
          "message": "fix perf test (#2686)\n\nCloses #2667\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T02:51:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e"
        },
        "date": 1776361905904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 49.78554175738116,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.985054129291676,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 313.19713541666664,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 471.22265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.1378920612,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1708106503484,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002207,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33628.92010472605,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2511873.3724920535,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69123077392578,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.10940514846939,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.75824889782659,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1657.26953125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3075.73046875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106652.93865591267,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3528.895772299176,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007723,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25807.481239804903,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 532868.1469669321,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67679595947266,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.241858476424134,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.266492588894856,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.584635416666668,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.37883903734,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3544.6075067782276,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002412,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66642.125422651,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2498815.609695544,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "6ef19a99a26f17c89a12f5dad18c737259c1509d",
          "message": "run validation ci manually (#2675)\n\nI am seeing the validation tests acting up again in the validation ci\njob, setting the ci job to be a manual trigger instead of automatically\nrunning on every PR\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T20:11:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ef19a99a26f17c89a12f5dad18c737259c1509d"
        },
        "date": 1776392107625,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.198671083012073,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.266030475604126,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.572395833333335,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.29296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.15532535355,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.804064240424,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00334,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115547.59096376719,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2494716.5738828066,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.269658127837648,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.181903584672433,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.603515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.46484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.58839153555,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.785058459097,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003664,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113015.03415563921,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2499218.3800707827,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.225838624397067,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.592404946616483,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.586067708333335,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.8203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.41170050869,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.846182721815,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002622,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106169.17293775841,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539807.2557109973,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.842264865352387,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.171980198019803,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.271744791666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.7890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.0731468643,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8683569234513,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002244,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 111531.15408604282,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545572.6064333174,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "6ef19a99a26f17c89a12f5dad18c737259c1509d",
          "message": "run validation ci manually (#2675)\n\nI am seeing the validation tests acting up again in the validation ci\njob, setting the ci job to be a manual trigger instead of automatically\nrunning on every PR\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T20:11:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ef19a99a26f17c89a12f5dad18c737259c1509d"
        },
        "date": 1776392113608,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.733466062659716,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.72440077369438,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1694.9169270833333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3184.31640625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.91647790163,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.0630528127035,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004048,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26496.632270571594,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543420.0284092956,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6742172241211,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.810786947508795,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.347739938080494,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.0203125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.2578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.60104385688,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3547.364786278896,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002287,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65655.66810154513,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2490834.4739618143,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.26978920306738,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.62944680851064,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 308.8111979166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 480.03125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.55693801558,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3478.0528203981503,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002539,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33573.237129425885,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2497643.731106456,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "495588ee584201ea956c15c6fc102dc3465de675",
          "message": "update container config to allow configurable wait_for setting (#2672)\n\n# Change Summary\n\nUpdate ContainerConfig struct to have a wait_for field of type WaitFor\nfrom the test container crate. Added additional functions to allow a\nuser to configure the WaitFor enum variant to use for a test container\n\n## What issue does this PR close?\n\n* Closes #2668\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-17T12:32:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/495588ee584201ea956c15c6fc102dc3465de675"
        },
        "date": 1776447561719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.87549072824287,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.237513288652647,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.917708333333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.43359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.2096368321,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8729325871336,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002166,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110869.221091421,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542898.4029245692,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.29198044417904,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.337541692402716,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.609114583333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.16015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.45894481485,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7136712928404,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004881,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116603.12428387564,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2461663.401519367,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.98159170968857,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.318089931119882,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.09375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.10289464757,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.8713955233698,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002227,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106650.29283491665,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545967.6021856553,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.10634466666046,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.363627699638766,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.072395833333335,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.83984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106658.39530811053,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7270451676472,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004653,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 112228.69501686188,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2491983.1573073408,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "495588ee584201ea956c15c6fc102dc3465de675",
          "message": "update container config to allow configurable wait_for setting (#2672)\n\n# Change Summary\n\nUpdate ContainerConfig struct to have a wait_for field of type WaitFor\nfrom the test container crate. Added additional functions to allow a\nuser to configure the WaitFor enum variant to use for a test container\n\n## What issue does this PR close?\n\n* Closes #2668\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-17T12:32:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/495588ee584201ea956c15c6fc102dc3465de675"
        },
        "date": 1776447566349,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.67118835449219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 35.546095629356756,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.04566087292391,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1711.0598958333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2801.59765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.17807282586,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3495.056199133758,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003327,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26104.486755911363,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542200.7654824936,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65809631347656,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.986201138308466,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.797806481553096,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.440104166666668,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.51171875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.82466603912,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3508.860464315536,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002386,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66362.53885522089,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2500451.5142811476,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67118835449219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.11104707700873,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.772298175069594,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 331.17565104166664,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 487.35546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.22971672987,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3495.091206356191,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002726,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33664.89491589615,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2511749.0945592383,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776481887073,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.946214752708386,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.96718278581821,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.024869791666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.44921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.14139180386,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8706447538057,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002205,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110413.06649386205,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542425.2953532445,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.97617741294613,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.259503420708743,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.719140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.16233508168,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.737251614167,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004479,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113484.07246612149,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2544687.939578795,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.234553901074435,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.407822468793345,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.319270833333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.3984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.6986762109,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.855803240594,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002458,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106560.50588658787,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546199.6882437351,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.071166875425053,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.268168492516587,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.871875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.74609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.05383675105,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8006619558446,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003398,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115590.71319751705,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2516086.203119518,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776481891409,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.70722198486328,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 35.93538935272604,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.14072251875919,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1736.0764322916666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3304.4375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106651.65011433056,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.8055377802802,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008448,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24780.03334748612,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535761.791386605,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67937469482422,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.6582202117097,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.76644625557435,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.8390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.36328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.69348133449,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3541.8680654145633,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002235,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66148.01318973025,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2479091.9678883087,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.594489026850326,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 62.78980742566631,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 325.2877604166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 485.453125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.52847862072,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.8493711948527,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007699,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33287.84116051398,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2459119.2238432113,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776533267268,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.109567239504067,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.47142439849335,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.710286458333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.15234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.70567563787,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.85603788805,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002454,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106765.31956156986,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 553587.7791886736,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.127791895597312,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.99399566697617,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.473828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.8125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.327707917,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.84336697017,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00267,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113817.49350200756,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2536021.6841316265,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.050342068538566,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.291324467676223,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.032421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.62109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.89553950846,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.6947838006645,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005203,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115479.4130361387,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2496391.4172740253,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.141676429618954,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.539267112237843,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.311979166666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.8828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.03639962591,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8671250160305,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002265,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110395.16244531843,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 551724.9278581326,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776533271191,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.6712015293846,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.20907318582701,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 343.04322916666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 524.20703125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.74242264476,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.85726978771,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002433,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33692.25870116562,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2487616.1430447455,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.98147119607722,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.24267326732673,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.579817708333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.9609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.85616354342,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8610828159317,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002368,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66429.82076173645,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2537379.2677249033,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.70722198486328,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.72232608424457,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.8070573719926,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1688.678125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2676.91796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.7882217389,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.875940980127,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007245,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25589.72011367697,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 551135.9834552688,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776564724555,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.003995951368712,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.880148606811145,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.564192708333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.28515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.1868884795,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.872169975694,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002179,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110790.54669250632,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545730.7212080202,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.912873755391097,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.021937075879087,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.286588541666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.2890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.44144791712,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8471799682698,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002605,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113098.36578071078,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2524661.6927648704,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.101068131752022,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.43050722410083,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.587239583333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.6171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.66017937925,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8545126801428,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00248,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106405.07533197841,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 553005.9154375605,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.159077754727875,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.624556222119672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.326953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.05859375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.61377631327,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8194332630733,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003078,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115625.31722137654,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2550547.088217683,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776564729436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.480651686435976,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.814703168255924,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 332.0397135416667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 471.75,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.51698423058,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.8824371782193,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007134,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33534.9939086299,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2502214.848623053,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.63609657749696,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.09137758633951,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1693.7572916666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2882.0078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.30321010307,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.142890141381,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002684,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26044.79359580922,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 548626.3913284141,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65023803710938,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.2376782146483,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.140104841196425,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.415755208333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.72317420937,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3517.106736518933,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002444,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65962.83803474603,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2493824.027011324,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776619481316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.28737585628122,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.163287671232876,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.0265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.38987964636,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.6778321557636,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005492,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114836.27802076406,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2508085.9124889667,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.049103163405075,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.162659236901167,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.142317708333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.12890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.5744365375,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8516382534476,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002529,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106862.39068716692,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547884.4170591182,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.65845026349081,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.940763476517315,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.88984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.99965241324,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.865893109473,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002286,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 111149.30130227133,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552939.6033455733,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.476877149785583,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.3428906009245,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.91640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.85546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.56696875635,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.5496736192604,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007677,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 112960.20216858404,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2575841.9297455074,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776619486217,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.87128693752386,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 62.96749903243285,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 314.3815104166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 505.0859375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.94272436654,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.0639307694532,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004033,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33019.40330591039,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2514847.3148051645,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 56.16027191566457,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.71547675942342,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.977604166666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.87890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.01015161425,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.866245082687,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00228,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66132.52913696531,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2481826.230140624,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 35.91362421120279,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.02784051624799,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1765.9266927083333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3035.3671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.81936383345,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.093254777069,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003532,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26437.198138576943,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546076.1241275386,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776650931335,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.906077860030255,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.988514118191638,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.60625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.7109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.03814949383,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8671836782696,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002264,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110690.98284268509,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549467.6956462662,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.603620886283974,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.44248277997059,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.395052083333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.05470943097,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7671674018766,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003969,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113535.3558537417,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2524654.9849370536,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.153439803982426,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.137491483431404,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.395963541666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.56568727581,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.851344944865,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002534,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106227.82051546517,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 550049.5697854412,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.486561928907978,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.723100107642626,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.540755208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.2890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.02234032808,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7996060757605,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003416,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115901.3506452209,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2560794.4532896606,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776650937525,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.90457620743878,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.52569837587007,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.202083333333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.484375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.00665188034,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.866127758274,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002282,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66480.1946056639,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2498831.800562094,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.60099792480469,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.83912113149793,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 79.26275215132956,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1771.1875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3023.375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103327.62964817675,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.106131741528,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003312,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26706.604237188792,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549030.5801014291,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.94824939009645,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.29177855207124,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 338.87265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 536.8125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104985.88289827961,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3477.732360921868,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008068,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33599.65608130044,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2503915.758495159,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "1e251126b4f960b4022cfa65b59ee21a50764def",
          "message": "test: Add JUnit XML upload and flaky test tracking workflow (#2699)\n\n# Change Summary\n\n- Implemented JUnit XML result uploads for both required and\nnon-required tests in the Rust-CI workflow.\n- Created a new workflow for detecting flaky tests from JUnit XML\nartifacts, which runs daily and on-demand.\n- The flaky test tracker parses JUnit results, identifies flaky tests,\nand creates or updates a tracking issue with a summary.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a. This is adding a new flaky test detection workflow. No changes to\nproduct code.\n\n## Are there any user-facing changes?\n\nNo. Test/infra only.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-20T18:48:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1e251126b4f960b4022cfa65b59ee21a50764def"
        },
        "date": 1776714760563,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.93193787383009,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.088930117501548,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.486848958333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.9453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.75292179432,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.8598464192123,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002427,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106238.28927212412,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546831.4725937351,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.59354400634766,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.866224738053992,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.981147008811252,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.39921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.37109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103329.2621604042,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.861317464092,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002364,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110431.10756882948,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 554058.0546366053,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.383387943490092,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.27291096155334,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.485677083333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.98828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.23231683807,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8066452882854,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003296,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114978.64560586895,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2562178.488873648,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.66857147216797,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.282366311908202,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.165474722564735,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.073958333333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.75390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.02585563627,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3497.73434707634,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004557,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114533.76037616469,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2510721.50077013,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "1e251126b4f960b4022cfa65b59ee21a50764def",
          "message": "test: Add JUnit XML upload and flaky test tracking workflow (#2699)\n\n# Change Summary\n\n- Implemented JUnit XML result uploads for both required and\nnon-required tests in the Rust-CI workflow.\n- Created a new workflow for detecting flaky tests from JUnit XML\nartifacts, which runs daily and on-demand.\n- The flaky test tracker parses JUnit results, identifies flaky tests,\nand creates or updates a tracking issue with a summary.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a. This is adding a new flaky test detection workflow. No changes to\nproduct code.\n\n## Are there any user-facing changes?\n\nNo. Test/infra only.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-20T18:48:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1e251126b4f960b4022cfa65b59ee21a50764def"
        },
        "date": 1776714766603,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 46.54113545173611,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.689683426443196,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 336.59869791666665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 513.546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.11857945757,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1032636821797,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003361,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33474.55457999779,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2503560.166033988,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.892028082862964,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.62222222222222,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1737.6390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3339.40625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.42487325157,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.9797074506805,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005472,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26008.13061963341,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544841.884478629,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.10093383804926,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.420380746014544,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.920963541666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.16796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.72014602127,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.867729893994,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00222,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65801.00677836277,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2512406.817671481,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "5a0d3fb508582e0922747323347ab66c85351dd3",
          "message": "Update docker.io/rust Docker tag to v1.95 (#2707)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.94` → `1.95` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-20T21:52:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a0d3fb508582e0922747323347ab66c85351dd3"
        },
        "date": 1776737832777,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.256411832474477,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.89973546601046,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.815755208333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.06518306013,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.812262776,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003151,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116078.66568551086,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2454748.152349799,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65547943115234,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.536985538787956,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.489057012454555,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.998567708333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104985.02913484538,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.2492958504117,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008556,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113149.27667032978,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2552639.059350809,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6871109008789,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.802363361001436,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.274092422014089,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.816666666666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.8046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.42150229088,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3533.6093623473785,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002388,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 109898.07677242439,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544089.074592237,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.014433007631894,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.2536403678232,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.9453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.58860036252,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.865423811963,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002294,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 105095.85801126607,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546876.5501953058,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "5a0d3fb508582e0922747323347ab66c85351dd3",
          "message": "Update docker.io/rust Docker tag to v1.95 (#2707)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.94` → `1.95` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-20T21:52:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a0d3fb508582e0922747323347ab66c85351dd3"
        },
        "date": 1776737837541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69123077392578,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 50.976673966552724,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 62.02643734643735,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 324.00403645833336,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 458.69921875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.82955269645,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3529.1899512357113,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002721,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33921.45821593666,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2539431.5799088767,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65926361083984,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 34.5845385476373,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.15642912113283,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1740.040625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2862.75390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.64290328593,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3563.2821694098207,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002826,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24680.571364289975,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539853.0099752194,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 56.1923938640618,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.55714020427112,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.377083333333335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.1484375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.43630789267,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.5565762567153,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007443,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 65875.64820252065,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2492053.110815765,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "693b34e1972d186270e39913fdadd6b8c3751bf9",
          "message": "Transport header support in OTLP/Fake Data receivers (#2702)\n\n# Change Summary\n\nUpdate the OTLP receiver to use the capture header policy when defined,\nextracts data from grpc/http headers\n\nUpdate the Fake Data Generator to allow user to specify transport\nheaders to tack on to generated OtapPdata, users can provide key/value\npairs or just the key (random value will be generated).\n\n## What issue does this PR close?\n\n* Closes #2692\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-21T17:41:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/693b34e1972d186270e39913fdadd6b8c3751bf9"
        },
        "date": 1776795346242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.921707962488787,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.395004625346902,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.831770833333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.35546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.77217024061,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8582670404476,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002416,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110334.7084292452,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 551882.1673142425,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64590454101562,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.988240232288756,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.65804006497022,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.55078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.7421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 101662.79839718765,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3409.870254436819,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002283,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 107488.74910057994,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552912.6886352791,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.999447917685536,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.97218089211298,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.9265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.25,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.49478865658,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8154443435346,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003146,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116703.7390317777,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2501614.2829554253,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.945215184998048,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 26.968443206437637,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.522265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.80886507059,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.7924495718903,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003538,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 112931.81750572998,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2561873.4827938224,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "693b34e1972d186270e39913fdadd6b8c3751bf9",
          "message": "Transport header support in OTLP/Fake Data receivers (#2702)\n\n# Change Summary\n\nUpdate the OTLP receiver to use the capture header policy when defined,\nextracts data from grpc/http headers\n\nUpdate the Fake Data Generator to allow user to specify transport\nheaders to tack on to generated OtapPdata, users can provide key/value\npairs or just the key (random value will be generated).\n\n## What issue does this PR close?\n\n* Closes #2692\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-21T17:41:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/693b34e1972d186270e39913fdadd6b8c3751bf9"
        },
        "date": 1776795353309,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.65495300292969,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.63801428694678,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.39277751901878,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1688.6115885416666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3280.203125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.70392578104,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3512.1562942716264,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002455,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25806.201438357068,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 550665.0296976169,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 47.67003708389454,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.6169398653355,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 331.165234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 506.3515625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.97690415163,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3478.066732076383,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002299,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33665.42929529767,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2469276.1393781225,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.66595458984375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.93272496989693,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.00269439702003,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.226822916666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.12109375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.54643890522,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3500.6015161523565,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002545,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66679.89793870626,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2490847.867150274,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "dd344850b0de4426753c6c5ac7ca8786a2545458",
          "message": "[query-engine] Tweak slice validation errors (#2721)\n\nRelates to #2636\n\n# Changes\n\n* Tweak the slice validation error messages\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-21T19:39:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd344850b0de4426753c6c5ac7ca8786a2545458"
        },
        "date": 1776824215144,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.791649376301976,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.233538223938224,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.359114583333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.765625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.27573631551,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.855099298412,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00247,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 109419.25328261132,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539763.628968941,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.749806185945198,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.18084803466419,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.794921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.81640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.72725660667,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8699994680196,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002216,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 105602.64345261952,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541457.664221495,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.151863425369605,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.967589043221853,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.70625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.0390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.8643156479,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8085224163806,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003264,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 114642.51795170846,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2473719.9161360166,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.559169636686008,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.862827369550224,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.525,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.47249301928,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.5465064326463,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007731,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113236.66861535868,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2577214.5590071,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "dd344850b0de4426753c6c5ac7ca8786a2545458",
          "message": "[query-engine] Tweak slice validation errors (#2721)\n\nRelates to #2636\n\n# Changes\n\n* Tweak the slice validation error messages\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-21T19:39:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd344850b0de4426753c6c5ac7ca8786a2545458"
        },
        "date": 1776824219979,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.108050961469104,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 66.15058205148026,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1703.015234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2814.578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.23671609431,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3478.0422128182786,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002722,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26779.353812293666,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547969.2930892443,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.61250902523499,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.84263365570599,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.541666666666668,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.74609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.07784050246,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.5667999863686,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007385,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66862.45456567862,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2519830.156566404,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.63871765136719,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.910528891745535,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 61.62303980328877,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 335.8678385416667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 496.74609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.01798639655,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3529.1825402884633,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002847,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33512.143262448684,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2453116.7149760495,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "3a6d2a3b70b838918f335bbc21b28c0fdab54f25",
          "message": "fix(benchmarks): Bump max_decoding_message_size to 32MiB to fix batch processor benchmarks (#2730)\n\n# Change Summary\n\nBatch processor benchmarks had 100% signal drop rate due to being over\nthe decompression limit on the backend engine. Bumping the limit fixes\nthe issue for both continuous and nightly.\n\n## What issue does this PR close?\n\n* Closes #2729\n\n## How are these changes tested?\n\nRan all scenarios locally and observed the dropped rate being 0 (or\nless):\n\n<img width=\"1891\" height=\"466\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6af43bf9-2e61-4af9-a48b-8285f8768a92\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-04-22T15:57:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3a6d2a3b70b838918f335bbc21b28c0fdab54f25"
        },
        "date": 1776881476689,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.552700642438957,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 23.17190802122587,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.875260416666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.58203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.00573756042,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3464.835189339494,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002854,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116640.68534001848,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2557350.2962292125,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.66857147216797,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.237541285187298,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.388913278713865,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.872005208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.4296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.30064605865,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3497.676872951554,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005543,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113391.26062643163,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2483548.097799924,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.062329109181356,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.37733353836691,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.505078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.40470112092,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8459480756724,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002626,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106419.23825995116,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 550815.2019963172,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.808880503383763,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.896801732137334,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.656640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.2734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.94715643975,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8641332444568,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002316,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 110023.53401052019,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552005.2798247191,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "3a6d2a3b70b838918f335bbc21b28c0fdab54f25",
          "message": "fix(benchmarks): Bump max_decoding_message_size to 32MiB to fix batch processor benchmarks (#2730)\n\n# Change Summary\n\nBatch processor benchmarks had 100% signal drop rate due to being over\nthe decompression limit on the backend engine. Bumping the limit fixes\nthe issue for both continuous and nightly.\n\n## What issue does this PR close?\n\n* Closes #2729\n\n## How are these changes tested?\n\nRan all scenarios locally and observed the dropped rate being 0 (or\nless):\n\n<img width=\"1891\" height=\"466\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6af43bf9-2e61-4af9-a48b-8285f8768a92\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-04-22T15:57:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3a6d2a3b70b838918f335bbc21b28c0fdab54f25"
        },
        "date": 1776881483732,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.49080481305493,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.08967567983977,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.358463541666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.40234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.17288949818,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8717006765105,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002187,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 67123.99635611259,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2497706.912098762,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.59354400634766,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.62121250390467,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 56.78534588747958,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1707.6755208333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3212.91015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103328.46311843835,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.834098486158,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002828,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25292.65009242488,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542394.1233547298,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.68743133544922,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 48.08921561001828,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.69362178988327,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 303.26770833333336,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 473.69921875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.08539594949,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3478.070325944681,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002237,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33581.0035556616,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2477983.8330639545,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "446e2510d411d9944a3a37faba577e2029677e8b",
          "message": "feat(config): add optional field to EngineConfig (#2727)\n\nAllow applications embedding the dataflow engine to carry their own\nengine-level configuration under `engine.custom`. The engine ignores\nthis field entirely; embedding binaries can read namespaced keys for\nconcerns like remote management, auth, or fleet coordination.\n\n# Change Summary\n\nAdd an optional `custom: HashMap<String, serde_json::Value>` field to\n`EngineConfig`. This gives embedding applications an escape hatch for\nengine-level config without forking the config crate or pre-parsing\nYAML. The field defaults to an empty map and is omitted from serialized\noutput when empty.\n\n## What issue does this PR close?\n\n* Closes #2561\n\n## How are these changes tested?\n\nThree new unit tests in `engine.rs`:\n- `from_yaml_accepts_custom_config` — parses a config with multiple\nnamespaced custom keys and verifies values\n- `custom_defaults_to_empty` — confirms the field defaults to an empty\nmap when omitted\n- `custom_roundtrips_through_json` — serializes to JSON and deserializes\nback, verifying data is preserved\n\n## Are there any user-facing changes?\n\nYes. A new optional `custom` key is available under the `engine` section\nof the YAML/JSON config. Example:\n\n```yaml\nengine:\n  custom:\n    remote_management:\n      server_url: \"ws://mgmt.example.com/v1\"\n      heartbeat_interval_secs: 10\n    custom_auth:\n      provider: \"oidc\"\n      token_endpoint: \"https://auth.example.com/token\"\n```\n\nExisting configs are unaffected since the field defaults to empty.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-22T23:05:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/446e2510d411d9944a3a37faba577e2029677e8b"
        },
        "date": 1776911190873,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.694522648423073,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.66680933550438,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.890364583333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.0546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.06089778198,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.8679462875484,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002251,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 109720.60686145522,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 551299.1951129456,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.59354400634766,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.543338153866422,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.45658056580566,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.731380208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103324.45776241155,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.6976579711804,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005154,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 115414.00924299612,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2508065.5490947063,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67381286621094,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.709745425709112,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.55463459759482,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.921223958333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.89728920504,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3492.1972265004624,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005202,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 113245.04656723421,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2555185.7814391577,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.64762115478516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.087422593463028,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.04159382239382,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.194661458333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.84375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.14489154673,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3519.870762078519,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002203,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 106752.92328484905,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 548213.2063632724,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "446e2510d411d9944a3a37faba577e2029677e8b",
          "message": "feat(config): add optional field to EngineConfig (#2727)\n\nAllow applications embedding the dataflow engine to carry their own\nengine-level configuration under `engine.custom`. The engine ignores\nthis field entirely; embedding binaries can read namespaced keys for\nconcerns like remote management, auth, or fleet coordination.\n\n# Change Summary\n\nAdd an optional `custom: HashMap<String, serde_json::Value>` field to\n`EngineConfig`. This gives embedding applications an escape hatch for\nengine-level config without forking the config crate or pre-parsing\nYAML. The field defaults to an empty map and is omitted from serialized\noutput when empty.\n\n## What issue does this PR close?\n\n* Closes #2561\n\n## How are these changes tested?\n\nThree new unit tests in `engine.rs`:\n- `from_yaml_accepts_custom_config` — parses a config with multiple\nnamespaced custom keys and verifies values\n- `custom_defaults_to_empty` — confirms the field defaults to an empty\nmap when omitted\n- `custom_roundtrips_through_json` — serializes to JSON and deserializes\nback, verifying data is preserved\n\n## Are there any user-facing changes?\n\nYes. A new optional `custom` key is available under the `engine` section\nof the YAML/JSON config. Example:\n\n```yaml\nengine:\n  custom:\n    remote_management:\n      server_url: \"ws://mgmt.example.com/v1\"\n      heartbeat_interval_secs: 10\n    custom_auth:\n      provider: \"oidc\"\n      token_endpoint: \"https://auth.example.com/token\"\n```\n\nExisting configs are unaffected since the field defaults to empty.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-22T23:05:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/446e2510d411d9944a3a37faba577e2029677e8b"
        },
        "date": 1776911197427,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.67118835449219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.499366877534925,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.45920037120099,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1705.052734375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3213.33984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.20905836306,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3494.8242139166045,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00731,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25426.86793043045,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 548117.4408239985,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6484375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 49.719200774463104,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.64295331161781,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 306.1015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 466.6875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.60693249777,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3574.562294846996,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007347,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33081.55227133692,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2505512.093444392,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.65547943115234,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 55.93027146934924,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.4217430202067,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.298828125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.94921875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.91040928956,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3511.6132226649775,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002337,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66838.15300653782,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2564362.803835505,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9c54c8e469806b54ec6fa8b9721790c951ca0a97",
          "message": "Task 1614: Fix tests that are ignored on Windows and MacOS (#2724)\n\n# Change Summary\n\nFix tests that are ignored on Windows and MacOS\n\n## What issue does this PR close?\n\n* Closes #1614\n\n## How are these changes tested?\n\nBy running the reenabled tests.\n\n## Are there any user-facing changes?\n\nYes, one production code change:\n\nWindows CA certificate hot-reload fix - The get_file_identity() function\nnow uses MetadataExt::last_write_time() (100ns-precision FILETIME) on\nWindows instead of the platform-fallback get_mtime() which truncated to\nwhole seconds. Previously, if a CA certificate file was replaced within\nthe same second (e.g., by automation or cert-manager), the file watcher\ncould miss the change entirely, leaving stale CA certificates in memory\nuntil the next reload event. This affects any Windows deployment using\nmTLS with watch_client_ca: true.\n\nAll other changes are test-only (removing #[ignore] attributes and\nfixing test assertions for cross-platform compatibility).",
          "timestamp": "2026-04-23T17:12:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9c54c8e469806b54ec6fa8b9721790c951ca0a97"
        },
        "date": 1776969970853,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.72750091552734,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.59583212616575,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.88046185820953,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.554296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.47265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.77372102297,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.2962450204764,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003736,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 318505.2821790555,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2544271.5473545487,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.78250122070312,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.513643324297924,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.94186281364095,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.781510416666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.58203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.38846376998,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3217.3837988217992,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002167,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 309214.3763930661,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 767585.7118286042,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.60407932536236,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.029301570113695,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.168619791666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.0234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.36513212745,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.882048537536,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 127268.28063065543,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 773358.9200774481,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.06735824048143,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.51232435778397,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.328385416666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.08203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.58041213754,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.791684373863,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003852,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 134789.77610396015,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2543797.539192791,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9c54c8e469806b54ec6fa8b9721790c951ca0a97",
          "message": "Task 1614: Fix tests that are ignored on Windows and MacOS (#2724)\n\n# Change Summary\n\nFix tests that are ignored on Windows and MacOS\n\n## What issue does this PR close?\n\n* Closes #1614\n\n## How are these changes tested?\n\nBy running the reenabled tests.\n\n## Are there any user-facing changes?\n\nYes, one production code change:\n\nWindows CA certificate hot-reload fix - The get_file_identity() function\nnow uses MetadataExt::last_write_time() (100ns-precision FILETIME) on\nWindows instead of the platform-fallback get_mtime() which truncated to\nwhole seconds. Previously, if a CA certificate file was replaced within\nthe same second (e.g., by automation or cert-manager), the file watcher\ncould miss the change entirely, leaving stale CA certificates in memory\nuntil the next reload event. This affects any Windows deployment using\nmTLS with watch_client_ca: true.\n\nAll other changes are test-only (removing #[ignore] attributes and\nfixing test assertions for cross-platform compatibility).",
          "timestamp": "2026-04-23T17:12:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9c54c8e469806b54ec6fa8b9721790c951ca0a97"
        },
        "date": 1776969977041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6882553100586,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.29991274112875,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.69138849929874,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1717.521484375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3186.38671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.45423636372,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3256.421534170477,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002367,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 56002.97298592144,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 762066.1664969231,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.97760884142733,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.49675915131732,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.9234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.73828125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.98349466297,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.869664401813,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00241,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84361.03659588224,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2553835.30786194,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.72293090820312,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.69310186633344,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.72592248781619,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 357.63111979166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 516.3671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98321.38892326897,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3222.0585735842856,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007289,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59867.540106965025,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2535667.795863005,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "da7688be82431d7cf4508c7376036fb240785034",
          "message": "Use latest weaver build and simplify crypto selection. (#2740)\n\n# Change Summary\n\n<!--\nUse latest weaver build that has the exclusion of openssl build in case\nof windows platforms. In case of windows platforms it uses SChannel TLS\ninstead of natively building openssl.\n\nSimplification of crypto selection.\n\n-->\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/2697\n\n## How are these changes tested?\n\nSearch cargo tree and check on windows platform no openssl dependency.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-24T00:33:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/da7688be82431d7cf4508c7376036fb240785034"
        },
        "date": 1776999396162,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.78250122070312,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 62.861140479870336,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.292333488588724,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.69375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.16796875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.12015053816,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3217.3751658435654,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002328,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 140257.85113581942,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2550414.468193516,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.75553894042969,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.47264179570494,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.81741009415033,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.913671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.59765625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 100008.95097609886,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7496675631473,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004629,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 318076.46784200514,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2536634.2637150604,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.694697729724325,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.01598883201489,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.099869791666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.59375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.35346631026,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.881669981768,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002188,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 127220.92534041623,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772976.3331190055,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.72750091552734,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.66075214038312,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.27633663366337,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.487760416666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.22265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.21847633796,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.3762496381596,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002269,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 308963.7422721426,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 767190.1536623855,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "da7688be82431d7cf4508c7376036fb240785034",
          "message": "Use latest weaver build and simplify crypto selection. (#2740)\n\n# Change Summary\n\n<!--\nUse latest weaver build that has the exclusion of openssl build in case\nof windows platforms. In case of windows platforms it uses SChannel TLS\ninstead of natively building openssl.\n\nSimplification of crypto selection.\n\n-->\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/2697\n\n## How are these changes tested?\n\nSearch cargo tree and check on windows platform no openssl dependency.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-24T00:33:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/da7688be82431d7cf4508c7376036fb240785034"
        },
        "date": 1776999402455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.94515514898427,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.1346527885661,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.866015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.91015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.98349466297,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.869664401813,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00241,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84423.40383929436,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2554825.495198207,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.77754974365234,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.46454172439365,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.11160726710474,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1702.34765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3285.0390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.70361386526,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3222.2793266050007,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003178,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 56216.00896360432,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 761093.5901962934,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.77754974365234,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.33641681014609,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.07721938380554,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 349.1713541666667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 496.83203125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.2139396079,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3222.199098096895,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004672,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 61024.35116221764,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2534198.7667410434,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "5507512bea4dd9e53a8ae74d1f5cd3a85b875d2c",
          "message": "Add live pipeline reconfiguration and shutdown control to the admin API (#2618)\n\n## Change Summary\n\nThis PR adds live pipeline lifecycle control to the OTAP Dataflow Engine\nadmin API.\n\nThe controller now runs as a resident runtime manager and supports\nin-memory pipeline operations on a running engine:\n\n- create a new pipeline in an existing group\n- replace an existing pipeline with a health-gated serial rolling\ncutover\n- resize a pipeline when only the effective core allocation changes\n- detect identical updates and return a successful `noop` result\n- shut down an individual logical pipeline and track shutdown progress\n\nThe PR also adds rollout and shutdown status resources, extends runtime\nand observed-state tracking with deployment generations so overlapping\ninstances remain distinguishable, updates the Rust admin SDK with typed\nlive-control methods/outcomes, and documents the feature.\n\n## Design Decisions\n\nThis PR keeps live reconfiguration scoped to one logical pipeline at a\ntime, keyed by `(group, pipeline)`. Reconfiguration mutations go through\none declarative endpoint, `PUT /api/v1/groups/{group}/pipelines/{id}`,\nwhile pipeline shutdown remains a separate operation through `POST\n/api/v1/groups/{group}/pipelines/{id}/shutdown`.\n\nThe controller classifies each reconfiguration request as `create`,\n`noop`, `resize`, or `replace` instead of exposing separate start,\nscale, and update APIs.\n\nTopology and node-configuration changes use a serial rolling cutover\nwith overlap: start the new instance on one core, wait for `Admitted`\nand `Ready`, then drain the old instance on that core. This preserves\ncontinuity while fitting the existing runtime model, without requiring a\nfull second serving fleet or a separate traffic-switching layer.\n\nPure core-allocation changes use a dedicated internal resize path so\nscale up/down only starts or stops the delta cores and leaves unchanged\ncores running.\n\nRuntime mutations remain intentionally narrow. This PR keeps changes in\nmemory only and rejects updates that would require topic-broker\nreconfiguration or broader engine/group-level policy mutation. Runtime\nconfig persistence remains out of scope.\n\nOperations are explicit and observable through rollout and shutdown ids.\nTerminal operation history is intentionally bounded in memory, so old\nrollout/shutdown ids may eventually return `404` or `Ok(None)` from the\nSDK after retention pruning.\n\nObserved status is deployment-generation-aware. During overlapping\nrollouts, `/status.instances` preserves old and new runtime instances,\nwhile aggregate readiness/liveness still uses the selected serving\ngeneration per core. After controller work completes, superseded\nobserved instances are compacted so status memory does not grow\nunbounded across rollouts.\n\nThe controller also hardens lifecycle cleanup around live operations:\nworker panics are converted into terminal rollout/shutdown failure\nstates, runtime exits are reported through supervised bookkeeping, and\nshutdown drain completion waits for upstream closure rather than queue\nemptiness alone.\n\n## What issue does this PR close?\n\n* Closes #2617\n\n## How are these changes tested?\n\nCommands run:\n\n- `cargo xtask check`\n\nAutomated coverage includes:\n\n- rollout planning for create, replace, resize, scale up/down, noop,\ntopic-mutation rejection, and conflict handling\n- rollout execution success/failure paths, rollback behavior, worker\npanic cleanup, and bounded operation retention\n- per-pipeline shutdown planning, progress tracking, conflict handling,\ntimeout behavior, and worker panic cleanup\n- observed-state generation selection, overlap-aware\n`/status.instances`, compaction of superseded generations, and\nshutdown-terminal status behavior\n- admin HTTP handlers for reconfigure, rollout status, shutdown,\nshutdown status, operation errors, wait timeouts, and missing retained\noperation ids\n- Rust admin SDK decoding for typed reconfigure/shutdown outcomes,\noperation rejection errors, pipeline details/status, and\nrollout/shutdown polling\n- engine shutdown-drain behavior where downstream nodes wait for\nupstream channel closure before completing shutdown drain\n- route/doc compatibility checks through the full workspace validation\nsuite\n\nManual validation covered:\n\n- creating a new pipeline through the admin API\n- replacing a pipeline with a topology/config change\n- resizing a pipeline by changing only core allocation\n- submitting an identical config and observing a noop rollout\n- shutting down a pipeline and observing shutdown tracking/status\n- checking generation-aware status during overlapping rollout behavior\n\n## Are there any user-facing changes?\n\nYes.\n\n- The admin API now supports live pipeline create/update/resize/noop via\n`PUT /api/v1/groups/{group}/pipelines/{id}`.\n- Pipeline shutdown is available via `POST\n/api/v1/groups/{group}/pipelines/{id}/shutdown`.\n- Rollout and shutdown progress can be queried through dedicated status\nendpoints.\n- New pipeline-scoped admin routes are under `/api/v1/groups/...`.\n- Pipeline status payloads now include deployment-generation-aware\ninstance data and rollout metadata while preserving the existing\naggregate status shape.\n- Terminal rollout and shutdown ids are retained only within a bounded\nin-memory window.\n- The Rust admin SDK now exposes typed clients, request options,\noperation outcomes, and error models for live reconfiguration and\nshutdown.\n- Operator documentation was added in\n`rust/otap-dataflow/docs/admin/live-reconfiguration.md`.\n- Support for the existing WebSocket log stream endpoint\n`/api/v1/telemetry/logs/stream` is preserved/restored in this branch.",
          "timestamp": "2026-04-24T05:43:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5507512bea4dd9e53a8ae74d1f5cd3a85b875d2c"
        },
        "date": 1777057641306,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.7005615234375,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 62.7498523282274,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.264002483122525,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.977734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.5546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.53405350354,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.2256617954868,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004149,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 136787.88229115456,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2541169.0769504756,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "5507512bea4dd9e53a8ae74d1f5cd3a85b875d2c",
          "message": "Add live pipeline reconfiguration and shutdown control to the admin API (#2618)\n\n## Change Summary\n\nThis PR adds live pipeline lifecycle control to the OTAP Dataflow Engine\nadmin API.\n\nThe controller now runs as a resident runtime manager and supports\nin-memory pipeline operations on a running engine:\n\n- create a new pipeline in an existing group\n- replace an existing pipeline with a health-gated serial rolling\ncutover\n- resize a pipeline when only the effective core allocation changes\n- detect identical updates and return a successful `noop` result\n- shut down an individual logical pipeline and track shutdown progress\n\nThe PR also adds rollout and shutdown status resources, extends runtime\nand observed-state tracking with deployment generations so overlapping\ninstances remain distinguishable, updates the Rust admin SDK with typed\nlive-control methods/outcomes, and documents the feature.\n\n## Design Decisions\n\nThis PR keeps live reconfiguration scoped to one logical pipeline at a\ntime, keyed by `(group, pipeline)`. Reconfiguration mutations go through\none declarative endpoint, `PUT /api/v1/groups/{group}/pipelines/{id}`,\nwhile pipeline shutdown remains a separate operation through `POST\n/api/v1/groups/{group}/pipelines/{id}/shutdown`.\n\nThe controller classifies each reconfiguration request as `create`,\n`noop`, `resize`, or `replace` instead of exposing separate start,\nscale, and update APIs.\n\nTopology and node-configuration changes use a serial rolling cutover\nwith overlap: start the new instance on one core, wait for `Admitted`\nand `Ready`, then drain the old instance on that core. This preserves\ncontinuity while fitting the existing runtime model, without requiring a\nfull second serving fleet or a separate traffic-switching layer.\n\nPure core-allocation changes use a dedicated internal resize path so\nscale up/down only starts or stops the delta cores and leaves unchanged\ncores running.\n\nRuntime mutations remain intentionally narrow. This PR keeps changes in\nmemory only and rejects updates that would require topic-broker\nreconfiguration or broader engine/group-level policy mutation. Runtime\nconfig persistence remains out of scope.\n\nOperations are explicit and observable through rollout and shutdown ids.\nTerminal operation history is intentionally bounded in memory, so old\nrollout/shutdown ids may eventually return `404` or `Ok(None)` from the\nSDK after retention pruning.\n\nObserved status is deployment-generation-aware. During overlapping\nrollouts, `/status.instances` preserves old and new runtime instances,\nwhile aggregate readiness/liveness still uses the selected serving\ngeneration per core. After controller work completes, superseded\nobserved instances are compacted so status memory does not grow\nunbounded across rollouts.\n\nThe controller also hardens lifecycle cleanup around live operations:\nworker panics are converted into terminal rollout/shutdown failure\nstates, runtime exits are reported through supervised bookkeeping, and\nshutdown drain completion waits for upstream closure rather than queue\nemptiness alone.\n\n## What issue does this PR close?\n\n* Closes #2617\n\n## How are these changes tested?\n\nCommands run:\n\n- `cargo xtask check`\n\nAutomated coverage includes:\n\n- rollout planning for create, replace, resize, scale up/down, noop,\ntopic-mutation rejection, and conflict handling\n- rollout execution success/failure paths, rollback behavior, worker\npanic cleanup, and bounded operation retention\n- per-pipeline shutdown planning, progress tracking, conflict handling,\ntimeout behavior, and worker panic cleanup\n- observed-state generation selection, overlap-aware\n`/status.instances`, compaction of superseded generations, and\nshutdown-terminal status behavior\n- admin HTTP handlers for reconfigure, rollout status, shutdown,\nshutdown status, operation errors, wait timeouts, and missing retained\noperation ids\n- Rust admin SDK decoding for typed reconfigure/shutdown outcomes,\noperation rejection errors, pipeline details/status, and\nrollout/shutdown polling\n- engine shutdown-drain behavior where downstream nodes wait for\nupstream channel closure before completing shutdown drain\n- route/doc compatibility checks through the full workspace validation\nsuite\n\nManual validation covered:\n\n- creating a new pipeline through the admin API\n- replacing a pipeline with a topology/config change\n- resizing a pipeline by changing only core allocation\n- submitting an identical config and observing a noop rollout\n- shutting down a pipeline and observing shutdown tracking/status\n- checking generation-aware status during overlapping rollout behavior\n\n## Are there any user-facing changes?\n\nYes.\n\n- The admin API now supports live pipeline create/update/resize/noop via\n`PUT /api/v1/groups/{group}/pipelines/{id}`.\n- Pipeline shutdown is available via `POST\n/api/v1/groups/{group}/pipelines/{id}/shutdown`.\n- Rollout and shutdown progress can be queried through dedicated status\nendpoints.\n- New pipeline-scoped admin routes are under `/api/v1/groups/...`.\n- Pipeline status payloads now include deployment-generation-aware\ninstance data and rollout metadata while preserving the existing\naggregate status shape.\n- Terminal rollout and shutdown ids are retained only within a bounded\nin-memory window.\n- The Rust admin SDK now exposes typed clients, request options,\noperation outcomes, and error models for live reconfiguration and\nshutdown.\n- Operator documentation was added in\n`rust/otap-dataflow/docs/admin/live-reconfiguration.md`.\n- Support for the existing WebSocket log stream endpoint\n`/api/v1/telemetry/logs/stream` is preserved/restored in this branch.",
          "timestamp": "2026-04-24T05:43:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5507512bea4dd9e53a8ae74d1f5cd3a85b875d2c"
        },
        "date": 1777057647437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.96241260559981,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.23143100115963,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.834375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.39453125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.5623946155,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8755590223113,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002301,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84398.83886713542,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2551642.268022297,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777080051388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.28851588593252,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.75508449497702,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.022916666666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.5,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.41770862117,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7717843844985,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00422,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 137980.4952722082,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2550734.604942115,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777080057610,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.00540042566139,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.44885860306644,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.144661458333335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.61019271254,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.857550753522,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002634,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84275.60352310241,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2553897.88829466,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777135294257,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75444793701172,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.207155258339064,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.873206614124555,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.774609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.15625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.660695498,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3245.2793437027353,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005004,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 139962.14495189625,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2545967.5935558914,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777135300411,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6994400024414,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.67341794227512,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.81972415930575,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.881640625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.0078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98312.7013877107,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8691236120144,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00242,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84457.06256211581,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2550420.1715602735,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777166647806,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.2250612662936,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.81581873394067,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.272916666666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.7265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.03470040462,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7261451133527,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005064,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 139927.89812234862,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2550700.671104961,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777166652836,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.89793896447564,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.08028669367404,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.069401041666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.5703125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.69351879912,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8602546850316,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002584,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84336.47169405213,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2557880.557559378,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777221744312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.200510303383204,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.702786377708975,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.906119791666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.36328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99989.6277426155,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.6634202478726,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006224,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 136858.79825348515,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2550764.0522871604,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777221750783,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6994400024414,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.97610361263075,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.86511136980985,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.0953125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.51953125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.66563680509,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3245.4289455003327,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002238,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84227.34935890303,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2547762.6847441983,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777253050182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.18598773403118,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.91013169173225,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.141145833333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.7890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.38740849789,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7205214057562,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005168,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 143435.4952619371,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2547925.9223891455,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777253054694,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.9410465551076,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.63155042959981,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.9921875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.08715558633,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8598761343487,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002591,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84150.0819994023,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2547886.665979575,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777309274124,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.32519290265811,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.70984686774942,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.616927083333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.0078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.75777570934,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.717006598408,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005233,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 139679.08076895867,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2566427.1976819616,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777309280040,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.92822444320446,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.85073351903436,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.291927083333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.6640625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.04015680979,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8715030884778,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002376,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84315.18512446918,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2569844.283564805,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d",
          "message": "Fix batch byte sizing and wakeup state (#2763)\n\n# Change Summary\n\nFixes batch processor behavior for OTLP byte-sized batches by comparing\n`min_size` against pending bytes instead of item count. Also avoids\nunnecessary wakeup set/cancel work for immediate size flushes, clears\nstale timer state after full drains, and adds wakeup scheduler metrics\nfor attribution.\n\n## What issue does this PR close?\n\n* Closes #NNN\n\n## How are these changes tested?\n\n- cargo xtask check\n- controlled benchmark\n\n## Are there any user-facing changes?",
          "timestamp": "2026-04-27T22:13:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d"
        },
        "date": 1777340122002,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.28242039141331,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.69563937727519,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.777083333333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.4921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.77821478568,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7836810879276,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 137483.18649965373,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2565457.0724910824,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d",
          "message": "Fix batch byte sizing and wakeup state (#2763)\n\n# Change Summary\n\nFixes batch processor behavior for OTLP byte-sized batches by comparing\n`min_size` against pending bytes instead of item count. Also avoids\nunnecessary wakeup set/cancel work for immediate size flushes, clears\nstale timer state after full drains, and adds wakeup scheduler metrics\nfor attribution.\n\n## What issue does this PR close?\n\n* Closes #NNN\n\n## How are these changes tested?\n\n- cargo xtask check\n- controlled benchmark\n\n## Are there any user-facing changes?",
          "timestamp": "2026-04-27T22:13:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d"
        },
        "date": 1777340126733,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.7005615234375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.99276960079696,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.0060710973725,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 63.20455729166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 64.4140625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.40179608486,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.320281260754,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002399,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 83463.40288575915,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2566700.02334982,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "9eab7837283aaa7f64ea77ae318fbb236d1f3794",
          "message": "Update Rust crate zip to v8 (#2762)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [zip](https://redirect.github.com/zip-rs/zip2) |\nworkspace.dependencies | major | `=4.6.1` → `=8.6.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>zip-rs/zip2 (zip)</summary>\n\n###\n[`v8.6.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#860---2026-04-25)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.1...v8.6.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add `compression not supported` as enum error\n([#&#8203;774](https://redirect.github.com/zip-rs/zip2/pull/774))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- allow for `[u8]` as filename\n([#&#8203;775](https://redirect.github.com/zip-rs/zip2/pull/775))\n\n##### <!-- 2 -->🚜 Refactor\n\n- mark `ZipFlags` as non-exhaustive and add test for `HasZipMetadata`\n([#&#8203;777](https://redirect.github.com/zip-rs/zip2/pull/777))\n- use and simplify is\\_dir\n([#&#8203;776](https://redirect.github.com/zip-rs/zip2/pull/776))\n\n###\n[`v8.5.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#851---2026-04-06)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.0...v8.5.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- change magic finder to stack buffer\n([#&#8203;763](https://redirect.github.com/zip-rs/zip2/pull/763))\n- simplify extra field parsing\n([#&#8203;764](https://redirect.github.com/zip-rs/zip2/pull/764))\n\n###\n[`v8.5.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#850---2026-04-01)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.4.0...v8.5.0)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- remove `zip64 comment` and add `zip64 extensible data sector`\n([#&#8203;747](https://redirect.github.com/zip-rs/zip2/pull/747))\n\n##### <!-- 2 -->🚜 Refactor\n\n- remove useless magic in struct\n([#&#8203;730](https://redirect.github.com/zip-rs/zip2/pull/730))\n- change extra\\_field from Arc\\<Vec<u8>> to Arc<\\[u8]>\n([#&#8203;741](https://redirect.github.com/zip-rs/zip2/pull/741))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup README\n([#&#8203;758](https://redirect.github.com/zip-rs/zip2/pull/758))\n\n###\n[`v8.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#840---2026-03-23)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.1...v8.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add a check for building benches\n([#&#8203;748](https://redirect.github.com/zip-rs/zip2/pull/748))\n\n##### <!-- 2 -->🚜 Refactor\n\n- split part of `read.rs` for code readability\n([#&#8203;744](https://redirect.github.com/zip-rs/zip2/pull/744))\n- remove unused allow\n([#&#8203;745](https://redirect.github.com/zip-rs/zip2/pull/745))\n\n##### <!-- 4 -->⚡ Performance\n\n- skip BufReader for Stored files in make\\_reader\n([#&#8203;739](https://redirect.github.com/zip-rs/zip2/pull/739))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- move pull request template to correct folder\n([#&#8203;749](https://redirect.github.com/zip-rs/zip2/pull/749))\n\n###\n[`v8.3.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#831---2026-03-21)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.0...v8.3.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- use `AexEncryption::new`\n([#&#8203;736](https://redirect.github.com/zip-rs/zip2/pull/736))\n- update tests to add big endian miri check\n([#&#8203;735](https://redirect.github.com/zip-rs/zip2/pull/735))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup repository files\n([#&#8203;743](https://redirect.github.com/zip-rs/zip2/pull/743))\n\n###\n[`v8.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#830---2026-03-19)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.2.0...v8.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add must\\_use\n([#&#8203;727](https://redirect.github.com/zip-rs/zip2/pull/727))\n- improve and fix extended timestamp extra field parsing\n([#&#8203;713](https://redirect.github.com/zip-rs/zip2/pull/713))\n- add crc32 ignore option\n([#&#8203;710](https://redirect.github.com/zip-rs/zip2/pull/710))\n- path related code in single file\n([#&#8203;712](https://redirect.github.com/zip-rs/zip2/pull/712))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Malformed ZIP64 file output\n([#&#8203;715](https://redirect.github.com/zip-rs/zip2/pull/715))\n([#&#8203;717](https://redirect.github.com/zip-rs/zip2/pull/717))\n\n##### <!-- 2 -->🚜 Refactor\n\n- refactor some imports\n([#&#8203;734](https://redirect.github.com/zip-rs/zip2/pull/734))\n- move code to distinct file (datetime, FixedSizeBlock)\n([#&#8203;733](https://redirect.github.com/zip-rs/zip2/pull/733))\n- move stream code to `src/read/stream.rs`\n([#&#8203;731](https://redirect.github.com/zip-rs/zip2/pull/731))\n- remove zip64 extra field update\n([#&#8203;732](https://redirect.github.com/zip-rs/zip2/pull/732))\n- improve part of the code with clippy help\n([#&#8203;725](https://redirect.github.com/zip-rs/zip2/pull/725))\n- simplify code for unicode extra field and improve error message\n([#&#8203;724](https://redirect.github.com/zip-rs/zip2/pull/724))\n- reorganize code\n([#&#8203;714](https://redirect.github.com/zip-rs/zip2/pull/714))\n\n##### Deps\n\n- avoid pulling in `zeroize_derive`\n([#&#8203;720](https://redirect.github.com/zip-rs/zip2/pull/720))\n\n###\n[`v8.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#820---2026-03-02)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.1.0...v8.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- allow custom salt\n([#&#8203;680](https://redirect.github.com/zip-rs/zip2/pull/680))\n- Support compressing bzip2 when feature `bzip2-rs` is enabled, since\n`bzip2/bzip2-sys` now supports it\n([#&#8203;685](https://redirect.github.com/zip-rs/zip2/pull/685))\n- enforce clippy in CI\n([#&#8203;674](https://redirect.github.com/zip-rs/zip2/pull/674))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- zip64 central header (issue 617)\n([#&#8203;629](https://redirect.github.com/zip-rs/zip2/pull/629))\n- allow aes password as bytes\n([#&#8203;686](https://redirect.github.com/zip-rs/zip2/pull/686))\n- handle extra field padding\n([#&#8203;682](https://redirect.github.com/zip-rs/zip2/pull/682))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Simplify 2 type conversions in src/write.rs\n([#&#8203;687](https://redirect.github.com/zip-rs/zip2/pull/687))\n\n##### <!-- 4 -->⚡ Performance\n\n- AI tweaks for string type conversions in src/types.rs\n([#&#8203;670](https://redirect.github.com/zip-rs/zip2/pull/670))\n\n###\n[`v8.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#810---2026-02-16)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.0.0...v8.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- *(writer)* Allow getting underlying writer of ZipWriter\n([#&#8203;464](https://redirect.github.com/zip-rs/zip2/pull/464))\n- add system to FileOption, so byte-for-byte identical archives can be\ncreated across platforms\n([#&#8203;660](https://redirect.github.com/zip-rs/zip2/pull/660))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Bugs in extra-data length calculation in src/write.rs\n([#&#8203;662](https://redirect.github.com/zip-rs/zip2/pull/662))\n\n###\n[`v8.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#800---2026-02-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.4.0...v8.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- document zip flags as enum\n([#&#8203;639](https://redirect.github.com/zip-rs/zip2/pull/639))\n- Migrate to Rust 2024\n([#&#8203;650](https://redirect.github.com/zip-rs/zip2/pull/650))\n- \\[**breaking**] Remove deprecated methods of `DateTime`\n([#&#8203;597](https://redirect.github.com/zip-rs/zip2/pull/597))\n\n###\n[`v7.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#740---2026-02-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.3.0...v7.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Increase MSRV to 1.88 and update dependencies\n([#&#8203;626](https://redirect.github.com/zip-rs/zip2/pull/626))\n\n###\n[`v7.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#730---2026-02-04)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.2.0...v7.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- cleanup the benchmarks and Cargo.toml\n([#&#8203;606](https://redirect.github.com/zip-rs/zip2/pull/606))\n- Add support for per-file comments\n([#&#8203;543](https://redirect.github.com/zip-rs/zip2/pull/543))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Document feature `unreserved` and make the mapping of extra fields\npublic ([#&#8203;616](https://redirect.github.com/zip-rs/zip2/pull/616))\n- Return an error if abort\\_file() fails when exceeding non-large-file\nlimit ([#&#8203;598](https://redirect.github.com/zip-rs/zip2/pull/598))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Bump version to 7.3.0 (semver checks fail if it's still 7.3.0-pre1)\n\n###\n[`v7.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#720---2026-01-20)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.1.0...v7.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add read\\_zipfile\\_from\\_stream\\_with\\_compressed\\_size\n([#&#8203;70](https://redirect.github.com/zip-rs/zip2/pull/70))\n- Allow choosing bzip2 rust backend\n([#&#8203;329](https://redirect.github.com/zip-rs/zip2/pull/329))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Need to include zip64 extra field in central directory (fix\n[#&#8203;353](https://redirect.github.com/zip-rs/zip2/issues/353))\n([#&#8203;360](https://redirect.github.com/zip-rs/zip2/pull/360))\n- Fails to extract file which might or might not be malformed\n([#&#8203;376](https://redirect.github.com/zip-rs/zip2/pull/376))\n([#&#8203;426](https://redirect.github.com/zip-rs/zip2/pull/426))\n- *(aes)* Allow AES encryption while streaming\n([#&#8203;463](https://redirect.github.com/zip-rs/zip2/pull/463))\n- Default \"platform\" field in zip files should be set to the local\nplatform, rather than always \"Unix\"\n([#&#8203;470](https://redirect.github.com/zip-rs/zip2/pull/470))\n([#&#8203;471](https://redirect.github.com/zip-rs/zip2/pull/471))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Define cfg\\_if! and cfg\\_if\\_expr! internal macros\n([#&#8203;438](https://redirect.github.com/zip-rs/zip2/pull/438))\n\n##### <!-- 4 -->⚡ Performance\n\n- Change an assert to debug\\_assert when encrypting/decrypting AES, and\neliminate a fallible operation\n([#&#8203;521](https://redirect.github.com/zip-rs/zip2/pull/521))\n- eliminate a String clone per new file added to archive, and other\nrelated refactors\n([#&#8203;522](https://redirect.github.com/zip-rs/zip2/pull/522))\n\n###\n[`v7.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#710---2026-01-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.0.0...v7.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- display the underlying error in Display impl for `ZipError`\n([#&#8203;483](https://redirect.github.com/zip-rs/zip2/pull/483))\n- Enable creation of `ZipArchive` without reparsing\n([#&#8203;485](https://redirect.github.com/zip-rs/zip2/pull/485))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Return InvalidPassword rather than panic when AES key is the wrong\nlength ([#&#8203;457](https://redirect.github.com/zip-rs/zip2/pull/457))\n- bench with auto zip64 comment\n([#&#8203;505](https://redirect.github.com/zip-rs/zip2/pull/505))\n- add condition for `getrandom` dependency\n([#&#8203;504](https://redirect.github.com/zip-rs/zip2/pull/504))\n- *(zipcrypto)* Support streaming ZipCrypto encryption, don't store\nentire file in memory\n([#&#8203;462](https://redirect.github.com/zip-rs/zip2/pull/462))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Clean up imports and move types\n([#&#8203;461](https://redirect.github.com/zip-rs/zip2/pull/461))\n- Replace handwritten `Ord` and `PartialOrd` for `DateTime`\n([#&#8203;484](https://redirect.github.com/zip-rs/zip2/pull/484))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Lock `lzma-rust2` to at least 0.15.5\n([#&#8203;491](https://redirect.github.com/zip-rs/zip2/pull/491))\n\n###\n[`v7.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v6.0.0...v7.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v6.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.1...v6.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v5.1.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#511---2025-09-11)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.0...v5.1.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- panic when reading empty extended-timestamp field\n([#&#8203;404](https://redirect.github.com/zip-rs/zip2/pull/404))\n([#&#8203;422](https://redirect.github.com/zip-rs/zip2/pull/422))\n- Restore original file timestamp when unzipping with `chrono`\n([#&#8203;46](https://redirect.github.com/zip-rs/zip2/pull/46))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Configure Amazon Q rules\n([#&#8203;421](https://redirect.github.com/zip-rs/zip2/pull/421))\n\n###\n[`v5.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#510---2025-09-10)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.1...v5.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Add legacy shrink/reduce/implode compression\n([#&#8203;303](https://redirect.github.com/zip-rs/zip2/pull/303))\n\n###\n[`v5.0.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#501---2025-09-09)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.0...v5.0.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- AES metadata was not copied correctly in raw copy methods, which could\ncorrupt the copied file.\n([#&#8203;417](https://redirect.github.com/zip-rs/zip2/pull/417))\n\n###\n[`v5.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#500---2025-09-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.6.1...v5.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Implement by\\_path\\*() methods on ZipArchive\n([#&#8203;382](https://redirect.github.com/zip-rs/zip2/pull/382))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNDEuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE0MS4zIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-28T03:20:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9eab7837283aaa7f64ea77ae318fbb236d1f3794"
        },
        "date": 1777397518733,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6994400024414,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.199579111039405,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.669629167965105,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.549739583333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.9453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98312.45561648443,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.861011786662,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00257,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 136815.1887028019,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2568976.0163524346,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "9eab7837283aaa7f64ea77ae318fbb236d1f3794",
          "message": "Update Rust crate zip to v8 (#2762)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [zip](https://redirect.github.com/zip-rs/zip2) |\nworkspace.dependencies | major | `=4.6.1` → `=8.6.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>zip-rs/zip2 (zip)</summary>\n\n###\n[`v8.6.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#860---2026-04-25)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.1...v8.6.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add `compression not supported` as enum error\n([#&#8203;774](https://redirect.github.com/zip-rs/zip2/pull/774))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- allow for `[u8]` as filename\n([#&#8203;775](https://redirect.github.com/zip-rs/zip2/pull/775))\n\n##### <!-- 2 -->🚜 Refactor\n\n- mark `ZipFlags` as non-exhaustive and add test for `HasZipMetadata`\n([#&#8203;777](https://redirect.github.com/zip-rs/zip2/pull/777))\n- use and simplify is\\_dir\n([#&#8203;776](https://redirect.github.com/zip-rs/zip2/pull/776))\n\n###\n[`v8.5.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#851---2026-04-06)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.0...v8.5.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- change magic finder to stack buffer\n([#&#8203;763](https://redirect.github.com/zip-rs/zip2/pull/763))\n- simplify extra field parsing\n([#&#8203;764](https://redirect.github.com/zip-rs/zip2/pull/764))\n\n###\n[`v8.5.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#850---2026-04-01)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.4.0...v8.5.0)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- remove `zip64 comment` and add `zip64 extensible data sector`\n([#&#8203;747](https://redirect.github.com/zip-rs/zip2/pull/747))\n\n##### <!-- 2 -->🚜 Refactor\n\n- remove useless magic in struct\n([#&#8203;730](https://redirect.github.com/zip-rs/zip2/pull/730))\n- change extra\\_field from Arc\\<Vec<u8>> to Arc<\\[u8]>\n([#&#8203;741](https://redirect.github.com/zip-rs/zip2/pull/741))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup README\n([#&#8203;758](https://redirect.github.com/zip-rs/zip2/pull/758))\n\n###\n[`v8.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#840---2026-03-23)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.1...v8.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add a check for building benches\n([#&#8203;748](https://redirect.github.com/zip-rs/zip2/pull/748))\n\n##### <!-- 2 -->🚜 Refactor\n\n- split part of `read.rs` for code readability\n([#&#8203;744](https://redirect.github.com/zip-rs/zip2/pull/744))\n- remove unused allow\n([#&#8203;745](https://redirect.github.com/zip-rs/zip2/pull/745))\n\n##### <!-- 4 -->⚡ Performance\n\n- skip BufReader for Stored files in make\\_reader\n([#&#8203;739](https://redirect.github.com/zip-rs/zip2/pull/739))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- move pull request template to correct folder\n([#&#8203;749](https://redirect.github.com/zip-rs/zip2/pull/749))\n\n###\n[`v8.3.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#831---2026-03-21)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.0...v8.3.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- use `AexEncryption::new`\n([#&#8203;736](https://redirect.github.com/zip-rs/zip2/pull/736))\n- update tests to add big endian miri check\n([#&#8203;735](https://redirect.github.com/zip-rs/zip2/pull/735))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup repository files\n([#&#8203;743](https://redirect.github.com/zip-rs/zip2/pull/743))\n\n###\n[`v8.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#830---2026-03-19)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.2.0...v8.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add must\\_use\n([#&#8203;727](https://redirect.github.com/zip-rs/zip2/pull/727))\n- improve and fix extended timestamp extra field parsing\n([#&#8203;713](https://redirect.github.com/zip-rs/zip2/pull/713))\n- add crc32 ignore option\n([#&#8203;710](https://redirect.github.com/zip-rs/zip2/pull/710))\n- path related code in single file\n([#&#8203;712](https://redirect.github.com/zip-rs/zip2/pull/712))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Malformed ZIP64 file output\n([#&#8203;715](https://redirect.github.com/zip-rs/zip2/pull/715))\n([#&#8203;717](https://redirect.github.com/zip-rs/zip2/pull/717))\n\n##### <!-- 2 -->🚜 Refactor\n\n- refactor some imports\n([#&#8203;734](https://redirect.github.com/zip-rs/zip2/pull/734))\n- move code to distinct file (datetime, FixedSizeBlock)\n([#&#8203;733](https://redirect.github.com/zip-rs/zip2/pull/733))\n- move stream code to `src/read/stream.rs`\n([#&#8203;731](https://redirect.github.com/zip-rs/zip2/pull/731))\n- remove zip64 extra field update\n([#&#8203;732](https://redirect.github.com/zip-rs/zip2/pull/732))\n- improve part of the code with clippy help\n([#&#8203;725](https://redirect.github.com/zip-rs/zip2/pull/725))\n- simplify code for unicode extra field and improve error message\n([#&#8203;724](https://redirect.github.com/zip-rs/zip2/pull/724))\n- reorganize code\n([#&#8203;714](https://redirect.github.com/zip-rs/zip2/pull/714))\n\n##### Deps\n\n- avoid pulling in `zeroize_derive`\n([#&#8203;720](https://redirect.github.com/zip-rs/zip2/pull/720))\n\n###\n[`v8.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#820---2026-03-02)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.1.0...v8.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- allow custom salt\n([#&#8203;680](https://redirect.github.com/zip-rs/zip2/pull/680))\n- Support compressing bzip2 when feature `bzip2-rs` is enabled, since\n`bzip2/bzip2-sys` now supports it\n([#&#8203;685](https://redirect.github.com/zip-rs/zip2/pull/685))\n- enforce clippy in CI\n([#&#8203;674](https://redirect.github.com/zip-rs/zip2/pull/674))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- zip64 central header (issue 617)\n([#&#8203;629](https://redirect.github.com/zip-rs/zip2/pull/629))\n- allow aes password as bytes\n([#&#8203;686](https://redirect.github.com/zip-rs/zip2/pull/686))\n- handle extra field padding\n([#&#8203;682](https://redirect.github.com/zip-rs/zip2/pull/682))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Simplify 2 type conversions in src/write.rs\n([#&#8203;687](https://redirect.github.com/zip-rs/zip2/pull/687))\n\n##### <!-- 4 -->⚡ Performance\n\n- AI tweaks for string type conversions in src/types.rs\n([#&#8203;670](https://redirect.github.com/zip-rs/zip2/pull/670))\n\n###\n[`v8.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#810---2026-02-16)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.0.0...v8.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- *(writer)* Allow getting underlying writer of ZipWriter\n([#&#8203;464](https://redirect.github.com/zip-rs/zip2/pull/464))\n- add system to FileOption, so byte-for-byte identical archives can be\ncreated across platforms\n([#&#8203;660](https://redirect.github.com/zip-rs/zip2/pull/660))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Bugs in extra-data length calculation in src/write.rs\n([#&#8203;662](https://redirect.github.com/zip-rs/zip2/pull/662))\n\n###\n[`v8.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#800---2026-02-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.4.0...v8.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- document zip flags as enum\n([#&#8203;639](https://redirect.github.com/zip-rs/zip2/pull/639))\n- Migrate to Rust 2024\n([#&#8203;650](https://redirect.github.com/zip-rs/zip2/pull/650))\n- \\[**breaking**] Remove deprecated methods of `DateTime`\n([#&#8203;597](https://redirect.github.com/zip-rs/zip2/pull/597))\n\n###\n[`v7.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#740---2026-02-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.3.0...v7.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Increase MSRV to 1.88 and update dependencies\n([#&#8203;626](https://redirect.github.com/zip-rs/zip2/pull/626))\n\n###\n[`v7.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#730---2026-02-04)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.2.0...v7.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- cleanup the benchmarks and Cargo.toml\n([#&#8203;606](https://redirect.github.com/zip-rs/zip2/pull/606))\n- Add support for per-file comments\n([#&#8203;543](https://redirect.github.com/zip-rs/zip2/pull/543))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Document feature `unreserved` and make the mapping of extra fields\npublic ([#&#8203;616](https://redirect.github.com/zip-rs/zip2/pull/616))\n- Return an error if abort\\_file() fails when exceeding non-large-file\nlimit ([#&#8203;598](https://redirect.github.com/zip-rs/zip2/pull/598))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Bump version to 7.3.0 (semver checks fail if it's still 7.3.0-pre1)\n\n###\n[`v7.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#720---2026-01-20)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.1.0...v7.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add read\\_zipfile\\_from\\_stream\\_with\\_compressed\\_size\n([#&#8203;70](https://redirect.github.com/zip-rs/zip2/pull/70))\n- Allow choosing bzip2 rust backend\n([#&#8203;329](https://redirect.github.com/zip-rs/zip2/pull/329))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Need to include zip64 extra field in central directory (fix\n[#&#8203;353](https://redirect.github.com/zip-rs/zip2/issues/353))\n([#&#8203;360](https://redirect.github.com/zip-rs/zip2/pull/360))\n- Fails to extract file which might or might not be malformed\n([#&#8203;376](https://redirect.github.com/zip-rs/zip2/pull/376))\n([#&#8203;426](https://redirect.github.com/zip-rs/zip2/pull/426))\n- *(aes)* Allow AES encryption while streaming\n([#&#8203;463](https://redirect.github.com/zip-rs/zip2/pull/463))\n- Default \"platform\" field in zip files should be set to the local\nplatform, rather than always \"Unix\"\n([#&#8203;470](https://redirect.github.com/zip-rs/zip2/pull/470))\n([#&#8203;471](https://redirect.github.com/zip-rs/zip2/pull/471))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Define cfg\\_if! and cfg\\_if\\_expr! internal macros\n([#&#8203;438](https://redirect.github.com/zip-rs/zip2/pull/438))\n\n##### <!-- 4 -->⚡ Performance\n\n- Change an assert to debug\\_assert when encrypting/decrypting AES, and\neliminate a fallible operation\n([#&#8203;521](https://redirect.github.com/zip-rs/zip2/pull/521))\n- eliminate a String clone per new file added to archive, and other\nrelated refactors\n([#&#8203;522](https://redirect.github.com/zip-rs/zip2/pull/522))\n\n###\n[`v7.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#710---2026-01-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.0.0...v7.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- display the underlying error in Display impl for `ZipError`\n([#&#8203;483](https://redirect.github.com/zip-rs/zip2/pull/483))\n- Enable creation of `ZipArchive` without reparsing\n([#&#8203;485](https://redirect.github.com/zip-rs/zip2/pull/485))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Return InvalidPassword rather than panic when AES key is the wrong\nlength ([#&#8203;457](https://redirect.github.com/zip-rs/zip2/pull/457))\n- bench with auto zip64 comment\n([#&#8203;505](https://redirect.github.com/zip-rs/zip2/pull/505))\n- add condition for `getrandom` dependency\n([#&#8203;504](https://redirect.github.com/zip-rs/zip2/pull/504))\n- *(zipcrypto)* Support streaming ZipCrypto encryption, don't store\nentire file in memory\n([#&#8203;462](https://redirect.github.com/zip-rs/zip2/pull/462))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Clean up imports and move types\n([#&#8203;461](https://redirect.github.com/zip-rs/zip2/pull/461))\n- Replace handwritten `Ord` and `PartialOrd` for `DateTime`\n([#&#8203;484](https://redirect.github.com/zip-rs/zip2/pull/484))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Lock `lzma-rust2` to at least 0.15.5\n([#&#8203;491](https://redirect.github.com/zip-rs/zip2/pull/491))\n\n###\n[`v7.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v6.0.0...v7.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v6.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.1...v6.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v5.1.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#511---2025-09-11)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.0...v5.1.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- panic when reading empty extended-timestamp field\n([#&#8203;404](https://redirect.github.com/zip-rs/zip2/pull/404))\n([#&#8203;422](https://redirect.github.com/zip-rs/zip2/pull/422))\n- Restore original file timestamp when unzipping with `chrono`\n([#&#8203;46](https://redirect.github.com/zip-rs/zip2/pull/46))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Configure Amazon Q rules\n([#&#8203;421](https://redirect.github.com/zip-rs/zip2/pull/421))\n\n###\n[`v5.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#510---2025-09-10)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.1...v5.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Add legacy shrink/reduce/implode compression\n([#&#8203;303](https://redirect.github.com/zip-rs/zip2/pull/303))\n\n###\n[`v5.0.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#501---2025-09-09)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.0...v5.0.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- AES metadata was not copied correctly in raw copy methods, which could\ncorrupt the copied file.\n([#&#8203;417](https://redirect.github.com/zip-rs/zip2/pull/417))\n\n###\n[`v5.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#500---2025-09-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.6.1...v5.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Implement by\\_path\\*() methods on ZipArchive\n([#&#8203;382](https://redirect.github.com/zip-rs/zip2/pull/382))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNDEuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE0MS4zIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-28T03:20:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9eab7837283aaa7f64ea77ae318fbb236d1f3794"
        },
        "date": 1777397524237,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.75499725341797,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.68690570633326,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.69288303640589,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.448307291666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.5390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.10348516752,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8735580936864,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002338,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84360.06374670964,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2580310.08498118,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "0ddaeb436473414388587dd60b58bfddcdf7e226",
          "message": "Add dfctl CLI and TUI for OTAP Dataflow Engine administration (#2756)\n\n# Change Summary\n\nAdds `dfctl`, an admin SDK-based command-line tool for operating local\nand remote OTAP Dataflow Engines.\n\nThe CLI supports engine, group, pipeline, telemetry, rollout, shutdown,\nreconfiguration, diagnosis, bundle, watch, shell completion, automation\nfriendly output, and an interactive TUI for operational workflows.\n\nMore details can be find here ->\nhttps://github.com/lquerel/otel-arrow/blob/962a01e30116433e448ed58b6d8b820e1bcdcd3a/rust/otap-dataflow/crates/enginectl/README.md\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nYes, the new CLI by itself.\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6ee53571-b1c5-47b2-bdcd-6d06999ea7d5\"\n/>\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/21e6273d-9fc5-423d-b4d8-1472d9f40059\"\n/>",
          "timestamp": "2026-04-29T00:24:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ddaeb436473414388587dd60b58bfddcdf7e226"
        },
        "date": 1777426672125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.31678569302939,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.77430979815946,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.241276041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.61328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.46686839702,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7734066571015,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00419,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 136995.0025364798,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2566202.840837621,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "0ddaeb436473414388587dd60b58bfddcdf7e226",
          "message": "Add dfctl CLI and TUI for OTAP Dataflow Engine administration (#2756)\n\n# Change Summary\n\nAdds `dfctl`, an admin SDK-based command-line tool for operating local\nand remote OTAP Dataflow Engines.\n\nThe CLI supports engine, group, pipeline, telemetry, rollout, shutdown,\nreconfiguration, diagnosis, bundle, watch, shell completion, automation\nfriendly output, and an interactive TUI for operational workflows.\n\nMore details can be find here ->\nhttps://github.com/lquerel/otel-arrow/blob/962a01e30116433e448ed58b6d8b820e1bcdcd3a/rust/otap-dataflow/crates/enginectl/README.md\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nYes, the new CLI by itself.\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6ee53571-b1c5-47b2-bdcd-6d06999ea7d5\"\n/>\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/21e6273d-9fc5-423d-b4d8-1472d9f40059\"\n/>",
          "timestamp": "2026-04-29T00:24:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ddaeb436473414388587dd60b58bfddcdf7e226"
        },
        "date": 1777426677019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.99605371438714,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.80652998065764,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.253125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.28125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.45587512333,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8720438790697,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002366,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84445.46697913231,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2579364.9006781527,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "dccbcb2de29f5c993d5dec24a558e8db3ad74322",
          "message": "[query-engine] Improve value equality logic (#2775)\n\n# Changes\n\n* Improve the value equality helper logic in expressions\n\n# Details\n\nThe way things were coded, left-side of equality comparison determined\nthe type. So...\n\n* `int(18) == string('value')` results in an error that string on\nright-side could not be converted to int (evaluates as `int(18) ==\ntoint(string('value'))`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)\n\nThis PR adjusts the logic so that if either side of a comparison is a\nstring we will apply string logic. So...\n\n* `int(18) == string('value')` results in `false` (evaluates as\n`tostring(int(18)) == string('value')`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)",
          "timestamp": "2026-04-29T16:33:32Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dccbcb2de29f5c993d5dec24a558e8db3ad74322"
        },
        "date": 1777482972868,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.34865463131153,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.62996758759068,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.026041666666668,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.70703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.45375911865,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7729740509153,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004198,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 137419.78839663067,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2568568.7346307808,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "dccbcb2de29f5c993d5dec24a558e8db3ad74322",
          "message": "[query-engine] Improve value equality logic (#2775)\n\n# Changes\n\n* Improve the value equality helper logic in expressions\n\n# Details\n\nThe way things were coded, left-side of equality comparison determined\nthe type. So...\n\n* `int(18) == string('value')` results in an error that string on\nright-side could not be converted to int (evaluates as `int(18) ==\ntoint(string('value'))`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)\n\nThis PR adjusts the logic so that if either side of a comparison is a\nstring we will apply string logic. So...\n\n* `int(18) == string('value')` results in `false` (evaluates as\n`tostring(int(18)) == string('value')`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)",
          "timestamp": "2026-04-29T16:33:32Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dccbcb2de29f5c993d5dec24a558e8db3ad74322"
        },
        "date": 1777482978186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.09994078992855,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.4196458527493,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.579036458333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.70703125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.17728677335,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8628504635203,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002536,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84127.99236023347,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2570605.5284652268,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Max Jacinto",
            "username": "luckymachi",
            "email": "77021922+luckymachi@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "775794600fb4ba7bea406b4e0bb08cd598b10cda",
          "message": "Isolation of setup-protoc jobs from RUST-CI (#2772)\n\n# Change Summary\n\nAs mentioned in issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768),\n`setup-protoc` jobs are dropped in favour of a targeted `compile_proto`\njob.\n\n## What issue does this PR close?\n\nThis issue closes issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768)\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-29T20:45:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/775794600fb4ba7bea406b4e0bb08cd598b10cda"
        },
        "date": 1777520206331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.57272516699801,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.98924411400247,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.38125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.89453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.61519861405,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.844301554264,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002879,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 336184.5216730724,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2563613.7835289855,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.7269515991211,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.53251167532193,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.8997460514091,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.460286458333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.09375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.12181707553,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.9230651337907,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002327,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 128764.9135132846,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 773504.4458819765,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.78250122070312,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.62080464503687,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.01051018038244,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.605859375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.6015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.70685098587,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3217.3618679304705,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002576,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 327043.65190928354,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 766364.6437793823,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 62.816370634847395,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.25437374206534,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.431119791666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.7421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.92693615063,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.821588892971,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003299,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 136767.94772756574,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2560168.56647111,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Max Jacinto",
            "username": "luckymachi",
            "email": "77021922+luckymachi@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "775794600fb4ba7bea406b4e0bb08cd598b10cda",
          "message": "Isolation of setup-protoc jobs from RUST-CI (#2772)\n\n# Change Summary\n\nAs mentioned in issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768),\n`setup-protoc` jobs are dropped in favour of a targeted `compile_proto`\njob.\n\n## What issue does this PR close?\n\nThis issue closes issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768)\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-29T20:45:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/775794600fb4ba7bea406b4e0bb08cd598b10cda"
        },
        "date": 1777520211151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.72293090820312,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.5442604092305,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.42508894044856,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 337.2220052083333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 513.86328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.42309994138,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3222.3218590007405,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002386,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59354.04041147149,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2552095.3698154986,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.72640228271484,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.90137887284403,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.33070836881629,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1665.6348958333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2917.5546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.3051365252,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3273.479044949289,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002217,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 55973.92210247091,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 761362.4820172135,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.7005615234375,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.78157319860078,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.51138613861386,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.948307291666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.91015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.46734644216,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.322444055908,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002359,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 83986.64034766838,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2571426.2988357986,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "524b78de5762aaebe99e31d67d3a2351d4d9e303",
          "message": "Make pipeline perf test a required CI check (#2779)\n\n## Summary\n\nMake the pipeline performance test a required CI check so that PRs which\nbreak the perf test are caught before merge.\n\n> **Dependency**: #2780 must be merged first (it fixes the currently\nbroken perf test).\n\n#2774 is an example of the kind of breakage this prevents — a route\nrename broke the perf test but the PR still merged because the perf test\nwas not a required check.\n\n### Changes\n\n- **rust-ci.yml**: Add `pipeline_perf_test` job (runs on\n`ubuntu-latest`) and include it in `rust-required-status-check`\naggregator\n- **pipeline-perf-on-label.yaml**: Simplify to only run on dedicated\nOracle bare-metal hardware when `pipelineperf` label is present — the\nbasic validation path is removed since `rust-ci.yml` now covers it\n\n### Motivation\n\nThe pipeline perf test has been broken by merged PRs several times\nbecause it was not a required check. This change ensures that if a PR\nbreaks the perf test (e.g. build failures, config issues, test\ninfrastructure breakage), it is caught before merge.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-30T12:25:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/524b78de5762aaebe99e31d67d3a2351d4d9e303"
        },
        "date": 1777571650894,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.72750091552734,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.607360256315204,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.92371632084236,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.768489583333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.3567994006,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.3807762603847,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002186,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 327669.699951466,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 767923.3407463747,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 62.96037019626165,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.357913367489516,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.711979166666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.0625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.47590662354,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.839704918577,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002964,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 137123.40724494684,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2576602.6966382572,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.49547200480895,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.71694665944105,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.753645833333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.40234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.16906679516,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7305792042403,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004982,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 336684.8191183491,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2572174.646457114,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.72750091552734,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.53787961575102,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.95990093646002,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.645963541666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.83683999289,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.3637605887675,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002498,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 128467.301730972,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 771239.6105701871,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "524b78de5762aaebe99e31d67d3a2351d4d9e303",
          "message": "Make pipeline perf test a required CI check (#2779)\n\n## Summary\n\nMake the pipeline performance test a required CI check so that PRs which\nbreak the perf test are caught before merge.\n\n> **Dependency**: #2780 must be merged first (it fixes the currently\nbroken perf test).\n\n#2774 is an example of the kind of breakage this prevents — a route\nrename broke the perf test but the PR still merged because the perf test\nwas not a required check.\n\n### Changes\n\n- **rust-ci.yml**: Add `pipeline_perf_test` job (runs on\n`ubuntu-latest`) and include it in `rust-required-status-check`\naggregator\n- **pipeline-perf-on-label.yaml**: Simplify to only run on dedicated\nOracle bare-metal hardware when `pipelineperf` label is present — the\nbasic validation path is removed since `rust-ci.yml` now covers it\n\n### Motivation\n\nThe pipeline perf test has been broken by merged PRs several times\nbecause it was not a required check. This change ensures that if a PR\nbreaks the perf test (e.g. build failures, config issues, test\ninfrastructure breakage), it is caught before merge.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-30T12:25:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/524b78de5762aaebe99e31d67d3a2351d4d9e303"
        },
        "date": 1777571656444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.6994400024414,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.12799322362675,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.8511240040226,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.413411458333332,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.1015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98312.79969654526,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8723683535113,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00236,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84367.90414941807,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2571641.9471574347,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.72640228271484,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.76234001064017,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.92201935733642,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1683.880078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2690.75390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.06182176525,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3273.471079797307,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002363,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 55995.79375833453,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 760827.3301044332,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.7055892944336,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.62270597080995,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.63925245288895,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 360.239453125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 532.03515625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.19280014296,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3239.297705858269,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003747,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 60211.66162658354,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2549863.117549355,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "993fa369f3ef1c12b381f01f758302e8243f211a",
          "message": "feat(engine): Extension System - Capability Registry & Resolver (#2732)\n\n# Extension system — Phase 1 (capabilities, registry, builder, proc\nmacro)\n\nImplements the Phase-1 extension/capability system for the OTAP dataflow\nengine. Extensions are first-class config siblings of nodes; nodes\nexplicitly bind to extension instances via named capabilities, and\nreceive typed handles resolved once at factory time — no hot-path\nregistry lookups.\n\nTracking docs:\n\n[`docs/extension-system-architecture.md`](rust/otap-dataflow/docs/extension-system-architecture.md)\n(rewritten in this PR).\n\n## What's in this PR\n\n### `#[capability]` proc macro (`engine-macros`)\n\n- New `capability.rs` expansion: from a single `#[capability] trait\n  Foo { ... }` source it generates `local::Foo` (`!Send`-friendly) and\n  `shared::Foo` (`Send + Clone`) trait variants plus a `SharedAsLocal`\n  adapter and an `ExtensionCapability` impl. The dual variants are\n  derived from one source, so authors can't accidentally let local\n  and shared semantics diverge.\n- New `pipeline_factory.rs` expansion to build the static\n  `PipelineFactory` registry used by `main.rs`.\n- All emitted paths use fully-qualified `::std::...` /\n  `::async_trait::...` / `::otap_df_engine::...` so generated code is\n  hygienic in any caller crate.\n\n### Capability registry (`engine::capability`)\n\n- `CapabilityRegistry`: typed-keyed (`(extension_name, TypeId)`)\n  storage with **typestate-enforced** single `.shared()` / `.local()`\n  registration per builder — duplicates are unrepresentable rather\n  than runtime errors.\n- Two execution models: native local (`Rc<dyn Local>`, lock-free) and\n  native shared (`Box<dyn Shared>`, `Send + Clone`). A shared-only\n  extension serves local consumers transparently via the\n  `SharedAsLocal` adapter generated by the proc macro.\n- Two **instance policies** chosen at build time, invisible to\n  consumers: `.cloned()` (clone a stored prototype) and\n  `.constructed()` (per-consumer construction via a closure;\n  Passive-only — `Active + Constructed` is statically rejected).\n- `resolve_bindings`: walks a node's `capabilities:` declaration and\n  produces a per-node `Capabilities` bundle with all bindings\n  resolved, surfacing config errors (unknown extension, unknown\n  capability, capability not provided by bound extension, multiple\n  bindings for the same capability).\n- `Capabilities`: per-node consumer API with `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`. Instances\n  are minted lazily at the call site, not at resolution time.\n- `ConsumedTracker`: cross-node, per-(capability, extension)\n  consumption flags driving `drop_local()` / `drop_shared()` cleanup\n  for extensions no node ever claimed.\n\n### One-shot consumption contract\n\nA binding is claimable **at most once per node**, regardless of\nexecution model. The guard is the `Cell<Option<_>>::take()` on each\nresolved entry's `produce` closure — no auxiliary flag.\n\n- Same accessor twice → `CapabilityAlreadyConsumed`.\n- Different accessors on a SharedAsLocal-fallback binding share one\n  underlying entry, so claiming either side consumes the other\n  naturally.\n- Different accessors on a native-dual binding (extension registered\n  both native local **and** native shared) take and drop the\n  alternative entry's `produce` closure on success, so the\n  per-binding contract holds uniformly. The cross-node tracker is\n  only flipped by actual consumption, not by invalidation, so\n  `drop_*` cleanup remains correct.\n\n### Documentation\n\n- `docs/extension-system-architecture.md`: rewritten to describe the\n  capability-based design, the local/shared duality, instance\n  policies, Active vs Passive lifecycle, and the typestate builder.\n\n## Tests\n\nNew, focused unit tests cover:\n\n- Registry: typestate single-registration, duplicate rejection,\n  `SharedAsLocal` adapter freshness per node, double-`Box` envelope\n  for shared `produce`.\n- `resolve_bindings`: every error path (unknown extension / unknown\n  capability / capability not provided / wrong extension), local-only\n  and shared-only binding shapes, fallback path, native-dual path.\n- One-shot contract: second-call rejection on each of `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`; fallback\n  cross-side rejection; native-dual cross-side rejection (both\n  directions).\n- `ConsumedTracker`: per-extension consumption flags, with the\n  invariant that mere invalidation does not flip a bucket.\n- Proc-macro end-to-end: `local-only`, `shared-only`, and `dual`\n  forms of `extension_capabilities!` against the registry.\n\n## Validation\n\n```text\ncargo xtask check\n✅ Cargo workspace structure complies with project policies.\n✅ Formatting completed successfully.\n✅ Clippy linting passed without warnings.\n✅ All tests passed successfully.\n```\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-01T02:10:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993fa369f3ef1c12b381f01f758302e8243f211a"
        },
        "date": 1777605308320,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 62.99417486243652,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.55374504777441,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.535807291666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.32421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.08425243566,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8267803303766,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003203,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 137104.97397628898,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2570166.49537121,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.5257939551019,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.90374639769453,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.364583333333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.62109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.36322359627,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.8029863786774,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003643,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 337309.7822170996,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2563423.7810039264,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67203521728516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.54557681925427,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.79009989932626,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.016015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.88671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.73118751416,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.3801218082044,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002198,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 128655.62707736962,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772182.1047498123,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67203521728516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.546157966686195,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.85279381044488,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.407942708333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.22265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.15799461755,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.327766481891,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003158,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 327265.87882389035,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 767078.4414006424,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "993fa369f3ef1c12b381f01f758302e8243f211a",
          "message": "feat(engine): Extension System - Capability Registry & Resolver (#2732)\n\n# Extension system — Phase 1 (capabilities, registry, builder, proc\nmacro)\n\nImplements the Phase-1 extension/capability system for the OTAP dataflow\nengine. Extensions are first-class config siblings of nodes; nodes\nexplicitly bind to extension instances via named capabilities, and\nreceive typed handles resolved once at factory time — no hot-path\nregistry lookups.\n\nTracking docs:\n\n[`docs/extension-system-architecture.md`](rust/otap-dataflow/docs/extension-system-architecture.md)\n(rewritten in this PR).\n\n## What's in this PR\n\n### `#[capability]` proc macro (`engine-macros`)\n\n- New `capability.rs` expansion: from a single `#[capability] trait\n  Foo { ... }` source it generates `local::Foo` (`!Send`-friendly) and\n  `shared::Foo` (`Send + Clone`) trait variants plus a `SharedAsLocal`\n  adapter and an `ExtensionCapability` impl. The dual variants are\n  derived from one source, so authors can't accidentally let local\n  and shared semantics diverge.\n- New `pipeline_factory.rs` expansion to build the static\n  `PipelineFactory` registry used by `main.rs`.\n- All emitted paths use fully-qualified `::std::...` /\n  `::async_trait::...` / `::otap_df_engine::...` so generated code is\n  hygienic in any caller crate.\n\n### Capability registry (`engine::capability`)\n\n- `CapabilityRegistry`: typed-keyed (`(extension_name, TypeId)`)\n  storage with **typestate-enforced** single `.shared()` / `.local()`\n  registration per builder — duplicates are unrepresentable rather\n  than runtime errors.\n- Two execution models: native local (`Rc<dyn Local>`, lock-free) and\n  native shared (`Box<dyn Shared>`, `Send + Clone`). A shared-only\n  extension serves local consumers transparently via the\n  `SharedAsLocal` adapter generated by the proc macro.\n- Two **instance policies** chosen at build time, invisible to\n  consumers: `.cloned()` (clone a stored prototype) and\n  `.constructed()` (per-consumer construction via a closure;\n  Passive-only — `Active + Constructed` is statically rejected).\n- `resolve_bindings`: walks a node's `capabilities:` declaration and\n  produces a per-node `Capabilities` bundle with all bindings\n  resolved, surfacing config errors (unknown extension, unknown\n  capability, capability not provided by bound extension, multiple\n  bindings for the same capability).\n- `Capabilities`: per-node consumer API with `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`. Instances\n  are minted lazily at the call site, not at resolution time.\n- `ConsumedTracker`: cross-node, per-(capability, extension)\n  consumption flags driving `drop_local()` / `drop_shared()` cleanup\n  for extensions no node ever claimed.\n\n### One-shot consumption contract\n\nA binding is claimable **at most once per node**, regardless of\nexecution model. The guard is the `Cell<Option<_>>::take()` on each\nresolved entry's `produce` closure — no auxiliary flag.\n\n- Same accessor twice → `CapabilityAlreadyConsumed`.\n- Different accessors on a SharedAsLocal-fallback binding share one\n  underlying entry, so claiming either side consumes the other\n  naturally.\n- Different accessors on a native-dual binding (extension registered\n  both native local **and** native shared) take and drop the\n  alternative entry's `produce` closure on success, so the\n  per-binding contract holds uniformly. The cross-node tracker is\n  only flipped by actual consumption, not by invalidation, so\n  `drop_*` cleanup remains correct.\n\n### Documentation\n\n- `docs/extension-system-architecture.md`: rewritten to describe the\n  capability-based design, the local/shared duality, instance\n  policies, Active vs Passive lifecycle, and the typestate builder.\n\n## Tests\n\nNew, focused unit tests cover:\n\n- Registry: typestate single-registration, duplicate rejection,\n  `SharedAsLocal` adapter freshness per node, double-`Box` envelope\n  for shared `produce`.\n- `resolve_bindings`: every error path (unknown extension / unknown\n  capability / capability not provided / wrong extension), local-only\n  and shared-only binding shapes, fallback path, native-dual path.\n- One-shot contract: second-call rejection on each of `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`; fallback\n  cross-side rejection; native-dual cross-side rejection (both\n  directions).\n- `ConsumedTracker`: per-extension consumption flags, with the\n  invariant that mere invalidation does not flip a bucket.\n- Proc-macro end-to-end: `local-only`, `shared-only`, and `dual`\n  forms of `extension_capabilities!` against the registry.\n\n## Validation\n\n```text\ncargo xtask check\n✅ Cargo workspace structure complies with project policies.\n✅ Formatting completed successfully.\n✅ Clippy linting passed without warnings.\n✅ All tests passed successfully.\n```\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-01T02:10:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993fa369f3ef1c12b381f01f758302e8243f211a"
        },
        "date": 1777605314470,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.7055892944336,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.93480108168953,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.93683038131333,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1717.8971354166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 2976.97265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.43860338906,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3239.3058036170733,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003597,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 55376.23741579022,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 760569.4819055035,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.7055892944336,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.06817687000691,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.26645934865012,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 354.76380208333336,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 506.3359375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.42057777592,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3239.305209780052,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003608,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59621.52096353413,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2551152.468226966,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.6994400024414,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.7826230174732,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.93657612913239,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.952864583333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.54296875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.23628182159,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3245.414774384401,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0025,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84345.29062517462,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2572900.3567682104,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ce3582596886edf41cb83c87829be8cd8d15fcce",
          "message": "fix(perf-test): add missing /api/v1 prefix to idle-state template endpoints (#2798)\n\nThe idle-state-template.yaml.j2 was using /telemetry/metrics instead of\n/api/v1/telemetry/metrics for both the Prometheus scraping endpoint and\nthe ready-check URL. This caused 404 errors during idle state\nbenchmarks.\n\nAll other test configs already had the correct /api/v1 prefix.\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-01T16:38:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce3582596886edf41cb83c87829be8cd8d15fcce"
        },
        "date": 1777658970305,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 62.92578930083411,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.4557441192454,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.160286458333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.53125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.11007662726,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7286325286996,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005018,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 143166.32128609254,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2559187.863873354,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.67203521728516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.476694531796895,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.96088923195677,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.998567708333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.53515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.79873689826,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.349090167792,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002767,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 128801.81720211706,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 773712.1466351065,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.53484365424944,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.87007895959127,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.833203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.10518181052,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.860470999747,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00258,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 336418.3830387304,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2561243.219120109,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.72750091552734,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.55697934621977,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.989707893101304,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.581640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.39453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.42354278252,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.3502354375582,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002746,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 327117.98631393973,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 767345.9280542071,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ce3582596886edf41cb83c87829be8cd8d15fcce",
          "message": "fix(perf-test): add missing /api/v1 prefix to idle-state template endpoints (#2798)\n\nThe idle-state-template.yaml.j2 was using /telemetry/metrics instead of\n/api/v1/telemetry/metrics for both the Prometheus scraping endpoint and\nthe ready-check URL. This caused 404 errors during idle state\nbenchmarks.\n\nAll other test configs already had the correct /api/v1 prefix.\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-01T16:38:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce3582596886edf41cb83c87829be8cd8d15fcce"
        },
        "date": 1777658976727,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.7055892944336,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 69.8727087903406,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.87073710456599,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 354.27161458333336,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 508.16796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.24775308919,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3239.3654043674483,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002493,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59885.62060299592,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2558642.4116949537,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69888305664062,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.07858780968374,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.64234164472666,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.502083333333335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.72265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.4607914025,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3245.9721627963218,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002363,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 84287.54877637258,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2578929.489722759,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.72640228271484,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.19726241416832,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.0614827426095,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1712.762890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 3353.8203125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.19181169517,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3273.475335147653,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002285,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 55451.87496656957,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 761267.0013960463,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Gyan ranjan",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f018901f3a2ba93bee9d45f6afb1204f90679863",
          "message": "Fix duplicate attribute keys in transform_attributes (#2423)\n\n# Fix Duplicate Attribute Keys in `transform_attributes`\n\n## Changes Made\nThis PR resolves issue #1650 by ensuring that dictionary keys are\ndeduplicated when transformations such as `rename` are applied, as\nrequired by the OpenTelemetry specification (\"Exported maps MUST contain\nonly unique keys by default\").\n\nTo accomplish this while maintaining strict performance requirements, we\nreplaced the previous `RowConverter` deduplication strategy with a new\nhigh-performance, proactive pre-filter:\n- We injected `filter_rename_collisions` into\n`transform_attributes_impl` inside\n`otap-dataflow/crates/pdata/src/otap/transform.rs`.\n- Before a rename is processed, this function reads the `parent_id`s and\ntarget keys. It uses the `IdBitmap` type to find any existing target\nkeys whose `parent_id` maps back to an old key that will be renamed.\n- It proactively strips those collision rows from the batch via\n`arrow::compute::filter_record_batch` *before* the actual transform\nhappens.\n\n## Testing\n- Extended the `AttributesProcessor` unit tests\n(`test_rename_removes_duplicate_keys`) to explicitly verify that\nrenaming an attribute resulting in a collision automatically discards\nduplicate keys.\n- Extended the `AttributesTransformPipelineStage` in `query-engine`\ntests with a parallel case ensuring OPL/KQL query pipelines\n(`project-rename`) properly drop duplicates when resolving duplicates.\n- Refactored `otap_df_pdata` `transform.rs` tests to properly expect\ndeduplicated keys using this plan-based method.\n- Validated logic with `cargo test --workspace --all-features`.\n\n## Validation Results\nAll tests pass. OTel semantic rules surrounding unique mapped keys map\ncleanly through down/upstream processors. The `IdBitmap` intersection\napproach completely resolves the multi-thousand percent `RowConverter`\nperformance regressions, dropping collision resolution overhead to\nessentially zero through efficient bitmap operations.\n\n---------\n\nSigned-off-by: Gyanranjan Panda <gyanranjanpanda438@gmail.com>\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-01T20:08:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f018901f3a2ba93bee9d45f6afb1204f90679863"
        },
        "date": 1777690087217,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.6036373490155,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.02486419562021,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.270182291666668,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.83984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.67219187206,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.879182331778,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002234,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 327623.7003906826,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 767685.7002696571,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.66462127827643,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23427864543065,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.947135416666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.58812938766,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.7774082697924,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004116,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 336310.84537434083,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2564310.2474429957,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.69999694824219,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 62.87369671818711,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.44811269359483,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.873958333333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.42578125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.97243317743,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3244.724090294855,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005102,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 136896.54832200715,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2568756.369518118,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 96.7269515991211,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.41274464100203,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.86151568884435,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.871354166666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.44140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.11515092639,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 3272.922846947396,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002331,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 128808.76455774998,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772648.2981770799,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}