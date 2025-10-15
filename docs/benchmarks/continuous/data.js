window.BENCHMARK_DATA = {
  "lastUpdate": 1760567237984,
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
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "877b2cb3dddef09143c7cb20164fd0ba789dd210",
          "message": "[query-engine] Expose allow_undefined_keys on BridgeOptions (#1286)\n\nRelates to #1281\n\n## Changes\n\n* Allow `allow_undefined_keys` on attributes schema to be set via\n`BridgeOptions`",
          "timestamp": "2025-10-15T17:12:44Z",
          "tree_id": "8e1b90c2eecf38a2918d279a1aff37c694e7aff1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/877b2cb3dddef09143c7cb20164fd0ba789dd210"
        },
        "date": 1760549176852,
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
            "value": 44.56822137867814,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 48.31473158988677,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.74921875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.50390625,
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
            "value": 18308095.725473758,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5830344.065247032,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 66.4247673710079,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 66.62334554231781,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.22265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.4765625,
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
            "value": 5881836.2689752905,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13970492.856147114,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 16.987650987994705,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.120892638103356,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.045703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5674867.567572457,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5642781.316975022,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
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
            "value": 52.19932170197998,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.14287223059592,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.784765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.13671875,
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
            "value": 14518643.933910523,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13920011.52718241,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "8164192+clhain@users.noreply.github.com",
            "name": "clhain",
            "username": "clhain"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "67d9813bd18a4accf723f190de8bf7b735ddbc86",
          "message": "[PerfTest] Update loadgen script to support static message body (#1287)\n\nThis PR updates the python loadgen script to support:\n\n1. Syslog headers as optional or either of the RFC formats (default\nrfc3164, no change to behavior).\n2. Message body may be supplied as a static string (e.g. a CEF or other\nformat message, default is still randomly-generated string).\n3. Renames the \"sent\" metric to \"logs_produced\" for consistency with the\nfake signal generator and easier reporting (no change to the existing\norchestrator.py required - it's looking at the unchanged final logged\nstring, not the metric).\n\nThis will enable additional nightly benchmarks for e.g. CEF parsing, etc\nin a followup PR.",
          "timestamp": "2025-10-15T22:14:04Z",
          "tree_id": "4552aff68287e808783b8121e734c9bc7bcdd1d7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67d9813bd18a4accf723f190de8bf7b735ddbc86"
        },
        "date": 1760567235346,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 66.06169952014723,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 66.94591546379046,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.96875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.859375,
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
            "value": 5894609.818441951,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13950976.791564692,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 17.82899661329179,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.369118849454882,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.01171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.80078125,
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
            "value": 5772704.23361472,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5640327.102239033,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
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
            "value": 43.99297583710191,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.506242782608695,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.28359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.9296875,
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
            "value": 18126888.83640864,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5753125.2062253775,
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
            "value": 52.150312447796146,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.144470217156936,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.63359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.73046875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14401669.184574902,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13864358.0087814,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}