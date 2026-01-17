window.BENCHMARK_DATA = {
  "lastUpdate": 1768609059526,
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
          "distinct": false,
          "id": "b793a1e733d3f1c1a626430c0d93e00e9e6d98e2",
          "message": "PerfTest - add passthrough scenario (#1810)\n\nAll existing tests add a dummy processor in the middle to force\nconversion into internal format. But there are real scenarios where\nengine can act as \"pass-through\" without having to do this conversion.\nThis adds a perf-test to continuously measure the throughput when\noperating as pass-through.\nModelled after saturation tests - where we put as much load as required\nto hit 100% CPU in engine. Local run shows it can do twice (minimum) the\nthroughput!",
          "timestamp": "2026-01-16T22:14:05Z",
          "tree_id": "1cf5cc0d17331750aa5a89bae24befe3b9d85c4a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b793a1e733d3f1c1a626430c0d93e00e9e6d98e2"
        },
        "date": 1768603839941,
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
            "value": 97.88627355647486,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.35741167310366,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.70872395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 565936.8956417819,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 565936.8956417819,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001149,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10151891.240092983,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10137779.0385659,
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
          "id": "c68e70eda406b6341cbd0ae73cf4521a56639d47",
          "message": "Update batch size variation perf tests (#1809)\n\nModified to use 10, 100, 512, 1024, 4096, 8192 as sizes.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T23:41:49Z",
          "tree_id": "2ebd0b963e9f0a0c3a4e59c7f3429710cd874ea8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c68e70eda406b6341cbd0ae73cf4521a56639d47"
        },
        "date": 1768609059170,
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
            "value": 96.16226525157725,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8441080331295,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.4703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.8359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 570846.5889158417,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 570846.5889158417,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000821,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10353526.431908403,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10311706.708369318,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}