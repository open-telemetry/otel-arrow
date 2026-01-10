window.BENCHMARK_DATA = {
  "lastUpdate": 1768041107365,
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
        "date": 1765189778425,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 23.988597762123607,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.654517560142793,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 23.988597762123607,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.654517560142793,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.456119791666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.94140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31922.358885295198,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2102003.7672666446,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
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
            "value": 21.42995936930783,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.396676213584733,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.42995936930783,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.396676213584733,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.081380208333332,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.3125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 27712.69772877773,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095711.1389132768,
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
          "id": "e8f26074e120041e52cf592b499b1694dfc185c2",
          "message": "Perf test - change loadgen template to allow null (#1582)\n\nOnce https://github.com/open-telemetry/otel-arrow/pull/1581 is merged,\nI'll be working on adding a new nightly test to stress the engine to its\nmax. This is a simple pre-req, to allow passing `null` to configuration,\nresulting in an outcome like below:\n\n```yaml\nconfig:\n      traffic_config:\n        max_batch_size: 1000\n        signals_per_second: null\n        metric_weight: 0\n        trace_weight: 0\n        log_weight: 100\n```",
          "timestamp": "2025-12-10T23:36:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8f26074e120041e52cf592b499b1694dfc185c2"
        },
        "date": 1765449295633,
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
            "value": 21.422415712782605,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.89766365658294,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.422415712782605,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.89766365658294,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.741145833333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.90234375,
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
            "value": 29377.049047285393,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083939.518865634,
            "unit": "bytes/sec",
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
            "value": 25.56555538501884,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 25.73884519367834,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 25.56555538501884,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 25.73884519367834,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.885026041666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.08984375,
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
            "value": 31820.569102313228,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2094480.812348879,
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
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "91a5aa4b5bb2729e869ab32a8e9263a42462aa20",
          "message": "Run perf test always but on cheap GH runners (#1604)\n\nFor catching issues like this\nhttps://github.com/open-telemetry/otel-arrow/pull/1602#issuecomment-3644189168,\nwe can run perf test in every PR. To save the dedicated hardware, we'll\njust run on the normal GH runners. if the label is applied\n(maintainers/approvers do this), then we run on the Oracle dedicated\nhardware.",
          "timestamp": "2025-12-12T00:30:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/91a5aa4b5bb2729e869ab32a8e9263a42462aa20"
        },
        "date": 1765535688257,
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
            "value": 27.317415768190184,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 27.762692540272617,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.317415768190184,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 27.762692540272617,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.754036458333335,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.8125,
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
            "value": 31804.796049110875,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2101356.773542523,
            "unit": "bytes/sec",
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
            "value": 21.120171927018845,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.333087358563578,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.120171927018845,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.333087358563578,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.659244791666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.78125,
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
            "value": 27638.990426172146,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2098173.716103008,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765622077347,
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
            "value": 28.04763290568186,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 29.500655394594595,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.04763290568186,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 29.500655394594595,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.646484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.31640625,
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
            "name": "logs_produced_rate",
            "value": 106664.19383510626,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.19383510626,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001391,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37964.618950284,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2091758.75944942,
            "unit": "bytes/sec",
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
            "value": 21.788549223766925,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.295938062106,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.788549223766925,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.295938062106,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.698567708333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.94921875,
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
            "name": "logs_produced_rate",
            "value": 106663.37076850992,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.37076850992,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001854,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33517.13915913016,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2071627.6121035211,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765622079719,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.563070490857264,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.7676872035275,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.563070490857264,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.7676872035275,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.082291666666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.60546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.22405593579,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001374,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13506.887848237358,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2101722.5460078144,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.65769273105602,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.06054879962851,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.65769273105602,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.06054879962851,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.53111979166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 68.55859375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.75025665373,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001078,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13745.589284804068,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093788.9577104365,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.4280683122715,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 62.29195735480874,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.4280683122715,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 62.29195735480874,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3054.740755208333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 5664.22265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.59559576884,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001165,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7754.578980897085,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721365.1813625533,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765708461086,
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
            "value": 21.709698647406757,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.00197405255452,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.709698647406757,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.00197405255452,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.437109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.24609375,
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
            "name": "logs_produced_rate",
            "value": 106664.50315499434,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.50315499434,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001217,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33517.0708303512,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2077135.158823462,
            "unit": "bytes/sec",
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
            "value": 29.800553977254175,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 30.150890641831584,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.800553977254175,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.150890641831584,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.071875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.171875,
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
            "name": "logs_produced_rate",
            "value": 106664.63292766552,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.63292766552,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001144,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37827.359074594104,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2080968.5094598932,
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765708463119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 36.926051988568226,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 49.265156955478126,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.926051988568226,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.265156955478126,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3068.9635416666665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 5107.921875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.36982723638,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001292,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7149.4780289427035,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720540.7463646851,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.22761881967357,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 40.88669687968473,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.22761881967357,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.88669687968473,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.39205729166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.8203125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.93429221655,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001537,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13888.56254819749,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2092782.231001341,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.93989632252675,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.214064601055576,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.93989632252675,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.214064601055576,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.898958333333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.87890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.9120288638,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000987,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13556.273308126973,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2094199.8943568654,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765795023697,
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
            "value": 29.927602747775783,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 30.85209847008877,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.927602747775783,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.85209847008877,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.63033854166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.7734375,
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
            "name": "logs_produced_rate",
            "value": 106664.33960632425,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.33960632425,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001309,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 38000.58829949029,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2102317.467016024,
            "unit": "bytes/sec",
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
            "value": 21.647191367317415,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.968220137802895,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.647191367317415,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.968220137802895,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.643489583333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.2578125,
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
            "name": "logs_produced_rate",
            "value": 106664.53870911943,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.53870911943,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001197,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33574.74898951883,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2081148.0962056655,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765795025744,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 36.91633213399575,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.50279170062757,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.91633213399575,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.50279170062757,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.387890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.046875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.69514755136,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001109,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13842.224977699383,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2094406.2721309687,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 36.87167623126879,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 50.61373878016809,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.87167623126879,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.61373878016809,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3095.7009114583334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 5095.61328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.43915762893,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001253,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7177.244814589038,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720151.1884682582,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.845787754663878,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.64118813254878,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.845787754663878,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.64118813254878,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.486067708333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.2890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.29338613883,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001335,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13550.927289157875,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088303.3644990614,
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
          "id": "102d2d694a9089a3f96a203be0532024658e0fe0",
          "message": "chore(deps): update rust crate rcgen to 0.14 (#1625)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [rcgen](https://redirect.github.com/rustls/rcgen) |\nworkspace.dependencies | minor | `0.13` -> `0.14` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rustls/rcgen (rcgen)</summary>\n\n###\n[`v0.14.6`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.6):\n0.14.6\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.5...v0.14.6)\n\n#### What's Changed\n\n- Use private cfg for docs.rs-like builds by\n[@&#8203;ctz](https://redirect.github.com/ctz) in\n[#&#8203;384](https://redirect.github.com/rustls/rcgen/pull/384)\n- Expand rustdoc for CertificateSigningRequestParams::from\\_der by\n[@&#8203;dwhjames](https://redirect.github.com/dwhjames) in\n[#&#8203;386](https://redirect.github.com/rustls/rcgen/pull/386)\n- Group imports by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;381](https://redirect.github.com/rustls/rcgen/pull/381)\n- examples: add signing new cert using existing ca pem files by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;379](https://redirect.github.com/rustls/rcgen/pull/379)\n- Tweak CSR parsing errors/documentation by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;390](https://redirect.github.com/rustls/rcgen/pull/390)\n- Rename invalid CSR signature error variant by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;393](https://redirect.github.com/rustls/rcgen/pull/393)\n- chore: fix some typos in comments by\n[@&#8203;black5box](https://redirect.github.com/black5box) in\n[#&#8203;395](https://redirect.github.com/rustls/rcgen/pull/395)\n- ci: sync cargo-check-external-types nightly by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;399](https://redirect.github.com/rustls/rcgen/pull/399)\n- Forward selected crypto backend to x509-parser by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;398](https://redirect.github.com/rustls/rcgen/pull/398)\n\n###\n[`v0.14.5`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.5):\n0.14.5\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.4...v0.14.5)\n\nImplement SigningKey for `&impl SigningKey` to make `Issuer` more\nbroadly useful.\n\n#### What's Changed\n\n- Forward signing and public key data through references by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;380](https://redirect.github.com/rustls/rcgen/pull/380)\n\n###\n[`v0.14.4`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.4):\n0.14.4\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.3...v0.14.4)\n\n#### What's Changed\n\n- Upgrade botan to 0.12 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;377](https://redirect.github.com/rustls/rcgen/pull/377)\n- Upgrade x509-parser to 0.18 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;376](https://redirect.github.com/rustls/rcgen/pull/376)\n- Add unstable support for ML-DSA algorithms by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;374](https://redirect.github.com/rustls/rcgen/pull/374)\n\n###\n[`v0.14.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.3):\n0.14.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.2...v0.14.3)\n\n#### What's Changed\n\n- docs: fix typo in `PKCS_RSA_SHA384` doc comment by\n[@&#8203;Bravo555](https://redirect.github.com/Bravo555) in\n[#&#8203;367](https://redirect.github.com/rustls/rcgen/pull/367)\n- Fix regression in key usage purpose encoding by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;369](https://redirect.github.com/rustls/rcgen/pull/369)\n\n###\n[`v0.14.2`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.2):\n0.14.2\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.1...v0.14.2)\n\n- Add a `CertifiedIssuer` type (see\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/issues/363))\n\n#### What's changed\n\n- Add a CertifiedIssuer by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/pull/363)\n- Provide a non-owning constructor for `Issuer` by\n[@&#8203;p-avital](https://redirect.github.com/p-avital) in\n[#&#8203;362](https://redirect.github.com/rustls/rcgen/pull/362)\n- Allow access to the CertifiedIssuer's Certificate by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;364](https://redirect.github.com/rustls/rcgen/pull/364)\n\n###\n[`v0.14.1`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.1):\n0.14.1\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.0...v0.14.1)\n\nDeclare 1.71 `rust-version` and check MSRV in CI.\n\n#### What's Changed\n\n- Check MSRV in CI by [@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;361](https://redirect.github.com/rustls/rcgen/pull/361)\n\n###\n[`v0.14.0`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.0):\n0.14.0\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.3...v0.14.0)\n\n0.14.0 contains a number of potentially breaking API changes, though\nhopefully the rate of API change should slow down after this. Here is a\nsummary of the most noticeable changes you might run into:\n\n- `signed_by()` methods now take a reference to an `&Issuer` type that\ncontains both the issuer's relevant certificate parameters and the\nsigning key (see\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/issues/356)). The\n`from_ca_cert_der()` and `from_ca_cert_pem()` constructors that were\npreviously attached to `CertificateParams` are now attached to `Issuer`\ninstead, removing a number of documented caveats.\n- The `RemoteKeyPair` trait is now called `SigningKey` and instead of\n`KeyPair` being an enum that contains a `Remote` variant, that variant\nhas been removed in favor of `KeyPair` implementing the trait (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). To\nalign with this change, the `CertifiedKey::key_pair` field is now called\n`signing_key`, and `CertifiedKey` is generic over the signing key type.\n- The `KeyPair::public_key_der()` method has moved to\n`PublicKeyData::subject_public_key_info()` (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)).\n- Output types like `Certificate` no longer contain their originating\n`CertificateParams`. Instead, `signed_by()` and `self_signed()` now take\n`&self`, allowing the caller to retain access to the input parameters\n(see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). In\norder to make this possible, `Certificate::key_identifier()` can now be\naccessed via `CertificateParams` directly.\n- String types have been moved into a module (see\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/issues/329)).\n\n#### What's Changed\n\n- Revert impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;325](https://redirect.github.com/rustls/rcgen/pull/325)\n- Move string types to separate module by\n[@&#8203;est31](https://redirect.github.com/est31) in\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/pull/329)\n- Unbundle params from output types by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/pull/328)\n- Deduplicate Issuer construction by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;332](https://redirect.github.com/rustls/rcgen/pull/332)\n- Extract write\\_extensions() method, reducing rightward drift by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;333](https://redirect.github.com/rustls/rcgen/pull/333)\n- Update 0.12-to-0.13.md by\n[@&#8203;Alirexaa](https://redirect.github.com/Alirexaa) in\n[#&#8203;338](https://redirect.github.com/rustls/rcgen/pull/338)\n- Distribute methods for parsing params elements from x509 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;336](https://redirect.github.com/rustls/rcgen/pull/336)\n- Eagerly derive Clone, Copy, where possible by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;341](https://redirect.github.com/rustls/rcgen/pull/341)\n- Updated `.gitignore` to be more specific by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;342](https://redirect.github.com/rustls/rcgen/pull/342)\n- Eagerly implemented `Debug` trait by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;343](https://redirect.github.com/rustls/rcgen/pull/343)\n- Minor tweaks to Debug impls and other style improvements by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;348](https://redirect.github.com/rustls/rcgen/pull/348)\n- tests: only test against openssl on Unix by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;350](https://redirect.github.com/rustls/rcgen/pull/350)\n- Eagerly implemented `PartialEq` and `Eq` traits by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;344](https://redirect.github.com/rustls/rcgen/pull/344)\n- Use Issuer directly in the public API by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/pull/356)\n- Tweak docstring for PublicKeyData::subject\\_public\\_key\\_info() by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;358](https://redirect.github.com/rustls/rcgen/pull/358)\n\n###\n[`v0.13.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.13.3):\n0.13.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.2...v0.13.3)\n\nThis release was yanked due to\n[#&#8203;324](https://redirect.github.com/rustls/rcgen/issues/324)\n\n#### What's Changed\n\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;305](https://redirect.github.com/rustls/rcgen/pull/305)\n- Add link to GitHub releases by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;304](https://redirect.github.com/rustls/rcgen/pull/304)\n- change signature of signed\\_by to accept \\&impl\nAsRef<CertificateParams> issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;307](https://redirect.github.com/rustls/rcgen/pull/307)\n- Clarify CertificateParams::signed\\_by() docs by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;308](https://redirect.github.com/rustls/rcgen/pull/308)\n- refactor: Generalize csr/crl signed\\_by to take \\&impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;312](https://redirect.github.com/rustls/rcgen/pull/312)\n- Fix: mark SAN as critical when subject is empty by\n[@&#8203;howardjohn](https://redirect.github.com/howardjohn) in\n[#&#8203;311](https://redirect.github.com/rustls/rcgen/pull/311)\n- Elide private key in KeyPair Debug impl by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;314](https://redirect.github.com/rustls/rcgen/pull/314)\n- derive Debug for non-sensitive struct types by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;316](https://redirect.github.com/rustls/rcgen/pull/316)\n- update LICENSE by\n[@&#8203;jasmyhigh](https://redirect.github.com/jasmyhigh) in\n[#&#8203;318](https://redirect.github.com/rustls/rcgen/pull/318)\n- Make `Certificate` cloneable (derive `Clone`) by\n[@&#8203;MadLittleMods](https://redirect.github.com/MadLittleMods) in\n[#&#8203;319](https://redirect.github.com/rustls/rcgen/pull/319)\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;321](https://redirect.github.com/rustls/rcgen/pull/321)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNDIuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-15T18:32:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/102d2d694a9089a3f96a203be0532024658e0fe0"
        },
        "date": 1765881133557,
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
            "value": 21.40641124405658,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.656962333772984,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.40641124405658,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.656962333772984,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.683854166666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.64453125,
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
            "name": "logs_produced_rate",
            "value": 106664.25605447983,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.25605447983,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001356,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33627.8470259964,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068694.546275071,
            "unit": "bytes/sec",
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
            "value": 27.130339600648085,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 28.137433377442683,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 27.130339600648085,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 28.137433377442683,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.444270833333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.55078125,
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
            "name": "logs_produced_rate",
            "value": 106664.55470848344,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.55470848344,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001188,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37876.868011197934,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2094512.9765301084,
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
          "id": "102d2d694a9089a3f96a203be0532024658e0fe0",
          "message": "chore(deps): update rust crate rcgen to 0.14 (#1625)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [rcgen](https://redirect.github.com/rustls/rcgen) |\nworkspace.dependencies | minor | `0.13` -> `0.14` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rustls/rcgen (rcgen)</summary>\n\n###\n[`v0.14.6`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.6):\n0.14.6\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.5...v0.14.6)\n\n#### What's Changed\n\n- Use private cfg for docs.rs-like builds by\n[@&#8203;ctz](https://redirect.github.com/ctz) in\n[#&#8203;384](https://redirect.github.com/rustls/rcgen/pull/384)\n- Expand rustdoc for CertificateSigningRequestParams::from\\_der by\n[@&#8203;dwhjames](https://redirect.github.com/dwhjames) in\n[#&#8203;386](https://redirect.github.com/rustls/rcgen/pull/386)\n- Group imports by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;381](https://redirect.github.com/rustls/rcgen/pull/381)\n- examples: add signing new cert using existing ca pem files by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;379](https://redirect.github.com/rustls/rcgen/pull/379)\n- Tweak CSR parsing errors/documentation by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;390](https://redirect.github.com/rustls/rcgen/pull/390)\n- Rename invalid CSR signature error variant by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;393](https://redirect.github.com/rustls/rcgen/pull/393)\n- chore: fix some typos in comments by\n[@&#8203;black5box](https://redirect.github.com/black5box) in\n[#&#8203;395](https://redirect.github.com/rustls/rcgen/pull/395)\n- ci: sync cargo-check-external-types nightly by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;399](https://redirect.github.com/rustls/rcgen/pull/399)\n- Forward selected crypto backend to x509-parser by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;398](https://redirect.github.com/rustls/rcgen/pull/398)\n\n###\n[`v0.14.5`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.5):\n0.14.5\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.4...v0.14.5)\n\nImplement SigningKey for `&impl SigningKey` to make `Issuer` more\nbroadly useful.\n\n#### What's Changed\n\n- Forward signing and public key data through references by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;380](https://redirect.github.com/rustls/rcgen/pull/380)\n\n###\n[`v0.14.4`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.4):\n0.14.4\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.3...v0.14.4)\n\n#### What's Changed\n\n- Upgrade botan to 0.12 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;377](https://redirect.github.com/rustls/rcgen/pull/377)\n- Upgrade x509-parser to 0.18 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;376](https://redirect.github.com/rustls/rcgen/pull/376)\n- Add unstable support for ML-DSA algorithms by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;374](https://redirect.github.com/rustls/rcgen/pull/374)\n\n###\n[`v0.14.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.3):\n0.14.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.2...v0.14.3)\n\n#### What's Changed\n\n- docs: fix typo in `PKCS_RSA_SHA384` doc comment by\n[@&#8203;Bravo555](https://redirect.github.com/Bravo555) in\n[#&#8203;367](https://redirect.github.com/rustls/rcgen/pull/367)\n- Fix regression in key usage purpose encoding by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;369](https://redirect.github.com/rustls/rcgen/pull/369)\n\n###\n[`v0.14.2`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.2):\n0.14.2\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.1...v0.14.2)\n\n- Add a `CertifiedIssuer` type (see\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/issues/363))\n\n#### What's changed\n\n- Add a CertifiedIssuer by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/pull/363)\n- Provide a non-owning constructor for `Issuer` by\n[@&#8203;p-avital](https://redirect.github.com/p-avital) in\n[#&#8203;362](https://redirect.github.com/rustls/rcgen/pull/362)\n- Allow access to the CertifiedIssuer's Certificate by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;364](https://redirect.github.com/rustls/rcgen/pull/364)\n\n###\n[`v0.14.1`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.1):\n0.14.1\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.0...v0.14.1)\n\nDeclare 1.71 `rust-version` and check MSRV in CI.\n\n#### What's Changed\n\n- Check MSRV in CI by [@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;361](https://redirect.github.com/rustls/rcgen/pull/361)\n\n###\n[`v0.14.0`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.0):\n0.14.0\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.3...v0.14.0)\n\n0.14.0 contains a number of potentially breaking API changes, though\nhopefully the rate of API change should slow down after this. Here is a\nsummary of the most noticeable changes you might run into:\n\n- `signed_by()` methods now take a reference to an `&Issuer` type that\ncontains both the issuer's relevant certificate parameters and the\nsigning key (see\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/issues/356)). The\n`from_ca_cert_der()` and `from_ca_cert_pem()` constructors that were\npreviously attached to `CertificateParams` are now attached to `Issuer`\ninstead, removing a number of documented caveats.\n- The `RemoteKeyPair` trait is now called `SigningKey` and instead of\n`KeyPair` being an enum that contains a `Remote` variant, that variant\nhas been removed in favor of `KeyPair` implementing the trait (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). To\nalign with this change, the `CertifiedKey::key_pair` field is now called\n`signing_key`, and `CertifiedKey` is generic over the signing key type.\n- The `KeyPair::public_key_der()` method has moved to\n`PublicKeyData::subject_public_key_info()` (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)).\n- Output types like `Certificate` no longer contain their originating\n`CertificateParams`. Instead, `signed_by()` and `self_signed()` now take\n`&self`, allowing the caller to retain access to the input parameters\n(see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). In\norder to make this possible, `Certificate::key_identifier()` can now be\naccessed via `CertificateParams` directly.\n- String types have been moved into a module (see\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/issues/329)).\n\n#### What's Changed\n\n- Revert impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;325](https://redirect.github.com/rustls/rcgen/pull/325)\n- Move string types to separate module by\n[@&#8203;est31](https://redirect.github.com/est31) in\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/pull/329)\n- Unbundle params from output types by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/pull/328)\n- Deduplicate Issuer construction by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;332](https://redirect.github.com/rustls/rcgen/pull/332)\n- Extract write\\_extensions() method, reducing rightward drift by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;333](https://redirect.github.com/rustls/rcgen/pull/333)\n- Update 0.12-to-0.13.md by\n[@&#8203;Alirexaa](https://redirect.github.com/Alirexaa) in\n[#&#8203;338](https://redirect.github.com/rustls/rcgen/pull/338)\n- Distribute methods for parsing params elements from x509 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;336](https://redirect.github.com/rustls/rcgen/pull/336)\n- Eagerly derive Clone, Copy, where possible by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;341](https://redirect.github.com/rustls/rcgen/pull/341)\n- Updated `.gitignore` to be more specific by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;342](https://redirect.github.com/rustls/rcgen/pull/342)\n- Eagerly implemented `Debug` trait by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;343](https://redirect.github.com/rustls/rcgen/pull/343)\n- Minor tweaks to Debug impls and other style improvements by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;348](https://redirect.github.com/rustls/rcgen/pull/348)\n- tests: only test against openssl on Unix by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;350](https://redirect.github.com/rustls/rcgen/pull/350)\n- Eagerly implemented `PartialEq` and `Eq` traits by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;344](https://redirect.github.com/rustls/rcgen/pull/344)\n- Use Issuer directly in the public API by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/pull/356)\n- Tweak docstring for PublicKeyData::subject\\_public\\_key\\_info() by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;358](https://redirect.github.com/rustls/rcgen/pull/358)\n\n###\n[`v0.13.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.13.3):\n0.13.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.2...v0.13.3)\n\nThis release was yanked due to\n[#&#8203;324](https://redirect.github.com/rustls/rcgen/issues/324)\n\n#### What's Changed\n\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;305](https://redirect.github.com/rustls/rcgen/pull/305)\n- Add link to GitHub releases by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;304](https://redirect.github.com/rustls/rcgen/pull/304)\n- change signature of signed\\_by to accept \\&impl\nAsRef<CertificateParams> issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;307](https://redirect.github.com/rustls/rcgen/pull/307)\n- Clarify CertificateParams::signed\\_by() docs by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;308](https://redirect.github.com/rustls/rcgen/pull/308)\n- refactor: Generalize csr/crl signed\\_by to take \\&impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;312](https://redirect.github.com/rustls/rcgen/pull/312)\n- Fix: mark SAN as critical when subject is empty by\n[@&#8203;howardjohn](https://redirect.github.com/howardjohn) in\n[#&#8203;311](https://redirect.github.com/rustls/rcgen/pull/311)\n- Elide private key in KeyPair Debug impl by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;314](https://redirect.github.com/rustls/rcgen/pull/314)\n- derive Debug for non-sensitive struct types by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;316](https://redirect.github.com/rustls/rcgen/pull/316)\n- update LICENSE by\n[@&#8203;jasmyhigh](https://redirect.github.com/jasmyhigh) in\n[#&#8203;318](https://redirect.github.com/rustls/rcgen/pull/318)\n- Make `Certificate` cloneable (derive `Clone`) by\n[@&#8203;MadLittleMods](https://redirect.github.com/MadLittleMods) in\n[#&#8203;319](https://redirect.github.com/rustls/rcgen/pull/319)\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;321](https://redirect.github.com/rustls/rcgen/pull/321)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNDIuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-15T18:32:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/102d2d694a9089a3f96a203be0532024658e0fe0"
        },
        "date": 1765881135452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.38073944685783,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 49.286862183676625,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.38073944685783,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.286862183676625,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3092.776822916667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 5156.421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.59204035148,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001167,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7239.545796644932,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 717203.1501625868,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.838970947035527,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.014870408383416,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.838970947035527,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.014870408383416,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.531119791666665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.43359375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.58670722587,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00117,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13658.06232104171,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2079364.529338438,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.034047504913694,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.048268426992344,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.034047504913694,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.048268426992344,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.10924479166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.91796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.93186389678,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003226,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13944.314721858875,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095112.3159583705,
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
          "id": "865730028aeacfbe679d94d531d15cb2257aeb44",
          "message": "Adds OTAP Dataflow processor that uses columnar query engine (#1638)\n\ncloses #1628 \n\nAdds a processor implementation to OTAP Dataflow that uses the columnar\nquery engine to transform telemetry data.\n\nExample config:\n```yaml\nnodes:\n  transform:\n    kind: processor\n    plugin_urn: urn:otel:transform:processor\n    out_ports:\n      out_port:\n        destinations:\n        - exporter\n        dispatch_strategy: round_robin\n    config:\n      query: logs | where event_name == \"gen_ai.system.message\"\n```\n\n**I'm flexible on the name of this plugin and open to suggestions**. I\nhesitated between some alternative names:\n- \"\\<_language_\\> processor\" - where \"_language_\" = KQL/OTTL/ anything\nthat can be transformed into our expression AST. I was thinking this\ncould eventually accept different ways to express the transform so\npinning it to one language seemed too specific.\n- \"transform processor\" - this is a good name that expresses the purpose\nof the processor, but it's not really API compatible with Go collector's\n[transform\nprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/transformprocessor)\nand I didn't want to cause confusion.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2025-12-17T00:44:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/865730028aeacfbe679d94d531d15cb2257aeb44"
        },
        "date": 1765967533524,
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
            "value": 17.790637607093256,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.02510743663282,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.790637607093256,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.02510743663282,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.00598958333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.37109375,
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
            "name": "logs_produced_rate",
            "value": 106664.0089551102,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.0089551102,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001495,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19317.98342504973,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1786716.176234561,
            "unit": "bytes/sec",
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
            "value": 18.350295447521695,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.28122889555915,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.350295447521695,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.28122889555915,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.58567708333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.25,
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
            "name": "logs_produced_rate",
            "value": 106664.94225010028,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.94225010028,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00097,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19811.649175880535,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1790633.0096597404,
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
          "id": "865730028aeacfbe679d94d531d15cb2257aeb44",
          "message": "Adds OTAP Dataflow processor that uses columnar query engine (#1638)\n\ncloses #1628 \n\nAdds a processor implementation to OTAP Dataflow that uses the columnar\nquery engine to transform telemetry data.\n\nExample config:\n```yaml\nnodes:\n  transform:\n    kind: processor\n    plugin_urn: urn:otel:transform:processor\n    out_ports:\n      out_port:\n        destinations:\n        - exporter\n        dispatch_strategy: round_robin\n    config:\n      query: logs | where event_name == \"gen_ai.system.message\"\n```\n\n**I'm flexible on the name of this plugin and open to suggestions**. I\nhesitated between some alternative names:\n- \"\\<_language_\\> processor\" - where \"_language_\" = KQL/OTTL/ anything\nthat can be transformed into our expression AST. I was thinking this\ncould eventually accept different ways to express the transform so\npinning it to one language seemed too specific.\n- \"transform processor\" - this is a good name that expresses the purpose\nof the processor, but it's not really API compatible with Go collector's\n[transform\nprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/transformprocessor)\nand I didn't want to cause confusion.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2025-12-17T00:44:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/865730028aeacfbe679d94d531d15cb2257aeb44"
        },
        "date": 1765967535231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.27961258032979,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.967549521301315,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.27961258032979,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.967549521301315,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 88.44440104166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 142.84375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.87291905374,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001009,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6887.091375561315,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1794091.667199535,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.09546507829661,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.66310849365914,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.09546507829661,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.66310849365914,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 133.099609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 141.140625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.78936637382,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001056,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4647.953761906719,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 350319.3923673622,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.67637329895706,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 38.2159228906675,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.67637329895706,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 38.2159228906675,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 143.66002604166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 176.41015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.0062480694,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000934,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7297.68951065597,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1811199.9525722766,
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
          "id": "ac99143900169732ae2a7c90ed7a61cc4bd37471",
          "message": "[otap-df-quiver] Segment file reader/writer for Quiver durable storage layer. (#1643)\n\nImplements Segment file reader/writer for Quiver durable storage layer",
          "timestamp": "2025-12-17T23:42:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac99143900169732ae2a7c90ed7a61cc4bd37471"
        },
        "date": 1766053922516,
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
            "value": 18.40980536619602,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.91037049733158,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.40980536619602,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 19.91037049733158,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.41614583333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.34765625,
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
            "name": "logs_produced_rate",
            "value": 106664.87114133578,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.87114133578,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00101,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19796.89945367534,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1796926.9086094613,
            "unit": "bytes/sec",
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
            "value": 17.755476985426014,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.172161089867636,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 17.755476985426014,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.172161089867636,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.30416666666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.3359375,
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
            "name": "logs_produced_rate",
            "value": 106664.40004816564,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.40004816564,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001275,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19126.832412522515,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1782444.6340309242,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
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
          "id": "ac99143900169732ae2a7c90ed7a61cc4bd37471",
          "message": "[otap-df-quiver] Segment file reader/writer for Quiver durable storage layer. (#1643)\n\nImplements Segment file reader/writer for Quiver durable storage layer",
          "timestamp": "2025-12-17T23:42:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac99143900169732ae2a7c90ed7a61cc4bd37471"
        },
        "date": 1766053924342,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.598309718529514,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.71164257804007,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.598309718529514,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 47.71164257804007,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 138.93059895833332,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 157.328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.7662560812,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001069,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 4615.990435716365,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349913.3527181492,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 29.69753764874861,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.83494807245704,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.69753764874861,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.83494807245704,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.06861979166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 98.45703125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.11739426093,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001434,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7000.76049590995,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1789536.4282410434,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.118638447072506,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 40.494039053549194,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.118638447072506,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.494039053549194,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 140.09361979166667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 165.5,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.56004160584,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001185,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7306.487173249939,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1788910.3785171837,
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
          "id": "2c5e35748dc82b3cee511b767e9ca66ee66df7ff",
          "message": "Support deleting attributes in columnar query engine (#1654)\n\ncloses #1639 \n\nAdds the ability to delete attributes to the columnar query engine.\n\nWe now support queries like:\n```kql\nlogs | project-away attributes[\"x\"], attributes[\"y\"]\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-19T03:20:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c5e35748dc82b3cee511b767e9ca66ee66df7ff"
        },
        "date": 1766140304366,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.213050461972834,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.264960838277215,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.213050461972834,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.264960838277215,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.42526041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.01073279942,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001494,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33137.423493424794,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723744.5000491759,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.18371353638219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.823998076745262,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.18371353638219,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.823998076745262,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.26393229166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.89453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.76447836681,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00107,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36776.23002826258,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723428.0149808847,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 22.407603660053134,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.419789502276412,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 22.407603660053134,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 23.419789502276412,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.70872395833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.29296875,
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
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.12272733962,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.198504549,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001431,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99984.79578775937,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2096300.1095321893,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.903205713835725,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.155274802641685,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.903205713835725,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.155274802641685,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.91809895833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.17578125,
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
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.88002992616,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.2386415860865,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001005,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 281725.90151713806,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2090936.4913230438,
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
          "id": "2c5e35748dc82b3cee511b767e9ca66ee66df7ff",
          "message": "Support deleting attributes in columnar query engine (#1654)\n\ncloses #1639 \n\nAdds the ability to delete attributes to the columnar query engine.\n\nWe now support queries like:\n```kql\nlogs | project-away attributes[\"x\"], attributes[\"y\"]\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-19T03:20:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c5e35748dc82b3cee511b767e9ca66ee66df7ff"
        },
        "date": 1766140306219,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.16200533607776,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.80549976826819,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.16200533607776,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.80549976826819,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 651.5256510416667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 933.49609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.28449764622,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.20707837525,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00134,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63771.27356438232,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2099649.490506909,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 33.61236692490533,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.575279173477256,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 33.61236692490533,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.575279173477256,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.501041666666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.85546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.34849482604,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.21047022578,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001304,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75885.28226118599,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2098788.2252069805,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 39.5584467835432,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 63.2596195093259,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.5584467835432,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 63.2596195093259,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3747.095182291667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6734.01171875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.15294812886,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.20010625083,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001414,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 56168.495493747396,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720314.4821087823,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766226692129,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.896090288687205,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.900452867089587,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.896090288687205,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.900452867089587,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.665625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.64453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.73905124029,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000883,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33699.97321819842,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723714.6198596825,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 22.19474326760634,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.41708989559972,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 22.19474326760634,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.41708989559972,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.98463541666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.66015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.17393193295,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.552218392447,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001196,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99148.61613864316,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2063263.4058940585,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.893551514263903,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.281487303701407,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.893551514263903,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.281487303701407,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.233723958333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.35546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.28564129729,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001688,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36045.76388985226,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723588.481011436,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 28.774451837040726,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 30.091543085378124,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.774451837040726,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 30.091543085378124,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.135546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.71484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.18115386774,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.55260115499,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001192,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278606.72971979255,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068752.883816113,
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
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766226694535,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 33.839091875342866,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.8070629493946,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 33.839091875342866,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.8070629493946,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.554296875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.36328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.32198178854,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.560065034792,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001114,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75559.92546087924,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2090323.931334878,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 40.12534768399119,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 58.45532308647725,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.12534768399119,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 58.45532308647725,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3911.1673177083335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 7160.55078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.30889950771,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.453371673909,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002229,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58302.28348188321,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722937.9389353241,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.378749952136396,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 38.48697208365608,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.378749952136396,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 38.48697208365608,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 678.407421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 932.4765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.23712389499,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.555567566434,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001161,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63781.38035788194,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2076220.3296460514,
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
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766313025665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.296562387406784,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.43149932848522,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.296562387406784,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.43149932848522,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.05846354166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.6875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.04213179224,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.545232984989,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001269,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 279314.58281420846,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2072968.9247748933,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 22.214759426892616,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.430319851393186,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 22.214759426892616,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.430319851393186,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.16263020833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.96484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.04912734395,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.49260374923,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001819,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99289.49284275023,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2063495.730776423,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.10506755417293,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.668970169125032,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.10506755417293,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.668970169125032,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.66393229166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.3359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.1955977402,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001184,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36985.831396437985,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724879.5590581017,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.017065864092453,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.382662177050197,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.017065864092453,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.382662177050197,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.77369791666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.3515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.50614192973,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001012,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33368.26122178618,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723671.1253350073,
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766313027118,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.738084909092926,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 39.796581732963304,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.738084909092926,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 39.796581732963304,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 667.0180989583333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 949.75390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.04393727145,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.545328675387,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001268,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63632.2934688148,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2076449.3327113248,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.57418966550172,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.16384560796727,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.57418966550172,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.16384560796727,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3909.36953125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 7404.1953125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.00241126478,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.543127797034,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001291,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57471.2610239175,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722566.4939860161,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 33.392766819494184,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.80570588071908,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 33.392766819494184,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.80570588071908,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.585807291666665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.0859375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.70992464133,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.52762600599,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001453,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75333.92023155367,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2079397.5627355233,
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766399480708,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.92660665165208,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.025312817970566,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.92660665165208,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.025312817970566,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.215625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.1875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.55850129989,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000983,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33289.19497086377,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723819.5399081942,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 22.42229452007846,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 24.852118320987653,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 22.42229452007846,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 24.852118320987653,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.88671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.34765625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.19740322453,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.553462370901,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001183,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99603.37270156298,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2069610.2719611607,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.391108990618044,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.440534075620505,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.391108990618044,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.440534075620505,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.01783854166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.66015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.21004161652,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.554132205675,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001176,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 280359.44344908616,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2078527.8370746942,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.960922891266865,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.104762883935436,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.960922891266865,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.104762883935436,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.24192708333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.23351292375,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001163,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36481.73358573406,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722714.8269031807,
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
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766399482404,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 33.45984589992966,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.66010776966249,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 33.45984589992966,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.66010776966249,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.840494791666668,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.4609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.44294965387,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5664763316545,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001047,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75440.34636151062,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088262.4886020138,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.24595821372303,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 39.20157448521443,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.24595821372303,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 39.20157448521443,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 685.071875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 962.37890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.18634142012,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.499876095267,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001743,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64416.83335641112,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2070242.702385319,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 38.67902102313984,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.33870895741557,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.67902102313984,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.33870895741557,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3874.844921875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 7168.5390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.0707927013,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.493752013169,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001807,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57257.40174139049,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721761.2757158639,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "9892ed711f59b7276e28cccc8b4e8ad0f9ebf884",
          "message": "Upgrade Collector dependencies to 0.142.0 and misc Go modules (#1682)\n\nSupersedes #1677, #1676, and #1675.\n\nTouched manually since already upgrading Collector dependencies.",
          "timestamp": "2025-12-22T21:24:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9892ed711f59b7276e28cccc8b4e8ad0f9ebf884"
        },
        "date": 1766485925639,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 20.187542408676237,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.388306444874274,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.187542408676237,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.388306444874274,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.37721354166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.51336390882,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.570208287168,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001008,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99493.19988231434,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2066175.963944236,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.951487900474492,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.466366723858105,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.951487900474492,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.466366723858105,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.8078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.80859375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.00782769863,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001288,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33671.06301076027,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 717937.149869832,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.06918780595184,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.37116893682588,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.06918780595184,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.37116893682588,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.264453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.19921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.49711445725,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5693470662345,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001017,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278965.44480968505,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2072909.537832103,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.036714732066741,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.23355514157512,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.036714732066741,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.23355514157512,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.059505208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.35546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.8886662791,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001354,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36003.84493899842,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722957.251425461,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "9892ed711f59b7276e28cccc8b4e8ad0f9ebf884",
          "message": "Upgrade Collector dependencies to 0.142.0 and misc Go modules (#1682)\n\nSupersedes #1677, #1676, and #1675.\n\nTouched manually since already upgrading Collector dependencies.",
          "timestamp": "2025-12-22T21:24:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9892ed711f59b7276e28cccc8b4e8ad0f9ebf884"
        },
        "date": 1766485928082,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.48503869429792,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 50.565895103965374,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.48503869429792,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.565895103965374,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 4085.336848958333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6718.984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.66817955993,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.472413516676,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00203,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58142.42050086195,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724701.7589894895,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 28.43287211842913,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 37.24486930315479,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.43287211842913,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.24486930315479,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 652.2936197916666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 959.36328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.25337326857,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.556428783234,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001152,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64294.012301905226,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2079411.902821414,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.990486942495238,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.88316276994578,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.990486942495238,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.88316276994578,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.131510416666668,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.55078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.53322435626,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.571260890882,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000997,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75449.17996507073,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088520.652004264,
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766572327264,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.864925492525487,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.10640114983386,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.864925492525487,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.10640114983386,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.123046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.87890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.01348830233,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000731,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33870.678559508735,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723882.5917210503,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.760240265211399,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.921528877897991,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.760240265211399,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.921528877897991,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.11875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.4609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.82932643619,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000833,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36713.71767097666,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723687.4561279776,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.05904672344728,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.338402079207924,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.05904672344728,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.338402079207924,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.3828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.6640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.95366338453,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.54054415938,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001318,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278896.6710101321,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2076736.703528111,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 20.529771846175628,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.7949804970964,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.529771846175628,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.7949804970964,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.921484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.47265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.84352944621,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.534707060649,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001379,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99512.9176963882,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2074509.1240417166,
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766572329772,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 29.629386093812588,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 36.17386083752612,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.629386093812588,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 36.17386083752612,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 708.1744791666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1012.7265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.6253047077,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.576141149508,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000946,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64044.835095683855,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2079234.4223595927,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 39.92658357935681,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 57.638757106435264,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.92658357935681,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.638757106435264,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3852.0412760416666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6722.16015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.48267050438,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.568581536732,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001025,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58580.53798139991,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721729.7149912812,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.20508120161121,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.12927582932506,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.20508120161121,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.12927582932506,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.637630208333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.35546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.67766419303,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.57891620223,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000917,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75437.08507429188,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2087302.9697575474,
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766658640042,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.956255994415658,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.152472121399814,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.956255994415658,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.152472121399814,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.86510416666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.9921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.99723870073,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00074,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34074.755092417814,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724674.8172378703,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.917937383376984,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.53074683783157,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.917937383376984,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.53074683783157,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.911328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.14504420344,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001212,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37241.80799349483,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 716205.6948520535,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 20.707146721554494,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.99455671181691,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.707146721554494,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.99455671181691,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.42265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.84377048149,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.587719835519,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000825,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99316.7743653734,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2074036.0175040965,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.05361375365599,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 31.402084247567146,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.05361375365599,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.402084247567146,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.028515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.71875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.44114416135,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.566380640552,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001048,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278305.69130532414,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2076227.8543418872,
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766658642454,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.47754419892082,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 50.07924823666073,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.47754419892082,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.07924823666073,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3931.753125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6069.75,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.85099250558,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5881026027955,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000821,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57925.67452158409,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721920.6595856037,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.306034958910605,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.65786899883495,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.306034958910605,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.65786899883495,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.468619791666665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.6015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.48267050438,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.568581536732,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001025,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75232.96124954421,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2089328.0711531115,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.490028158202982,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 38.69803716877278,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.490028158202982,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 38.69803716877278,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 672.4915364583334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 944.27734375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.99001665713,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.595470882828,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000744,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63793.86519254718,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088948.9738467939,
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766745053528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.997388133742342,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.178760898185175,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.997388133742342,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.178760898185175,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.752604166666664,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.33203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.83654845835,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000829,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34383.3028317246,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724104.6633100674,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 20.63220633464578,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.98488505761445,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.63220633464578,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.98488505761445,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.10390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2578125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.66683119192,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.578342053172,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000923,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99350.0078118916,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2075308.0945382942,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.68611721891793,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.041029015735884,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.68611721891793,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.041029015735884,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.995442708333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.94921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.32559276569,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001112,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37009.68840980556,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 715833.3309683396,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.274152784270193,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 31.6326263197026,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.274152784270193,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 31.6326263197026,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.300130208333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.0078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.49530896293,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.569251375035,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001018,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278359.922252711,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2065501.8026332024,
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766745055322,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.40171917746399,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 37.07622017262639,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.40171917746399,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.07622017262639,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 678.955078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 965.52734375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.23712389499,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.555567566434,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001161,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64685.506597533946,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2081571.7528012826,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.37070260899891,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.6497093422381,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.37070260899891,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.6497093422381,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.560416666666665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.39453125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.73182923112,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.581786949249,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000887,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75137.54340396919,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2090873.1516440227,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 36.58551233671361,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.35432436026285,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 36.58551233671361,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 51.35432436026285,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 4048.015234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6776,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108328.71852992396,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.42208208597,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002556,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 56885.52565755876,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 714454.3269831126,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766831488096,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.164923635146334,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.315418802144023,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.164923635146334,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.315418802144023,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.606640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.05862611017,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.599107183839,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000706,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 98973.78633371166,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2059128.163855064,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.9385057186854,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 10.978817644327503,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.9385057186854,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 10.978817644327503,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.994140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.20703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.3562860811,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001095,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33483.98554330086,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723397.1551769009,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.745385327026344,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.205773766635716,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.745385327026344,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.205773766635716,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.183203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.6640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.04935370946,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001265,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36215.79104451316,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723348.075822908,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.06210585938555,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.49661232552747,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.06210585938555,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.49661232552747,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.12421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.5546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.89613017801,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.590494899434,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000796,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278791.6812609454,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2064312.1655934243,
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766831489616,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.604177523437443,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.47328322230829,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.604177523437443,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.47328322230829,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 710.1692708333334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1071.8203125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.3833684327,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.563318526933,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00108,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64274.717134165745,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068659.9556995158,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.435090248622366,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.59728889987639,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.435090248622366,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.59728889987639,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.770963541666667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.9765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.84377048149,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.587719835519,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000825,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75592.09780595456,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2084367.5737108008,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.34703097413801,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.865327638502016,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.34703097413801,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 52.865327638502016,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 4080.1623697916666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 7028.4609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.69526106748,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.4738488365765,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002015,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57116.82853547039,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721749.0757736205,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766917833938,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.827479071379162,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.29719832211129,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.827479071379162,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.29719832211129,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.19544270833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.82421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.70294120407,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.580255883816,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000903,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278932.63312620786,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2066176.3227096389,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.889029958891909,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.607221094834234,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.889029958891909,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.607221094834234,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.901041666666664,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.6796875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.52419687924,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001002,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33427.05555006843,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723294.404720069,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.267620390089863,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.58415635179153,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.267620390089863,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.58415635179153,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.893619791666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.93765686985,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.592695814102,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000773,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99380.17823656539,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2062248.6555330337,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.815346366252685,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.35279665096161,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.815346366252685,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.35279665096161,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.290625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.08203125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.05838110733,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00126,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36508.17079552772,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723231.747976446,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766917836647,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.497557394443234,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 99.4465896130883,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.497557394443234,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.4465896130883,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3889.052734375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6070.01953125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108325.04285672003,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.227271406162,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004592,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57762.2605566965,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720435.0428746046,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.45078302673996,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.7115638693777,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.45078302673996,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.7115638693777,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.039583333333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.00390625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.45017162451,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5668590960995,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001043,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75239.65785112369,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2073869.508363904,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.368545253644825,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 40.349405447973545,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.368545253644825,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.349405447973545,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 706.3223958333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1008.10546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.5004907455,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.516526009511,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001569,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63937.94976454146,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2073138.767323923,
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1767004296828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.071511582604646,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.461179549899114,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.071511582604646,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.461179549899114,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.96145833333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.10546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.6397486986,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000938,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36193.73434534787,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722832.2651094928,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.501815129099143,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.856842768112585,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.501815129099143,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.856842768112585,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.37135416666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.05078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.95366338453,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.54054415938,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001318,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278718.0226460111,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2067431.6321944313,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.125439859275618,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.118203877376795,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.125439859275618,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.118203877376795,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.21471354166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.0351544454,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000719,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33585.62995442089,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724268.3903033861,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.223581343824947,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.611651025957972,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.223581343824947,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.611651025957972,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.01315104166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.7137742124,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.580830033257,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000897,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99078.22071432261,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2053964.0983612756,
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1767004298821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.4542670031732,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.80309341605274,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.4542670031732,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 51.80309341605274,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 4007.857421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6449.140625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108328.98392462876,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.436148005324,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002409,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 56894.59047634933,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722279.6160902601,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.231253059351953,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.982393555383425,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.231253059351953,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.982393555383425,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.807421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.8203125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.5621122928,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.572791951518,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000981,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75304.97831369823,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2082478.9983510985,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 28.781937658730666,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 35.979042768850434,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.781937658730666,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 35.979042768850434,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 705.512109375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 972.7890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.65238719378,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.577576521271,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000931,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64200.45853937906,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2071873.465258874,
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
          "id": "66b4c7e30dca8c44340dace1056ed5a5887366ae",
          "message": "chore(deps): update dependency psutil to v7.2.1 (#1698)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.1.3` ->\n`==7.2.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.1.3/7.2.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.1`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#721)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.0...release-7.2.1)\n\n\\=====\n\n2025-12-29\n\n**Bug fixes**\n\n- 2699\\_, \\[FreeBSD], \\[NetBSD]: `heap_info()`\\_ does not detect small\nallocations\n(<= 1K). In order to fix that, we now flush internal jemalloc cache\nbefore\n  fetching the metrics.\n\n###\n[`v7.2.0`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#720)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.1.3...release-7.2.0)\n\n\\=====\n\n2025-12-23\n\n**Enhancements**\n\n- 1275\\_: new `heap_info()`\\_ and `heap_trim()`\\_ functions, providing\ndirect\n  access to the platform's native C heap allocator (glibc, mimalloc,\n  libmalloc). Useful to create tools to detect memory leaks.\n- 2403\\_, \\[Linux]: publish wheels for Linux musl.\n- 2680\\_: unit tests are no longer installed / part of the distribution.\nThey\n  now live under `tests/` instead of `psutil/tests`.\n\n**Bug fixes**\n\n- 2684\\_, \\[FreeBSD], \\[critical]: compilation fails on FreeBSD 14 due\nto missing\n  include.\n- 2691\\_, \\[Windows]: fix memory leak in `net_if_stats()`\\_ due to\nmissing\n  `Py_CLEAR`.\n\n**Compatibility notes**\n\n- 2680\\_: `import psutil.tests` no longer works (but it was never\ndocumented to\n  begin with).\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi41OS4wIiwidXBkYXRlZEluVmVyIjoiNDIuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-29T19:06:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66b4c7e30dca8c44340dace1056ed5a5887366ae"
        },
        "date": 1767090724247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.863592911249262,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.047672468256426,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.863592911249262,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.047672468256426,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.85299479166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.73046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.9304348342,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000777,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34122.37505724343,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724634.512934206,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.968636986773237,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.026832757316036,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.968636986773237,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.026832757316036,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.20846354166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.35546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.21501767574,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002281,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36281.93874912277,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722167.8153728961,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.504650157823267,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.82209305105853,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.504650157823267,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.82209305105853,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.99270833333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.03154342096,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.597671801311,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000721,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 99311.8560371935,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2065860.9100811714,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.148442913208356,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.23555261789368,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.148442913208356,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.23555261789368,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.102473958333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.06640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.75168975866,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.582839557209,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000876,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 282900.41194181575,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2100712.079156466,
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
          "id": "66b4c7e30dca8c44340dace1056ed5a5887366ae",
          "message": "chore(deps): update dependency psutil to v7.2.1 (#1698)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.1.3` ->\n`==7.2.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.1.3/7.2.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.1`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#721)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.0...release-7.2.1)\n\n\\=====\n\n2025-12-29\n\n**Bug fixes**\n\n- 2699\\_, \\[FreeBSD], \\[NetBSD]: `heap_info()`\\_ does not detect small\nallocations\n(<= 1K). In order to fix that, we now flush internal jemalloc cache\nbefore\n  fetching the metrics.\n\n###\n[`v7.2.0`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#720)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.1.3...release-7.2.0)\n\n\\=====\n\n2025-12-23\n\n**Enhancements**\n\n- 1275\\_: new `heap_info()`\\_ and `heap_trim()`\\_ functions, providing\ndirect\n  access to the platform's native C heap allocator (glibc, mimalloc,\n  libmalloc). Useful to create tools to detect memory leaks.\n- 2403\\_, \\[Linux]: publish wheels for Linux musl.\n- 2680\\_: unit tests are no longer installed / part of the distribution.\nThey\n  now live under `tests/` instead of `psutil/tests`.\n\n**Bug fixes**\n\n- 2684\\_, \\[FreeBSD], \\[critical]: compilation fails on FreeBSD 14 due\nto missing\n  include.\n- 2691\\_, \\[Windows]: fix memory leak in `net_if_stats()`\\_ due to\nmissing\n  `Py_CLEAR`.\n\n**Compatibility notes**\n\n- 2680\\_: `import psutil.tests` no longer works (but it was never\ndocumented to\n  begin with).\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi41OS4wIiwidXBkYXRlZEluVmVyIjoiNDIuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-29T19:06:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66b4c7e30dca8c44340dace1056ed5a5887366ae"
        },
        "date": 1767090726145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 38.569592667437036,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 54.976693271389856,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.569592667437036,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.976693271389856,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3984.61796875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6335.46484375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.15046065155,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.550974414532,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001209,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57507.52874998281,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720820.2335365994,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.766536332328865,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.38481838004312,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.766536332328865,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.38481838004312,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.668489583333333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.4140625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.47544852938,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.568198772057,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001029,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 76977.47712341417,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2084722.1128037449,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 28.1291097101669,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.58944059613444,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.1291097101669,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.58944059613444,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 696.8893229166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 958.38671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.5476683226,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.572026421098,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000989,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64519.6878303943,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2079854.2142475834,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767177149658,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.52167866656504,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.12416172746202,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.52167866656504,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.12416172746202,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.41744791666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.21875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.41225628932,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.564849583334,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001064,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116809.61811805537,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2047744.297016182,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.830399711180595,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.097449362359118,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.830399711180595,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.097449362359118,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.075520833333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.38671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.27480857457,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001694,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53427.42951228684,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722433.9857893734,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.725082358542327,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.097352636061004,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.725082358542327,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.097352636061004,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.06276041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.73046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.49666365983,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.463323173971,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002125,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296000.01374545175,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2050477.1017911844,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.119899762820642,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.787018443741806,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.119899762820642,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.787018443741806,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.89401041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.15234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.30753788237,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001122,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50975.27896395004,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723952.3331319158,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767177151824,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 29.344098672329398,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 37.680054622692516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.344098672329398,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.680054622692516,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 747.7328125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1019.96875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.53322435626,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.571260890882,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000997,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64410.70040784302,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068233.6960605816,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.458104661929042,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.43527808517252,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.458104661929042,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.43527808517252,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.097135416666667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.62890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.14865516878,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.550878723946,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00121,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75423.02605196676,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2082393.056509353,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 41.31027928980244,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 76.28617597028784,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.31027928980244,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.28617597028784,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3891.005078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6866.46484375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.90311007357,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.537864833899,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001346,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58922.6103462866,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724121.9824113116,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767263503568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.923636719634194,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.386388796905223,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.923636719634194,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.386388796905223,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.813802083333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.0390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.97015604221,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000755,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50868.34732587599,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723526.1651818999,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.287892275242115,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.295908363074545,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.287892275242115,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.295908363074545,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.194661458333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.61328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.68849719632,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.579490351404,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000911,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116688.1304552087,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2062263.5958150185,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.577660512306377,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.52595985773929,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.577660512306377,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.52595985773929,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.092838541666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108321.94328099734,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.062993892859,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006309,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 295985.4079592304,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2065268.4004769246,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.038853285734596,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.394661255799566,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.038853285734596,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.394661255799566,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.19765625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.30859375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.36327587352,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001645,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53530.39601617955,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723963.8481741215,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767263505132,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 38.77893507946342,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 56.61690435643565,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.77893507946342,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 56.61690435643565,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 4071.5013020833335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6858.4609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.68103715927,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.526094969441,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001469,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58933.78295103652,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723979.9282696061,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.348333507912738,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 38.2556728756033,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.348333507912738,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 38.2556728756033,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 671.98359375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 955.33984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.58558375259,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5740359388865,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000968,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64190.67559530955,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2064835.7337787114,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.539340842975523,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.70793258266863,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.539340842975523,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.70793258266863,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.5265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.80078125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.91599076579,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.591547510587,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000785,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75735.21739345885,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2080231.8799037368,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767349854630,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.91489595228867,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.95854603960396,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.91489595228867,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.95854603960396,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.17356770833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.76025674086,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001979,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53457.00264572141,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722724.7936699,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.07209886974818,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.186170406855554,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.07209886974818,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.186170406855554,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.89309895833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.9936276788,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000742,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50562.35648502215,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722357.0866216298,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.167631518524047,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.736468356196234,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.167631518524047,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.736468356196234,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.223307291666664,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.42578125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.39059039541,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.563701290957,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001076,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 117011.13429170763,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068297.4917048248,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.40722029681245,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.13110631983993,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.40722029681245,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.13110631983993,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.9609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.94644148008,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.540161398444,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001322,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 298260.6783174372,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2082084.6018500149,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767349857170,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.9034034612018,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 50.63865283462133,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.9034034612018,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 50.63865283462133,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3718.23671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6667.0625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.47115630202,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.216971284008,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001235,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58784.99955130473,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723635.2049074195,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.34162341652928,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.83187743307209,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.34162341652928,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.83187743307209,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.8,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.73828125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.60002773289,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.574801469843,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00096,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75400.08605543354,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2074046.4494377205,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.135197536325908,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.955758561686295,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.135197536325908,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.955758561686295,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 692.69765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1020.4765625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.5819727581,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.573844556179,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00097,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63645.90865651132,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2066609.4121252694,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767436213871,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.225517447181137,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.693140721209772,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.225517447181137,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.693140721209772,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.364453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.3359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.88349162594,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.589825056175,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000803,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 117352.93929020116,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068514.6007916809,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.66759789561296,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.7590835812118,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.66759789561296,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.7590835812118,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.0578125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.64453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.8904717532,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.537195002919,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001353,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296128.9218070924,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2065158.9029157283,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.190421497062252,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.197628278954692,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.190421497062252,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.197628278954692,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.19700520833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.17578125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.08209778511,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000693,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53826.604343281855,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722725.3683010639,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.126518772228401,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.46166705517669,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.126518772228401,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.46166705517669,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.333072916666666,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.5546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.94487890648,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000769,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 51048.59498809878,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724157.9062176223,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767436215528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 29.3006433449731,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 36.56701474075221,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.3006433449731,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 36.56701474075221,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 676.2766927083334,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 933.18359375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.91418525751,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.591451818648,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000786,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63120.081988856924,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2073690.2049842565,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.592749071671268,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 34.77518324587881,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.592749071671268,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.77518324587881,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.25234375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.546875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.91057424112,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5912604347795,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000788,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75625.91202259573,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2078647.5490356397,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.01863646252403,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 57.75832916562889,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.01863646252403,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 57.75832916562889,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3948.8548177083335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6375.78125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.2350886098,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.502459696319,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001716,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 56972.32241111673,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721883.6037797726,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767522709848,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.841942912682256,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.00117310195228,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.841942912682256,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.00117310195228,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.47734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.89453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.0225158609,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.597193340627,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000726,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116830.46764242998,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2054626.3371139644,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.32338817212683,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.24760941140263,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.32338817212683,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.24760941140263,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.02747395833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.58984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.0116827908,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.596619187912,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000732,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296836.92062819557,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2055780.423345054,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.84924058034993,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 10.941641776814734,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.84924058034993,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 10.941641776814734,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.56328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.17578125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.0351544454,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000719,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 51144.648246963465,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724721.5665651477,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.793793453570903,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.738365279531804,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.793793453570903,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.738365279531804,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.20729166666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.44921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.97015604221,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000755,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 54034.15391245125,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724452.0991546864,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767522711762,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.428826920041825,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.01159033643852,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.428826920041825,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.01159033643852,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.874479166666667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.4609375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.30911220395,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.50638294681,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001675,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75457.5818726519,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068453.8849299026,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 37.709811528507394,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.58022400983254,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.709811528507394,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 51.58022400983254,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3883.448307291667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6116.1015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108328.89365417507,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.431363671279,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002459,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57709.7495809357,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722327.3900733532,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.496576886432308,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 37.39373144139265,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.496576886432308,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.39373144139265,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 695.6373697916666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1035.47265625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.16802105597,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.2539051159665,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000843,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 64604.172725545715,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2069149.2060514928,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767609169504,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6060800,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.585514459970526,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.059105001549426,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.585514459970526,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.059105001549426,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.918359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1875,
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
            "name": "logs_received_total",
            "value": 339200,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.93513921957,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.241562378637,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000974,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 116811.23463203668,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2046834.515110597,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.745518503313864,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.874772427214456,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.745518503313864,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.874772427214456,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.970052083333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.09765625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.75349526152,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000875,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53646.13803725722,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723255.9625649119,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.263776466378836,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.53803618796935,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.263776466378836,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.53803618796935,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.317708333333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.3671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.64697069548,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.577289446861,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000934,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296126.3976635383,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2054252.1395955987,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.263667642370885,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.61836944791908,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.263667642370885,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.61836944791908,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.95065104166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.4453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.22064499346,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001724,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50713.11084352375,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723066.4727071767,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767609172571,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 30.21829106937811,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 36.33208039653036,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.21829106937811,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 36.33208039653036,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 676.8087239583333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 946.92578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.55308481098,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.572313494982,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000986,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 63591.0215005332,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2059220.1579930359,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 31.30407204847127,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 32.07011826086957,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.30407204847127,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.07011826086957,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.000130208333335,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.78515625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.15046065155,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.550974414532,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001209,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 75199.0527037313,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068111.1039154835,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6155500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 40.80335002800932,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 65.34281152741312,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.80335002800932,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34281152741312,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3862.7971354166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 7240.48828125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 344500,
            "unit": "count",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.17212644941,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.552122701819,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001197,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57160.27260412416,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720684.2546706762,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "4bf9bf85b4d0c2e7992e18e3dedda40427aa2485",
          "message": "[batch_processor] Support bytes-based batching via new `format = [otap|otlp|preserve]` (#1633)\n\nFixes #1570.\n\nAdds dual format configuration to batch processor, with separate\n`FormatConfig` structs for each payload format.\nThis supports forcing payload into one or the other format, or allowing\nboth to be preserved.\n\nThe new bytes-based batching routines operate by scanning through\ntop-level fields. Unlike the items-based batching mode, this may produce\nbatches that are less than the limit; like that mode, it can also\nproduce outputs greater than the limit.",
          "timestamp": "2026-01-06T01:10:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4bf9bf85b4d0c2e7992e18e3dedda40427aa2485"
        },
        "date": 1767695614634,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.019352357093101,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 10.802998749613122,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.128515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.28515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.11592889024,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001782,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52180.21108908626,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 725922.9090265393,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.876712325869246,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.732544019820377,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.34544270833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.55078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.89432467039,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000797,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53517.194275713984,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723486.3916208673,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.38775848302962,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.868700285383724,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.32018229166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.8125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.52600237452,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.57087812585,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001001,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 119424.48283195528,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2070109.214203181,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.821075792526628,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.01963991350016,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.187109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.6015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.78960533146,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.584849082567,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000855,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296267.95376124786,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2075696.0136627213,
            "unit": "bytes/sec",
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
          "id": "4bf9bf85b4d0c2e7992e18e3dedda40427aa2485",
          "message": "[batch_processor] Support bytes-based batching via new `format = [otap|otlp|preserve]` (#1633)\n\nFixes #1570.\n\nAdds dual format configuration to batch processor, with separate\n`FormatConfig` structs for each payload format.\nThis supports forcing payload into one or the other format, or allowing\nboth to be preserved.\n\nThe new bytes-based batching routines operate by scanning through\ntop-level fields. Unlike the items-based batching mode, this may produce\nbatches that are less than the limit; like that mode, it can also\nproduce outputs greater than the limit.",
          "timestamp": "2026-01-06T01:10:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4bf9bf85b4d0c2e7992e18e3dedda40427aa2485"
        },
        "date": 1767695617741,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.42632638000111,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 48.37423286056763,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3772.7877604166665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6432.53125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.76206217735,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.477389295399,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001978,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58171.68688330759,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724217.6719156253,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 28.99415634773289,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.990982905797665,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 696.0907552083333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 978.38671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.02046604638,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.544084700458,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001281,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 67395.46024139786,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2087673.973783487,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.14887360337889,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.26405121778052,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.300130208333332,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.41015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.37253549045,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5627443809935,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001086,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 79101.08536048981,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093136.9492966435,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb",
          "message": "feat: Add 'tls' option to internal telemetry OTLP configuration (#1724)\n\nAdd 'tls' option to internal telemetry OTLP configuration with ca file.",
          "timestamp": "2026-01-06T21:31:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb"
        },
        "date": 1767782007782,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.229565813954036,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.180654182210574,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.766927083333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.75145042376,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.52982687246,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00143,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 297922.9439487783,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2086899.055935769,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.050492316069647,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.395238540779182,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.91380208333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.96293400221,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000759,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53520.04866626681,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723421.0349118926,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.840769512575156,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.62567868325477,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.101953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.68359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.37068241287,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000729,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52267.5079069014,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720556.2566047922,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.04754455852137,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.437582938623684,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.817708333333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.6875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.14120517249,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.497483874142,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001768,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 118778.99683149463,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2071801.366054916,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb",
          "message": "feat: Add 'tls' option to internal telemetry OTLP configuration (#1724)\n\nAdd 'tls' option to internal telemetry OTLP configuration with ca file.",
          "timestamp": "2026-01-06T21:31:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb"
        },
        "date": 1767782010308,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.45897900662289,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 34.00676546651181,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.713151041666666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.7137742124,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.580830033257,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000897,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 79576.30039074308,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2101154.7474697465,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.9756478901903,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.35695733931472,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3726.75,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6119.3671875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.1262827257,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5653.198692984462,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001429,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58953.15730345839,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 716687.9057929962,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.171803889276244,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.84601257293198,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 674.7841145833333,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 923.83984375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.76252277676,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.583413707168,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00087,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66852.70637227825,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2080343.8059623397,
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
          "id": "9b0aa84cfc1ef5df2f4434a3290bab3abbd7299c",
          "message": "[kql_processor] Experimental KQL recordset processor (#1730)\n\nRelates to #1642\n\n## Changes\n\n* Adds a processor which executes KQL query using the RecordSet engine\n(OTLP-bytes form)\n\n## Details\n\n@drewrelmas took @jmacd's original work and added tests + config. I\ncleaned it up a bit and improved the bridge API to allow the processor\nto own the pipeline memory instead of storing it in a static. The static\npath is in place for callers needing to invoke things using FFI (from\nnon-Rust platforms).\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@microsoft.com>",
          "timestamp": "2026-01-08T00:49:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b0aa84cfc1ef5df2f4434a3290bab3abbd7299c"
        },
        "date": 1767868441941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 20.884010579745443,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 22.222799609876926,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.641536458333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.55078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.70631370523,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.527434626377,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001455,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 118992.52400901263,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2071167.9175814362,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.346055443704646,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.83285739493847,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.192317708333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.55263194648,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.4662894931635,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002094,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 297157.85888050945,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2070182.6282128931,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 9.924751593428068,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.439755954847689,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.637890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.80078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.4391051019,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001603,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52215.56776614152,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724875.6529387231,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.722181281579253,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.993687993803253,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.23216145833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.57421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.264206187,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001146,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53953.26850233212,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722753.5239942359,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "9b0aa84cfc1ef5df2f4434a3290bab3abbd7299c",
          "message": "[kql_processor] Experimental KQL recordset processor (#1730)\n\nRelates to #1642\n\n## Changes\n\n* Adds a processor which executes KQL query using the RecordSet engine\n(OTLP-bytes form)\n\n## Details\n\n@drewrelmas took @jmacd's original work and added tests + config. I\ncleaned it up a bit and improved the bridge API to allow the processor\nto own the pipeline memory instead of storing it in a static. The static\npath is in place for callers needing to invoke things using FFI (from\nnon-Rust platforms).\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@microsoft.com>",
          "timestamp": "2026-01-08T00:49:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b0aa84cfc1ef5df2f4434a3290bab3abbd7299c"
        },
        "date": 1767868443664,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.803999408744904,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.3039714046771,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.626822916666665,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.2890625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.5819727581,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.573844556179,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00097,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 80057.9641727593,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083770.0179912841,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 37.31575552568431,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 51.8059325514467,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3970.232421875,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6485.93359375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108327.53058861147,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.359121196408,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003214,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 57323.010122967426,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 720974.6578142623,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 30.118544544031145,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 36.20695684726001,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 692.1326822916667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 990.91015625,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108321.66708978776,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.048355758751,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006462,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 66802.0257773238,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2075514.4419750073,
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
          "id": "7cffafe2cb2c3ac605852d8d87ba77b4a41b716c",
          "message": "Internal Telemetry Guidelines (#1727)\n\nThis PR defines a set of guidelines for our internal telemetry and for\ndescribing how we can establish a telemetry by design process.\n\nOnce this PR is merged, I will follow up with a series of PRs to align\nthe existing instrumentation with these recommendations.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2026-01-08T22:23:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7cffafe2cb2c3ac605852d8d87ba77b4a41b716c"
        },
        "date": 1767954753509,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.959087628887598,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.594870078667284,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.05833333333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.06640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.02432137278,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000725,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 54029.533036377936,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723855.3113861278,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.4737600217189,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 32.922365091583586,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.51588541666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.28125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108328.97309216637,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.435573884818,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002415,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296963.9439865307,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2064213.693585362,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.11095999029154,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.62881628083198,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.78346354166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.88505533108,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.536907932547,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001356,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 118910.98304981987,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2063189.7271862007,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.033133786565134,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.304219134185796,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.698567708333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.22265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.62711020635,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000945,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52224.62181330029,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 725390.0438388955,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "7cffafe2cb2c3ac605852d8d87ba77b4a41b716c",
          "message": "Internal Telemetry Guidelines (#1727)\n\nThis PR defines a set of guidelines for our internal telemetry and for\ndescribing how we can establish a telemetry by design process.\n\nOnce this PR is merged, I will follow up with a series of PRs to align\nthe existing instrumentation with these recommendations.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2026-01-08T22:23:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7cffafe2cb2c3ac605852d8d87ba77b4a41b716c"
        },
        "date": 1767954755216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.137832181538382,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 37.42163266367435,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 693.6358072916667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1003.34375,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.0116827908,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.596619187912,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000732,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 67405.83738862778,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083671.4518812364,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.46750933629358,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.73532943246399,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.085416666666667,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.92578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.20126183014,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.606666876997,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000627,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 80063.1334105283,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2084089.972932309,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTLP-FILTER-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.008198476398924,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.701492368155336,
            "unit": "%",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 3875.5783854166666,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 7413.2578125,
            "unit": "MiB",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.05093279007,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.492699437874,
            "unit": "logs/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001818,
            "unit": "seconds",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58903.89913920592,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724029.3876501112,
            "unit": "bytes/sec",
            "extra": "Nightly - OTel Collector/OTAP-FILTER-OTAP (Go Collector) - Network Utilization"
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
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768041106965,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.028619131423772,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.634206568642625,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.95364583333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.2890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108332.02432137278,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000725,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52180.382876691445,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 726355.5146033977,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 31.506628026957756,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.13952840682132,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.24557291666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.76171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.44632693872,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.5136553277525,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001599,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296480.3436854653,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2066971.6116356032,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 14.900113214761543,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.240035775781976,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.16458333333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.48828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.37614647095,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001084,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53489.48665933257,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722973.4838918763,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 94.70000457763672,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 21.312058971387675,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 21.550030506005424,
            "unit": "%",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.73046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.91796875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.46281007548,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 5741.567528934001,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001036,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 118272.46239762347,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2057468.4405070622,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTLP-FILTER-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}