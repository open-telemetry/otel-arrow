window.BENCHMARK_DATA = {
  "lastUpdate": 1762561237470,
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
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2e2dad73f0b53c44690e7fea357cfb5d5957664a",
          "message": "Update github workflow dependencies (major) (#1290)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | major | `v5.0.0` -> `v6.0.0` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | major | `v3.30.8` -> `v4.30.8` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.0.0`](https://redirect.github.com/actions/setup-node/releases/tag/v6.0.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v5.0.0...v6.0.0)\n\n#### What's Changed\n\n**Breaking Changes**\n\n- Limit automatic caching to npm, update workflows and documentation by\n[@&#8203;priyagupta108](https://redirect.github.com/priyagupta108) in\n[#&#8203;1374](https://redirect.github.com/actions/setup-node/pull/1374)\n\n**Dependency Upgrades**\n\n- Upgrade ts-jest from 29.1.2 to 29.4.1 and document breaking changes in\nv5 by [@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot]\nin\n[#&#8203;1336](https://redirect.github.com/actions/setup-node/pull/1336)\n- Upgrade prettier from 2.8.8 to 3.6.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1334](https://redirect.github.com/actions/setup-node/pull/1334)\n- Upgrade actions/publish-action from 0.3.0 to 0.4.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1362](https://redirect.github.com/actions/setup-node/pull/1362)\n\n**Full Changelog**:\n<https://github.com/actions/setup-node/compare/v5...v6.0.0>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.30.8`](https://redirect.github.com/github/codeql-action/releases/tag/v4.30.8)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.30.7...v4.30.8)\n\n### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n#### 4.30.8 - 10 Oct 2025\n\nNo user facing changes.\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.30.8/CHANGELOG.md)\nfor more information.\n\n###\n[`v4.30.7`](https://redirect.github.com/github/codeql-action/releases/tag/v4.30.7)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v3.30.8...v4.30.7)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 4.30.7 - 06 Oct 2025\n\n- \\[v4+ only] The CodeQL Action now runs on Node.js v24.\n[#&#8203;3169](https://redirect.github.com/github/codeql-action/pull/3169)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.30.7/CHANGELOG.md)\nfor more information.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-16T15:08:49Z",
          "tree_id": "85e0d70b13b38da2b231b1aecc2937b43c090e79",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2e2dad73f0b53c44690e7fea357cfb5d5957664a"
        },
        "date": 1760628147371,
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
            "value": 43.425886744504034,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.983537932752185,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.275,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18045334.941159442,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5730788.344330749,
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
            "value": 52.52990688736323,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.27933487297921,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.088671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.25390625,
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
            "value": 14475489.602449581,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13931726.418723544,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 64.47659056661799,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 65.03250019224807,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.033984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.90234375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2600000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2600000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5829199.3204696085,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13848587.347319975,
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
            "value": 13.9265896987845,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.590494733902249,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.287890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.12109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5672587.855949966,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5641456.243110531,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ac76786947bcac5a29e688218a68038ec5fd0f83",
          "message": "Update taiki-e/install-action action to v2.62.32 (#1297)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.62.30` -> `v2.62.32` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.32`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...HEAD\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.9...v2.22.10\n\n[2.22.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.8...v2.22.9\n\n[2.22.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.7...v2.22.8\n\n[2.22.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.6...v2.22.7\n\n[2.22.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.5...v2.22.6\n\n[2.22.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.4...v2.22.5\n\n[2.22.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.3...v2.22.4\n\n[2.22.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.2...v2.22.3\n\n[2.22.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.1...v2.22.2\n\n[2.22.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.0...v2.22.1\n\n[2.22.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.27...v2.22.0\n\n[2.21.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.26...v2.21.27\n\n[2.21.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.25...v2.21.26\n\n[2.21.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.24...v2.21.25\n\n[2.21.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.23...v2.21.24\n\n[2.21.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.21.22...v2.21.23\n\n[2.21\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-16T16:01:16Z",
          "tree_id": "f052a304c90f254b4a9eae454eefc4817659da42",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac76786947bcac5a29e688218a68038ec5fd0f83"
        },
        "date": 1760631397552,
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
            "value": 42.6823206067354,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.10518230297383,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.35,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.0234375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18031934.22387833,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5721798.413656323,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 10.14442957570986,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 10.732169602736532,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.541015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1640625,
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
            "value": 5672751.461412268,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5642145.411946182,
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
            "value": 63.67720575707325,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.50635996924497,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.75625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.84765625,
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
            "value": 5845033.995455782,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13915928.77247093,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 52.713168908323226,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.33550853015752,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.797265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.09375,
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
            "value": 14514415.143957987,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13968511.165609399,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f6ca6eef6de794993da517fc9eb87bf4cf1a3e66",
          "message": "Upgrade various Go dependencies (including Collector v0.137.0/v1.43.0) (#1298)\n\nFollowing up closed Renovate PR\nhttps://github.com/open-telemetry/otel-arrow/pull/1272\n\nTargets non-breaking upgrades mentioned in that PR",
          "timestamp": "2025-10-16T16:53:36Z",
          "tree_id": "684c533661403a615afef61b393abedc5a691391",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f6ca6eef6de794993da517fc9eb87bf4cf1a3e66"
        },
        "date": 1760634438499,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 52.14357815418047,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.00036929930394,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.96484375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.3359375,
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
            "value": 14430350.64000969,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13883945.899474109,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 63.486668475774366,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.14900284879249,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.790234375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.56640625,
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
            "value": 5841587.188988263,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13845097.562520806,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 42.983990099892985,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.241426111712826,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.56328125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.98046875,
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
            "value": 18218503.413228087,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5905093.741979321,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 18.637911785310152,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.889339880165924,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.705859375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.39453125,
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
            "value": 5796549.484026316,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5715334.6118500875,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "622e1c22113ac730b5a85e39a57e8b60385fd32f",
          "message": "OTLP exporter Ack/Nack support (#1266)\n\nAdds Ack and Nack handling in the OTLP exporter.\n\nCopied from #1197.\n\nPart of #1253.\n\nTest cleanup: Modifies test_subscribe_to() to consume and return\nOtapPdata, vs declaring many `mut` vars.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-16T22:04:31Z",
          "tree_id": "e2d7d04edab3172da2434b8b87b8a7deb034cc20",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/622e1c22113ac730b5a85e39a57e8b60385fd32f"
        },
        "date": 1760653142232,
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
            "value": 65.86102350435279,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 66.19286574164914,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.084765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.19921875,
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
            "value": 5784252.276598511,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13701357.128937462,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.51312709744889,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.23425191898813,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.263671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.86328125,
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
            "value": 14312824.159606611,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13784105.545314655,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 44.74331552136076,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.42066360493827,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.640234375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.18359375,
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
            "value": 18426161.84097417,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5855240.227754915,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 15.829785749371132,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.829591562091,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.49765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.76953125,
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
            "value": 5815513.788607576,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5786718.888772544,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "ceebc61082869f03ce38d7095d2fbe5ce69baca2",
          "message": "Promote @utpilla to OTel-Arrow approver (#1300)\n\nFixes #1299\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-16T23:21:49Z",
          "tree_id": "15cd921ebc66fe62f783500fe30be5aeabfbe6db",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ceebc61082869f03ce38d7095d2fbe5ce69baca2"
        },
        "date": 1760657675501,
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
            "value": 42.48366001599919,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.991327100957086,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.151953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.18359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18041366.426227383,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5727824.96529569,
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
            "value": 67.99292844584596,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 68.51567447763503,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.06640625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.38671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5858055.773820559,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13871313.6123841,
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
            "value": 12.195950229682504,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.787447339165443,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.586328125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.015625,
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
            "value": 5643218.071081933,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5611719.04410184,
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
            "value": 52.77789091050934,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.217922023910525,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.63203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.890625,
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
            "value": 14503759.306875521,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13973484.925029656,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
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
          "id": "8b3120c2ccb0af05d8a7d872192753ee5c93c694",
          "message": "Batch processor metrics set: non-optional (#1303)\n\nThis is a cleanup. Makes the MetricSet non-optional using the default\nregistry, which removes a bunch of `if let Some(_)` phrases.\n\nPart of #1304\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-17T15:55:15Z",
          "tree_id": "13dab84be4e71692b9689d583b1ed013dcae25ed",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b3120c2ccb0af05d8a7d872192753ee5c93c694"
        },
        "date": 1760717424669,
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
            "value": 42.49960697646625,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.88300321027784,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.16484375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.81640625,
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
            "value": 18032488.365212336,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5724771.481010858,
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
            "value": 65.613626444311,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 65.99713500891542,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.276953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.9296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5847507.304441553,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13887832.898018066,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.03039990952636,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.01382874074074,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.04765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.1953125,
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
            "value": 14401451.934482265,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13861753.259835955,
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
            "value": 18.338849473416282,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.96132776119403,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.959375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.06640625,
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
            "value": 5777642.486125084,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5746120.16565449,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "882d49b75f71e09832dd712e9a818c5d7759b3be",
          "message": "Upgrade opentelemetry-proto and prost in query_engine (#1308)\n\nHopefully resolving some new test and clippy failures that just popped\nup [on main\nbranch](https://github.com/open-telemetry/otel-arrow/actions/runs/18598146365).\n\nPotentially a rust toolchain update?",
          "timestamp": "2025-10-17T20:32:04Z",
          "tree_id": "39bff9f7188ef860cbc3c48c01d825a813dad759",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/882d49b75f71e09832dd712e9a818c5d7759b3be"
        },
        "date": 1760733943386,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 17.452552644918402,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.19790566849888,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.783203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.14453125,
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
            "value": 5742584.757555239,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5717907.200125346,
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
            "value": 67.55062066102202,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 68.10164091070465,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.833203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.0703125,
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
            "value": 5854471.655310044,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13902767.250181835,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 40.97678711625735,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.37301207130069,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.848046875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.3359375,
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
            "value": 18035738.263075866,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5725330.367231988,
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
            "value": 52.33654804389947,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.85019023432063,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.978515625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.20703125,
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
            "value": 14510276.717738483,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13880107.46337227,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
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
          "id": "769b048f4a31a333a443ac2823e5a692c98c6219",
          "message": "OTLP receiver Ack/Nack support (#1268)\n\nAdds Ack and Nack handling in the OTLP receiver via a new `slots::State`\nvector with configurable max size, which is based on the `slotmap`\ncrate.\n\nCopied from https://github.com/open-telemetry/otel-arrow/pull/1197.\n\nReplaces #1246 \n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1253.\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-10-17T23:53:36Z",
          "tree_id": "0d9f8245bbb92d33baea306236d0ec5c3fa8e157",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/769b048f4a31a333a443ac2823e5a692c98c6219"
        },
        "date": 1760746129367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 2499000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.95999908447266,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 4.011062785191815,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 5.209508606791986,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 196.779296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 275.91015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 40152.22713555761,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5186127.397694262,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0480270580473659,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06495469504202328,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.1640625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3236.0793471888965,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4825.700534714814,
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
            "value": 17.04112451610674,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.540530273258074,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.1171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.83203125,
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
            "value": 5671495.957424251,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5639882.612474774,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.036107809602728004,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06936103702553753,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.805859375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.86328125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32102.104273487366,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4771.183819145253,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "d4a5b3fdb684aad1380416e181c9c5024dfdb9e2",
          "message": "Disable OTLP receiver backpressure temporarily (#1311) (#1312)\n\nSee #1311",
          "timestamp": "2025-10-18T20:06:31Z",
          "tree_id": "77869524d864cd8aa22da9cb56919d17ce7a474e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d4a5b3fdb684aad1380416e181c9c5024dfdb9e2"
        },
        "date": 1760818777819,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.206834721103673,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.45803774007782,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.87734375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.11328125,
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
            "value": 5662628.534813679,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5631869.551814811,
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
            "value": 41.7918959862664,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.24970057425896,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.766015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.66015625,
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
            "value": 18297067.301915187,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5836554.047845672,
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
            "value": 69.1835633120357,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 69.96545093129299,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.043359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.4921875,
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
            "value": 5872234.035780202,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13905577.372051757,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.732873235213425,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.07965287210114,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.77421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.88671875,
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
            "value": 14368269.47295244,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13838600.589628022,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8b32b594e159bae5197d69e9ce15550bdfa91e10",
          "message": "Update github workflow dependencies (#1314)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.30.8` -> `v4.30.9` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.62.32` -> `v2.62.33` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.30.9`](https://redirect.github.com/github/codeql-action/releases/tag/v4.30.9)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.30.8...v4.30.9)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 4.30.9 - 17 Oct 2025\n\n- Update default CodeQL bundle version to 2.23.3.\n[#&#8203;3205](https://redirect.github.com/github/codeql-action/pull/3205)\n- Experimental: A new `setup-codeql` action has been added which is\nsimilar to `init`, except it only installs the CodeQL CLI and does not\ninitialize a database. Do not use this in production as it is part of an\ninternal experiment and subject to change at any time.\n[#&#8203;3204](https://redirect.github.com/github/codeql-action/pull/3204)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.30.9/CHANGELOG.md)\nfor more information.\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.33`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...HEAD\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]: https://redirect.github.com/taiki-e/install-action/compar\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-20T10:55:28Z",
          "tree_id": "bcf8294afdedc2703626a3f75c00dbb3a45d8f4d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b32b594e159bae5197d69e9ce15550bdfa91e10"
        },
        "date": 1760958372478,
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
            "value": 64.08860165745608,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.98265915144007,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.16953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.078125,
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
            "value": 5840820.03516317,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13875819.65559842,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 41.445667194395064,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.07398741663443,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.947265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.9375,
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
            "value": 18317426.750971295,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5828837.350178412,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 15.49110869321806,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.29225630202692,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.29609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.9375,
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
            "value": 5772127.982021761,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5745848.213967973,
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
            "value": 51.51314840847172,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.97361223915229,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.574609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.734375,
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
            "value": 14451162.525198733,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13854529.505253782,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "afaf61d8b090151204377c14cbc2f30aa2d25203",
          "message": "Update all patch versions (#1313)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.1.0` ->\n`==7.1.1` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.1.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.1.0/7.1.1?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.12.2`\n-> `==2.12.3` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.12.3?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.12.2/2.12.3?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.1.1`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#711)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.1.0...release-7.1.1)\n\n\\=====\n\n2025-10-19\n\n**Enhancements**\n\n- 2645\\_, \\[SunOS]: dropped support for SunOS 10.\n- 2646\\_, \\[SunOS]: add CI test runner for SunOS.\n\n**Bug fixes**\n\n- 2641\\_, \\[SunOS]: cannot compile psutil from sources due to missing C\ninclude.\n- 2357\\_, \\[SunOS]: `Process.cmdline()`\\_ does not handle spaces\nproperly. (patch\n  by Ben Raz)\n\n**Compatibility notes**\n\n- 2645\\_: SunOS 10 is no longer supported.\n\n</details>\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.12.3`](https://redirect.github.com/pydantic/pydantic/blob/HEAD/HISTORY.md#v2123-2025-10-17)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.12.2...v2.12.3)\n\n[GitHub\nrelease](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.12.3)\n\n##### What's Changed\n\nThis is the third 2.13 patch release, fixing issues related to the\n`FieldInfo` class, and reverting a change to the supported\n[*after* model\nvalidator](https://docs.pydantic.dev/latest/concepts/validators/#model-validators)\nfunction signatures.\n\n- Raise a warning when an invalid after model validator function\nsignature is raised by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;12414](https://redirect.github.com/pydantic/pydantic/pull/12414).\nStarting in 2.12.0, using class methods for *after* model validators\nraised an error, but the error wasn't raised concistently. We decided\n  to emit a deprecation warning instead.\n- Add\n[`FieldInfo.asdict()`](https://docs.pydantic.dev/latest/api/fields/#pydantic.fields.FieldInfo.asdict)\nmethod, improve documentation around `FieldInfo` by\n[@&#8203;Viicos](https://redirect.github.com/Viicos) in\n[#&#8203;12411](https://redirect.github.com/pydantic/pydantic/pull/12411).\nThis also add back support for mutations on `FieldInfo` classes, that\nare reused as `Annotated` metadata. **However**, note that this is still\n*not* a supported pattern. Instead, please refer to the [added\nexample](https://docs.pydantic.dev/latest/examples/dynamic_models/) in\nthe documentation.\n\nThe [blog\npost](https://pydantic.dev/articles/pydantic-v2-12-release#changes)\nsection on changes was also updated to document the changes related to\n`serialize_as_any`.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-20T12:17:09Z",
          "tree_id": "9159e50890b3e666c7984aecef51ae533f975d17",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/afaf61d8b090151204377c14cbc2f30aa2d25203"
        },
        "date": 1760963474723,
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
            "value": 44.642646473867515,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.60702250019256,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.000390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.80078125,
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
            "value": 18347202.68906995,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5829916.348693403,
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
            "value": 51.94446101927157,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.439956606811144,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.85,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.12890625,
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
            "value": 14565553.665452352,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14010323.262398258,
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
            "value": 14.779178100977086,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.21574302467703,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.682421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.62109375,
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
            "value": 5755917.8799051065,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5725776.0869172495,
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
            "value": 63.52246692275393,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.70942956387017,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.62890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.75,
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
            "value": 5841415.232366545,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13839634.694604497,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "706de2a99b0ca2c57e3ee637d6d0ffc16743ac0d",
          "message": "Update opentelemetry-python monorepo to v1.38.0 (#1316)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n|\n[opentelemetry-exporter-otlp](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.37.0` -> `==1.38.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp/1.38.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp/1.37.0/1.38.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n|\n[opentelemetry-proto](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.37.0` -> `==1.38.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-proto/1.38.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-proto/1.37.0/1.38.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n|\n[opentelemetry-sdk](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.37.0` -> `==1.38.0` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-sdk/1.38.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-sdk/1.37.0/1.38.0?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-python\n(opentelemetry-exporter-otlp)</summary>\n\n###\n[`v1.38.0`](https://redirect.github.com/open-telemetry/opentelemetry-python/blob/HEAD/CHANGELOG.md#Version-1380059b0-2025-10-16)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-python/compare/v1.37.0...v1.38.0)\n\n- Add `rstcheck` to pre-commit to stop introducing invalid RST\n\n([#&#8203;4755](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4755))\n- logs: extend Logger.emit to accept separated keyword arguments\n\n([#&#8203;4737](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4737))\n- logs: add warnings for classes that would be deprecated and renamed in\n1.39.0\n\n([#&#8203;4771](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4771))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNDMuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE0My4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-20T16:39:56Z",
          "tree_id": "2f9a710ce4bbcdb98d0b8e13950fe8f9c246594c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/706de2a99b0ca2c57e3ee637d6d0ffc16743ac0d"
        },
        "date": 1760981151779,
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
            "value": 67.65408639117784,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 67.83005497289764,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.63828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.703125,
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
            "value": 5857634.047388502,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13914895.59716672,
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
            "value": 18.426457219380783,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.718110929642446,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.112890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.78515625,
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
            "value": 5953336.8150899755,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5745708.308063505,
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
            "value": 44.039574673349705,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.52133913473424,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.01875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.15625,
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
            "value": 18337117.133293696,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5829476.189358055,
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
            "value": 51.2881809038148,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.66361184886337,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.976953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.109375,
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
            "value": 14408784.060047949,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13863249.46509865,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
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
          "id": "392b4476a28e188def1fd4a984b3853b67c55808",
          "message": "perf_exporter: Ack the request (#1319)\n\nSee #1311 #1312",
          "timestamp": "2025-10-20T17:40:27Z",
          "tree_id": "402f7a6c098b43712c6d6c51186a57a5a240ffd8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/392b4476a28e188def1fd4a984b3853b67c55808"
        },
        "date": 1760982896519,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.82639084106173,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.191666106235566,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.0171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.2890625,
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
            "value": 14468859.192161437,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13921694.017260695,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 43.34991180411836,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.91425497536946,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.810546875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.203125,
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
            "value": 18376398.1134952,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5860844.9333437,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 15.281383604125073,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.71128490454757,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.233203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.265625,
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
            "value": 5752228.51551626,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5622561.683576505,
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
            "value": 61.926282527326,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 62.61069759950823,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.920703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.4375,
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
            "value": 5863200.857901671,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13887163.543566376,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "cf9629d679e9138e1a989a3304a918e7025ac96a",
          "message": "[PerfTest] remove perf orchestrator v1 code and pipeline (#1320)\n\nThis PR removes the original pipeline perf orchestration components\nwhich are unused in the new framework now active in CI jobs.\n\nSpecifically:\n\n1. Remove the original CI job that was demonstrating the go\notel-collector with and without batches enabled.\n2. Remove the original python backend (same functionality is supported\nwith a df_engine running perf-exporter for both otap and otlp).\n3. Remove the original orchestrator.py script and associated library\nfiles.\n4. Remove the original (go collector) system under test config\ndirectories - these are now located inside the various test_suites.\n5. Remove the report_templates directory - these are now located inside\nthe various test_suites.\n6. Remove the collector_batch_comparison and dataflow_engine_e2e_logs\ntest suites that were setup just to verify parity with the older\nscripts.\n7. Update readme files to reference the current state of the project.\n\n> NOTE: The new framework does not yet have the support for kubernetes\nthat the old script had. Since this isn't in use anywhere in our current\npipelines, it seems safe to remove.",
          "timestamp": "2025-10-20T21:43:57Z",
          "tree_id": "00d733815ee9d35754079fc63f5b0ac0c2b2ba24",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cf9629d679e9138e1a989a3304a918e7025ac96a"
        },
        "date": 1760997366649,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 50.943871120328566,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.84287054309144,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.610546875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.8046875,
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
            "value": 14415242.258474957,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13870469.69316941,
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
            "value": 16.633855846353633,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.965733529548448,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.66875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.10546875,
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
            "value": 5778382.25463629,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5648693.878461651,
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
            "value": 57.34804289944291,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 58.39206932922127,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.96015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.71484375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5889372.133020032,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13955693.925892126,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 43.413876904846155,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.38819239628483,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.221875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.40625,
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
            "value": 18452920.457898498,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5755426.680512549,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "32b3217385531d25d10f05571a390b06720f5e90",
          "message": "add start_periodic_telemetry calls in various processors and receivers (#1306)\n\nFixes https://github.com/open-telemetry/otel-arrow/issues/1302\n\nThis PR adds missing calls to start_periodic_telemetry in nodes which\nhave an existing metrics implementation, allowing those metrics to be\nexported prior to shutdown.\n\ndf_engine::shared::receiver and df_engine::local::processor were missing\nthe start_periodic_telemetry method definition, so it was added to those\nlocations.\n\nhttps://github.com/open-telemetry/otel-arrow/issues/1305 will further\nrevamp the api for this. Since it's likely that effort will affect how\nthe timers are canceled on shutdown, and given the current inconsistency\nin handling the shutdown signal between node implementations, I ignored\nthis cleanup for now (but happy to add it in if it makes sense).",
          "timestamp": "2025-10-21T21:01:45Z",
          "tree_id": "74722bd2484e5857e9b96b2c56a59f3c26d4c6cb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32b3217385531d25d10f05571a390b06720f5e90"
        },
        "date": 1761081384008,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.51964658018653,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.47059484708063,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.894140625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.2421875,
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
            "value": 14520744.209973251,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13946032.027315533,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 60.00787360999988,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 60.61815715434581,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.696484375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.82421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5902545.487223781,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13916279.242786821,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 42.333045615938694,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.23162113103024,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.378515625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.66796875,
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
            "value": 18422567.936725102,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5855485.524826328,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 15.939621838934464,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.52337968907108,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.508984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.6328125,
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
            "value": 5776527.491235163,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5722736.188441796,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "e74cd49091d06d5f88cb2ce56eed50b4f519ca65",
          "message": "OTLP receiver: new Settings with support for wait_for_result (#1321)\n\nIntroduces `wait_for_result` in the OTLP receiver.\n\nFixes #1311 \n\nAs I begun to understand this code, I could see ways to shorten it while\nadding new configuration support.\n\nThe new struct `Settings` follows collector practice where there is a\nwidely-used non-configuration object used to pass startup conditions,\nthis follows that convention.\n\nComments added: it was not clear at first why we construct Grpc<.> and\nOtapBatchService<.> in each request.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-10-21T21:12:37Z",
          "tree_id": "06cc0e64a3036286d3e222db1c50c14933ebf85c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e74cd49091d06d5f88cb2ce56eed50b4f519ca65"
        },
        "date": 1761082303332,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 16.841199750558282,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.263101789019125,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.251953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.0703125,
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
            "value": 5675050.2248086985,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5620374.830167895,
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
            "value": 43.93142444275937,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.09419454249632,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.462109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.9296875,
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
            "value": 18333784.458521686,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5916762.534415936,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 433000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 18.041667938232422,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 34.25844385165116,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 36.20561010526316,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.648046875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.87890625,
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
            "value": 1967000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10191986.01500082,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9768344.414320093,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04503945222159212,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.061232929481205345,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.8015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.8828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59097.82514997564,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4820.218636720484,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "david@ddahl.com",
            "name": "David Dahl",
            "username": "daviddahl"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "897561e7b02e7c12ffcfcfd92e97093b2eb8dc8b",
          "message": "[otap-dataflow] Fix batch_processor loss of NodeConfig (#1322)\n\n- Preserve NodeUserConfig out_ports/default_out_port in batch processor\nfactory to prevent lost port wiring; Fixes #1310.\n- Introduce OtapBatchProcessor::build_from_json and refactor from_config\nto delegate; no functional change beyond preserving config.\n- Add smoke test and repo config (fake-batch-perf-issue-1310.yaml)\nvalidating batch processor presence and routing to exporter.",
          "timestamp": "2025-10-21T21:41:33Z",
          "tree_id": "d28a776743ff80635de7557657dba0422f184aea",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/897561e7b02e7c12ffcfcfd92e97093b2eb8dc8b"
        },
        "date": 1761083644917,
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
            "value": 44.13400310756799,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.44655500038577,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.10859375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.4296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18247033.29835695,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5787167.3029397605,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04525230809718032,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07554452183355309,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.075390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.2265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58819.430933540614,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4796.576267822738,
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
            "value": 10.002849609744821,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 10.411024826198034,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.932421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.23046875,
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
            "value": 5667336.805032956,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5611709.801727704,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 447000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 18.625,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 34.93306376014077,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 37.90898984823722,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.903125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.20703125,
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
            "value": 1953000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10141878.27526881,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9722329.75114949,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
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
          "id": "039b5b60f8d993f29324c27502f1e526528e529e",
          "message": "revert wait_for_result=true (#1328)\n\nPart of #1311",
          "timestamp": "2025-10-22T00:15:06Z",
          "tree_id": "a010ab5266f0374fe49e5ad3f68f48f9e2fd89fb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/039b5b60f8d993f29324c27502f1e526528e529e"
        },
        "date": 1761092907483,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 17.444279084431503,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.19173831683168,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.92421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.27734375,
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
            "value": 5776378.625655549,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5619707.021135333,
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
            "value": 51.821680713539834,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.167358067213875,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.754296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.02734375,
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
            "value": 14556757.993941784,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13935601.394898001,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 43.44853025834382,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.12072468268639,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.544921875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.2421875,
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
            "value": 18408481.322386943,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5727856.925060007,
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
            "value": 62.30550727050479,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 62.76955393379523,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.672265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.8828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5894587.811876774,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13946979.70506702,
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
          "distinct": false,
          "id": "378a2b6f2c95a495e1262b62f0b3b778ed2f40a9",
          "message": "[query-engine] GetType support (#1329)\n\n## Changes\n\n* Adds `GetType` expression and recordset engine support\n* Adds `gettype` support in KQL parser which uses a constant\nmap/dictionary to translate engine types into KQL types",
          "timestamp": "2025-10-22T17:13:57Z",
          "tree_id": "cb5de10e2ca94c35e1ca10ff9976e9edec2ebcf8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/378a2b6f2c95a495e1262b62f0b3b778ed2f40a9"
        },
        "date": 1761154016462,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.38049110403332,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.27731768129402,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.0140625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.21875,
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
            "value": 14534669.09343568,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13913563.179402774,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 61.357545581431985,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 62.849696959155146,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.90703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.45703125,
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
            "value": 5891624.19839066,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13933771.112197911,
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
            "value": 12.565840610432677,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.095072582719878,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.006640625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.03125,
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
            "value": 5666567.520532641,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5611539.729957471,
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
            "value": 42.154119334943864,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.62179885908634,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.71875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.7109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18058602.347513814,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5723905.035815579,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5365df8b7edba509f23f097be5a58f534c7a928d",
          "message": "Add license header (#1331)\n\n## Changes\n- Add license header to `smoke_fake_batch_issue_1310.rs` to fix this:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/18701333499/job/53330573847?pr=1309",
          "timestamp": "2025-10-22T17:56:20Z",
          "tree_id": "22851c5e8be3fbd0d7992461f7b49c49bb83004a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5365df8b7edba509f23f097be5a58f534c7a928d"
        },
        "date": 1761156573045,
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
            "value": 40.52839897401455,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.29564705078428,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.64296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.2890625,
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
            "value": 18149273.823902175,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5754847.595540806,
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
            "value": 60.41393024757513,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 60.81244836577078,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.14765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.765625,
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
            "value": 5890621.9857249325,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13894395.781263117,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.89570321316703,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.871062633484854,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.09296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.35546875,
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
            "value": 14436593.118679598,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13843563.368254889,
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
            "value": 16.606471114443625,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.139692705918954,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.023828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.53125,
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
            "value": 5702406.411381865,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5646948.821231583,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "43d034e59fab780f5a22aa546a9e10c789edf924",
          "message": "Streamline CONTRIBUTING documents (#1332)\n\nand re-alphabetize the maintainers list.",
          "timestamp": "2025-10-22T18:29:49Z",
          "tree_id": "87a04ebcdf270cb7a796403c390e47554a334022",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43d034e59fab780f5a22aa546a9e10c789edf924"
        },
        "date": 1761158571899,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 52.0869554549695,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.28212152180637,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.032421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.44140625,
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
            "value": 14634793.221341485,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14053476.875255043,
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
            "value": 17.760727019082474,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.642124578833695,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.848828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.1875,
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
            "value": 5803233.212726985,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5747233.665948245,
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
            "value": 43.36632549143104,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.919724141351516,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.416796875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.84375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18148082.883005556,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5754614.893373813,
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
            "value": 63.24923338227838,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 63.75961623725672,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.165625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.40625,
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
            "value": 5887736.675922832,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13929162.16992467,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "david@ddahl.com",
            "name": "David Dahl",
            "username": "daviddahl"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d2008b55170aa13e599bb1dcb50124bb5179a7d4",
          "message": "[otap-dataflow] Add internal telemetry to syslog_cef_receiver (#1309)\n\nMoved the internal telemetry to this new PR from #1199, the conflicts\nduring rebase were a bit too interconnected and complex.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-10-22T18:42:27Z",
          "tree_id": "de6572015aa7e4267d768811f3db71d4eca1ad68",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2008b55170aa13e599bb1dcb50124bb5179a7d4"
        },
        "date": 1761159339524,
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
            "value": 58.019472018894,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 58.66874257483647,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.658203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.52734375,
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
            "value": 5873650.684378852,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13885858.653501987,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 41.94321970227589,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.40353711587584,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.20703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.4453125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18134916.37236073,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5748483.959828253,
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
            "value": 51.66665854838709,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.488762274021084,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.980078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.109375,
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
            "value": 14546795.26085271,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13967560.164391574,
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
            "value": 13.985960051044643,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.904703900928792,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.991015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.84765625,
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
            "value": 5802467.343465077,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5748511.242818603,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "66996f8f9b72effcb725d9a56d38963316896964",
          "message": "Consolidate otap_batch_processor tests (#1335)\n\nThe OTAP batch processor tests are split across a number of modules and\nfiles.\nThis moves them into one module and removes the configs/ test file,\ninlining the test instead.\nThis also consolidates the test `use` statements that had been\nduplicated within various scopes.\nNo functional changes.",
          "timestamp": "2025-10-22T19:11:54Z",
          "tree_id": "3eb617209e301a2e0040c80aeb1c01f12ea15ff6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66996f8f9b72effcb725d9a56d38963316896964"
        },
        "date": 1761161066746,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 8.400513083789088,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 8.983037329418128,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.023828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.1484375,
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
            "value": 5668653.696159247,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5614122.379237578,
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
            "value": 44.48112439429474,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.07768538116592,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.824609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.59765625,
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
            "value": 18248485.35892963,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5896377.178682212,
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
            "value": 51.276206018211646,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.18697435960973,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.778125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.90625,
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
            "value": 14439259.88596409,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13825140.151860869,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 58.3326420692057,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.0073091780716,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.63125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.73046875,
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
            "value": 5927330.202502765,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13984229.10388415,
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
          "id": "198624c7ce3df23926c1bf75bd5b0bb65a9679b3",
          "message": "[query-engine] Fix up DateTime attributes in OTLP bridge when schema is present (#1336)\n\n## Changes\n\n* In OTLP bridge if schema says an attribute key should be a DateTime\nbut it is found to be a String attempt an automatic conversion\n\n## Details\n\n@drewrelmas has found users to be making multiple calls into the\nrecordset engine while processing the same batch of records. In the\nfirst call a DateTime is created. In a later call that DateTime is used\nin a `summary` (eg: `bin(TimeGenerated, 10m)`). The problem is, the\nbridge input and output is OTLP where \"extended\" things like DateTimes\nbecome Strings. This is a stopgap while we look at better solutions.",
          "timestamp": "2025-10-22T23:02:21Z",
          "tree_id": "c50be36f010b5287a8bee5250fe25d2a02c5aad6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/198624c7ce3df23926c1bf75bd5b0bb65a9679b3"
        },
        "date": 1761174826431,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 12.89182174153964,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.638501363530597,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.87421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.88671875,
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
            "value": 5799914.137772666,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5644442.718206954,
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
            "value": 62.650719850584146,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 64.89768254312011,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.865625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.44921875,
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
            "value": 5922039.930786775,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13931848.062827373,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 43.18156563306261,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.72280277997059,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.430078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.78515625,
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
            "value": 18407649.019260205,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5855911.364792781,
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
            "value": 51.50034425347479,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.89557892649679,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.845703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.1484375,
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
            "value": 14538116.615125671,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13925968.313215706,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "197425009+otelbot[bot]@users.noreply.github.com",
            "name": "otelbot[bot]",
            "username": "otelbot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "55cd0f3f0484948b0800f7b49375485068d114bb",
          "message": "chore(release) Prepare Release v0.44.0 (#1337)\n\n## Release v0.44.0\n\nThis PR prepares the repository for release v0.44.0.\n\n### Changes included:\n- Updated CHANGELOG.md with release notes\n- Updated collector/otelarrowcol-build.yaml version to v0.44.0\n- Updated collector/cmd/otelarrowcol/main.go version to v0.44.0\n\n### Release Notes:\n- Upgrade to v0.137.0 / v1.43.0 of collector dependencies.\n[#1298](https://github.com/open-telemetry/otel-arrow/pull/1298)\n\n### Checklist:\n- [x] Verify CHANGELOG.md formatting and content\n- [x] Verify collector version update in\ncollector/otelarrowcol-build.yaml\n- [x] Verify collector main.go version update in\ncollector/cmd/otelarrowcol/main.go\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-10-23T00:06:33Z",
          "tree_id": "66a91744558f8489c68e439c1b2053c45aefe598",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/55cd0f3f0484948b0800f7b49375485068d114bb"
        },
        "date": 1761178644601,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.31360199757425,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.913556408668725,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.597265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.7578125,
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
            "value": 14494282.078419697,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13875062.648105463,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 59.429894766368484,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 60.04667342318059,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.151953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.98046875,
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
            "value": 5924052.563382923,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14001584.338963013,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 43.654509461138616,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.479097134896676,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.334765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.4140625,
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
            "value": 18379876.878931604,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5725706.360403511,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 13.366410967203429,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.33031407545412,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.030078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.09765625,
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
            "value": 5693892.243855907,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5639803.076657874,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
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
          "id": "c10c580add1fdeabdcd04d18e7584369c0238b4f",
          "message": "[query-engine] Implement fmt_with_indent for summary expressions (#1338)\n\nRelates to #1178\n\n## Details\n\nExamples:\n\n```\nPipeline\n├── Query: \"source\\n | extend t = gettype(EventName) | summarize Count = count(), max(SeverityNumber) by Timestamp, EventName | extend t = gettype(Count)\"\n├── Constants:\n│   └── 0 = Map: {\"Array\":\"array\",\"Boolean\":\"bool\",\"DateTime\":\"datetime\",\"Double\":\"real\",\"Integer\":\"long\",\"Map\":\"dictionary\",\"Null\":\"null\",\"Regex\":\"regex\",\"String\":\"string\",\"TimeSpan\":\"timespan\"}\n├── Initializations: []\n└── Expressions:\n    ├── Set\n    │   ├── Source(Scalar): Constant(Reference)\n    │   │                   ├── ValueType: Map\n    │   │                   ├── Id: 0\n    │   │                   └── Accessor:\n    │   │                       └── GetType(Scalar): Source\n    │   │                           └── Accessor:\n    │   │                               └── String: \"EventName\"\n    │   └── Destination(Mutable): Source\n    │                             └── Accessor:\n    │                                 ├── String: \"Attributes\"\n    │                                 └── String: \"t\"\n    └── Summary\n        ├── GroupBys:\n        │   ├── EventName = Source\n        │   │               └── Accessor:\n        │   │                   └── String: \"EventName\"\n        │   └── Timestamp = Source\n        │                   └── Accessor:\n        │                       └── String: \"Timestamp\"\n        ├── Aggregations:\n        │   ├── Count = Count\n        │   └── max_SeverityNumber = Maximum(Scalar): Source\n        │                                             └── Accessor:\n        │                                                 └── String: \"SeverityNumber\"\n        └── PostExpressions:\n            └── Set\n                ├── Source(Scalar): Constant(Reference)\n                │                   ├── ValueType: Map\n                │                   ├── Id: 0\n                │                   └── Accessor:\n                │                       └── GetType(Scalar): Source\n                │                           └── Accessor:\n                │                               └── String: \"Count\"\n                └── Destination(Mutable): Source\n                                          └── Accessor:\n                                              └── String: \"t\"\n```",
          "timestamp": "2025-10-23T20:04:49Z",
          "tree_id": "db450c1fa4aef988fe45cb5baa8c5057b4b73c79",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c10c580add1fdeabdcd04d18e7584369c0238b4f"
        },
        "date": 1761250583332,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 17.69202285671741,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 22.71500006501045,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.865625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.06640625,
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
            "value": 5773252.882288132,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5617665.387374309,
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
            "value": 44.01129090753109,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.63564642461538,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.84609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.57421875,
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
            "value": 18362480.107943874,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5823353.706694357,
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
            "value": 52.50179486363681,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.371301566933994,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.55078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.75,
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
            "value": 14617517.255907035,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14007591.234449487,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 59.42027293250883,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.73027461401471,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.209765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.0546875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5874388.9484791085,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13905073.305971626,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "fb291bfcb973afe17b8b98827bb12ebcb1d59203",
          "message": "Retry processor (stateless) Ack/Nack (#1295)\n\nReplaces stateful retry processor with new stateless Ack/Nack support.\n\nThe component has been restructured with exactly same configuration used\nby the OTel Collector retry configuration, with 3 duration fields (in\nseconds) instead of a retry limit.\n\nNew tests written. Metrics updated.\n\nNote: stores the num_items calculated from the initial data in a 3-word\ncalldata, as a temporary measure. In the 10/21/2025 SIG meeting we\ndiscussed placing num_items in another field of the context,\npotentially, that could be a separate change.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-10-24T15:24:14Z",
          "tree_id": "3f9f90fbfec88cbaa54f93732616cffa97854c9c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb291bfcb973afe17b8b98827bb12ebcb1d59203"
        },
        "date": 1761320268236,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.745172861815854,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.59263733230413,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.650390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.75390625,
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
            "value": 5879120.529619323,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5746714.077791205,
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
            "value": 43.784412018889284,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.528987594328655,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.6734375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.46875,
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
            "value": 18421360.69692671,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5857937.523788573,
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
            "value": 61.7491920463315,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 62.27404322122801,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.46484375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.4296875,
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
            "value": 5915100.385280378,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13984510.068977896,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.08409991700302,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.156495857750286,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.32734375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.71875,
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
            "value": 14462018.137093475,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13896069.688184801,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7da17f2bc9413923cd04b871e236b1878dd6523b",
          "message": "[otap-df-otap][perf] Avoid unnecessary allocation (#1346)\n\n## Changes\n- Avoid one unnecessary clone of `EffectHandler` instance in OTLP\nExporter",
          "timestamp": "2025-10-27T12:16:25Z",
          "tree_id": "39dbd1acdc022d7c1ee3e98282786a9baa050bc6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7da17f2bc9413923cd04b871e236b1878dd6523b"
        },
        "date": 1761568163328,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.504395960105356,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.570686792452825,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.812890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.19140625,
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
            "value": 14458248.261518415,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13883941.098167429,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 42.926932281921744,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.78167682328354,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.283984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.23046875,
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
            "value": 18459665.4419578,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5753707.954020978,
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
            "value": 60.6921898713621,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 61.83661329412675,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.994921875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.671875,
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
            "value": 5884390.218161703,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13923595.835131273,
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
            "value": 14.927447019171552,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.8898449054864,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.558984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.1953125,
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
            "value": 5775646.449241087,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5620159.900903176,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "129437996+c1ly@users.noreply.github.com",
            "name": "c1ly",
            "username": "c1ly"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8dbbaea29b12a972c03a7652f25979d144a93c9e",
          "message": "[otap-dataflow]basic filter processor logs followup (#1341)\n\nAdd a basic filter processor that is based on the golang version minus\nthe OTTL support\n\nUpdate to #1279 \n- fixed edge case where optional record batches are not present\n- fixed edge case where logs with no resource attribute/record\nattributes are not being filtered out\n- fixed string column not guaranteed to be a string array when downcast\nto apply regex filter\n- utilized booleanbuilder to optimize building the id filter\n\nAdd support for log filtering\n\nDefined to following rust structs that will allow us to filter on \n- resource attributes\n- log record attributes\n- log record bodies\n- log record severity text\n- log record severity number (optional)\n\n```rust\n/// struct that describes the overall requirements to use in order to filter logs\n#[derive(Debug, Clone, Deserialize)]\npub struct LogFilter {\n    // Include match properties describe logs that should be included in the Collector Service pipeline,\n    // all other logs should be dropped from further processing.\n    // If both Include and Exclude are specified, Include filtering occurs first.\n    include: LogMatchProperties,\n    // Exclude match properties describe logs that should be excluded from the Collector Service pipeline,\n    // all other logs should be included.\n    // If both Include and Exclude are specified, Include filtering occurs first.\n    exclude: LogMatchProperties,\n\n    // LogConditions is a list of OTTL conditions for an ottllog context.\n    // If any condition resolves to true, the log event will be dropped.\n    // Supports `and`, `or`, and `()`\n    #[allow(dead_code)]\n    log_record: Vec<String>,\n}\n\n/// LogMatchProperties specifies the set of properties in a log to match against and the type of string pattern matching to use.\n#[derive(Debug, Clone, Deserialize)]\npub struct LogMatchProperties {\n    // LogMatchType specifies the type of matching desired\n    match_type: LogMatchType,\n\n    // ResourceAttributes defines a list of possible resource attributes to match logs against.\n    // A match occurs if any resource attribute matches all expressions in this given list.\n    resource_attributes: Vec<KeyValue>,\n\n    // RecordAttributes defines a list of possible record attributes to match logs against.\n    // A match occurs if any record attribute matches at least one expression in this given list.\n    record_attributes: Vec<KeyValue>,\n\n    // SeverityTexts is a list of strings that the LogRecord's severity text field must match\n    // against.\n    severity_texts: Vec<String>,\n\n    // SeverityNumberProperties defines how to match against a log record's SeverityNumber, if defined.\n    severity_number: Option<LogSeverityNumberMatchProperties>,\n\n    // LogBodies is a list of values that the LogRecord's body field must match\n    // against.\n    bodies: Vec<AnyValue>,\n}\n\n/// LogSeverityNumberMatchProperties specifies the requirements needed to match on the log severity field\n#[derive(Debug, Clone, Deserialize)]\npub struct LogSeverityNumberMatchProperties {\n    // Min is the minimum severity needed for the log record to match.\n    // This corresponds to the short names specified here:\n    // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#displaying-severity\n    // this field is case-insensitive (\"INFO\" == \"info\")\n    min: i32,\n    // MatchUndefined lets logs records with \"unknown\" severity match.\n    // If MinSeverity is not set, this field is ignored, as fields are not matched based on severity.\n    match_undefined: bool,\n}\n```\n\nFuture PR\n- add trace and metric support\n\nPart of #1213\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-10-27T15:02:19Z",
          "tree_id": "c522d7ed42e1c4c0bde763a1aa7c01c2a891ba84",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8dbbaea29b12a972c03a7652f25979d144a93c9e"
        },
        "date": 1761578140561,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.336467217003227,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.317822077782058,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.1671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.50390625,
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
            "value": 5850130.448893509,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5824926.311393777,
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
            "value": 58.18341697440188,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 58.78778312210201,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.7046875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.2734375,
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
            "value": 5984585.868355459,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14035236.73711234,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.44107635398202,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.0645350196653,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.848046875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.1328125,
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
            "value": 14615901.72648356,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14036706.987960417,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 40.17495959943192,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 40.68736366648603,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.558203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.13671875,
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
            "value": 18509551.43780158,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5816665.421187701,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "l.querel@f5.com",
            "name": "Laurent Quérel",
            "username": "lquerel"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "36d4a6b782d67ae44d7324f6815064a5bce42731",
          "message": "Improve status and health endpoints (#1345)\n\nThis PR replaces the concept of aggregated phases with the concept of\n*conditions*, which aligns better with Kubernetes best practices (see\nhttps://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#typical-status-properties).\nI maintained the state machine at the `PipelineRuntime` status level,\nbut we now also track a set of three-state conditions (`True`, `False`,\n`Unknown`) commonly used in K8S (`Accepted`, `Ready`).\n\nThis change simplifies the creation of the following three global\nendpoints:\n\n* `/status`\n* `/livez`\n* `/readyz`\n\nThis PR fixes the issue #1318 (see status output below).\nThis PR also adds an alias for the metrics endpoint, which is now\naccessible from `/metrics` as well.\n\nIn a future PR, I will add events in the different exporters to track\nthe actual state of outgoing connections, in order to properly update\nthe system's readiness.\n\nBelow the output of the `/status` endpoint after a shutdown request\n(fixing #1318 ).\n\n```json\n{\n  \"generatedAt\": \"2025-10-25T02:38:05.426649+00:00\",\n  \"pipelines\": {\n    \"default_pipeline_group:default_pipeline\": {\n      \"conditions\": [\n        {\n          \"type\": \"Accepted\",\n          \"status\": \"True\",\n          \"lastTransitionTime\": \"2025-10-25T02:35:56.020482+00:00\",\n          \"reason\": \"ConfigValid\",\n          \"message\": \"Pipeline configuration validated and resources quota is not exceeded.\"\n        },\n        {\n          \"type\": \"Ready\",\n          \"status\": \"False\",\n          \"lastTransitionTime\": \"2025-10-25T02:37:39.788879+00:00\",\n          \"reason\": \"QuorumNotMet\",\n          \"message\": \"Pipeline is not ready; ready quorum all cores ready not met (0 of 1 cores ready).\"\n        }\n      ],\n      \"totalCores\": 1,\n      \"runningCores\": 0,\n      \"cores\": {\n        \"1\": {\n          \"phase\": \"stopped\",\n          \"lastHeartbeatTime\": \"2025-10-25T02:37:39.788879+00:00\",\n          \"conditions\": [\n            {\n              \"type\": \"Accepted\",\n              \"status\": \"True\",\n              \"lastTransitionTime\": \"2025-10-25T02:35:56.020482+00:00\",\n              \"reason\": \"ConfigValid\",\n              \"message\": \"Pipeline admission successful.\"\n            },\n            {\n              \"type\": \"Ready\",\n              \"status\": \"False\",\n              \"lastTransitionTime\": \"2025-10-25T02:37:39.788879+00:00\",\n              \"reason\": \"Drained\",\n              \"message\": \"Pipeline runtime (core) drained; waiting for shutdown or deletion.\"\n            }\n          ],\n          \"deletePending\": false,\n          \"recentEvents\": [\n            {\n              \"time\": \"2025-10-25T02:37:39.788879+00:00\",\n              \"type\": {\n                \"Success\": \"Drained\"\n              }\n            },\n            {\n              \"time\": \"2025-10-25T02:36:39.137776+00:00\",\n              \"type\": {\n                \"Request\": \"ShutdownRequested\"\n              },\n              \"message\": \"Shutdown requested via the `/pipeline-groups/shutdown` endpoint.\"\n            },\n            {\n              \"time\": \"2025-10-25T02:36:24.421752+00:00\",\n              \"type\": {\n                \"Success\": \"Ready\"\n              },\n              \"message\": \"Pipeline initialization successful.\"\n            },\n            {\n              \"time\": \"2025-10-25T02:35:56.020482+00:00\",\n              \"type\": {\n                \"Success\": \"Admitted\"\n              },\n              \"message\": \"Pipeline admission successful.\"\n            }\n          ]\n        }\n      }\n    }\n  }\n}\n```\n\nCloses #1318",
          "timestamp": "2025-10-27T19:06:46Z",
          "tree_id": "ba1855759fe99af8ed5468b73b0795fb283e3d5c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/36d4a6b782d67ae44d7324f6815064a5bce42731"
        },
        "date": 1761592867139,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.59804553983962,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.159240612529004,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.574609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.72265625,
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
            "value": 14465713.458812729,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13844425.339241708,
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
            "value": 13.970557531185321,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.46326685025282,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.855859375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.15234375,
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
            "value": 5758534.821138042,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5704034.550577207,
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
            "value": 59.16135525644022,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.642915656791075,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.79375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.94921875,
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
            "value": 5947111.127704246,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13936867.409526175,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 42.885973496608365,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.35582139563766,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.2765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.0546875,
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
            "value": 18453290.322141297,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5812336.035800853,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f71769d880ddce56b4525bc88d709d9a7edec65a",
          "message": "chore(deps): update github workflow dependencies (#1349)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | minor | `v4.30.9` -> `v4.31.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.62.33` -> `v2.62.40` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.31.0`](https://redirect.github.com/github/codeql-action/compare/v4.30.9...v4.31.0)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.30.9...v4.31.0)\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.40`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...HEAD\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.9...v2.22.10\n\n[2.22.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.8...v2.22.9\n\n[2.22.8]: https://redirect.github.com/taiki-e/ins\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNTYuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE1OS40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-28T06:49:11Z",
          "tree_id": "aeae92dee2f282249de21bb822e354f5549c6f0c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f71769d880ddce56b4525bc88d709d9a7edec65a"
        },
        "date": 1761635080817,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.719257488729863,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.50139606968641,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.56953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.7421875,
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
            "value": 6004821.069481811,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5781711.499282402,
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
            "value": 51.91743518732201,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.71168236927348,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.19765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.47265625,
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
            "value": 14694832.11679018,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14072442.377573323,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 59.4872272950187,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.85316749825784,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.559375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.6328125,
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
            "value": 5987370.3237113645,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14013670.694936598,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 45.030106517148106,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.71498272152874,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.59296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.37109375,
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
            "value": 18383047.033090785,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5915398.301742558,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "4165b8b384840c89e3eab8cf7b44694b293d3975",
          "message": "[query-engine] Improve accessor diagnostics\\logging in recordset engine (#1344)\n\n## Details\n\nThe goal here is to re-parent diagnostics for accessors to the root\nexpression instead of the selector.\n\nThat turns something like this:\n```\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(GetType): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Resolved constant with id '0' on line 2 at column 27 as: string\n```\n\nInto:\n```\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'Map' constant with id '0' defined on line 2 at column 27\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Evaluated as: 'string'\n```\n\nI know it is still hard to read but I think it is an improvement 🤣\n\n### Full example\n\nBefore:\n\n```\nln   1: source\nln   2:  | extend EventNameType = gettype(EventName)\n                  |               |       | [Verbose] StaticScalar(String): Evaluated as: 'EventName'\n                  |               |       | [Verbose] StaticScalar(String): Resolved 'event_name' value for key 'EventName' specified in accessor expression\n                  |               |       | [Verbose] ScalarExpression(Source): Evaluated as: 'event_name'\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(GetType): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Resolved constant with id '0' on line 2 at column 27 as: string\n                  | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'EventNameType'\n                  | [Verbose] StaticScalar(String): Resolved 'Map' value for key 'Attributes' specified in accessor expression\n                  | [Verbose] MutableValueExpression(Source): Evaluated as: Map write for 'EventNameType' key\n                  | [Verbose] SetTransformExpression: Map key 'EventNameType' created on target map\n```\n\nAfter:\n```\nln   1: source\nln   2:  | extend EventNameType = gettype(EventName)\n                  |               |       | [Verbose] StaticScalar(String): Evaluated as: 'EventName'\n                  |               |       | [Verbose] ScalarExpression(Source): Resolved 'event_name' value for key 'EventName' specified in accessor expression\n                  |               |       | [Verbose] ScalarExpression(Source): Evaluated as: 'event_name'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'Map' constant with id '0' defined on line 2 at column 27\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Evaluated as: 'string'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'EventNameType'\n                  | [Verbose] MutableValueExpression(Source): Resolved 'Map' value for key 'Attributes' specified in accessor expression\n                  | [Verbose] MutableValueExpression(Source): Evaluated as: Map write for 'EventNameType' key\n                  | [Verbose] SetTransformExpression: Map key 'EventNameType' created on target map\n```",
          "timestamp": "2025-10-28T22:19:06Z",
          "tree_id": "c83d37d5f6f7920f8c9b8bfc75e02f45dc2ff7af",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4165b8b384840c89e3eab8cf7b44694b293d3975"
        },
        "date": 1761690616049,
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
            "value": 55.7830360464185,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 56.69948474618115,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.058984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.12890625,
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
            "value": 5961476.069253576,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13968737.970686484,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 43.55628893488374,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.57739117556309,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.33359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.375,
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
            "value": 18338376.98567369,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5888809.134619648,
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
            "value": 52.11397396888018,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.85490196078432,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.77578125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.08203125,
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
            "value": 14744997.272321215,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14122157.840314716,
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
            "value": 18.75357477311113,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.53438111816633,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.658984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.85546875,
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
            "value": 6033846.5395208895,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5798780.987413925,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "distinct": false,
          "id": "752873fe128a034720f2b3ae2a38b7729419f74d",
          "message": "OpenTelemetry data model primer for OTAP-based query engines (#1353)\n\nThis document captures some of the details required to \"correctly\"\naggregate OpenTelemetry data in a pipeline, as we are interested in\ndoing.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2025-10-29T15:38:40Z",
          "tree_id": "2ba47bd8752b841405ec0243be7bca15f9982f98",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/752873fe128a034720f2b3ae2a38b7729419f74d"
        },
        "date": 1761753001168,
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
            "value": 40.23176112306739,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 40.62869206280455,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.56640625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.41796875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18156397.66128782,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5809158.62093327,
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
            "value": 52.180016497100134,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.72434469616246,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.973828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.20703125,
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
            "value": 14534324.977873448,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13983591.004048456,
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
            "value": 8.990812832391365,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 10.221610036352386,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.98359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.19140625,
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
            "value": 5853434.110806581,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5803891.596098909,
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
            "value": 56.57898638169596,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 57.21475892895837,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.9078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5983340.575607426,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14014991.429870447,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "04bd314ab4e064e4702a8c3730b48dfa30b57107",
          "message": "Add tonic timeout config (#1356)\n\nfixes: #1327\n\n## Summary\nAdds optional timeout configuration field to all gRPC clients\n(exporters) and servers (receivers) using Tonic's built-in timeout\nsupport.\n\n## Changes\n\n- Added optional `timeout` field to OTAP Exporter, OTLP Exporter, OTLP\nReceiver, and OTAP Receiver configs\n- Applied timeout via `Endpoint::timeout()` for clients and\n`Server::builder().timeout()` for servers\n- Uses `humantime-serde` for human-readable duration format (e.g.,\n`\"30s\"`, `\"5m\"`, `\"500ms\"`)\n- Added config parsing tests to verify timeout deserialization\n\n## Default behavior\n\n- When timeout is not specified, no timeout is applied (requests can run\nindefinitely). This matches Tonic's default behavior and maintains\nbackward compatibility with existing configurations.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2025-10-29T16:22:57Z",
          "tree_id": "5a96657207ddf8c000d50bbbd3f656e25fc0569a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/04bd314ab4e064e4702a8c3730b48dfa30b57107"
        },
        "date": 1761757070648,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.540947294638094,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.56819072405533,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.93671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.08203125,
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
            "value": 14483051.362330597,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13913660.951251928,
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
            "value": 17.52696880358709,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.26793799464627,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.162890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.43359375,
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
            "value": 5857925.387023851,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5804021.735729629,
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
            "value": 43.58447021628022,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.10744153929372,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.982421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.828125,
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
            "value": 18359495.277700637,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5881772.863638913,
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
            "value": 61.096248222523805,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 63.50028788117893,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.866015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.47265625,
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
            "value": 5981582.757231381,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14010740.574490607,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
          "distinct": false,
          "id": "ae81a06ed253da7215d1cd9eb7338b7d431a2700",
          "message": "[PerfTest] add workflow to manually trigger perf-tests for a given pr (#1361)\n\nThis workflow will allow folks with the correct permissions to run a\ntext-output-only version of the benchmark suite on the same bare metal\nrunner used on commits to main / nightly. This is needed to evaluate\nperformance of some PRs prior to merge.",
          "timestamp": "2025-10-29T17:54:48Z",
          "tree_id": "dff4b836d0db52b3732669c047264ae033d45800",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ae81a06ed253da7215d1cd9eb7338b7d431a2700"
        },
        "date": 1761761191222,
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
            "value": 42.84230227580637,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.321830848845295,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.069140625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.6875,
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
            "value": 18473954.66528983,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5913703.520198574,
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
            "value": 58.18311531393474,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.49196218226716,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.743359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.13671875,
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
            "value": 5955890.871823018,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13981575.067116518,
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
            "value": 15.490171512988491,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.912387365819756,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.981640625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.328125,
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
            "value": 5855172.626089239,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5699107.500167718,
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
            "value": 51.838907742111196,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.09051723731181,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.849609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.17578125,
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
            "value": 14669774.129976358,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14036755.3151788,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cecabd3ade984053be54f9beb316cb163e58b444",
          "message": "[otap-df-otap] Fix ACK and NACK metrics for OTLP Receiver (#1359)\n\n## Changes\n- Fix counting of ACKs and NACKs by tracking the ACK/NACK responses\nseparately\n- Minor refactoring for `route_ack_response` and `route_nack_response`\nmethods",
          "timestamp": "2025-10-29T19:39:22Z",
          "tree_id": "253e629993cb95a11c4b0c1de72a78316168e6ec",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cecabd3ade984053be54f9beb316cb163e58b444"
        },
        "date": 1761767598414,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 18.531912000850852,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 19.624399409594094,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.141015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.83984375,
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
            "value": 5858048.1212230455,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5804336.392012816,
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
            "value": 57.45845113694662,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 58.04856169818603,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.935546875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.015625,
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
            "value": 5946307.782099507,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13938597.988617858,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -2005000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -83.54166412353516,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 51.77385518576854,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.3877886853082,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.85859375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.99609375,
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
            "value": 4405000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14471648.565948447,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13898401.651861945,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 43.51349886278755,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.1172271689956,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.61953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.7734375,
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
            "value": 18410423.635264806,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5911435.637585981,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fa2dd90f0da9daa8ea2221505d32f6863067663e",
          "message": "chore(deps): update python docker tag to v3.14 (#1350)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| python | final | minor | `3.13-slim` -> `3.14-slim` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNTYuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE1Ni4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-29T19:58:01Z",
          "tree_id": "c8b86efb6be6dc437288b5bd99a27262f65e73e9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fa2dd90f0da9daa8ea2221505d32f6863067663e"
        },
        "date": 1761768564186,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.32253155254228,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.89925755893947,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.227734375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.49609375,
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
            "value": 14448408.034617525,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13881643.478592044,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 42.94435534476084,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.488856056031125,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.93671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.19140625,
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
            "value": 18462666.671194468,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5912714.912432585,
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
            "value": 58.92488213417651,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.307118648252455,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.62890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.859375,
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
            "value": 5943143.638527498,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13933884.211972734,
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
            "value": 16.229243631447044,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.346059127233683,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.029296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.375,
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
            "value": 5860261.414000209,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5801362.932109455,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lalit_fin@yahoo.com",
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "64824b400b6924b6da86f7d17196e8bafe906d3a",
          "message": "Add Microsoft Geneva exporter for Otel Arrow (#1355)\n\nfixes: #1354 \n\n## Implementation Details\n\n- Currently implements multi-stage transformation:\n  - OTAP (Arrow RecordBatch) → OTLP (protobuf) → Geneva-format\n- The long-term goal is to implement zero-copy OTAP → Geneva-format\nconversion\n- Located in experimental/ directory as this is Microsoft internal\nfunctionality. Will be moved to contrib once repo is available.\n\n\nTODO - the test coverage is less. If it is ok, I can extend that in\nsubsequent PR.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-10-29T20:46:32Z",
          "tree_id": "90d89250033155939f543f18db7f6843b3db6c37",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/64824b400b6924b6da86f7d17196e8bafe906d3a"
        },
        "date": 1761771857398,
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
            "value": 43.07512698303258,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.648384523497185,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.92265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.734375,
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
            "value": 18453797.32586316,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5782646.112533479,
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
            "value": 50.970046588224726,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.008461677119634,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.54296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.62890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14353404.318925565,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13795055.100489423,
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
            "value": 17.03740112193943,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.89064338858195,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.4296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.76171875,
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
            "value": 5857755.051967626,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5700869.237144751,
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
            "value": 61.496533376732124,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 62.60547866831073,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.795703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.640625,
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
            "value": 5932990.891122847,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13903922.675397053,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
          "distinct": false,
          "id": "634c94119bda341454aeebdb879ba72960693dff",
          "message": "[PerfTest] fix bad default path to suite configs (#1364)\n\nFix the accidentally set default test_suite path relative to the repo\nroot instead of to the tools/pipeline_perf_test directory.",
          "timestamp": "2025-10-29T21:12:50Z",
          "tree_id": "a6bef0c33f8f5f703a0963c906622fe161579c01",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/634c94119bda341454aeebdb879ba72960693dff"
        },
        "date": 1761773076460,
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
            "value": 43.2538697234312,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.00848171169091,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.871875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.921875,
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
            "value": 18453769.801330823,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5909513.919311536,
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
            "value": 52.329397035702385,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.97157959694232,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.30859375,
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
            "value": 14587806.550543841,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14035251.947600335,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 58.77325157859955,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.316048843147875,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.62109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.85546875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5961579.61564251,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13985982.536374029,
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
            "value": 10.494437670278797,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.16009789999232,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.443359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2200000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2200000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5763802.088026488,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5708402.310615012,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "de6644e935e34a8c85e1aabd64fa9c2b688d9a02",
          "message": "[otap-df-otap] Add ACK and NACK support for OTAP Receiver (#1358)\n\nTowards #1324 \n\n## Changes\n- Add ACK and NACK support for OTAP Receiver",
          "timestamp": "2025-10-29T23:20:45Z",
          "tree_id": "1f289f207f5ea8779bc1bfb42598282ba6b45994",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de6644e935e34a8c85e1aabd64fa9c2b688d9a02"
        },
        "date": 1761780855960,
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
            "value": 59.428175656366946,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.82070931692308,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.278515625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.83984375,
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
            "value": 5971107.2193762055,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13987552.286667803,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 40.9629129461421,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.79035432831185,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.98046875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18161780.372703627,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5811662.6849075835,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 17.02416385265078,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.157730233713796,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.90625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.140625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2200000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2200000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5761308.231938024,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5705561.439779666,
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
            "value": 50.8580211958931,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.76992227371022,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.84140625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.140625,
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
            "value": 14439736.409013519,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13867488.301635932,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "66651184+utpilla@users.noreply.github.com",
            "name": "Utkarsh Umesan Pillai",
            "username": "utpilla"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "71bc65ad49125e3d320b2b118c74034b725f5f03",
          "message": "[otap-df-otap] Use a better name for OTAP Receiver config setting (#1365)\n\nRelated to\nhttps://github.com/open-telemetry/otel-arrow/pull/1356#discussion_r2474120888\n\n## Changes\n- Rename `message_size` to `response_stream_channel_size` to better\nreflect the purpose of this setting",
          "timestamp": "2025-10-30T00:53:51Z",
          "tree_id": "563943c8277f8b0f45a6d48a2daee9a5e5d70aed",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/71bc65ad49125e3d320b2b118c74034b725f5f03"
        },
        "date": 1761787664771,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 52.145184337010086,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.54971751839517,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.91171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.234375,
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
            "value": 14636375.143505907,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14077388.84126378,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.299503295043326,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 21.380408060860116,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 336.37578125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 551.11328125,
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
            "name": "network_tx_bytes_rate_avg",
            "value": 212768.58694144353,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12142989.809113704,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03815452682144027,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.05561511291076156,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 6.95078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 7.04296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3303.178991908068,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4923.6958896426395,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.026091181281444734,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.046242107695863184,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 8.634375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 8.69921875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3304.6180413113107,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4925.723458633528,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
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
          "id": "142f76db4b67b1eb84621d42d67b5559d3c90d14",
          "message": "[query-engine] Add select expression (#1363)\n\nRelates to #1362\n\n## Changes\n\n* Adds an expression for selecting data from a scalar result using a\npath potentially not known until execution time\n\n## Details\n\nWorking towards KQL `extract_json` expression support.",
          "timestamp": "2025-10-30T16:09:59Z",
          "tree_id": "d60040c5cee5a807d555094f3b0a6632e95ec90a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/142f76db4b67b1eb84621d42d67b5559d3c90d14"
        },
        "date": 1761841351986,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.3505168682758,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.65357364474493,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.743359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.07421875,
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
            "value": 14454057.66463612,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13879321.542450476,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.026456356771975303,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.043614062258014556,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.625390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.80859375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3303.646302893491,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4924.241219828184,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2400000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.341300652647705,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.76479760710391,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 337.80390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 554.0703125,
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
            "name": "network_tx_bytes_rate_avg",
            "value": 215148.8178595285,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12333977.286959933,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.022036136152129362,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.03494170381299992,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 8.65625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 8.65625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3281.388444915627,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4892.470894431907,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "b2f68af4ca3dc84ff779b52bed48efcf50f0d904",
          "message": "[query-engine] KQL extract_json expression (#1366)\n\nRelates to #1362\n\n## Changes\n\n* Implement support for `extract_json` in KQL parser",
          "timestamp": "2025-10-30T16:45:42Z",
          "tree_id": "f4d69b2e996d5c1f2e1b99939467b9fdd689c8f3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b2f68af4ca3dc84ff779b52bed48efcf50f0d904"
        },
        "date": 1761843668167,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.023386244128822536,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.03629384077669903,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.49375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.5546875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3281.0869638583963,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4891.893032059574,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03481242318268538,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06652787855317172,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 6.79609375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 6.87109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3265.103983205533,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4868.321850308849,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.146602889266402,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.320864348767103,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 341.318359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 554.79296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 210814.97257745196,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12146678.074853085,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.79353791489134,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.48706760628824,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.9546875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.21484375,
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
            "value": 14542535.26527341,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13995834.517276261,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
          "distinct": false,
          "id": "6fbe82db3912ebda6bda8694c4ff02d924814928",
          "message": "[PipelinePerf] Try cleanup cache to fix disk space issues in runners (#1383)\n\nTrying to see if this can unblock CI immediately. Can move the cleanup\nto a common script etc. once this is confirmed to be helping.",
          "timestamp": "2025-11-04T19:26:16Z",
          "tree_id": "08f051ff9876a7beaa0dcaca0563b9138ecc8ef9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6fbe82db3912ebda6bda8694c4ff02d924814928"
        },
        "date": 1762285722360,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.437521213634675,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.33970182099482,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.873828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.1875,
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
            "value": 14539278.550131505,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13916456.959177572,
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
            "value": 16.11183268916865,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.265184752215802,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.60234375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.4375,
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
            "value": 5862344.300589802,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5808432.06396172,
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
            "value": 45.73223214303021,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.56096205461437,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.2125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.65625,
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
            "value": 18471304.6331133,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5915515.432016499,
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
            "value": 56.200582902747996,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 57.15060738974241,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.84375,
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
            "value": 5987458.771025588,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14038562.334332693,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "b06c055168630b8a3f2ee859f2d62428e82a89de",
          "message": "[PipelinePerf] setup verification of manual workflow runs for PRs (#1381)\n\nThese changes allow automated verification of a manually triggered perf\ntest workflow for a given PR number. Because of how github actions work\nfor this weird case, it's a 2 step thing:\n\n- The new workflow file below (pipeline-perf-verify-pr.yaml) inserts a\nplaceholder status into PRs on create/update that will remain \"pending\"\nindefinitely.\n- The original manual workflow now updates that placeholder status with\nthe result of it's own run (for the specified PR number).\n\nSo the flow is:\n- New PR created, new job inserts the pending status result\n- Maintainer triggers the manual run with target PR number\n- Manual job runs and sets the previously pending result to pass/fail\n\nNew commits will clear the result back to pending.",
          "timestamp": "2025-11-04T22:41:54Z",
          "tree_id": "c7fbd740e10ba97e3132daff51a0b69fa1aff57e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b06c055168630b8a3f2ee859f2d62428e82a89de"
        },
        "date": 1762297015717,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.965402461639286,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.86835361303211,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.04375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.3671875,
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
            "value": 14606402.613772955,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14031461.130036533,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 43.53137031773665,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.86984034674923,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.363671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.02734375,
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
            "value": 18451811.24447623,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5918405.101417085,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 14.545566299681361,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.14520285292518,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.75390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.234375,
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
            "value": 5736552.738144288,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5680214.1255550515,
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
            "value": 57.63466928911751,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 58.31960146104647,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.575,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.609375,
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
            "value": 5926461.117912276,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13849550.035061542,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
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
          "id": "a371625656c267fdb101840e8799e5fd602dd58f",
          "message": "Copy otel-arrow-rust pdata/otlp to otap-df-pdata/src/otlp (#1385)\n\nThis copies the procedural macros from otel-arrow-rust.\n\nThe `cargo xtask compile-proto` is updated to call these procedural\nmacros, to build the builder pattern for OTLP message objects.\n\nSome `use` statements in `otap-df-otap` now refer to\n`otap-df-pdata/src/proto`, which is generated by the xtask, i.e., we've\nmoved the generated protos from otap-df-otap to otap-df-pdata.\n\nNo functional changes. The clippy settings in otap-dataflow are more\nstrict, so minor changes were required.\n\nFollowing this PR, the otel-arrow-rust/src/pdata package is future dead\ncode. The rest of otel-arrow-rust can be moved, likely in a single move,\nto refer to the now-exact copies of the proto in otap-df-pdata.\n\nCorrespondences:\n\n- otel-arrow-rust/src/pdata/otlp/* copied to\notap-dataflow/crates/pdata/src/otlp/* (`derive` -> `macros`)\n- otel-arrow-rust/build.rs copied to otap-dataflow/xtask/src/genproto.rs\n- otel-arrow-rust/src/proto/mod.rs copied to\notap-dataflow/crates/pdata/src/proto/mod.rs\n- otel-arrow-rust/src/pdata/mod.rs (portions) added to\notap-dataflow/crates/pdata/src/lib.rs",
          "timestamp": "2025-11-05T15:54:34Z",
          "tree_id": "cb38da9ed7a3a410a68a15ecf8c75da1b15afd6d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a371625656c267fdb101840e8799e5fd602dd58f"
        },
        "date": 1762359548374,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.87031149012161,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.82370262901354,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.3765625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.078125,
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
            "value": 6003383.238473505,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5783806.4063565135,
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
            "value": 42.82994277104441,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.28254969180734,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.578125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.0546875,
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
            "value": 18150463.009745046,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5808846.844697029,
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
            "value": 51.43793085646446,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.16612775860735,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.0671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.40234375,
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
            "value": 14635702.245201519,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14014119.620257948,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 59.88934582761074,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 60.71656562829425,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.071875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.171875,
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
            "value": 5963348.525113507,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13958434.916502178,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "026a521ba988bb1f512de5b20c6f33ad78ec6dde",
          "message": "[PipelinePerf] - move disk cleanup to helper script (#1388)\n\nThis was indicated in a [slack\nthread](https://cloud-native.slack.com/archives/C08RRSJR7FD/p1762220210415279).",
          "timestamp": "2025-11-05T19:37:31Z",
          "tree_id": "7a1142ae8ff654d9877cce3f768e7161e372f309",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/026a521ba988bb1f512de5b20c6f33ad78ec6dde"
        },
        "date": 1762372322122,
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
            "value": 62.29184326722898,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 63.01724411423264,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.222265625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.6484375,
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
            "value": 5942613.135045031,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13875608.795377752,
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
            "value": 10.99491121520899,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.027928543328429,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.378515625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.1953125,
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
            "value": 5916461.934333503,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5862189.765683402,
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
            "value": 45.255344220662366,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.66261059322688,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.357421875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.8046875,
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
            "value": 18432462.597018596,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5916603.760002383,
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
            "value": 51.04939106865527,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 51.87625732480148,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.598828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.8671875,
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
            "value": 14551112.086649437,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13973772.386529138,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
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
          "id": "e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a",
          "message": "[otel-arrow-rust] Replace Snafu with thiserror (#1389)\n\nPart of #1377 #867 \nReplaces snafu with thiserror.\n\nHere's my retrospective on Snafu vs. Thiserror:\n\nSnafu has a little more sugar, maybe too much magic. E.g., options and\nresults both have context extensions, with thiserror you choose\nok_or_else() or map_err() explicitly. While I admire Snafu for its\nautomation, I prefer the more sparing approach taken by thiserror.",
          "timestamp": "2025-11-05T22:16:25Z",
          "tree_id": "913396d4b14dc04b9c5d186c579067b2bedfd7bc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a"
        },
        "date": 1762381867883,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 51.69665504931047,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.36858729937195,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.183203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.41796875,
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
            "value": 14595796.166668957,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14027696.704155225,
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
            "value": 17.260990116687733,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.629191810925995,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.112890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.5625,
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
            "value": 6032822.0685384255,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5810832.658947052,
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
            "value": 57.44389752653885,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 57.974008614065966,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.280078125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.25390625,
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
            "value": 5984443.490036525,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14025915.891229484,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 44.895962048198136,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.35114628659954,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.55390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.7421875,
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
            "value": 18457623.725937642,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5912553.861093126,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "66e06074e5ef4ab6bd641a93cc98978b3822870b",
          "message": "Initial support for directly encoding OTAP from OTLP bytes (#1386)\n\npart of #387 \n\nHandles `MetricsData`, `ResourceMetrics`, `ScopeMetrics` and `Metrics`\nproto messages. Metrics data will be handled in immediate followup PRs.\n\nAlso fixes an issue where metrics metadata was not getting decoded\ncorrectly when converting from OTAP to OTLP Bytes.",
          "timestamp": "2025-11-06T12:26:15Z",
          "tree_id": "785000474e999f4f801710aedc6ada7f1a34ec7c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66e06074e5ef4ab6bd641a93cc98978b3822870b"
        },
        "date": 1762433012202,
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
            "value": 41.452440960221466,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 42.21809561972594,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.00625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.13671875,
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
            "value": 18357113.680329494,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5890052.718904549,
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
            "value": 52.385470396405964,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.8066564528534,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.206640625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.4453125,
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
            "value": 14650521.923240045,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14022158.127451682,
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
            "value": 18.5755685579867,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.847795239643826,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.54296875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.33984375,
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
            "value": 5832320.918162941,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5678273.4780000355,
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
            "value": 57.938094711034275,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 58.41590494168534,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.687890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.1484375,
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
            "value": 5986003.573677956,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14021631.952983458,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "b2a371d7fd8e489fec38d89f6b3b3538f0165235",
          "message": "[PipelinePerf] replace marker + manual verification logic with labels (#1391)\n\nSince the PR marker and manual workflow steps weren't working well, this\nPR switches the logic to a single label-based workflow (the manual\nworkflow is left in place for executing specific suites):\n\nWhen a new PR is created:\n- Job 1 (on the standard ubuntu-latest runer) checks for the existence\nof a label \"pipelineperf\" on the PR.\n- If label is not found, Job 1 fails (so there's a better visual\nindicator that it hasn't run, rather than the 'skipped' icon we see with\nthe cargo bench job).\n- If the label is found, Job 1 sets an output that tells Job 2 to run\n(on the dedicated hardware).\n\nUpdates to the PR once the label is applied will automatically run the\nperf tests again, so we should avoid doing this until the PR is\nbasically ready to merge in order to avoid over-use of the dedicated\nrunner.",
          "timestamp": "2025-11-06T15:42:08Z",
          "tree_id": "e2a40d7085d993b3b0d45e8093133efa066b0c2d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b2a371d7fd8e489fec38d89f6b3b3538f0165235"
        },
        "date": 1762444606769,
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
            "value": 64.02700930830903,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 65.61233854460824,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.91171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.03515625,
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
            "value": 5954537.97720263,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13953162.975504633,
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
            "value": 13.014293959375618,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.342748353804557,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.563671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.20703125,
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
            "value": 6045716.880551123,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5811005.244155669,
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
            "value": 51.61732991942195,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.427359975213015,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.276953125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.5859375,
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
            "value": 14633065.185270656,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14014765.170031458,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 45.166279079131364,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.770469309838475,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.716796875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.76171875,
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
            "value": 18453771.03900703,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5809985.757702436,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d2b7a28b6a6bcc4844e45296fbe9be4c6a27e8ee",
          "message": "OTLP bytes -> OTAP encoding for Metric data: Gauge & NumberDataPoint (#1393)\n\npart of #387\n\nSupport direct encoding OTLP Bytes -> OTAP for Metric data of type Gauge\nincluding the NumberDataPoints.\n\nWill continue to support additional data types and other metric messages\nin followup PRs.",
          "timestamp": "2025-11-06T16:30:01Z",
          "tree_id": "3c53a8c24c8003d6e0c0e789d6c29b45969301e4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2b7a28b6a6bcc4844e45296fbe9be4c6a27e8ee"
        },
        "date": 1762447524078,
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
            "value": 59.010869693942816,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.54253847953216,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.88203125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.26171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2500000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5953848.132021574,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13976136.493525088,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 51.42196783841897,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.62464945958429,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.59140625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.90625,
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
            "value": 14503559.311961764,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13915067.388623819,
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
            "value": 17.800118369429583,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.579353984217857,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.56328125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.8671875,
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
            "value": 5831026.283506715,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5773292.693584088,
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
            "value": 43.86413750948147,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.417695336017346,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.42890625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.41796875,
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
            "value": 18425409.83602916,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5918959.940143057,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
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
          "id": "e1540ad5c3bd034d482399bcb4f400181f5fee81",
          "message": "Move otel-arrow-rust into otap-dataflow (#1395)\n\nPart of #867 \nFixes #1377\n\nThis is a move, no real code changes were required, only to replace\n`otel_arrow_rust::` with `crate::` (in crates/pdata) or\n`otap-df-pdata::` (in crates/otap). Top-level module files were merged\nwith the existing, so several files were manually combined, but there\nwere no name conflicts.\n\nAll relative paths change from otel-arrow-rust/${x} to\notap-dataflow/crates/pdata/${x}.\n\nIt is a pleasure to finally combine these two codebases. Thanks to\n@v0y4g3r for the original code structure here. A lot has changed since\nthe first commit!\n\nThis crate had a `#[allow(missing_docs)]` declaration. I've removed it\nfrom the crate-level and applied it in dozens of locations instead. I\nwill file a new issue reminding us that there are a lot of missing docs\nin this crate; I think it will be easier to finish this documentation\nnow that it's more solidified.",
          "timestamp": "2025-11-06T17:36:38Z",
          "tree_id": "3159cfe3417f33210d824dde47d4c49d60ed49fb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1540ad5c3bd034d482399bcb4f400181f5fee81"
        },
        "date": 1762451504698,
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
            "value": 61.249142283070434,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 62.13811890558767,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.1734375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.3515625,
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
            "value": 5905286.691540011,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13803320.322883602,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 52.11168564101285,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.82048832391422,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.119921875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.3359375,
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
            "value": 14600568.92720384,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14033998.785031086,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 44.061651183653375,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.42456262508727,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.783984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.94921875,
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
            "value": 18361866.17189018,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5893217.712054471,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
            "value": 17.892953070029243,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.808531293754818,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.86015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.1796875,
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
            "value": 5839008.640487211,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5784931.551274592,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "54a6d2df04a9e83266893c889a837ce48905174a",
          "message": "chore(deps): update github workflow dependencies (#1373)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[actions/github-script](https://redirect.github.com/actions/github-script)\n| action | minor | `v7` -> `v7.1.0` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.31.0` -> `v4.31.2` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.25.3` -> `1.25.4` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | patch | `v2.13.1` -> `v2.13.2` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.62.40` -> `v2.62.47` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/github-script (actions/github-script)</summary>\n\n###\n[`v7.1.0`](https://redirect.github.com/actions/github-script/releases/tag/v7.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/github-script/compare/v7.0.1...v7.1.0)\n\n#### What's Changed\n\n- Upgrade husky to v9 by\n[@&#8203;benelan](https://redirect.github.com/benelan) in\n[#&#8203;482](https://redirect.github.com/actions/github-script/pull/482)\n- Add workflow file for publishing releases to immutable action package\nby [@&#8203;Jcambass](https://redirect.github.com/Jcambass) in\n[#&#8203;485](https://redirect.github.com/actions/github-script/pull/485)\n- Upgrade IA Publish by\n[@&#8203;Jcambass](https://redirect.github.com/Jcambass) in\n[#&#8203;486](https://redirect.github.com/actions/github-script/pull/486)\n- Fix workflow status badges by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;497](https://redirect.github.com/actions/github-script/pull/497)\n- Update usage of `actions/upload-artifact` by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;512](https://redirect.github.com/actions/github-script/pull/512)\n- Clear up package name confusion by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;514](https://redirect.github.com/actions/github-script/pull/514)\n- Update dependencies with `npm audit fix` by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;515](https://redirect.github.com/actions/github-script/pull/515)\n- Specify that the used script is JavaScript by\n[@&#8203;timotk](https://redirect.github.com/timotk) in\n[#&#8203;478](https://redirect.github.com/actions/github-script/pull/478)\n- chore: Add Dependabot for NPM and Actions by\n[@&#8203;nschonni](https://redirect.github.com/nschonni) in\n[#&#8203;472](https://redirect.github.com/actions/github-script/pull/472)\n- Define `permissions` in workflows and update actions by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;531](https://redirect.github.com/actions/github-script/pull/531)\n- chore: Add Dependabot for .github/actions/install-dependencies by\n[@&#8203;nschonni](https://redirect.github.com/nschonni) in\n[#&#8203;532](https://redirect.github.com/actions/github-script/pull/532)\n- chore: Remove .vscode settings by\n[@&#8203;nschonni](https://redirect.github.com/nschonni) in\n[#&#8203;533](https://redirect.github.com/actions/github-script/pull/533)\n- ci: Use github/setup-licensed by\n[@&#8203;nschonni](https://redirect.github.com/nschonni) in\n[#&#8203;473](https://redirect.github.com/actions/github-script/pull/473)\n- make octokit instance available as octokit on top of github, to make\nit easier to seamlessly copy examples from GitHub rest api or octokit\ndocumentations by\n[@&#8203;iamstarkov](https://redirect.github.com/iamstarkov) in\n[#&#8203;508](https://redirect.github.com/actions/github-script/pull/508)\n- Remove `octokit` README updates for v7 by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;557](https://redirect.github.com/actions/github-script/pull/557)\n- docs: add \"exec\" usage examples by\n[@&#8203;neilime](https://redirect.github.com/neilime) in\n[#&#8203;546](https://redirect.github.com/actions/github-script/pull/546)\n- Bump ruby/setup-ruby from 1.213.0 to 1.222.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;563](https://redirect.github.com/actions/github-script/pull/563)\n- Bump ruby/setup-ruby from 1.222.0 to 1.229.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;575](https://redirect.github.com/actions/github-script/pull/575)\n- Clearly document passing inputs to the `script` by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;603](https://redirect.github.com/actions/github-script/pull/603)\n- Update README.md by\n[@&#8203;nebuk89](https://redirect.github.com/nebuk89) in\n[#&#8203;610](https://redirect.github.com/actions/github-script/pull/610)\n\n#### New Contributors\n\n- [@&#8203;benelan](https://redirect.github.com/benelan) made their\nfirst contribution in\n[#&#8203;482](https://redirect.github.com/actions/github-script/pull/482)\n- [@&#8203;Jcambass](https://redirect.github.com/Jcambass) made their\nfirst contribution in\n[#&#8203;485](https://redirect.github.com/actions/github-script/pull/485)\n- [@&#8203;timotk](https://redirect.github.com/timotk) made their first\ncontribution in\n[#&#8203;478](https://redirect.github.com/actions/github-script/pull/478)\n- [@&#8203;iamstarkov](https://redirect.github.com/iamstarkov) made\ntheir first contribution in\n[#&#8203;508](https://redirect.github.com/actions/github-script/pull/508)\n- [@&#8203;neilime](https://redirect.github.com/neilime) made their\nfirst contribution in\n[#&#8203;546](https://redirect.github.com/actions/github-script/pull/546)\n- [@&#8203;nebuk89](https://redirect.github.com/nebuk89) made their\nfirst contribution in\n[#&#8203;610](https://redirect.github.com/actions/github-script/pull/610)\n\n**Full Changelog**:\n<https://github.com/actions/github-script/compare/v7...v7.1.0>\n\n###\n[`v7.0.1`](https://redirect.github.com/actions/github-script/releases/tag/v7.0.1)\n\n[Compare\nSource](https://redirect.github.com/actions/github-script/compare/v7.0.0...v7.0.1)\n\n#### What's Changed\n\n- Avoid setting `baseUrl` to undefined when input is not provided by\n[@&#8203;joshmgross](https://redirect.github.com/joshmgross) in\n[#&#8203;439](https://redirect.github.com/actions/github-script/pull/439)\n\n**Full Changelog**:\n<https://github.com/actions/github-script/compare/v7.0.0...v7.0.1>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.31.2`](https://redirect.github.com/github/codeql-action/compare/v4.31.1...v4.31.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.1...v4.31.2)\n\n###\n[`v4.31.1`](https://redirect.github.com/github/codeql-action/compare/v4.31.0...v4.31.1)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.0...v4.31.1)\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.25.4`](https://redirect.github.com/actions/go-versions/releases/tag/1.25.4-19122936812):\n1.25.4\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.25.3-18483679098...1.25.4-19122936812)\n\nGo 1.25.4\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.13.2)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.13.1...v2.13.2)\n\n##### What's Changed\n\n- Fixed an issue where there was a limit of 512 allowed endpoints when\nusing block egress policy. This restriction has been removed, allowing\nfor an unlimited number of endpoints to be configured.\n- Harden Runner now automatically detects if the agent is already\npre-installed on a custom VM image used by a GitHub-hosted runner. When\ndetected, the action will skip reinstallation and use the existing\nagent.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.13.1...v2.13.2>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.47`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...HEAD\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]: https://redirect.github.com/taiki-e/install-action/comp\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNTkuNCIsInVwZGF0ZWRJblZlciI6IjQxLjE1OS40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-06T20:27:18Z",
          "tree_id": "ed163e263fd959b2c5cdf95b9c36ac0e945c185b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/54a6d2df04a9e83266893c889a837ce48905174a"
        },
        "date": 1762462400394,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 10.4233866307097,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.219957426430833,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.621875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.16796875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1900000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1900000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5188737.4197293315,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5139760.702554198,
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
            "value": 51.896127159825525,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.50653978321462,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.7015625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.0390625,
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
            "value": 14583605.236985654,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13970290.935615236,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 42.578820013026046,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.27613186542459,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.986328125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.125,
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
            "value": 18380926.119575895,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5893516.385964999,
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
            "value": 56.12648865581029,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 57.306661843993524,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.71171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.51953125,
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
            "value": 5979444.82515608,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14001957.430056319,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "99deb083032f33365657140d51358363e1b3bc83",
          "message": "chore(deps): update docker.io/rust docker tag to v1.91 (#1374)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.90` -> `1.91` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNTkuNCIsInVwZGF0ZWRJblZlciI6IjQxLjE1OS40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-11-06T21:24:46Z",
          "tree_id": "2a2efcdd3f84b80d576578aca5b0290a0ae413b4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99deb083032f33365657140d51358363e1b3bc83"
        },
        "date": 1762465179538,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 17.065945580591492,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.526640068158933,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.93828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.2265625,
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
            "value": 5761872.295071496,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5707534.985464909,
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
            "value": 51.594053460492006,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.034657230199166,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.048828125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.7109375,
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
            "value": 14628539.72329109,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14012193.589321494,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 60.790406904766535,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 61.15035475146651,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.08984375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.37890625,
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
            "value": 5968521.303455589,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13983832.520536995,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 44.15800812583215,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.9063784856235,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.726171875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.69140625,
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
            "value": 18444120.345191564,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5913783.569647076,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "a.lockett@f5.com",
            "name": "albertlockett",
            "username": "albertlockett"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ecafada8477aeb3a7539ed574a64671b783f158d",
          "message": "OTLP bytes -> OTAP encoding for Metrics Sum and Histogram data (#1398)\n\npart of #387 \n\nWill handle ExpHistogram and Exemplars in a followup PR",
          "timestamp": "2025-11-06T21:59:52Z",
          "tree_id": "514bc0c830a6bce61608197dba1267960eedc2d0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ecafada8477aeb3a7539ed574a64671b783f158d"
        },
        "date": 1762467433587,
        "tool": "customSmallerIsBetter",
        "benches": [
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
            "value": 15.750918975294198,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.478733472641434,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.225390625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.22265625,
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
            "value": 5857919.603092515,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5803665.087541463,
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
            "value": 59.74453331616374,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 60.33219038750192,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.834375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.96875,
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
            "value": 5988453.257598741,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14041526.596045261,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 52.385134942720356,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.037116701030925,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.068359375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.3203125,
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
            "value": 14607482.686195513,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14021374.940802569,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
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
            "value": 44.57891613749686,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.683991199632295,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.238671875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.2734375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2300000,
            "unit": "count",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18413038.793494985,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5923233.896297545,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "dbd017c4dbf2305c619c31da35ce0722ff46b369",
          "message": "[query-engine] Add special handling for invalid equality expressions in KQL comparisons (#1400)\n\n## Changes\n\n* Add a rule in KQL pest to allow `where [scalar] = [scalar]` so that a\nmore intentional error message can be generated hinting that `==` is the\nmost likely solution",
          "timestamp": "2025-11-06T23:37:57Z",
          "tree_id": "790c27046521a574aca315a77a80294ef350184c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dbd017c4dbf2305c619c31da35ce0722ff46b369"
        },
        "date": 1762473180652,
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
            "value": 43.84424078563296,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.615986125735525,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.017578125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.8671875,
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
            "value": 18356101.70135247,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5886817.115220994,
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
            "value": 58.53581048546399,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 59.55263529302469,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.878515625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.05859375,
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
            "value": 5965188.90573033,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13938646.324284341,
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
            "value": 9.870663409296839,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 10.520835566625156,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.920703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1328125,
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
            "value": 5757770.102909572,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5699089.454359612,
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
            "value": 52.3167721164988,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 53.22668097200861,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.8875,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.1640625,
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
            "value": 14558498.42436878,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14003976.38349178,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "drewrelmas@gmail.com",
            "name": "Drew Relmas",
            "username": "drewrelmas"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "tree_id": "7f669511fe1e7f92b99dcc5884c8a0c591bb8565",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762561234801,
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
            "value": 54.54222443836117,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 54.77172873241626,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.7375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.8046875,
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
            "value": 5902833.694163572,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13821118.436712345,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTAP - Network Utilization"
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
            "value": 44.18604319878927,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.42291205948416,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.070703125,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.89453125,
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
            "value": 18441642.795981247,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5920761.250339371,
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
            "value": 51.94624666334393,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 52.66626546130031,
            "unit": "%",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.106640625,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.40625,
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
            "value": 14594000.41856562,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14021284.073087469,
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
            "value": 13.43219044836394,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.21095748723503,
            "unit": "%",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.812109375,
            "unit": "MiB",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.08984375,
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
            "value": 5760869.388270346,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5704218.023345944,
            "unit": "bits/sec",
            "extra": "CI 100kLRPS/OTAP-ATTR-OTAP - Network Utilization"
          }
        ]
      }
    ]
  }
}