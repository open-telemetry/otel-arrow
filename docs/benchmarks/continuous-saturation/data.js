window.BENCHMARK_DATA = {
  "lastUpdate": 1765570628662,
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
          "id": "3b6b188e013d94a185e56ab21d57f48a4667974b",
          "message": "Add perf test to saturate the df engine (#1612)\n\nTrying to work towards\nhttps://github.com/open-telemetry/otel-arrow/issues/1531\nThis adds a continuous tests, matching the existing back pressure ones.\nThere is only one difference - the load generator is not capped at 100K\nRPS - it goes as much as it can go.\n\n\nNeed to try tweaking few settings to get it right. The engine and\nload-generator is now running in just 1 core, we might need to run\nload_generator on more than one core to fully saturate the engine..\nAlso, in future, we want to run engine of multiple cores to see if we\ntruly scale with number of cores..\n\nAdd as continuous to begin with, once things are stabilized, we can move\nthis to nightly runs.",
          "timestamp": "2025-12-12T19:52:23Z",
          "tree_id": "a5761259a6d63ecf008583c74a872c1c4d7f95e9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b6b188e013d94a185e56ab21d57f48a4667974b"
        },
        "date": 1765570627680,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1001000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.50298309326172,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05009308118185862,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06937203408210689,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05009308118185862,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.06937203408210689,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.01028645833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.1640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1006000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16766.343914546313,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33172919754628,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001155,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10681.333568296035,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 657.114062629953,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 361000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.997342586517334,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 54.947276736753395,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.9796264029055,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 54.947276736753395,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.9796264029055,
            "unit": "%",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.40078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.53515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 9031000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 8670000,
            "unit": "count",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 150513.796870273,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 144497.24491919688,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001144,
            "unit": "seconds",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2820683.117041999,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2703093.384457995,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}