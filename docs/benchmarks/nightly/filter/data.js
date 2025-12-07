window.BENCHMARK_DATA = {
  "lastUpdate": 1765103241088,
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764238888313,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 26.212712470792194,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 26.983411369006255,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.056640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.3203125,
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
            "value": 209316.10490026668,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13880809.512416402,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
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
            "value": 30.95041382900027,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.11623829341548,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.752734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.515625,
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
            "value": 238573.5665390667,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13851234.689957434,
            "unit": "bits/sec",
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764325278392,
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
            "value": 30.628480486680665,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 31.199239520680326,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.78203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.3828125,
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
            "value": 239715.0240716356,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13850897.790708894,
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
            "value": 26.421081400249335,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 27.333864333307567,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.629296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.53125,
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
            "value": 209469.0918127711,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13945223.913945306,
            "unit": "bits/sec",
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764412053817,
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
            "value": 32.38576674610646,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.166940790348775,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.140234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.3359375,
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
            "value": 239251.16087389365,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13812634.317326397,
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
            "value": 26.16158408370556,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 27.045839648175296,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.258984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.296875,
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
            "value": 206843.3059020142,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13713111.025205994,
            "unit": "bits/sec",
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764498062198,
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
            "value": 31.33893540267248,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.093204642939504,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.44609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.59375,
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
            "value": 253422.6028356283,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13882370.946449343,
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
            "value": 26.415875357524865,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 27.042039130771016,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.671484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.4453125,
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
            "value": 207723.48011795597,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13814755.646279573,
            "unit": "bits/sec",
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764584551139,
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
            "value": 32.611761499459774,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.182628132072544,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.96484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.6171875,
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
            "value": 251016.19447308523,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13808857.523726916,
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
            "value": 26.303094190772647,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 27.159754198614316,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.923046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 207713.3163071948,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13801517.738200601,
            "unit": "bits/sec",
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
          "id": "e8cecc80e28d47b04d086c50dfcd1e4e6ac83fbb",
          "message": "Update all patch versions (#1487)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.1` -> `v1.18.2` |\n[![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.1/v1.18.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.12.4`\n-> `==2.12.5` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.12.5?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.12.4/2.12.5?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.2`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.2)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.1...v1.18.2)\n\n##### What's Changed\n\n- Fix invalid encoding on level 9 with single value input by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1115](https://redirect.github.com/klauspost/compress/pull/1115)\n- flate: reduce stateless allocations by\n[@&#8203;RXamzin](https://redirect.github.com/RXamzin) in\n[#&#8203;1106](https://redirect.github.com/klauspost/compress/pull/1106)\n- build(deps): bump github/codeql-action from 3.30.5 to 4.31.2 in the\ngithub-actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1111](https://redirect.github.com/klauspost/compress/pull/1111)\n\n`v1.18.1` is marked \"retracted\" due to invalid flate/zip/gzip encoding.\n\n##### New Contributors\n\n- [@&#8203;RXamzin](https://redirect.github.com/RXamzin) made their\nfirst contribution in\n[#&#8203;1106](https://redirect.github.com/klauspost/compress/pull/1106)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.1...v1.18.2>\n\n</details>\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.12.5`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.12.5):\n2025-11-26\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.12.4...v2.12.5)\n\n#### v2.12.5 (2025-11-26)\n\nThis is the fifth 2.12 patch release, addressing an issue with the\n`MISSING` sentinel and providing several documentation improvements.\n\nThe next 2.13 minor release will be published in a couple weeks, and\nwill include a new *polymorphic serialization* feature addressing\nthe remaining unexpected changes to the *serialize as any* behavior.\n\n- Fix pickle error when using `model_construct()` on a model with\n`MISSING` as a default value by\n[@&#8203;ornariece](https://redirect.github.com/ornariece) in\n[#&#8203;12522](https://redirect.github.com/pydantic/pydantic/pull/12522).\n- Several updates to the documentation by\n[@&#8203;Viicos](https://redirect.github.com/Viicos).\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.12.4...v2.12.5>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi4xOS45IiwidXBkYXRlZEluVmVyIjoiNDIuMTkuOSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-12-02T00:43:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8cecc80e28d47b04d086c50dfcd1e4e6ac83fbb"
        },
        "date": 1764670909679,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.89691362740091,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.020968738089703,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.6171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.26171875,
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
            "value": 232977.3283463955,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13701370.560711157,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
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
            "value": 25.734286582620314,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 26.697204559649258,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.6015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.80859375,
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
            "value": 237961.8920252482,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13810159.171309924,
            "unit": "bits/sec",
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
          "id": "89d92af81eab57dc4f7318396d950c313927457f",
          "message": "[otap-dataflow] fix exclude filter returning empty batch #1483 (#1504)\n\nFixes #1483\n\nAdds check for fields where an empty vec provided so nothing should be\nexcluded",
          "timestamp": "2025-12-02T20:03:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/89d92af81eab57dc4f7318396d950c313927457f"
        },
        "date": 1764757863562,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 21.601959315710467,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.367379820377828,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.87890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.93359375,
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
            "value": 232467.34920275593,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13606688.907473397,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
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
            "value": 25.976715064901963,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 26.5503572015444,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.972265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.08984375,
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
            "value": 236321.92849833472,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13710913.763132554,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "7b23c8139eec1c18112a0a0506f692069e691ad6",
          "message": "Minor refactor otap_df_otap::otap_grpc::otlp::server_new (#1516)\n\nFrom https://github.com/lquerel/otel-arrow/pull/5\n\n---------\n\nCo-authored-by: querel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-12-03T22:02:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7b23c8139eec1c18112a0a0506f692069e691ad6"
        },
        "date": 1764843724483,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03948225333846749,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.051946113733075436,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.78515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.78515625,
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
            "value": 46140.73435495276,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63627.260103671135,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03369330359505027,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.053956885119506554,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 8.71953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 8.79296875,
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
            "value": 46135.64571669382,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63620.24380352128,
            "unit": "bits/sec",
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
          "id": "5a5ca878dcea95586eb414b1b0fe4979319972d7",
          "message": "  Add TLS to OTLP/OTAP receivers (server-side, with reload) (#1510)\n\nThis PR adds server-side TLS for receivers, following the Go collector's\napproach. Plan is to submit changes incrementally to make the review\neasier and to get feedback on the implementation direction before\nraising further PRs.\n\n**TL;DR:** This enables TLS for receivers. mTLS and exporter client-side\nTLS are not part of this PR.\n\n  ### What's in this PR\n  **Server-side TLS for receivers:**\n  - OTLP and OTAP receivers can now accept TLS connections\n- Certificate and key configuration via `cert_file`/`key_file` or\n`cert_pem`/`key_pem`\n- Async certificate hot-reload without service restart or connection\ndrops\n\n  **Certificate reload behavior:**\n  - Similar to the Go collector's implementation\n  - File-based certificates are monitored via mtime\n- Reload triggered asynchronously when files change and\n`reload_interval` has elapsed\n- Active connections continue using the current certificate while reload\nhappens in background\n- Zero downtime during rotation assumes the new certificate is deployed\nbefore the old certificate expires, creating an overlap window where\nboth are valid.\nThe `reload_interval` should be set appropriately based on your\ncertificate rotation schedule (e.g., if certificates are rotated daily,\nuse `reload_interval:\n  \"5m\"` to ensure timely pickup).\n\n  **Implementation notes:**\n  - Uses `rustls` for TLS\n  - Certificate reload runs in spawned async tasks\n  - Compare-exchange used to ensure single reload task at a time\n- **NUMA note:** Each receiver creates its own TLS resolver instance\n(per-core design). Arc-wrapped atomics are used for local coordination\nbetween the resolver and its async reload task. No cross-NUMA sharing.\n\n  ### What's NOT in this PR\n\n  **Deferred to follow-up PRs:**\n  - mTLS (mutual TLS / client certificate verification)\n  - Client-side TLS for exporters\n  - Comprehensive benchmarks (planned after full TLS implementation)\n\n  ### Feature flag\n\nCurrently gated behind `experimental-tls` feature flag. This will be\nremoved once we're confident the implementation is stable and\nproduction-ready.\n\n  ### Testing\n\n  Manual E2E testing completed:\n  - Basic TLS handshake and data transmission\n  - Certificate hot-reload during active connections  \n  - Plaintext rejection when TLS is enabled\n  - Certificate chain validation\n  - Backward compatibility (plaintext mode works without feature flag)\n\n  Automated tests included:\n  - Unit tests for reload logic (`tls_utils::tests`)\n- Integration test for end-to-end certificate rotation\n(`tests/tls_reload.rs`)\n\n  ---\n\n  **Configuration example:**\n\n  ```yaml\n  receivers:\n    otlp:\n      config:\n        listening_addr: \"0.0.0.0:4319\"\n        tls:\n          cert_file: \"/path/to/cert.pem\"\n          key_file: \"/path/to/key.pem\"\n          reload_interval: \"5m\"  # Optional, defaults to 5 minutes\n\n \n```",
          "timestamp": "2025-12-04T22:58:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a5ca878dcea95586eb414b1b0fe4979319972d7"
        },
        "date": 1764930522382,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.034019302609049985,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.05626438230965843,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.034019302609049985,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.05626438230965843,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.62109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.73828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46063.35920189108,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63499.885328845674,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03389575986794718,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.05614509870713014,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.03389575986794718,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.05614509870713014,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.70625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.8515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45990.81662950524,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63399.58593122342,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "dfca971bd964d35f4e00a57f179d004ea83b1d07",
          "message": "Batch processor Ack/Nack support (#1486)\n\nAdds routing state for inbound and outbound batches, calls notify_ack\nand notify_nack appropriately. This depends on behavior referred to as\n\"in-line\" delivery, which requires that points are not re-ordered by the\nbatcher; this property is required by the present algorithm to align Ack\nand Nack responses appropriately. This is tested lightly, now, in this\nPR. This requires more substantive testing in the lower-level library!\nThis only tests logs and traces, because metric batching has known\ncurrent defects.\n\nThe batch processor automatically tracks inbound and outbound context\nonly as needed, considering the whether the arriving data\n`has_subscribers()`, which will be determined by `wait_for_result: true`\nin the receiver.\n\nNew testing revealed a few more cases where protocol buffer form and\nOTAP-records-converted forms have insignificant differences: TraceID,\nSpanID, Resource, and scope presence information is lost, default values\nare filled in. The equivalence tests now canonicalize these.\n\nFixes #1326",
          "timestamp": "2025-12-06T01:23:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dfca971bd964d35f4e00a57f179d004ea83b1d07"
        },
        "date": 1765016864160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
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
            "value": 21.34887166773927,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.48280966213592,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.34887166773927,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.48280966213592,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.233984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.33203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 27709.99619017087,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2099300.579148578,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
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
            "value": 26.84554795222789,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 28.139743215258857,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 26.84554795222789,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.139743215258857,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.226171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.04296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31776.048899426787,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2100012.7720285156,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "6564239839d48e773a1f0cc45c69064f60695217",
          "message": "Columnar query engine filter handle `== null` filters (#1538)\n\npart of #1508 \n\nHandles filter predicates like `severity_text == null` or\n`attributes[\"x\"] == null` (e.g. doesn't exist) when filtering using the\ncolumnar query engine.\n\nIt handles all the cases, including when the optional column is not\npresent, when then the ID column (used to join attributes) is not\npresent or null (meaning no attributes), and when attribute batches are\nentirely absent.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-06T13:43:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6564239839d48e773a1f0cc45c69064f60695217"
        },
        "date": 1765103240328,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
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
            "value": 21.414485865695333,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.678990215580978,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.414485865695333,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.678990215580978,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.072526041666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.3515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 28670.63524303081,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2097659.8107290003,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
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
            "value": 23.912121654224684,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.61922130350346,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.912121654224684,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.61922130350346,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.353776041666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.9140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31793.686138182402,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2099290.9997663656,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          }
        ]
      }
    ]
  }
}