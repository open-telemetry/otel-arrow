window.BENCHMARK_DATA = {
  "lastUpdate": 1787362460924,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca",
          "message": "Add uncompressed bytes-per-log metric to traffic generator benchmarks (#3026)\n\nCloses #2987\n\nAdds a `logs_bytes_produced` counter metric to the traffic generator\nreceiver that tracks the total protobuf-encoded (uncompressed) bytes of\nlog payloads produced. The benchmark report SQL then computes\n`uncompressed_bytes_per_log` from this counter, enabling direct\ncomparison of uncompressed payload size against the egress (compressed)\nbytes per log.\n\n### Changes\n- **metrics.rs**: Added `logs_bytes_produced: Counter<u64>` with unit\n`By`\n- **mod.rs**: Record payload bytes in `export_pdata()` for log signals\n(captured before ownership move)\n- **integration_report_logs.yaml** & **report_logs.yaml**: Added\n`logs_bytes_produced` to metric filter and `uncompressed_bytes_per_log`\ncomputed metric to report SQL",
          "timestamp": "2026-06-02T22:17:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca"
        },
        "date": 1780455968432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6160192489624023,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.47408178599363,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.35160046457608,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38390.18416572147,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37385.88961489604,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002317,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 796196.3065881141,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11124206.070510978,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.296706184861723,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7432125806808472,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54058971134614,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.76902103873924,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1318.6716145833334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1614.35546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66174.20498624118,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65682.38994842726,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002232,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 285518.3912872831,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19036273.863235302,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.3469549678607535,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "a7a04541634b883904108e499e52bbeb94ccfb6e",
          "message": "feat: added comment support in OPL (#3152)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for comments in OPL programs. Both inline comments (`//`)\nand block comments (`/* ... */`) are supported\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/3151\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\nYes - this comment syntax is now available for OPL programs written in\nthe transform processor config.\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-03T18:14:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7a04541634b883904108e499e52bbeb94ccfb6e"
        },
        "date": 1780514063907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.452617645263672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5971745271886,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.99425467525414,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1292.5494791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1509.28125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66015.37958035023,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64396.27465550858,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007229,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 285893.53983942076,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18991858.159844812,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.439597497973664,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3471508026123047,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.45533132309367,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.737843866171,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.306510416666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.60546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 36908.207491389505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36041.91623093505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002914,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 767896.9009324142,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10708894.619574782,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.305662440703486,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "ee0b8953fa04dee164e3f9149751559be8b05f2e",
          "message": "task(comparison_dashboard): Add comparisons to fluent bit (#3190)\n\n# Change Summary\n\nThis PR adds comparisons to fluent bit for logs across all comparisons.\nMetrics and traces are not included though we may want to add them in\nthe future as I think there is some support there.\n\n## What issue does this PR close?\n\n* Closes #3169\n\n## How are these changes tested?\n\n<img width=\"2335\" height=\"1572\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/17be789e-6aca-40da-aefb-b0fac929da7d\"\n/>\n\n## Are there any user-facing changes?\n\nYes new comparisons on the dashboard.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-04T01:18:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee0b8953fa04dee164e3f9149751559be8b05f2e"
        },
        "date": 1780543770193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.408416271209717,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55414105672197,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69982393822394,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1299.6213541666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1587.6015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 68714.57087225506,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 67059.6380143739,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00183,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 295943.4656998458,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19769685.07659088,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.413138431144108,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.540890693664551,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.36853005049703,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.87234082106895,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.176692708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.81640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37945.2595299591,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36981.111950431834,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002225,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 788912.8998170386,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10994417.537635822,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.33286043087264,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780600535985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4590530395507812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.4826499315711,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.22447612370492,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.817057291666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.75,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 36366.92835766778,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 35472.64624010026,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002318,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 757248.4859359392,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10553923.196644189,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.347392038654938,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.4185566902160645,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.582690016832,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78883404255319,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1325.2359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1594.8515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 67304.40860375801,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65676.61321629326,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002013,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 290914.60259256524,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19407886.74434532,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.429500675293564,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780628309896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.615307092666626,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.67513207084167,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.49956050758279,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.991145833333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38019.65859503187,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37025.32777893781,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003169,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 789457.4510116484,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11037377.017979432,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.322092156081826,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9766973257064819,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5797219756506,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78350209010682,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1335.7598958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1561.08203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66222.10191064545,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65575.31238440961,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007156,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 287341.4164612473,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19134197.655843504,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.381853566732869,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tom Tan",
            "username": "ThomsonTan",
            "email": "Tom.Tan@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "082af67b453a42301bdb6f62fd00b1640b641d40",
          "message": "fix(otap): Transform processor config error on leading whitespace (#3219)\n\n# Change Summary\n\nIn `SignalScope::try_from`\n(`rust/otap-dataflow/crates/core-nodes/src/processors/transform_processor/mod.rs`),\nthe query slice is now trimmed of leading whitespace before the keyword\nchecks.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #3209 \n\n## How are these changes tested?\n\nAdded test and passed locally.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-05T16:49:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/082af67b453a42301bdb6f62fd00b1640b641d40"
        },
        "date": 1780685365015,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5303401947021484,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.00516563530262,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.19341822952845,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.297265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38011.81349432379,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37049.985284755036,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002399,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 792859.5201453088,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11054240.993878838,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.399725642308066,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6789209842681885,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56521958689292,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.79658151416628,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 720.369921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 779.91796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 59246.158623524025,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 57659.000828333024,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007266,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 257861.0423815205,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17031375.422511958,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.472173271771479,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780713737314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7682926654815674,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.74245452149962,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.29717637977615,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 62.2890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.81640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38128.48502819488,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37835.546667612405,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002384,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 795208.4778204166,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11088898.929162156,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.017496715624905,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.516103982925415,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.60806765542523,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82782716240241,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 730.6626302083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 795.31640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55833.27009918143,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54428.4469586184,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001859,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 245460.3065319071,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16093154.340875933,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.509779724535021,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780768707033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1765857934951782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58368201973822,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82677192167789,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 723.6776041666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 801.58984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 58694.73071879846,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 59385.3246073767,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001979,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 256649.33440114863,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16917830.19217641,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.321763602337341,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.576439142227173,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.65926036527839,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.19610041583243,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.26731770833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.9921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38013.51620575077,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37034.12110147301,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002342,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 792532.3076479014,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11049703.59217317,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.40005713856077,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780800407929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.497713804244995,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.97122507152113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.21328555849821,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.752604166666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.74609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38634.6499379399,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37669.66697671715,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003132,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 803300.953004136,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11191317.5093091,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.324875356626855,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6347036361694336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55014189319722,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.72638891028109,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 731.5885416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 796.4296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 56034.641014393994,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54558.29438432467,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002169,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 245772.8732284854,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16093670.680429447,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.504775598320377,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780855456435,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1561051607131958,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5766444769752,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.84198423736672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 730.3934895833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 787.96484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 57544.917301180714,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56879.63752568589,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001824,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 252155.73688547636,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16575749.399461001,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.433145987816942,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5160183906555176,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.26971029018158,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.70096183679802,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37402.811312158046,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36461.74971327928,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003511,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 781144.0140327186,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10882213.104976097,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.423656850681194,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780887000299,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1648443937301636,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.96922899088644,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.9577020084073,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.73033854166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.9296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37953.45719196514,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37511.358496610446,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002439,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 787876.6494046665,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10999700.078290742,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.00368211073613,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.636530876159668,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54720373034979,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74236147553634,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 718.378125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 803.4609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 58911.30619635921,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 57358.091468989274,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007157,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 257310.81295257568,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16955719.189640857,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.486042097333226,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "66541e92c02992528ccb1693cf5ea2269a13b230",
          "message": "chore(ci): restore continuous/nightly pipeline perf workflows (#3364)\n\n## Root cause\n\nAll four pipeline perf workflows set `TMPDIR`/`TMP`/`TEMP` in job-level\n`env:` using `${{ runner.temp }}`. The `runner` context is not available\nat job-level `env` scope (only inside `steps`), so GitHub Actions\nrejects the workflow at evaluation time:\n\n> Unrecognized named-value: 'runner'. Located at position 1 within\nexpression: runner.temp\n\nFailing since #3164 introduced the pattern on June 8.\n\n## Fix\n\nReplace the job-level `env` block with a `Route temp files to\nRUNNER_TEMP` step at the top of `steps:` that writes\n`TMPDIR`/`TMP`/`TEMP` to `$GITHUB_ENV` using `$RUNNER_TEMP`. Placed\nbefore `harden-runner` so every subsequent step inherits the routed temp\ndirs.",
          "timestamp": "2026-07-08T11:04:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66541e92c02992528ccb1693cf5ea2269a13b230"
        },
        "date": 1783533253237,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8626484870910645,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.58605224914098,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.56161815795569,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 62.659765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.4765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28756.768485848228,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27933.56328193141,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003265,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2693447.0687542204,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7288786.83924687,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.42332564483293,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.508539080619812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51150396441602,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.73360720055389,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 797.6279947916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 884.38671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52989.83498470403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53259.30900321879,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002074,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 598474.2836322527,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13327516.601798719,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.236989266910753,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "66541e92c02992528ccb1693cf5ea2269a13b230",
          "message": "chore(ci): restore continuous/nightly pipeline perf workflows (#3364)\n\n## Root cause\n\nAll four pipeline perf workflows set `TMPDIR`/`TMP`/`TEMP` in job-level\n`env:` using `${{ runner.temp }}`. The `runner` context is not available\nat job-level `env` scope (only inside `steps`), so GitHub Actions\nrejects the workflow at evaluation time:\n\n> Unrecognized named-value: 'runner'. Located at position 1 within\nexpression: runner.temp\n\nFailing since #3164 introduced the pattern on June 8.\n\n## Fix\n\nReplace the job-level `env` block with a `Route temp files to\nRUNNER_TEMP` step at the top of `steps:` that writes\n`TMPDIR`/`TMP`/`TEMP` to `$GITHUB_ENV` using `$RUNNER_TEMP`. Placed\nbefore `harden-runner` so every subsequent step inherits the routed temp\ndirs.",
          "timestamp": "2026-07-08T11:04:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66541e92c02992528ccb1693cf5ea2269a13b230"
        },
        "date": 1783566568440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8358864784240723,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.48798897127355,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74468893656146,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.4333333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 858.34765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53091.59812293468,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52647.81262732396,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001961,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 596158.8530408948,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13338801.431957202,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.323525580462412,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.609276533126831,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.55760045056181,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.92988611880578,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.19739583333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.95703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28311.044947020902,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27572.33147862577,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008382,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2654373.81977249,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7190738.860628038,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.26947296169625,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "d2b3ed04a389abf5f9e460123d95ea42b30312bf",
          "message": "feat: add opamp controller extension (#3410)\n\n# Change Summary\n\nAdds initial implementation of OpAMP Agent Controller Extension.\n\nDesign doc can be found in #3388 \n\nAdds:\n- controller extension\n- OpAMP proto definitions & prost generated structs\n\nCurrently only works with websocket.\n\nFollowups will need to include:\n- [plain HTTP\ntransport](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport)\n(plain meaning traditional HTTP request/response, not necessarily\nplaintext)\n- mTLS\n- connection setting management (this is ignored)\n- metrics collection\n\nCurrently this lives in the controller crate. I wasn't sure where was\nthe right place for this, and hesitated between controller or\ncore-nodes. Happy to move if anyone has feelings/suggestions\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Relates to #3387 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: cijothomas <cijo.thomas@gmail.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-09T17:01:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2b3ed04a389abf5f9e460123d95ea42b30312bf"
        },
        "date": 1783620318745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8620887994766235,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.57291280684358,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78002326483133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.5321614583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 840.73046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55069.680486857105,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54594.930942823994,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002164,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 617842.0842648426,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13769768.620801952,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.316839743087032,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.320697069168091,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.01055086096446,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.76931485639079,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.529296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.08984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28312.21122602841,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27655.170600071884,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002378,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2644717.3336146376,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7161015.526963046,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.63192980656439,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "d2b3ed04a389abf5f9e460123d95ea42b30312bf",
          "message": "feat: add opamp controller extension (#3410)\n\n# Change Summary\n\nAdds initial implementation of OpAMP Agent Controller Extension.\n\nDesign doc can be found in #3388 \n\nAdds:\n- controller extension\n- OpAMP proto definitions & prost generated structs\n\nCurrently only works with websocket.\n\nFollowups will need to include:\n- [plain HTTP\ntransport](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport)\n(plain meaning traditional HTTP request/response, not necessarily\nplaintext)\n- mTLS\n- connection setting management (this is ignored)\n- metrics collection\n\nCurrently this lives in the controller crate. I wasn't sure where was\nthe right place for this, and hesitated between controller or\ncore-nodes. Happy to move if anyone has feelings/suggestions\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Relates to #3387 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: cijothomas <cijo.thomas@gmail.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-09T17:01:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2b3ed04a389abf5f9e460123d95ea42b30312bf"
        },
        "date": 1783649575370,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3010380268096924,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.62972206477265,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.2276621673737,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.08059895833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.56640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28897.926092170786,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28232.973813579312,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004306,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2691168.2221408235,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7344827.625983535,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.32004102403279,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5557712316513062,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5043112768093,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.79374892987781,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 780.5826822916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 840.171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 56672.951272811486,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56357.97932571404,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002169,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 637323.5788314602,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14202486.732490264,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.308488814833595,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "021c7430e5267668d8257f337e2d122f51387597",
          "message": "feat(contrib-extensions): Add Azure Identity Auth extension (#3438)\n\n# Change Summary\n\nAdds a new `azure_identity_auth` extension (in a new\n`otap-df-contrib-extensions`\ncrate) that acquires and refreshes Azure OAuth access tokens and exposes\nthem\nto data-path nodes through the shared `BearerTokenProvider` capability\n(merged\nin #3372).\n\n## What issue does this PR close?\n\n* Related to #3356\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes — a new opt-in extension is available. It is not enabled by default;\nusers\nmust build with `--features azure-identity-auth-extension` and reference\nthe\nURN `urn:microsoft:extension:azure_identity_auth` in their pipeline\nconfig.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T16:07:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/021c7430e5267668d8257f337e2d122f51387597"
        },
        "date": 1783711111854,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9185105562210083,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54898725323585,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.68777555110219,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.5259114583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 833.19921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55813.24932441286,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55300.598737886256,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001882,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 626645.8300009132,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14034127.093323836,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.33162830607113,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8504343032836914,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.73224525726577,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.69442965925698,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.518880208333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.2421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28781.863539501916,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 29026.634373204673,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003064,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2703748.681863706,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7266120.798141679,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.1471643284836,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783735264593,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.682443380355835,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.85142901158679,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.08044547563804,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.559895833333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.69921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29084.158158025537,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28303.992114013497,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005175,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2714837.4385104747,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7360617.553945722,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.91712107516948,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.858862578868866,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.61866539215609,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.90511814671814,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.2670572916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 834.3671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55653.137474999174,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55175.15347153049,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002008,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 627302.9620042914,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13923336.315776054,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.369301624651934,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783791147337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9658743739128113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5172445956672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.66580007719028,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 799.3611979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 868.70703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55674.705989104084,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56212.453718088225,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002113,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 623265.0789603846,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13988802.742839735,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.087668972539946,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5391969680786133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.99681326303666,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.91154250386398,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.634765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28763.823080238966,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28033.452964549535,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002455,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2685923.1437542606,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7290844.599362488,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.81135606629756,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783821135302,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.026169776916504,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.50609044661043,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69956060838838,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 791.0140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 873.65234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53176.688401105464,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53722.371521307614,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001856,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 600775.9579043501,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13346396.479796775,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.182975376767715,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.34257435798645,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.0599873709891,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.15320065050724,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.006510416666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.76953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28613.038056557125,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27942.756392776788,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007609,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2700942.6632235083,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7281891.0232060915,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.65985077698716,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783877666297,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.620288848876953,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.69710420181508,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.2319463923952,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.707942708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28518.538658284833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27771.27062113404,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006581,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2656487.590908207,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7198430.492900342,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.65596141238889,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0606679916381836,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53747117991144,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78643672874989,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.9307291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 892.07421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52991.649082086675,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52429.583613067494,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001907,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 602278.6480534318,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13359537.992581362,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.48738186628132,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783907538216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.821826457977295,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.21279177888816,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.89800930232559,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.781640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.43359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28725.551157763373,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27914.965968821543,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00233,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2681434.4116627336,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7242599.839954915,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.05723376692094,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6144409775733948,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.52578761633795,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.84913765150931,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 791.1873697916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 854.6484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53526.65081227614,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53197.76115037451,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001886,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 602373.8141054881,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13395313.081833947,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.3232925799782,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "035e910605e727284dd65a849142868778665c89",
          "message": "chore: Optimize per-event field extraction in the ETW receiver (#3427)\n\n## Summary\n\nCaches per-schema field readers instead of rebuilding them every event\nand drops a redundant per-field copy.\n\n## Problem\nPer event, `extract_decoded_fields` called\n`EventFormat::try_get_field_data_closure(name)` once for every field.\nEach call does two costly things:\n- Finds the field by name by re-walking the field list from index 0.\nReading all n fields is therefore `1 + 2 + ... + n = O(n^2)` name\ncomparisons per event.\n- Heap-allocates a boxed closure (`Box<dyn FnMut>`) for the result, so\n`n` allocations per event.\n\nThe result was then `.to_vec()`'d before interpretation, adding another\n`n` copies. So, a single event with `n` fields cost `O(n^2)`\nname-finding + `~2n` allocations, repeated on every event on the decode\nthread.\n\n### Performance\n\nCaching field readers per schema (instead of rebuilding a boxed closure\nper field on every event) turns per-event field extraction from an\nO(n^2) name-walk + n allocations into an O(n) reuse. A microbenchmark\nreading 16 fixed-size fields:\n\n| | time (16 fields) |\n|---|---|\n| `closure_per_field` (before) | ~990 ns |\n| `cached_refs` (after) | ~72 ns |\n\n~14x faster. The gap compounds two effects the change removes: the\nO(n^2) re-walk to find each field by name, and a heap allocation (boxed\nclosure) per field per event.\n\n<details>\n<summary>Benchmark used to measure this (criterion, against\none_collect's public API)</summary>\n\n```rust\nuse criterion::{black_box, criterion_group, criterion_main, Criterion};\nuse one_collect::event::{EventField, EventFormat, LocationType};\n\nconst FIELD_COUNT: usize = 16;\n\nfn bench_field_reads(c: &mut Criterion) {\n    // All-fixed-size schema (the common TraceLogging shape after struct\n    // flattening), so absolute offsets are valid for the cached-reader path.\n    let mut format = EventFormat::new();\n    let mut names = Vec::with_capacity(FIELD_COUNT);\n    for i in 0..FIELD_COUNT {\n        let name = format!(\"f{i}\");\n        format.add_field(EventField::new(\n            name.clone(),\n            \"u32\".to_string(),\n            LocationType::Static,\n            i * 4,\n            4,\n        ));\n        names.push(name);\n    }\n\n    let data = vec![0xABu8; FIELD_COUNT * 4];\n    let data = data.as_slice();\n\n    // Readers resolved once (a consumer caches these per schema_id).\n    let refs: Vec<_> = names\n        .iter()\n        .map(|n| format.get_field_ref(n).expect(\"field exists\"))\n        .collect();\n\n    let mut group = c.benchmark_group(\"tdh_field_reads\");\n\n    // Before: a fresh boxed closure per field, per event (re-walks the field\n    // list each call, so reading all n fields is O(n^2) plus n allocations).\n    group.bench_function(\"closure_per_field\", |b| {\n        b.iter(|| {\n            for n in &names {\n                let mut reader = format\n                    .try_get_field_data_closure(n)\n                    .expect(\"field exists\");\n                black_box(reader(data));\n            }\n        });\n    });\n\n    // After: O(1), allocation-free reads via readers resolved once per schema.\n    group.bench_function(\"cached_refs\", |b| {\n        b.iter(|| {\n            for r in &refs {\n                black_box(format.get_data(*r, data));\n            }\n        });\n    });\n\n    group.finish();\n}\n\ncriterion_group!(benches, bench_field_reads);\ncriterion_main!(benches);\n```\n</details>\n\n## Changes\n- Cache field readers per `SchemaId`. The name-finding walk and the\nclosure boxing now happen once per schema (on first sight), and every\nlater event of that schema reuses the cached closures. Per-event\nextraction becomes an `O(n)` pass with no per-field closure allocations.\n- Drop the `to_vec`. The cached closure's borrowed slice is passed\nstraight to `interpret_field_value`, so numeric fields allocate nothing.\n- Bound the cache (MAX_CACHED_SCHEMAS = 4096) against pathological\nproducers that emit unbounded distinct schemas; beyond the cap, new\nschemas fall back to the original per-event path so the map can't grow\nwithout bound.\n- Pre-size to 128 to avoid warmup rehashes.\n\n## How are these changes tested?\n- Existing unit tests and integration tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Swapnil Ashtekar <46826200+swashtek@users.noreply.github.com>",
          "timestamp": "2026-07-13T19:25:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/035e910605e727284dd65a849142868778665c89"
        },
        "date": 1783973105312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6759742498397827,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56959187406316,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80217687074831,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.5819010416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 842.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55209.76100974915,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54836.55722483312,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002071,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 624099.2213785121,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13916330.922720125,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.381079574701753,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3968732357025146,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.64429481501956,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.04544849469855,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.74765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.90234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 25476.913844616956,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 24866.26453462768,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003343,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2392410.7072698055,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6439029.200850434,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.21110174945007,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "035e910605e727284dd65a849142868778665c89",
          "message": "chore: Optimize per-event field extraction in the ETW receiver (#3427)\n\n## Summary\n\nCaches per-schema field readers instead of rebuilding them every event\nand drops a redundant per-field copy.\n\n## Problem\nPer event, `extract_decoded_fields` called\n`EventFormat::try_get_field_data_closure(name)` once for every field.\nEach call does two costly things:\n- Finds the field by name by re-walking the field list from index 0.\nReading all n fields is therefore `1 + 2 + ... + n = O(n^2)` name\ncomparisons per event.\n- Heap-allocates a boxed closure (`Box<dyn FnMut>`) for the result, so\n`n` allocations per event.\n\nThe result was then `.to_vec()`'d before interpretation, adding another\n`n` copies. So, a single event with `n` fields cost `O(n^2)`\nname-finding + `~2n` allocations, repeated on every event on the decode\nthread.\n\n### Performance\n\nCaching field readers per schema (instead of rebuilding a boxed closure\nper field on every event) turns per-event field extraction from an\nO(n^2) name-walk + n allocations into an O(n) reuse. A microbenchmark\nreading 16 fixed-size fields:\n\n| | time (16 fields) |\n|---|---|\n| `closure_per_field` (before) | ~990 ns |\n| `cached_refs` (after) | ~72 ns |\n\n~14x faster. The gap compounds two effects the change removes: the\nO(n^2) re-walk to find each field by name, and a heap allocation (boxed\nclosure) per field per event.\n\n<details>\n<summary>Benchmark used to measure this (criterion, against\none_collect's public API)</summary>\n\n```rust\nuse criterion::{black_box, criterion_group, criterion_main, Criterion};\nuse one_collect::event::{EventField, EventFormat, LocationType};\n\nconst FIELD_COUNT: usize = 16;\n\nfn bench_field_reads(c: &mut Criterion) {\n    // All-fixed-size schema (the common TraceLogging shape after struct\n    // flattening), so absolute offsets are valid for the cached-reader path.\n    let mut format = EventFormat::new();\n    let mut names = Vec::with_capacity(FIELD_COUNT);\n    for i in 0..FIELD_COUNT {\n        let name = format!(\"f{i}\");\n        format.add_field(EventField::new(\n            name.clone(),\n            \"u32\".to_string(),\n            LocationType::Static,\n            i * 4,\n            4,\n        ));\n        names.push(name);\n    }\n\n    let data = vec![0xABu8; FIELD_COUNT * 4];\n    let data = data.as_slice();\n\n    // Readers resolved once (a consumer caches these per schema_id).\n    let refs: Vec<_> = names\n        .iter()\n        .map(|n| format.get_field_ref(n).expect(\"field exists\"))\n        .collect();\n\n    let mut group = c.benchmark_group(\"tdh_field_reads\");\n\n    // Before: a fresh boxed closure per field, per event (re-walks the field\n    // list each call, so reading all n fields is O(n^2) plus n allocations).\n    group.bench_function(\"closure_per_field\", |b| {\n        b.iter(|| {\n            for n in &names {\n                let mut reader = format\n                    .try_get_field_data_closure(n)\n                    .expect(\"field exists\");\n                black_box(reader(data));\n            }\n        });\n    });\n\n    // After: O(1), allocation-free reads via readers resolved once per schema.\n    group.bench_function(\"cached_refs\", |b| {\n        b.iter(|| {\n            for r in &refs {\n                black_box(format.get_data(*r, data));\n            }\n        });\n    });\n\n    group.finish();\n}\n\ncriterion_group!(benches, bench_field_reads);\ncriterion_main!(benches);\n```\n</details>\n\n## Changes\n- Cache field readers per `SchemaId`. The name-finding walk and the\nclosure boxing now happen once per schema (on first sight), and every\nlater event of that schema reuses the cached closures. Per-event\nextraction becomes an `O(n)` pass with no per-field closure allocations.\n- Drop the `to_vec`. The cached closure's borrowed slice is passed\nstraight to `interpret_field_value`, so numeric fields allocate nothing.\n- Bound the cache (MAX_CACHED_SCHEMAS = 4096) against pathological\nproducers that emit unbounded distinct schemas; beyond the cap, new\nschemas fall back to the original per-event path so the map can't grow\nwithout bound.\n- Pre-size to 128 to avoid warmup rehashes.\n\n## How are these changes tested?\n- Existing unit tests and integration tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Swapnil Ashtekar <46826200+swashtek@users.noreply.github.com>",
          "timestamp": "2026-07-13T19:25:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/035e910605e727284dd65a849142868778665c89"
        },
        "date": 1784021837262,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.935360312461853,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54262776344336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7238749233599,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 798.6983072916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 870.64453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52236.58950185379,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51747.989146295666,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002003,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 591073.6527751565,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13102972.379442684,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.422156928729047,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.353952169418335,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.22124532365036,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.7812194367069,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.210286458333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.1328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27048.217568123102,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26411.51542492473,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007651,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2541178.373097964,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6874034.207513289,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.21478859557737,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "339e7ac67cc04ab693a08cb4692203f35bd99a0f",
          "message": "feat(recordset_kql_processor): Record signals.dropped in recordset_kql_processor (#3482)\n\n# Change Summary\n\nExtends the `signals.dropped` flow metric (#2859) to the recordset_kql\nprocessor so pipelines can observe how many records it filters out.\nTogether with the earlier transform-processor change, this gives every\nKQL-based decision node the same queryable dropped count already\nprovided by `filter_processor` and `log_sampling_processor`.\n\nVery similar to recent PR #3473.\n\n### Validation\n\nRan `configs/trafficgen-flow-metrics-demo.yaml` (with `--features\nrecordset-kql-processor`), which places the recordset_kql processor as\none of four interior decision nodes in a single `ingest_pipeline` flow\nrange: the sampler keeps ~2/3, filter drops `worker-1`, transform drops\n`worker-3`, and recordset drops `worker-2`. Each decision node's drops\nare tagged with a distinct `flow.node.decision`, and the counts\nreconcile exactly against incoming/outgoing (480 − 160 − 48 − 48 − 48 =\n176).\n\n| `flow.node.decision` | Metric | Sum | Count |\n| --- | --- | ---: | ---: |\n| _(range)_ | signals.incoming | 480 | 48 |\n| sampler | signals.dropped | 160 | 48 |\n| filter | signals.dropped | 48 | 48 |\n| transform | signals.dropped | 48 | 48 |\n| **recordset** | **signals.dropped** | **48** | 48 |\n| _(range)_ | signals.outgoing | 176 | 48 |\n\nThe `recordset` row confirms the processor records `signals.dropped`\nunder its own decision attribute, exactly like the existing\n`filter`/`transform`/`sampler` decision nodes.\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Related to #2859 \n\n## How are these changes tested?\n\n* Unit tests and demo config\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nAdditional `flow.dropped` metric source\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-14T23:32:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/339e7ac67cc04ab693a08cb4692203f35bd99a0f"
        },
        "date": 1784074473533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.7503223419189453,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.54119865197562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.55490280777538,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.56471354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.1640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27146.620381587258,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26400.00082661451,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003786,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2531756.648210979,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6852599.097031876,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.8998700355589,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.277421474456787,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56653143476626,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.71638028715562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 804.4287760416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 884.50390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52739.970197625305,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52066.25853535045,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00193,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 598597.0167176226,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13229480.759331442,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.49683179772183,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "69856c386f1368f22d98802cd273dca511d31df7",
          "message": "chore: ETW tracestats metrics polling (#3425)\n\n# Change Summary\n\n1. Expose ETW session trace stats as receiver metrics\n2. Poll `query_stats(handle)` off-thread while `ProcessTrace` is blocked\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\nhttps://github.com/microsoft/one-collect/issues/299\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-15T01:25:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/69856c386f1368f22d98802cd273dca511d31df7"
        },
        "date": 1784082254962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.2005748748779297,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.80610941998545,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.9461133477857,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.93736979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.83203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28979.210557644667,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28341.501282417554,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008536,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2725181.1382425376,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7333640.466359351,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.15514404429875,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.44864001870155334,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58634660759833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74018211033744,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.1061197916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 850.16015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52081.6302640237,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52315.289290040215,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001962,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 590721.5982706068,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12960585.240551017,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.291567078901128,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "ed8deab4e1a56145bcc3b1460a1507813a03c9b6",
          "message": "chore(deps): update all patch versions (#3485)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [syn](https://redirect.github.com/dtolnay/syn) |\nworkspace.dependencies | patch | `2.0.118` → `2.0.119` |\n| [xxhash-rust](https://redirect.github.com/DoumanAsh/xxhash-rust) |\nworkspace.dependencies | patch | `0.8.16` → `0.8.17` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>dtolnay/syn (syn)</summary>\n\n###\n[`v2.0.119`](https://redirect.github.com/dtolnay/syn/releases/tag/2.0.119)\n\n[Compare\nSource](https://redirect.github.com/dtolnay/syn/compare/2.0.118...2.0.119)\n\n- Preserve attributes on tail-call expressions in statement position\n([#&#8203;1994](https://redirect.github.com/dtolnay/syn/issues/1994))\n- Parse field-representing types builtin in type position\n([#&#8203;1996](https://redirect.github.com/dtolnay/syn/issues/1996))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjI1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-15T16:50:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed8deab4e1a56145bcc3b1460a1507813a03c9b6"
        },
        "date": 1784138948351,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8108738660812378,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51853471992484,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.75235836627141,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.3907552083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 859.14453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53551.582196595504,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53117.34639610618,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001962,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 603842.7547045256,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13396097.877370685,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.368089629356765,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5742557048797607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.25682774587185,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.60171592583973,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.13229166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.93359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27875.41970622514,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27157.83513950809,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002684,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2638623.197920076,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7067655.749989676,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 97.15881933761048,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "aa051fe5dc924e81dcc0ef075de4833a0f259b6d",
          "message": "feat(metrics): Add datapoint-level enum-attribute mechanism for metric sets (#3454)\n\n# Change Summary\n\nAdds the core plumbing for datapoint-level enum attributes on metric\nsets\n\n### Outcome\n\nInstrumentation can now declare closed-set (`enum`) attributes on the\n**existing** `metric_set` unit — no new instrument family, no per-signal\nset explosion — in two flavors:\n\n- **Registration** attributes — a value fixed once at registration,\nattached to every datapoint of the set (e.g. `signal = logs` on a\njournald receiver).\n- **Measurement** attributes — values that vary per recorded datapoint\n(e.g. `signal` × `outcome` for durable-buffer loss). Each combination is\nrecorded through a generated `with(attrs)` and exported as its own\ndatapoint with the right attributes.\n\nWorst-case cardinality of a metric set is known at compile time, and a\nset that exceeds the budget (2000) is **rejected with a hard build\nerror** at the declaration site — so cardinality blowups are caught at\ncompile time rather than in production.\n\nPlain metric sets are unaffected. **No node instrumentation is migrated\nin this PR** — that is a separate sub-issue.\n\n### Usage\n\n```rust\n#[derive(Debug, Clone, Copy, AttributeEnum)]\npub enum Signal {\n    #[attribute_value = \"log-records\"] // optional rename\n    Logs,\n    Metrics,\n    Traces,\n}\n\n#[derive(Debug, Clone, Copy, AttributeEnum)]\npub enum LossOutcome {\n    Dropped,\n    Expired,\n}\n\n#[attribute_set(name = \"durable_buffer.loss.attrs\", measurement)]\n#[derive(Debug, Clone, Copy)]\npub struct LossAttributes {\n    pub signal: Signal,\n    #[attribute_key = \"loss.outcome\"] // optional rename\n    pub outcome: LossOutcome,\n}\n\n#[metric_set(name = \"processor.durable_buffer.loss\", measurement_attributes = LossAttributes)]\n#[derive(Debug, Default, Clone)]\npub struct LossMetrics {\n    #[metric(unit = \"{items}\")]\n    pub lost_items: Counter<u64>,\n}\n\nlet mut loss = LossMetrics::register(&pipeline_ctx);\nloss.with(LossAttributes {\n    signal: Signal::Metrics,\n    outcome: LossOutcome::Expired,\n})\n.lost_items\n.add(80); // signal=metrics, loss.outcome=expired\n\n#[attribute_set(name = \"signal.attrs\")]\n#[derive(Debug, Clone, Copy)]\npub struct SignalAttributes {\n    pub signal: Signal,\n}\n\n#[metric_set(name = \"receiver.journald\", registration_attributes = SignalAttributes)]\n#[derive(Debug, Default, Clone)]\npub struct JournaldMetrics {\n    #[metric(unit = \"{records}\")]\n    pub records: Counter<u64>,\n}\n\nlet mut metrics = JournaldMetrics::register(\n    &pipeline_ctx,\n    &SignalAttributes {\n        signal: Signal::Logs,\n    },\n);\nmetrics.records.add(42); // signal=log-records\n```\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Part of #3300\n* Closes #3430\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-15T23:07:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/aa051fe5dc924e81dcc0ef075de4833a0f259b6d"
        },
        "date": 1784167900459,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.552866220474243,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.7469282355323,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.66877659986072,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.746484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.17578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28781.13685290584,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28046.39292611319,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004579,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2706691.8448977014,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7314915.913944915,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.50766328591149,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.3731178641319275,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.61729674957336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.99698417623333,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 783.7415364583334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 842.00390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 54346.6141687718,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54143.83724994833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001898,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 613737.1203705351,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13704143.866323179,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.335308902050912,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "fa9396b54203f1b6d85bb44839aa05cdfc060ba5",
          "message": "chore(deps): update all patch versions (#3498)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [bitflags](https://redirect.github.com/bitflags/bitflags) |\nworkspace.dependencies | patch | `2.13.0` → `2.13.1` |\n| [clap](https://redirect.github.com/clap-rs/clap) |\nworkspace.dependencies | patch | `4.6.1` → `4.6.2` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>bitflags/bitflags (bitflags)</summary>\n\n###\n[`v2.13.1`](https://redirect.github.com/bitflags/bitflags/blob/HEAD/CHANGELOG.md#2131)\n\n[Compare\nSource](https://redirect.github.com/bitflags/bitflags/compare/2.13.0...2.13.1)\n\n#### What's Changed\n\n- Lower the LLVM IR output of the generated output by\n[@&#8203;bolshoytoster](https://redirect.github.com/bolshoytoster) in\n[#&#8203;492](https://redirect.github.com/bitflags/bitflags/pull/492)\n\n#### New Contributors\n\n- [@&#8203;bolshoytoster](https://redirect.github.com/bolshoytoster)\nmade their first contribution in\n[#&#8203;492](https://redirect.github.com/bitflags/bitflags/pull/492)\n\n**Full Changelog**:\n<https://github.com/bitflags/bitflags/compare/2.13.0...2.13.1>\n\n</details>\n\n<details>\n<summary>clap-rs/clap (clap)</summary>\n\n###\n[`v4.6.2`](https://redirect.github.com/clap-rs/clap/compare/clap_complete-v4.6.1...clap_complete-v4.6.2)\n\n[Compare\nSource](https://redirect.github.com/clap-rs/clap/compare/v4.6.1...v4.6.2)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjI1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-16T15:36:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fa9396b54203f1b6d85bb44839aa05cdfc060ba5"
        },
        "date": 1784225868881,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.885127604007721,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58173851303454,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80758854327073,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.453515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 868.18359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52186.71902415819,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51724.799979520634,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001856,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 592222.230095504,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13123178.186795037,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.449483232994272,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7314282655715942,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.39313251800624,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.56719250716,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.55846354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.1875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28401.961518591983,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28194.22155243235,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002898,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2669324.9659167146,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7225093.149645507,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.67631375998847,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "a1904eee5d1e84b07820ba4a93e0a2b22c05282f",
          "message": "  feat(pdata): add retained memory sizing (#3443)\n\n# Change Summary\n\nAdds a pdata-level retained memory size API without changing existing\nencoded-size semantics.\n\nThe new API gives retention sites a way to estimate how much memory a\npayload keeps alive:\n  - `OtapArrowRecords::retained_memory_bytes()`\n  - `OtapPayload::retained_memory_bytes()`\n  - `OtapPayloadHelpers::retained_memory_bytes()`\n\nFor OTAP Arrow records, this walks Arrow buffers and dedupes shared\nbuffers within one pdata accounting call. `num_bytes()` is unchanged and\nstill represents encoded/wire size.\n\n## What issue does this PR close?\n\n* Closes #3442\n\n## How are these changes tested?\n\n  - `cargo fmt --all`\n  - `cargo check -p otap-df-pdata`\n  - `cargo clippy -p otap-df-pdata --all-targets -- -D warnings`\n  - `cargo test -p otap-df-pdata`\n  - `python3 tools/sanitycheck.py`\n\n## Are there any user-facing changes?\n\n  Yes. This adds a public pdata helper API.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-16T19:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a1904eee5d1e84b07820ba4a93e0a2b22c05282f"
        },
        "date": 1784254416661,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.2315254211425781,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.10429350866536,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.50633755013885,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.04440104166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.5390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28819.338972951682,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28464.421497847066,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008316,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2671465.3889759774,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7322969.862576271,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.85279055040822,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.7204297780990601,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.59819123265888,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.89353009348683,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 785.2440104166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 857.09765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55994.89816113308,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56398.30208695909,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001895,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 629606.9274763775,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14052019.18943429,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.163579472757934,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "601c95329fbe7b723fb4e3a42fa969a8f6d9a951",
          "message": "feat(metrics): Improve datapoint attribute API ergonomics (#3499)\n\n# Change Summary\n\nFollow-up to #3454.\n\n- Make typed metric datapoint attributes easier for component authors to\ndeclare, register, record, and inspect.\n- Standardize datapoint dimensions on canonical pipeline-domain types\nsuch as `otap_df_config::SignalType`, and replace stale static/dynamic\nterminology with registration/measurement terminology.\n- Preserve legacy named measurement sets for compatibility and document\nthe complete contract with self-contained examples.\n\n### Problem: datapoint attributes looked like scope attributes\n\nFixed and variable datapoint dimensions used named scope-style\ndeclarations, even though their names are not exported as scope/entity\nmetadata.\n\n```rust\n#[attribute_set(name = \"component.fixed.attrs\")]\nstruct FixedAttributes {\n    signal: SignalType,\n}\n\n#[attribute_set(name = \"component.variable.attrs\", measurement)]\nstruct VariableAttributes {\n    outcome: Outcome,\n}\n```\n\n### Improvement: declare the per-item lifecycle explicitly\n\n```rust\n#[attribute_set(item, registration)]\nstruct FixedAttributes {\n    signal: SignalType,\n}\n\n#[attribute_set(item, measurement)]\nstruct VariableAttributes {\n    outcome: Outcome,\n}\n```\n\n`registration` marks values fixed for a metric-set registration;\n`measurement` marks values supplied for each recording. Scope/entity\nattributes remain explicitly named:\n\n```rust\n#[attribute_set(name = \"component.scope\")]\nstruct ScopeAttributes { /* ... */ }\n```\n\nLegacy named measurement declarations remain supported, so existing\n`AttributeSetHandler` users do not need to migrate immediately.\n\n### Problem: reporting was implicit when inspecting buckets\n\nComponent authors need to inspect a measurement bucket in diagnostics\nand tests without marking it for export.\n\n### Improvement: separate inspection from recording\n\n```rust\nlet mut metrics = MyMetrics::register(&pipeline_ctx, &fixed);\nmetrics.with(variable).records.add(1);\n\n// Inspecting a bucket does not cause it to be reported.\nlet count = metrics.get(variable).records.get();\n```\n\n`with(...)` is the explicit recording path and marks the bucket for\nreporting; `get(...)` only reads it.\n\n### Problem: registration plumbing leaked through the API\n\nComponent authors should not need to select an entity scope or call\nregistry/registrar helpers.\n\n### Improvement: register through the generated metric-set API\n\n```rust\nlet metrics = MyMetrics::register(&pipeline_ctx, &fixed);\n```\n\n`PipelineContext` supplies the registrar internally. Registration\nattributes are borrowed, allowing callers to reuse them. Low-level\nregistrar and registry helpers remain available for macro expansion and\nexisting engine code, but are hidden from generated documentation.\n\n## What issue does this PR close?\n\nRelated to #3300.\n\n## Are there any user-facing changes?\n\nYes. The component-facing metric declaration and measurement APIs are\nsimplified, and the previous named measurement declaration form remains\ncompatible.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-07-17T18:23:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/601c95329fbe7b723fb4e3a42fa969a8f6d9a951"
        },
        "date": 1784320592110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6067293882369995,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.36241181084782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.79089521165857,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.011328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29520.178357674467,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 29045.86893744107,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003025,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2718780.032609753,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7445668.823914697,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.6029849361868,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.48932576179504395,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5845534220089,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.88207663782447,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 780.6748697916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 832.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55271.48634449799,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55001.028715623754,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002005,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 622756.7210718349,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13894277.155489994,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.322637696318811,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "4ab1d242e454fc5eaacb68118e06ca37151b576a",
          "message": "[otap-dataflow] add kafka exporter into contrib-nodes (#3262)\n\n# Change Summary\n\nAdd Kafka Exporter implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3249 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes",
          "timestamp": "2026-07-17T23:40:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4ab1d242e454fc5eaacb68118e06ca37151b576a"
        },
        "date": 1784341601048,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6460716724395752,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56632344123555,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.84257806491308,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 789.1734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 874.53515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52438.22846184846,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52777.017016441794,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002027,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 595020.1852372101,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13214817.978186212,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.274229179186872,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9032351970672607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.46227650200548,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.30128712871287,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.85338541666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.40625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28590.268126050833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28332.03075816208,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002935,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2684801.908477083,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7245722.396621559,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.7620709363951,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "4ab1d242e454fc5eaacb68118e06ca37151b576a",
          "message": "[otap-dataflow] add kafka exporter into contrib-nodes (#3262)\n\n# Change Summary\n\nAdd Kafka Exporter implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3249 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes",
          "timestamp": "2026-07-17T23:40:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4ab1d242e454fc5eaacb68118e06ca37151b576a"
        },
        "date": 1784397340635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.836111307144165,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56151794667569,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.89111931119312,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.793359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 862.16796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51995.01669147032,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51560.280487099204,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001904,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 586016.6752341342,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13094995.684381384,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.365661119333133,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.373145818710327,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.80372983346588,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.94922816207055,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.8390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28646.904807680257,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27967.072042007505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002992,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2694975.640827196,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7270752.368872807,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.36245212867654,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784427070616,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5429253578186035,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58033807549461,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 101.96642135308184,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.652734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 870.59375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51580.071982774534,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50268.42920106387,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001855,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 584139.721762941,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12945617.296874756,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.620409291615152,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.803083658218384,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.58241389244316,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.12442121445042,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.536067708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.01171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29291.86581085621,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28470.790280073634,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003006,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2750176.5449795118,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7426163.08623985,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.59642454337937,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784483637355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.960800051689148,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.59740626615093,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.50463539247562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.84361979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.4921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29080.071140096778,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28800.669803250126,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003292,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2715581.4029058972,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7340259.021440549,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.28882805355613,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6376490592956543,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51248630440672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69634152010536,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 798.3341145833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 903.5703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51040.015522164525,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 49693.759073560635,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001941,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 579533.9437633716,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12779056.885059172,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.662107165318275,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784518900083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8136403560638428,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.49560375804035,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.5286175720578,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.80651041666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.05078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27490.30956171797,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26716.83107910573,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002962,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2577641.3562565953,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6951746.521577456,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.48005591024138,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5212888717651367,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53238024505058,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80956413449564,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 783.6563802083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 858.4375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55348.039756925275,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53952.55584698042,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002125,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 619742.7521445901,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13917762.449970458,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.486809890940052,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784570660568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.64194917678833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53288174562833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78524289783664,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 797.6110677083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 878.484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52565.81433933689,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51177.05228817858,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00668,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 595211.0506443417,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13136631.714523448,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.63042856186209,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6737515926361084,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.99522262439388,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.1951780565729,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.207421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.60546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28933.50076243549,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28159.890848322542,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003109,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2700278.33017913,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7321939.805223716,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.8909373876349,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "7502e7dbe636b6bd14d15e0a69367fa00bc10343",
          "message": "feat(metrics): Require scope keyword on scope attribute_set declarations (#3531)\n\n# Change Summary\n\nRequire `scope` on every scope-level `#[attribute_set]` declaration so\nthe intended telemetry attachment point is explicit and unambiguous.\n\nFollow-up from\nhttps://github.com/open-telemetry/otel-arrow/pull/3499#issuecomment-5004841970\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #3513\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-20T22:13:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7502e7dbe636b6bd14d15e0a69367fa00bc10343"
        },
        "date": 1784599821144,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6646950840950012,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55769536512315,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78032938993273,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 795.5170572916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 871.984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53536.431005484934,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53180.57698349785,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002132,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 605370.6254849818,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13450782.694783207,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.383303074594899,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.451296806335449,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.16364897205369,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.2697689513948,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.021484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 26024.00111209065,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 25386.075597798306,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002303,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2428175.223801455,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6597281.121865657,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.64988548336501,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "3b1e9ab64362fe2cca00a0081fff7fcc3664ca63",
          "message": "chore(deps): update all patch versions (#3536)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [libc](https://redirect.github.com/rust-lang/libc) |\nworkspace.dependencies | patch | `0.2.186` → `0.2.188` |\n| [time](https://time-rs.github.io)\n([source](https://redirect.github.com/time-rs/time)) |\nworkspace.dependencies | patch | `>=0.3.47, <0.3.54` → `>=0.3.47,\n<0.3.55` |\n| [tokio](https://tokio.rs)\n([source](https://redirect.github.com/tokio-rs/tokio)) |\nworkspace.dependencies | patch | `1.53.0` → `1.53.1` |\n| [tokio-util](https://tokio.rs)\n([source](https://redirect.github.com/tokio-rs/tokio)) |\nworkspace.dependencies | patch | `0.7.18` → `0.7.19` |\n| [xxhash-rust](https://redirect.github.com/DoumanAsh/xxhash-rust) |\nworkspace.dependencies | patch | `0.8.17` → `0.8.18` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rust-lang/libc (libc)</summary>\n\n###\n[`v0.2.188`](https://redirect.github.com/rust-lang/libc/releases/tag/0.2.188)\n\n[Compare\nSource](https://redirect.github.com/rust-lang/libc/compare/0.2.187...0.2.188)\n\n##### Changed\n\n- Restore `Send` and `Sync` for `DIR`\n([35b062263401](https://redirect.github.com/rust-lang/libc/commit/35b062263401733cd89065c6a553640f2ba51ff1))\n\nThese were removed in 0.2.187 because `libc` does not actually make\n`Send` and `Sync`\nguarantees about `DIR` (or other extern types), but this caused some\ncrates to break.\nThe traits are added back for now to allow time to migrate, but will be\nremoved again\nin the future; please make sure your crates are not relying on\n`libc::DIR: Send` or\n`libc::DIR: Sync`.\n\n###\n[`v0.2.187`](https://redirect.github.com/rust-lang/libc/releases/tag/0.2.187)\n\n[Compare\nSource](https://redirect.github.com/rust-lang/libc/compare/0.2.186...0.2.187)\n\nThis release contains a number of improvements related to 64-bit\n`time_t` configuration.\nOf note the existing `RUST_LIBC_UNSTABLE_*` environment variables have\nbeen replaced\nwith configuration options. The new way to use these is:\n\n```sh\nRUSTFLAGS='--cfg=libc_unstable_musl_v1_2_3' cargo ...\nRUSTFLAGS='--cfg=libc_unstable_gnu_time_bits=\"64\"' cargo ...\n```\n\nBeing able to set this via `RUSTFLAGS` makes it easier to only apply\nconfiguration to\nspecific targets (and notably, not the host if build scripts are used).\n\nThere are two other notable changes:\n\n- The 32-bit `windows-gnu` targets now respect\n`libc_unstable_gnu_time_bits`\n- uClibc now supports a similar configuration option:\n\n  ```sh\n  RUSTFLAGS='--cfg=libc_unstable_uclibc_time64'\n  ```\n\nAs a reminder, these options are under active development and may change\nin the future\n(hence the \"unstable\" in the name). It likely that we will harmonize\neverything under a\nsingle configuration option before considering them stable.\n\n##### Support\n\n- Add support for `aarch64-unknown-linux-pauthtest`\n([#&#8203;5065](https://redirect.github.com/rust-lang/libc/pull/5065))\n- Add support for new QNX targets\n([#&#8203;5241](https://redirect.github.com/rust-lang/libc/pull/5241))\n- Better document breaking change policy and recommended usage\n([#&#8203;5179](https://redirect.github.com/rust-lang/libc/pull/5179))\n\n##### Added\n\n- Android: Add `POSIX_SPAWN_*` constants\n([#&#8203;5104](https://redirect.github.com/rust-lang/libc/pull/5104))\n- Android: Add `getpwent`, `setpwent`, and `endpwent`\n([#&#8203;5160](https://redirect.github.com/rust-lang/libc/pull/5160))\n- Android: Add `preadv2` and `pwritev2`\n([#&#8203;5157](https://redirect.github.com/rust-lang/libc/pull/5157))\n- Android: Add `seccomp_notif*` structures\n([#&#8203;5224](https://redirect.github.com/rust-lang/libc/pull/5224))\n- Android: Add `timer_[create, delete, getoverrun, gettime, settime]`\n([#&#8203;5108](https://redirect.github.com/rust-lang/libc/pull/5108))\n- Apple: Add `PROC_PIDT_SHORTBSDINFO` and `proc_bsdshortinfo`\n([#&#8203;5110](https://redirect.github.com/rust-lang/libc/pull/5110))\n- Apple: Add `SIOC*` constants from `sockio.h`\n([#&#8203;5263](https://redirect.github.com/rust-lang/libc/pull/5263))\n- Apple: Add `_IOR`, `_IOW`, `_IOWR`\n([#&#8203;5264](https://redirect.github.com/rust-lang/libc/pull/5264))\n- Apple: Add `bpf_program` and `bpf_insn`\n([#&#8203;5235](https://redirect.github.com/rust-lang/libc/pull/5235))\n- Apple: Add additional `kqueue` constants\n([#&#8203;5077](https://redirect.github.com/rust-lang/libc/pull/5077))\n- Apple: Update `vm_statistics64` with recently added fields\n([#&#8203;5253](https://redirect.github.com/rust-lang/libc/pull/5253))\n- Apple: add `IN6_IFF_*` and `SIOCGIFAFLAG_IN6`\n([#&#8203;5239](https://redirect.github.com/rust-lang/libc/pull/5239))\n- Dragonfly: Add `O_*`, `POSIX_FADV_*`, `NI*`, and a few other missing\nconstants\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Dragonfly: add `fdatasync`, `dlvsym`, `reallocarray`, `qsort_r`,\n`pthread_*affinity_np`, `ftok`, `extattr_*`, and `dup3`\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Emscripten: Add `in6_pktinfo`\n([#&#8203;5256](https://redirect.github.com/rust-lang/libc/pull/5256))\n- FreeBSD: Add SOL\\_LOCAL\n([#&#8203;5185](https://redirect.github.com/rust-lang/libc/pull/5185))\n- FreeBSD: Add `DLT_*` constants\n([#&#8203;5235](https://redirect.github.com/rust-lang/libc/pull/5235))\n- FreeBSD: Add `PROC_LOGSIGEXIT_*` and `PPROT_*`\n([#&#8203;4657](https://redirect.github.com/rust-lang/libc/pull/4657))\n- FreeBSD: Add `SO_RERROR`\n([#&#8203;5260](https://redirect.github.com/rust-lang/libc/pull/5260))\n- FreeBSD: add `IN6_IFF_*`, `in6_ifreq`, and `SIOCGIFAFLAG_IN6`\n([#&#8203;5239](https://redirect.github.com/rust-lang/libc/pull/5239))\n- FreeBSD: add `_IO*` helpers from `sys/ioccom.h`\n([#&#8203;5239](https://redirect.github.com/rust-lang/libc/pull/5239))\n- Glibc: Add `PTHREAD_*_MUTEX_INITIALIZER_NP` for riscv64\n([#&#8203;5094](https://redirect.github.com/rust-lang/libc/pull/5094))\n- Glibc: Add new fields to `struct tcp_info`\n([#&#8203;5215](https://redirect.github.com/rust-lang/libc/pull/5215))\n- Linux: Add `OPEN_TREE_NAMESPACE`\n([#&#8203;5145](https://redirect.github.com/rust-lang/libc/pull/5145))\n- Linux: Add `SECCOMP_IOCTL_*` constants\n([#&#8203;5224](https://redirect.github.com/rust-lang/libc/pull/5224))\n- Linux: Add `SO_DETACH_REUSEPORT_BPF`\n([#&#8203;5081](https://redirect.github.com/rust-lang/libc/pull/5081))\n- Linux: Add `futex_waitv`\n([#&#8203;5125](https://redirect.github.com/rust-lang/libc/pull/5125))\n- Linux: Add constants for `fsopen`, `fsconfig`, `fsmount`, and `fspick`\n([#&#8203;5145](https://redirect.github.com/rust-lang/libc/pull/5145))\n- Linux: Add fields to `statx` present since 6.16\n([#&#8203;4621](https://redirect.github.com/rust-lang/libc/pull/4621))\n- Linux: Add network entry API\n([#&#8203;5049](https://redirect.github.com/rust-lang/libc/pull/5049))\n- Linux: add `ifaddrmsg` and `rtattr`\n([#&#8203;5234](https://redirect.github.com/rust-lang/libc/pull/5234))\n- Linux: add `sockaddr_iucv`\n([#&#8203;5041](https://redirect.github.com/rust-lang/libc/pull/5041))\n- MacOS: Add `ENOTCAPABLE`\n([#&#8203;4925](https://redirect.github.com/rust-lang/libc/pull/4925))\n- Musl: Add `renameat2`\n([#&#8203;5113](https://redirect.github.com/rust-lang/libc/pull/5113))\n- NuttX: Add `F_SETFD`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `POLLRD*` and `POLLWR*` constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `SO_KEEPALIVE` and TCP keepalive constants\n([#&#8203;5111](https://redirect.github.com/rust-lang/libc/pull/5111))\n- NuttX: Add `TCP_MAXSEG`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `eventfd` and `EFD_*` constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `pipe2`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `strerror_r`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `netinet` structs and constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add socket structs, functions and constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- QuRT: Add POSIX timer functions\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- QuRT: Add missing pthread functions from QuRT SDK headers\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- QuRT: Add missing unistd process and file functions\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- QuRT: Add mqueue subsystem (message queues, select/pselect)\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- Redox: Add `*at` and `dirent` functions\n([#&#8203;5117](https://redirect.github.com/rust-lang/libc/pull/5117))\n- Solarish: Add IP TTL and IPv6 Hop Limit consts\n([#&#8203;5089](https://redirect.github.com/rust-lang/libc/pull/5089))\n- Solarish: Add `port_alert` and `PORT_ALERT*` constants\n([#&#8203;5203](https://redirect.github.com/rust-lang/libc/pull/5203))\n- Solarish: add AI\\_CANONNAME\n([#&#8203;5085](https://redirect.github.com/rust-lang/libc/pull/5085))\n- aarch64: Add SYS\\_sendfile and SYS\\_fadvise64 constants\n([#&#8203;5133](https://redirect.github.com/rust-lang/libc/pull/5133))\n\n##### Deprecated\n\n- Dragonfly: Deprecate compatibility aliases `CPUCTL_RSMSR` and\n`UTX_DB_LASTLOG`\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n\n##### Fixed\n\n- **breaking** NetBSD: Correct `ts` from `*const timespec` to `*mut\ntimespec` in \\_lwp\\_park\\`\n([#&#8203;5169](https://redirect.github.com/rust-lang/libc/pull/5169))\n- **breaking** Linux GNU: Change overflowing\n`PTRACE_*ET_SYSCALL_USER_DISPATCH_CONFIG` constants from `u8` to\n`c_uint`\n([#&#8203;4936](https://redirect.github.com/rust-lang/libc/pull/4936))\n- Fix the soundness bug in the representation of extern types\n([#&#8203;5021](https://redirect.github.com/rust-lang/libc/pull/5021))\n- Cygwin: fix `cpuset_t` typo in `CPU_ZERO`\n([#&#8203;5098](https://redirect.github.com/rust-lang/libc/pull/5098))\n- Dragonfly: ABI fixes including regex offsets, `ifaddrs`, pthread\nbarriers, process sizing fields, and `mcontext` alignment\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Dragonfly: Correct values of `CPUCTL_CPUID*`, `EV_HUP`, and\n`EV_SYSFLAGS`\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Emscripten: fix pthread type sizes for wasm64 (MEMORY64)\n([#&#8203;5156](https://redirect.github.com/rust-lang/libc/pull/5156))\n- Horizon: Fix the value of `POLLOUT`\n([#&#8203;5090](https://redirect.github.com/rust-lang/libc/pull/5090))\n- Linux: Correct the value of `EPIOC[GS]PARAMS` with nonstandard \\_IOC\n([#&#8203;5188](https://redirect.github.com/rust-lang/libc/pull/5188))\n- Make VxWorks shims `unsafe`\n([#&#8203;3727](https://redirect.github.com/rust-lang/libc/pull/3727))\n- NetBSD: Correct getmntinfo to link `__getmntinfo13`\n([#&#8203;5251](https://redirect.github.com/rust-lang/libc/pull/5251))\n- QNX: Fix the value of `PTHREAD_MUTEX_INITIALIZER`\n([#&#8203;5241](https://redirect.github.com/rust-lang/libc/pull/5241))\n- QuRT: fix type and definition inaccuracies against SDK headers\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- Windows: Correctly link to 32-bit time routines on 32-bit platforms\n([#&#8203;5059](https://redirect.github.com/rust-lang/libc/pull/5059))\n- uClibc: Fix constants accidentally removed\n([#&#8203;5141](https://redirect.github.com/rust-lang/libc/pull/5141))\n- uclibc: Fix build issues\n([#&#8203;5046](https://redirect.github.com/rust-lang/libc/pull/5046))\n- uclibc: Fix type of PRIO\\_PROCESS and friends\n([#&#8203;5046](https://redirect.github.com/rust-lang/libc/pull/5046))\n\n##### Changed\n\n- AIX, TeeOS: Drop unneeded `-> c_void`\n([#&#8203;5240](https://redirect.github.com/rust-lang/libc/pull/5240))\n- Apple: Change `AIO_LISTIO_MAX` to account for changes in macOS 27\n([#&#8203;5253](https://redirect.github.com/rust-lang/libc/pull/5253))\n- Glibc: Update the value of `MS_NOUSER`\n([#&#8203;5215](https://redirect.github.com/rust-lang/libc/pull/5215))\n- L4Re: Update definitions and test infra\n([#&#8203;5275](https://redirect.github.com/rust-lang/libc/pull/5275))\n- Linux: Update the value of `SW_MAX` and `SW_CNT`\n([#&#8203;5215](https://redirect.github.com/rust-lang/libc/pull/5215))\n- MacOS: Add `swapped_count` to `vm_statistics64`\n([#&#8203;4926](https://redirect.github.com/rust-lang/libc/pull/4926))\n- Windows: Windows-GNU now respects `libc_unstable_gnu_time_bits` for\n64-bit `time_t` config\n([#&#8203;5062](https://redirect.github.com/rust-lang/libc/pull/5062))\n\n##### Removed\n\n- Dragonfly: Remove FreeBSD-only `Elf32_Lword`, `ip_mreq_source`, and\n`IP_` constants\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Dragonfly: Remove private VM type bindings\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Linux: Remove `KERN_REALROOTDEV` and `VM_LAPTOP_MODE`\n([#&#8203;5177](https://redirect.github.com/rust-lang/libc/pull/5177))\n- VxWorks: Remove non-user-facing (kernel) API\n([#&#8203;5129](https://redirect.github.com/rust-lang/libc/pull/5129))\n\n##### Other\n\n- Print config information if `LIBC_BUILD_VERBOSE` is set\n([#&#8203;5272](https://redirect.github.com/rust-lang/libc/pull/5272))\n- Annotate `*LAST` constants as potentially changing\n([#&#8203;5120](https://redirect.github.com/rust-lang/libc/pull/5120))\n- Annotate `*MAX` constants as potentially changing\n([#&#8203;5122](https://redirect.github.com/rust-lang/libc/pull/5122))\n- BSD: Annotate `ELAST` constants as potentially changing\n([#&#8203;5118](https://redirect.github.com/rust-lang/libc/pull/5118))\n- FreeBSD: Annotate `RAND_MAX` as potentially changing\n([#&#8203;5119](https://redirect.github.com/rust-lang/libc/pull/5119))\n- Linux, L4re: Annotate `*NUM` constants as potentially changing\n([#&#8203;5123](https://redirect.github.com/rust-lang/libc/pull/5123))\n- QNX: Restructure to support new platforms\n([#&#8203;4984](https://redirect.github.com/rust-lang/libc/pull/4984))\n- Unix: Annotate `*COUNT` constants as potentially changing\n([#&#8203;5121](https://redirect.github.com/rust-lang/libc/pull/5121))\n- uClibc: Add unstable support of 64-bit `time_t`\n([#&#8203;5046](https://redirect.github.com/rust-lang/libc/pull/5046))\n- (internal) FreeBSD: Replace unstable env to set version with an\nunstable cfg\n([#&#8203;5201](https://redirect.github.com/rust-lang/libc/pull/5201))\n- (internal) Glibc: Remove public configuration for file offset bits\n([#&#8203;5268](https://redirect.github.com/rust-lang/libc/pull/5268))\n- (internal) Linux: Delete config via\n`RUST_LIBC_UNSTABLE_LINUX_TIME_BITS64`\n([#&#8203;5197](https://redirect.github.com/rust-lang/libc/pull/5197))\n- (internal) Replace `RUST_LIBC_UNSTABLE` env with `libc_unstable*` cfg\n([#&#8203;4977](https://redirect.github.com/rust-lang/libc/pull/4977))\n\n</details>\n\n<details>\n<summary>time-rs/time (time)</summary>\n\n###\n[`v0.3.54`](https://redirect.github.com/time-rs/time/blob/HEAD/CHANGELOG.md#0354-2026-07-20)\n\n[Compare\nSource](https://redirect.github.com/time-rs/time/compare/v0.3.53...v0.3.54)\n\n##### Added\n\n- `PrimitiveDateTime` has been renamed to `PlainDateTime`.\n- `Duration` has been renamed to `SignedDuration`.\n- Iteration is now possible over `Date`, `Month`, and `Weekday`.\nRelevant iterator methods have been\n  overridden to ensure maximum performance.\n\nFor both `PlainDateTime` and `SignedDuration`, a non-deprecated type\nalias has been added for\nbackwards compatibility. The new names should be preferred.\n\n##### Changed\n\n- The associated metadata type (for `powerfmt` implementations) for\nvarious types has been changed\nto `()` and made public. This guarantees that no additional metadata\nwill be present.\n\n##### Performance\n\n- More gains when parsing RFC 2822.\n\n</details>\n\n<details>\n<summary>tokio-rs/tokio (tokio)</summary>\n\n###\n[`v1.53.1`](https://redirect.github.com/tokio-rs/tokio/releases/tag/tokio-1.53.1):\nTokio v1.53.1\n\n[Compare\nSource](https://redirect.github.com/tokio-rs/tokio/compare/tokio-1.53.0...tokio-1.53.1)\n\n### 1.53.1 (July 20th, 2026)\n\n##### Fixed\n\n- signal: restore MSRV by removing `OnceLock::wait` from the Windows\nhandler ([#&#8203;8300])\n\n##### Fixed (unstable)\n\n- time: fix alt timer cancellation and insertion race ([#&#8203;8252])\n\n##### Documented\n\n- runtime: remove dead link definition in Runtime::block\\_on\n([#&#8203;8301])\n\n[#&#8203;8252]: https://redirect.github.com/tokio-rs/tokio/pull/8252\n\n[#&#8203;8300]: https://redirect.github.com/tokio-rs/tokio/pull/8300\n\n[#&#8203;8301]: https://redirect.github.com/tokio-rs/tokio/pull/8301\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNzIuNCIsInVwZGF0ZWRJblZlciI6IjQzLjI3Mi40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-21T15:43:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b1e9ab64362fe2cca00a0081fff7fcc3664ca63"
        },
        "date": 1784657407572,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4988996982574463,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58579673970729,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77320976212543,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 793.141796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 871.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53769.79547778404,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52426.14223691682,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002088,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 606057.9916108923,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13553345.460941873,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.560224837297405,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8913056254386902,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.61436117743628,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.05085498419551,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.330859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.8046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28273.327632670527,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28021.32587190801,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003549,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2664038.8065917185,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7147844.873281077,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.07183274516197,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "d25a5680f19dfa4f53f2e5c3179ce07ea2c8d3f8",
          "message": "chore: Fix flaky quiver WAL replay tests by disabling time-based segment finalization (#3533)\n\n# Change Summary\n\n## Problem\n`wal_replay_reads_from_rotated_files` intermittently fails in CI with:\n\n```\nassertion `left == right` failed: no segments should be finalized\n  left: 2, right: 0\n```\n\nBoth this test and `wal_replay_finalizes_segments_if_threshold_exceeded`\nset a large `target_size_bytes` and assert that ingesting 20 bundles\nfinalizes zero segments (so the data stays in the WAL). But segment\nfinalization also triggers on `max_open_duration`, which defaulted to 5\nseconds. On a loaded CI runner, ingesting the bundles occasionally\nexceeded 5 seconds of wall-clock time, causing time-based finalization\nand breaking the assertion.\n\nHere's a sample CI run that fails with these tests:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/29763886758/job/88433587864?pr=3528\n\n## Fix\n\nSet max_open_duration: `Duration::from_secs(3600)` in both tests'\n`SegmentConfig` so only size/stream limits (which the tests never hit)\ncan trigger finalization. This removes the wall-clock dependency and\nmakes the tests deterministic. It matches the convention already used\nelsewhere in this file.\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nTest-only change; no production behavior is affected.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-21T21:54:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d25a5680f19dfa4f53f2e5c3179ce07ea2c8d3f8"
        },
        "date": 1784689491682,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1338180303573608,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.37912442512962,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.16034275921164,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 62.229427083333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.0078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28657.1846578483,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28332.264348404035,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002405,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2664358.701271149,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7284502.959601358,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.03973747058565,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.727518618106842,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.50092144682652,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.72116085211485,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.9076822916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 869.19140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51763.18224958333,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51386.5954740556,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002107,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 585906.4072548823,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12979602.426504234,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.401930831372134,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1784743716182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.47598934173584,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.03461085921995,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.99182272832192,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.860416666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29222.368508133088,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28498.825730437133,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001981,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2721926.771511556,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7423600.518442099,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.51013775997448,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.482624053955078,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.52589200414907,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.88887513538604,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 783.37734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 847.3046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55150.94437825935,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53781.75368703613,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002599,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 620774.6022854776,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13875091.575134873,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.542476020731037,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1784773577581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5377023220062256,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58784255743502,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78596664607782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.45078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 873.74609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55718.29385261212,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54304.32944459095,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003632,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 624365.9517105863,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13969944.47318832,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.497535428508584,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6360185146331787,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.66636719776656,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.65489358408792,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.0640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28983.976382567424,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28219.95336531365,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002119,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2713681.260137826,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7328576.075957334,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.16179109187784,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1784830279906,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.4244155883789062,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.50690849809804,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.6528956695947,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 800.4510416666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 885.7265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51616.75760734297,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50881.52045915649,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003769,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 594561.2319261235,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13135928.482184669,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.685209611677946,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.297182321548462,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.52582792886335,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.8703184516925,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.420703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.61328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28982.306876631454,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28316.530456213008,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002125,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2694385.033548688,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7369496.54156671,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.15237178209824,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1784859220987,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3403823375701904,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 87.40826399108883,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.41129440840284,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.897265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.83203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 26322.379585779676,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 25706.33524045312,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002174,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2456756.1529973275,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6674539.841873648,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.57006590076753,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.722036361694336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56988401831029,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80042010966098,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.78359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 881.2109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55808.51583292927,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54289.387701592845,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008763,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 624999.1971195197,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13974568.515159123,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.512364084024883,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1784916758540,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6625423431396484,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.18361814183878,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.00534505358932,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.927734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27479.061590046702,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26747.419908943444,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002049,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2572799.4317020318,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6983843.905055761,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.18869559982394,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.4287362098693848,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.63491226629779,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.81545810613623,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 780.9947916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 859.00390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 54904.69224985141,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53571.202118973175,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009443,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 618324.1064584025,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13757970.42914735,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.542098776973537,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1784945876395,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.503253698348999,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56413746395725,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74130528920506,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.52890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 865.5234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 54806.79197577635,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53434.83894742694,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003512,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 615610.2476750889,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13769296.106701773,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.520765474389673,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.475487232208252,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.98500057176976,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.4794493000232,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.24856770833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.21484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27535.789192851054,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26854.144247936634,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001912,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2571578.3772697314,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6975708.270876115,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.7609504710738,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785002314178,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4155936241149902,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.60146426195553,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78545454545456,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 784.8033854166666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 826.3984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 60832.88590280781,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 59363.410564033446,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003729,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 675507.334283571,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15210920.528516117,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.379186739193875,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.434478759765625,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.52668532357434,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.17118452611219,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.39140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.0859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28615.657487812598,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27919.0153893906,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002116,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2688833.903285196,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7246081.170873751,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.30833558360263,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785032377680,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8432347774505615,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.68641451791544,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.1994369682908,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.26875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.1875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29201.623805594343,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28371.35313205538,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006937,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2730437.311757776,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7357134.866607246,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.23923466212088,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.510692834854126,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56994065569538,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 101.2300640086373,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.3208333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 866.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51433.43150382508,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50142.09606215769,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003774,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 586758.323099626,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12914602.677917551,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.70191055380418,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785088849477,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.371584177017212,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5418302387469,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74949961508852,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 779.4756510416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 834.98828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55376.76784265025,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54063.46116265014,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003502,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 623606.5817993546,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13902576.272429524,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.534714359541868,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.62395977973938,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.95972852315944,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.40840819482025,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.770703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.4921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29242.365411038227,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28475.057475597558,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001986,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2717767.5891354415,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7389405.4690026995,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.44379643358063,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785118753896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6942734718322754,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.91845319936692,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.38250331022665,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.062890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27910.781429715655,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27158.788613754466,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001903,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2620851.4454035633,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7047311.275575874,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.50104364655805,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.486748218536377,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.52559415731557,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.87423367803675,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 799.0153645833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 889.33203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51624.263620551246,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50340.498157395254,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00473,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 587329.1158120262,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13007761.22290775,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.667129593665827,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785176590920,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4595706462860107,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55313146914222,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.67340986658392,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 804.7076822916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 870.3203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52310.84943983243,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51024.22719961181,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002849,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 592811.2978878858,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13147379.771154499,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.61823177779351,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3553569316864014,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.80039538479045,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.88270307799121,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.346354166666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.0234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27595.66632376243,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26945.68988540832,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002175,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2601800.514134316,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7021692.463334223,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.5572054454337,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785204915182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.3708176612854004,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.89632551041902,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.11477991800108,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.36380208333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.21484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27741.762536312013,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27361.473582699564,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007002,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2631369.518211811,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7113556.958603031,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.17060683002119,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5255727767944336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.62103195548463,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80299892456598,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 777.4494791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 864.04296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55476.48842042225,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54075.38927718803,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.020021,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 622149.8101170236,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13961572.806154616,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.505230354013941,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785272770739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.042961359024048,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5447092979916,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7528078817734,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.8984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 853.05859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52136.83442520315,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50550.330752417816,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003643,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 587537.893379858,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13053052.480242273,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.622829853625758,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.4651379585266113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.1909845467574,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.19297742854943,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.563541666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.38671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28492.456715415057,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27505.153758102708,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001846,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2678768.4640768245,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7176886.995515711,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 97.39151024697289,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785291943367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3732590675354004,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.19738529463184,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.97176579781886,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.66484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.53125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28357.45945095792,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27684.463523782408,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001849,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2666786.8280113507,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7184593.6220396105,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.32792146109101,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.583625316619873,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53448821999702,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77967474685485,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.9092447916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 836.25390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51821.35213351175,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50482.48250156567,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008083,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 583153.3947128855,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12969567.89437196,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.551599006542508,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785348598639,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.746673107147217,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.502485300681,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.67910115854133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.8662760416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 853.75390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52221.6438890194,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50787.28607041521,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003856,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 592843.9556705356,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13097642.86505222,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.673078078016875,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.765043258666992,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.33179550695935,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.37557358053303,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.14921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28472.309279276255,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27685.037594814505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002158,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2649208.156201576,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7209022.0076288,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.69097196016743,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785380792033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.750943660736084,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.44934625443746,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.73151243627375,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.937109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.92578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28259.043431379847,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27481.653079376596,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002031,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2648928.4228305235,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7162631.035077259,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.38897686320014,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.547166585922241,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56700083701449,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80997153189197,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 785.5080729166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 856.1015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 56459.227222492984,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55021.11671752053,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007906,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 631322.9138477163,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14168195.516335478,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.474193028268406,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785440204176,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9320504069328308,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.04948135569101,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.46740230596612,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.250651041666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.8515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28059.016531470574,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27797.49236455929,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002103,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2615609.9906583335,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7136679.659088078,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.09517795184767,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.562166690826416,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54879706398177,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.73312403816558,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 798.1842447916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 866.7421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51257.16462451019,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 49943.87060489104,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003319,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 579048.0213153195,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12902088.140586488,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.59397568314645,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785464501077,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.642548084259033,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.3725132799818,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.88807033780657,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.44752604166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27340.665542630046,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26618.175331146624,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002197,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2547423.0132930283,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6930209.278697808,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.70239062601041,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5287444591522217,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56271511715862,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74340136054423,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.5236979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 843.76171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55543.26834436513,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54138.72094592346,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003671,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 623732.4766613712,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13955061.2478591,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.52100503601456,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785525862083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5232057571411133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56813244958045,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77951762523192,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 790.0712239583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 868.7109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55725.89392649415,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54319.81497136799,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00801,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 626698.9434497138,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13997521.612345634,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.53720688813184,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.7297635078430176,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.95760188248228,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.6970053393175,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.08229166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28104.079591393383,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27336.90471637054,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001965,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2623879.8970201476,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7154355.601705929,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.9830648072184,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785550611207,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.76141095161438,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.59863291408988,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 101.4812664092664,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 794.4294270833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 867.4375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51739.536722404926,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50310.79553891698,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00247,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 583474.2476144962,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12972412.114919733,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.597396569950092,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.454725980758667,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.30976013916099,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.25567175690266,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.27174479166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.91015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27700.80517162583,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27020.826318968146,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001866,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2588645.7930878643,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7029430.372027121,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.8018738039362,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785607241306,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.816155195236206,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.98099567119426,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.82871287128712,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.823177083333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.5625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28782.439638582557,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27971.881473080262,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001863,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2652813.745176285,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7293109.087753537,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.83858809173474,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.517977714538574,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56795872341941,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.81564356435644,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 779.8115885416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 831.8984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55250.46951733002,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53859.274960914525,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00311,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 621478.4316566924,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13867519.058624243,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.538930520466474,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785637113414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4184389114379883,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.46056673582098,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59402128644146,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 809.0997395833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 883.9609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 58694.90065989307,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 57275.4003436699,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003509,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 651794.1038552638,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14671101.350118631,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.380000837083635,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.1603846549987793,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.11945726198381,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.24568778979908,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.266015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.79296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28077.328632923753,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27189.977051987822,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002147,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2629766.7517627305,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7118096.457108235,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.71824094351237,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785693615407,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9458481073379517,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54964829551952,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.71755890305137,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.2057291666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 887.546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51119.743712307354,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50636.228567202066,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002257,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 579552.7295431743,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12850943.85232797,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.44541657114172,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5407445430755615,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.22463593873746,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.1988712794743,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.66184895833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.47265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29143.989189308286,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28403.514871425876,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002081,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2707842.167412018,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7350625.685385702,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.33475626765211,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785723476334,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.433568000793457,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53247086807629,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.6517723276261,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 780.4828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 840.29296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55128.57303490986,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53786.98165387953,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003367,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 621828.5425085848,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13811376.18591375,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.5609488279165,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.7212469577789307,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.8569933456281,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.85254562326014,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.905078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.95703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28764.026337708467,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27981.286166796584,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002031,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2698113.8080931986,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7293671.940755962,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.42565363184983,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785786104833,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4999849796295166,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.6153814478463,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.9009329626197,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.4875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 877.046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55536.29977915642,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54147.900617705716,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002197,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 624597.5845750748,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13910767.14588339,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.535028642843429,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.536369562149048,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.73591216081911,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.39425564840607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.864453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.5703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28617.34841959189,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27891.50673397657,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002065,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2669836.517222252,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7259643.230601269,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.72220470864487,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785810864778,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.311155319213867,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54061061064952,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.70903076209615,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 790.7932291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 872.90625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53528.46923277117,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52291.34313043701,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003584,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 606888.0135575051,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13395222.793903524,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.605898361487224,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.8776538372039795,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.14605815499581,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.5225721581676,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.212109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.82421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27162.390798372628,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26380.75125459813,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002082,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2522796.141335629,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6877032.733241101,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.63018569821467,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785872249098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.056751012802124,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.38437491070884,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.88000618333591,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.82643229166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.75,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29044.069621636452,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28737.146120085952,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001922,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2708533.868382369,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7386436.3849376105,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.25201295438407,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6821088790893555,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56389834490471,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77829039993811,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 789.3895833333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 856.484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52882.38320340448,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51464.02005540759,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002969,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 597974.7100708153,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13325901.007088017,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.619277107132694,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785896095188,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.469449996948242,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.45531319590516,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.98305497725345,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.62486979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.59375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28422.37502558872,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27720.49869051915,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002023,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2700609.5507833124,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7228103.267176381,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 97.42283430517661,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.795238494873047,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51358894643568,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.8427836564455,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 781.630078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 850.50390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55581.353863186574,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54027.72244807407,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003936,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 622907.276617647,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13988477.50984629,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.529400988840903,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785954967457,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4606213569641113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53586927220294,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.94286948472693,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 798.4408854166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 862.41015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52414.99100407364,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51125.256579472116,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003826,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 596609.3565359714,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13180038.611952376,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.669562100066267,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3885998725891113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.24551771302981,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.68270286948473,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.31197916666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.86328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29150.741130635768,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28454.44657132136,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001905,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2710096.0248622815,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7413950.222559396,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.24332227194783,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1785983335550,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.446242094039917,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56409871890601,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.86532427424335,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 791.3325520833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 885.06640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55833.31573716717,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54467.497709012496,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003601,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 629822.2274594272,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13977868.546787748,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.563267158410572,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0886969566345215,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.3964235430487,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.72557059961315,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.190494791666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.92578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28560.782234443475,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28249.841863230304,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001858,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2690466.6577195455,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7215185.904749983,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.23829091664504,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786071522350,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1315678358078003,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.6084218396812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 101.19039853132409,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 779.91328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 834.3359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55316.584200925594,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54690.63951967633,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003705,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 617772.5194998459,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13828165.787634669,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.295763313895547,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.745424270629883,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.15918900669426,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.34686885880078,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.098307291666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.77734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28045.72666739453,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27275.7524743629,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002011,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2646684.947144256,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7128527.600230494,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 97.0343512844214,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786125471526,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.87778377532959,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.44648346325347,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.00818956171597,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.43776041666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.58984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27914.1430358088,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27110.83436405169,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001842,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2628299.6427845275,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7065046.610688039,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.94646824553615,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.7277703285217285,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56066712320207,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77302038295244,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 794.3923177083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 895.4921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51767.95689130136,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50355.84595594627,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002368,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 586569.0600867934,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12957798.311118772,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.648479912341307,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786157753606,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.754962205886841,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.40238888980556,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.95415570345148,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.98619791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.65625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28212.412668266592,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27435.171365574435,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001958,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2628560.662607078,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7119621.432256235,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.80988679026031,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.9615252017974854,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56673752547256,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.8641976071015,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 789.8205729166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 848.47265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52628.85453153953,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51070.23781362616,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003206,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 592650.016360712,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13204143.973339569,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.604606552323235,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786211209587,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5832908153533936,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53723033571885,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.67534136546186,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 791.65,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 844.03125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52383.548099436484,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51030.32866110038,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003572,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 597318.8363054134,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13175236.02098198,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.705173217133131,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.8711698055267334,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.21318433009914,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.68808130582076,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.53763020833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28447.446247134525,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27630.67171688696,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001871,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2635816.8285765583,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7190608.389701327,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.3945982777405,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786240781523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.63822078704834,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.85085080987027,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.63164400494438,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.35182291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.03125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 25290.83375520833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 24623.605729251318,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001976,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2375200.3030333403,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6394447.938167515,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.46029623564633,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.648946523666382,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56923932561565,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82101109741059,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.940234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 854.6953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55048.80006464801,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53590.58681517577,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004941,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 619370.2499477896,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13793238.898111504,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.557444819253154,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786297664023,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6422338485717773,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.61674972657433,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.88730870304529,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.4791666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 874.54296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 54819.2345199618,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53370.782116300106,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002662,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 616811.0275409537,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13736340.648899505,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.55709178473088,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.168104648590088,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.69227448594577,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.58646449017485,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.32799479166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.08984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27620.694878551858,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26745.642332483934,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002111,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2581844.47783803,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6928433.584020929,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.53327617793832,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786327254314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8417468667030334,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55863412482084,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.96387096774193,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.1337239583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 861.25390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55066.78226435635,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54603.25934954534,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003506,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 617243.7556750322,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13788308.152899064,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.304155887906198,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.748706102371216,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.73417254752682,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.78598065764022,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.444661458333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.95703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28657.366624832524,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27869.65986347394,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002024,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2669034.4313719207,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7260858.00388377,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.76846091580634,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786385539661,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.373774766921997,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.4516581732221,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.8338509124652,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 778.4901041666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 865.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 64437.78965965548,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 62908.181691068254,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00361,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 709264.9246376592,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16128049.02379081,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.2746053942672,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6802725791931152,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.98892022727452,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.52949010477299,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.536328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 26417.443505777916,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 25709.384022248185,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002021,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2467336.494525803,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6681481.908209524,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.97026876998058,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786413881513,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4723995327949524,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5398776874166,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74777477407895,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 795.9424479166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 852.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51958.4946005713,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51713.04291872981,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003663,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 590740.3958020423,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13106019.394219259,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.423431352326855,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3535027503967285,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.44531827103424,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.53371321812237,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.024479166666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.38671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28975.691151731226,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28293.747443836055,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00202,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2744896.6573268245,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7326927.24277927,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 97.01424891755775,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786479138558,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6166656017303467,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.7008228435541,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.55429633070135,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.80130208333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28820.677637079087,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28066.53685001043,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002059,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2714181.428499301,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7294928.022995098,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.70524878092654,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8973239064216614,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54269741321211,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74920358115997,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.60078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 886.140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 54927.130972858846,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54434.256717324126,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003134,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 621901.5695395271,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13842885.410637535,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.424819719116364,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786501359693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.9696860313415527,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54327893255343,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78002616793657,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 799.3106770833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 862.09765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51514.60007262709,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 49984.77811065146,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002407,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 582877.2530418788,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12897153.618124595,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.6610951388353,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6831431984901428,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.4911659088005,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.45399216168447,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.05494791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.60546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28929.052091393136,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28731.425233633177,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001966,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2730069.8663577135,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7307636.091727578,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.02034250503792,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786557526148,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.7422640323638916,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58912216821338,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.88247218788628,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.4869791666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 838.56640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52350.164403381445,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50914.58463714329,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003632,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 594720.2233419793,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13155124.56628462,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.680743888620826,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.509474754333496,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.53961256136071,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.41455778120185,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.282552083333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.94921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28584.093407837416,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27866.78282520473,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001903,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2683520.091377965,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7291429.262633339,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.29816646616257,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786588578945,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9261189699172974,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53676162447644,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.83232622798887,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 806.587890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 890.96875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52762.53686890384,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52273.893013761146,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002801,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 597987.5655933383,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13187402.389074238,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.439507010429042,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8655802607536316,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.35843149670391,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.70571228288807,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.620182291666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.30078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28839.09349116126,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28589.468004389062,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001886,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2722747.081331523,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7299090.820708831,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.23601771510846,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786653522205,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.060776948928833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53407682481948,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.72040609137056,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 802.28984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 877.42578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52593.65655638976,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52035.755153613376,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003434,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 596374.807761154,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13191167.753406018,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.46086582198359,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7952980995178223,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.01827544280745,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.3053301339113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.06015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.59765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28072.7007618095,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27849.439125971698,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001352,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2644760.397295466,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7076564.566981065,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.96637922697079,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786673440761,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8233467936515808,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.52868078200491,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.98759517177345,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.40833333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.33984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28454.105592614207,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28219.829623355505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001886,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2677523.3058132394,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7197217.1321165515,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.88091677198673,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7622635960578918,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56128165184515,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7596875725002,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 785.2240885416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 842.32421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55072.75486735991,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54652.95531452168,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002446,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 621056.6824837994,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13820487.194999266,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.363643171895962,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786730463508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8641825914382935,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.25055364221764,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.50237623762376,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.350390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.88671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28849.051347028206,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28599.742878455014,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001973,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2699699.11557966,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7309677.865414537,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.39592261556375,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.040090799331665,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.57963402500938,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.73340278317656,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 785.8690104166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 857.6328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 54332.75877370059,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53767.648751155684,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002475,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 610991.3820159846,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13600709.657464255,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.363550317101264,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786759096966,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7257716059684753,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.33181841583243,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.04786382456679,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.723697916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.0234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28564.080712669063,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28356.770717929063,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001931,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2661606.721191459,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7251721.776932711,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.86141841280296,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.1254005432128906,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54629872304352,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.75470928885801,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.9938802083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 873.625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55139.928442751996,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54519.38342807332,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003705,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 618315.7491010068,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13812983.503172493,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.341209496925847,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786816058214,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.40668803453445435,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.60663631394257,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.79730427390061,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 784.1100260416666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 844.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55169.61490128899,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54945.24669037307,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004044,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 622390.099601884,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13894158.474532042,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.327460282582237,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8671593070030212,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.78829131769945,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.75190860624524,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.30703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.91015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28642.262112730317,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28393.888068586235,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002244,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2649423.6961215013,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7226373.785665458,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.30964782708672,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786845478292,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6486299633979797,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.15551708268404,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.47378758449227,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.231510416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.85546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28890.72242155552,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28703.328546212015,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001961,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2719000.27307311,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7326777.062900709,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.72769921772493,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0046756267547607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55161717642638,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82692004634994,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 797.03359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 853.53125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52394.95036603803,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51868.55102055577,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003874,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 591603.9630223322,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13206700.626360115,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.405831691497927,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786902469700,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.9075829982757568,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.46841375271474,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.74800706550955,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.215494791666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28132.437784063874,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27595.788200742274,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00191,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2594127.70826515,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7095021.829214378,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.00447957472629,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0473531484603882,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.57147565404321,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82615634926783,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 799.5266927083334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 874.2578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51631.1863365015,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51090.425489806716,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002495,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 585605.75642993,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12952744.325019337,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.462142873458097,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786931853571,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9948931932449341,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55204414541699,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.81996895374108,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.0904947916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 855.5703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55152.718515877394,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54604.00788087399,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002482,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 617491.1313747218,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13804719.556229342,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.308531284404287,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8035569190979004,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.89079567222052,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.62537860623405,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.613932291666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.64453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28675.77246049544,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28445.346312617483,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001871,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2679537.7842162056,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7248562.706423138,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.19951350803716,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1786988969928,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6668432354927063,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.38687045301708,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.88822915855842,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.94348958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.37109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28314.29544594832,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28125.4834775528,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001493,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2637562.486164215,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7210773.918607065,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.77838742822954,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5938845276832581,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53239993394823,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.81351421782784,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 793.8927083333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 869.3203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52649.80755122098,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52337.12849029543,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004018,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 597533.2835881714,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13239209.83141396,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.417005495419346,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787019092725,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9978599548339844,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.50565603946858,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74106763771597,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 802.5930989583334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 879.5,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53736.002982004014,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53199.79290231447,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002602,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 605746.5880165551,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13499312.30844727,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.386258385043487,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.052944302558899,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.19558597907223,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.75767718053521,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.78541666666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.69921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28897.2882147429,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28593.01588592546,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00217,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2705253.277677448,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7321223.320353897,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.61237976680431,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787090379806,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8605612516403198,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.51084963780545,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.51465758724754,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.694010416666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.4609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29043.930215237073,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28793.989421389648,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00221,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2713438.9491634634,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7318179.810066676,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.23629735543982,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0421839952468872,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.52965127793479,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7500154655119,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.43828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 862.8359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55704.59462662398,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55124.05025749355,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004027,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 623717.12478036,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13935701.764325045,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.314791309181278,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787107201719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8222119808197021,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.85449657786556,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.58056364199442,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.674739583333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29022.335932388454,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28783.71079980551,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002062,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2735515.816111137,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7351757.962259996,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.03694068971888,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9496801495552063,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53528983263185,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78124614435534,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 798.1709635416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 869.51953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52361.53366647128,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51864.26658555204,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003972,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 589487.0233150625,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13067359.128219927,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.365956989725898,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787161976535,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6836581230163574,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.90843886912589,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.66835841630007,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.45390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.87890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27790.75235091432,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27600.758601708672,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001974,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2604768.4435791853,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7074314.677508715,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.3730743479613,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8751699924468994,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.551747930646,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7575808197989,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 784.3673177083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 827.4921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55139.21113379751,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54656.6492905516,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002672,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 617217.9389467378,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13826972.887492811,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.292641370414838,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787191089028,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.0889675617218018,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.21167148435515,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.63854736516767,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.48359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.15625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28249.330490867367,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27941.704448271244,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001422,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2644760.9607330123,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7170235.846523144,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.65281424149642,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.1511324644088745,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56550390707389,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77140132287342,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 794.844140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 854.34375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51872.94669585995,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51275.820332111856,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002375,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 591911.2791781334,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12976974.37392802,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.543672540865126,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787257969072,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1387686729431152,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53202463569025,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.73811448328851,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 800.1244791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 877.31640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52411.111975704094,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51814.27060516418,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002543,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 590192.9932009574,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13151782.056809302,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.39054909598852,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9683077931404114,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.39898963488864,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.12776207029741,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.14765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.8125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28502.282549183354,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28226.292724008243,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002212,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2655464.877796504,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7237583.744199457,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.07770633434492,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787292473932,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9570167660713196,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54736431491725,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.91780588098378,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 782.9044270833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 866.35546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 56064.67417893193,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55528.125864318594,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003916,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 624397.9458454625,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14108413.846525876,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.244714928271868,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.199400544166565,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.53245139127263,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.75118981050016,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.645442708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.56640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28910.265964280698,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28563.516097645068,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002907,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2700042.1293615713,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7335526.510264853,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.52765269273615,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787334691842,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8719736933708191,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.30114176628956,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.962348941431,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.123046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.78515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28912.42018272923,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28660.311478495805,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001895,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2711183.247615441,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7333966.023277093,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.59713128552968,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0283305644989014,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.46511921667734,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.68829045963692,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.0509114583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 876.515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55355.05446894029,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54785.82151983864,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003554,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 622344.2279652759,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13861591.366223643,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.35958557708069,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
        "date": 1787362459669,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7302457690238953,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53087558526062,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.75063449232904,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 802.1703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 868.6015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52284.69257590696,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51902.8858290501,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004178,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 589902.1536110651,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13104921.170097211,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.365498164283117,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.231982707977295,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.65645795473046,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.40856746337703,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.7734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29369.004390751154,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 29007.18332315201,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002034,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2710515.2262658435,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7408949.107520128,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.44289640498988,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}