window.BENCHMARK_DATA = {
  "lastUpdate": 1766572328078,
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
      }
    ]
  }
}