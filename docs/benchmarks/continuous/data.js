window.BENCHMARK_DATA = {
  "lastUpdate": 1760491816009,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "4ed8aa6d1fdb70a2bbdd72574f0929d51e654251",
          "message": "Print ACK/NACK in debug_processor at detailed level (#1265)\n\nPart of #1249 \nThis prints the ACK or NACK by after storing the u128 of SystemTime\nmicroseconds as two u64 in the CallData.\nManually tested, e.g., changed fake-debug-noop.yaml to use `detailed`\nlevel, then\n\n```\ncargo run --bin df_engine -- --num-cores=1 --pipeline ./configs/fake-debug-noop.yaml \n```\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-15T00:47:48Z",
          "tree_id": "37ed568ff25aadcf1c4873c07609febf8d985d2a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4ed8aa6d1fdb70a2bbdd72574f0929d51e654251"
        },
        "date": 1760491813718,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.19379590352533,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.97888933726525,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.974609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.48828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18470544.444100913,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5756411.89193423,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 52.014536655224894,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.21472411209166,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.761328125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.859375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14319618.340186846,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13790399.392755594,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 17.34289827812465,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.300790239219634,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.57265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.64453125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5774355.621784603,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5743162.417160369,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 66.3652822349325,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 67.08935043250732,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.333203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.03515625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5862431.535521744,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13928106.743467327,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      }
    ]
  }
}