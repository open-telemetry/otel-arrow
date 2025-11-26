window.BENCHMARK_DATA = {
  "lastUpdate": 1764152520548,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "0dc05d926ef58004a70332e3cb286f1c7825de7d",
          "message": "[otap-dataflow benchmark] filter processor benchmarks + internal telemetry (#1448)\n\nAdd filter processor scenarios to the nightly benchmark suite\nCollect internal metrics inside the filter processor tracking, number of\nsignals before and after the filtering\n\n\n```rust\n/// Pdata-oriented metrics for the OTAP FilterProcessor\n#[metric_set(name = \"filter.processor.pdata.metrics\")]\n#[derive(Debug, Default, Clone)]\npub struct FilterPdataMetrics {\n    /// Number of log signals consumed\n    #[metric(unit = \"{log}\")]\n    pub log_signals_consumed: Counter<u64>,\n    /// Number of span signals consumed\n    #[metric(unit = \"{span}\")]\n    pub span_signals_consumed: Counter<u64>,\n\n    /// Number of log signals sent\n    #[metric(unit = \"{log}\")]\n    pub log_signals_sent: Counter<u64>,\n    /// Number of span signals sent\n    #[metric(unit = \"{span}\")]\n    pub span_signals_sent: Counter<u64>,\n}\n\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-11-25T03:30:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0dc05d926ef58004a70332e3cb286f1c7825de7d"
        },
        "date": 1764066304357,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 36.73575747808186,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 37.531594367940585,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.463671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.70703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 373184.88544259855,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13748368.255068475,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.11056827906434,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 31.354828518833628,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.957421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.07421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 334924.53053086984,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13755878.420081716,
            "unit": "bits/sec",
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
          "id": "261dd531b0111ca72990d1aec9f69cb25160e1ce",
          "message": "OTLP Exporter Optimizations (#1474)\n\nSupport for multiple simultaneous client connections to improve\nthroughput and the ack/nack system.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-11-25T23:19:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/261dd531b0111ca72990d1aec9f69cb25160e1ce"
        },
        "date": 1764152519573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.067793589774684,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.14354063957979,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.29296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.58984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 242858.67220339738,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13955142.051708246,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 26.298659767537497,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 26.843679566261326,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.378515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.26171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 209184.38100461414,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13934851.875845104,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}