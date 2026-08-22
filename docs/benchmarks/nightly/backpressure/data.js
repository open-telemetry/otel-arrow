window.BENCHMARK_DATA = {
  "lastUpdate": 1787419430586,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
        "date": 1780514050341,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75487335285932,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.96794984908288,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.511588541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.02734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.12940995005,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.12940995005,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001955,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2997966.156203938,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2857000.5113352756,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.488784812893506,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.57807992704699,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.9175969534468,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.600390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.15234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.0941265498,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.57589110729,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005344,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2909008.9044475257,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1113169.2110444095,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 29.58577627295541,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780543758494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72008000129598,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.04003084753606,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.1421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.65625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.27690055966,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.27690055966,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001865,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3085796.193800051,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2935145.5667694635,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.381953667441447,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.67520010471344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.58991902431984,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.95596316074608,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.309765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.81640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.35179976517,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98321.2129144155,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002189,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3031107.435137011,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149627.5323724258,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.82862126381072,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780600519961,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75281734667266,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1939004566917,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.797916666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.33984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.98519733736,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.98519733736,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002043,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3089943.5069099697,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2939433.137138484,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.42422426596319,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.55625133660708,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.88473347219008,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.034244791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.69921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.86693016425,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.28581466152,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00308,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3032290.2003434226,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1151833.6041105974,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.838432453291933,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780628298001,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77937946521138,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24201072177765,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.398828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.01953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.72010758048,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.10810578747,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001968,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3096893.902359785,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2942996.407264687,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.494869293014734,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.48223795949961,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.01847225446947,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.194921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98320.1649859029,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96653.72151156556,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008036,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3028734.034514845,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150367.7580253002,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.33592775475725,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780685353016,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73113355867267,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19445520018586,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.362890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.81640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.91800751453,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.91800751453,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002084,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3089983.782069205,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2941396.571217538,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.4246553305685,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.51610727489067,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.83158856788785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.092447916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.70703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99985.67038700571,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96652.81470743885,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008599,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3033518.3956777463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150541.777982582,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.385722235405034,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780713722445,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.56000446765825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.79811416685925,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.887890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.65234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.33013468406,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.72463243933,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002202,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3033319.418397287,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150595.4680064747,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.84844821579602,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75926934910933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16603282750077,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.941145833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.70703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.814764795,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.814764795,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002147,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3098572.7042318103,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2944188.8203696827,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.512036421949933,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780768694570,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73689095713144,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12456930578129,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.076692708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.93275649217,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99996.54178626323,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002075,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3089142.1628271225,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2938732.991969323,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.892489956602535,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.52435594509353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.8727542963307,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.173697916666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.71484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.3374171351,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.81512684951,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005198,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3026649.007091678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149340.680869605,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.782147957125346,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780800395879,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.60560875665504,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.92164341324289,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.378776041666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.90234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.96192050191,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.37922182688,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003023,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3040179.341887313,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1151566.5120847404,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.918635758540557,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82377181958246,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15920055559843,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.023828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.49512284595,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.27861875108,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002103,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3093577.582623423,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2949644.6916118157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.003648405355456,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780855443326,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73920208230452,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18360371517028,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.55,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.0703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.00158511296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.00158511296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002033,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3086813.37961019,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2934722.5620139344,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.392386147154603,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.54376524074374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.01801748819933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.049609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.96484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.71194631657,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.13341387796,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003173,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3034997.0437018406,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150810.0161600863,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.866008926733915,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1780886987429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.52934771145084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.86739781276661,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.30078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.71856040737,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.11214156274,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002169,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3036850.8935643663,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1152074.8137680588,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.885183419118167,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75896051306924,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.29712871287128,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.561458333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.01171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.57426445604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.57426445604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001887,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3089654.860620554,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2939620.1062935865,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.422059211089667,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783533240696,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.61782411727445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.92905154960286,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.81875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99990.90891743539,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.34932722239,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003855,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3030381.806179072,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1148733.550913479,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.82025792099568,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8051563873133,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10866063838009,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.0390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.17769064495,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.17769064495,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002129,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3094615.4917908097,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2938946.9236355047,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.472636197563084,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783566552627,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76601552587104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12822447244338,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.821744791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.19140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.92128506475,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.31244972466,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002082,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3088206.9725596975,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2936171.299874126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.948077241465295,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.61079152295082,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.93870628288455,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.083854166666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.2119464476,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.61397139414,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002473,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3021383.5954797743,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1140381.3288597653,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.72803459253439,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783620305510,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.6204497704588,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.94686548808609,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.176953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.07889134868,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.51646816157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003753,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3025713.470551747,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1147736.3048351945,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.772726673225005,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74020964249198,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16663366336634,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.093619791666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.43002228955,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.43002228955,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002992,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3101801.1614871128,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2947185.4572143457,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.545313606491852,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783649563361,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76531247838753,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33375821795963,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.122916666666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.07860773141,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.07860773141,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001986,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3094176.0773073663,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2948833.189957473,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.46723892747992,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.63096045332335,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.06312456560353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.305989583333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.74194315283,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96661.58387838106,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003155,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3036228.2396280584,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150803.766685412,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.410909254789573,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783711096023,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.65164861996821,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.94908331399397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.949739583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.39453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.36521482229,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99995.36521482229,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002781,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3038395.3246418554,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149899.6457354617,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.385361542651527,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77205472044429,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15014637184677,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.4515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.23265333034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.23265333034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001892,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3093305.72853901,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2937174.6730080065,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.45833834691169,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783735250290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.69773906825321,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16682746965739,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.877213541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.43359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.8803157029,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.8803157029,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002107,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3099713.719397733,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2942372.1625805516,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.523619366215385,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73298848730182,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1859286592866,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.891536458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.72219636948,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96659.63145649049,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004367,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3035124.093857319,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150720.038680789,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.400120692820177,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783791131923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.703593134880066,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72685829448916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1771578379212,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.16875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.64453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.95406057878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96654.81162964033,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002062,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3092349.3656842518,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2940655.763828986,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.99374468322844,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.67335981555067,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22076917124507,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.845052083333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.65861865212,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.08097500792,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003205,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3030611.8155271686,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150931.7951612845,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.821427464830318,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783821123277,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78177463071226,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21297447366392,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.326302083333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.84765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.02689748176,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.02689748176,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003238,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3099920.4332934683,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2946844.005317682,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.526315854232394,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.60391366164508,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.0467019620056,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.193880208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.55388782048,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.01132302347,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004468,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3031864.1062228843,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149662.8778365613,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.834812329186384,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783877650907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.66965214644482,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13244814557791,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.736328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.21875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.09888831765,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.53613183885,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003741,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3033985.1900403486,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149313.618125195,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.856847226537813,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74777497677445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3207084552595,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.2578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.3350087988,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.3350087988,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002033,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3099666.9369834084,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2942115.9034613012,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.523959606004126,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783907522598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74069125435737,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.07164631318597,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.985416666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.85546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.05238725201,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.05238725201,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002002,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3092027.881229212,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2941283.57858646,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.445400527723887,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.57488167740367,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.91086848635236,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.487239583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.07421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.05222873583,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.49024993747,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003769,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3033711.5726093473,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1151612.7035792626,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.8540788251,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1783973090417,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77561434169344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.02321018125723,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.523307291666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.05859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.68987662048,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.68987662048,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001613,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3095718.8846118366,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2940353.55429634,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.482733300215436,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.62177838441525,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.91178488641633,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.119661458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.99858348753,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.41527376273,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003001,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3034873.196957556,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150525.6032957956,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.86466092744363,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784021821343,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.55380355395943,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.85940312355034,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.807421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.25390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99986.85137841383,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98320.35941565459,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00629,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3032700.254728639,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150006.1024314675,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.845089183489822,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76071243614429,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.04557247777349,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.344270833333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.25559633317,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.6410947004,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001878,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3096500.9141235254,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2944868.360228294,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.03377070278073,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784074457617,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.59686604765186,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10323998143276,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.680729166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.5,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99984.79731156878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98318.38402304263,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009123,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3028746.5515168235,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149420.6497188169,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.80549565183032,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75149401015054,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.05877639081523,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.787109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.7557690526,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.14973906866,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002183,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3095444.5469740233,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2938054.0883431663,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.02300520239439,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784082243128,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677965961396694,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72132444679013,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10323241732651,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.0625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.76887921146,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98321.23585520024,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002175,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3098311.384245788,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2944280.9477952854,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.512128151122273,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.50669167665978,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.82476573372506,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.349348958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.83067456132,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96659.24597608541,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002951,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3032729.479442258,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150250.2087039992,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.375472142546915,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784138932539,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74927516703329,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.96588289819249,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.036067708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.11629969506,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.11629969506,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001963,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3100243.781538014,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2947211.085339547,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.528934350987118,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6752445697784424,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.61866793011487,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.01071445194322,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.344140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.8671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.9970937551,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98316.88648882437,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003202,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3027897.718778934,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150080.6022325268,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.797331230816727,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784167884768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.62178842872093,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.97140647954845,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.427994791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.84375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.88192862662,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96661.7191976724,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003071,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3037389.8301420286,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149943.664446745,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.42288235046381,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79941229253969,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17042183622829,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.203385416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.57421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.76887921146,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.76887921146,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002175,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3097743.5485388744,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2943939.1352238557,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.503618729584836,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784225855043,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74699624190006,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16235912499035,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.904947916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.23828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.20643276877,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.20643276877,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001908,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3091394.6024555857,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2940221.3742456837,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.43891093698926,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667555570602417,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.58043572009629,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.004802473908,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.395963541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.73828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99989.57719718733,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.9953595362,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003054,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3036057.168191628,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150156.449321345,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.878403949042887,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784254402705,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75927119287232,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.06638633377135,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.627604166666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.97208712083,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.97208712083,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002051,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3094777.6574364915,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2944709.1127174133,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.473390989010998,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.66879715713141,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.02798443579766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.999479166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.27878511099,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.72413869247,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004033,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3029475.7964023207,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1148624.9171987535,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.81029926441121,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784320580122,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.71946200185783,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.04041781874041,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.990755208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.92586449842,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.92586449842,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001469,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3094327.3682276295,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2945552.0541677247,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.46850638314605,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.61057692715457,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.03865119876257,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.215234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.8671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.43376451613,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96660.31930569893,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00394,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3031097.7666291567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1151169.6996098936,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.358242848784467,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784341589195,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.71643179751935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08249845392703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.740755208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.25304025959,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.25304025959,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0031,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3095651.7383132013,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2940315.290670832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.48283064731879,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.56829693123095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.90865058365759,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.492447916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.02734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.596958611,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.02034263415,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003242,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3037034.321580624,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1148872.1595377368,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.886763620357293,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784397324596,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.67520010471344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.63978013749905,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.97548267613242,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.427994791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.0703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.88859461133,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.7742208744,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003067,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3037817.779708614,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150393.3118132777,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.89732257606884,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.66298996552653,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.9791886690786,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.856770833333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.05859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.56731090532,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.56731090532,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002298,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3093564.672087693,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2942598.9252975886,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.461184633369164,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784427057559,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.69237558793465,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.91202365206567,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.279166666666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.62109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.6033637124,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.6033637124,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002276,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3096946.8237052658,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2939227.1812312575,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.495569164963847,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.65936340053896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.0420329033753,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.87265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99990.07378927729,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99990.07378927729,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002756,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3038735.695784116,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1151297.7938389706,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.390373570361174,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784483624057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.49475093520064,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.96464055835595,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.69140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.09765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99990.16237218854,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.61522472813,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004303,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3039754.1955110636,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150879.9478870889,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.91580988517775,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.68772802495477,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13630533879316,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.092578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.06786529915,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.06786529915,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003213,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3094831.616983969,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2940743.9120908463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.474549273393816,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784518884665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.59144112074814,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.96226391879422,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.01953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.01519859566,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.4038356494,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001991,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3025813.734260589,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149033.3304860117,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.772842729765593,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75255115609914,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.07824851339872,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.221354166666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.20703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.23009819124,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.23009819124,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003114,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3097311.0795148155,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2939254.0873441626,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.499713525015345,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784570647144,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.58789961682908,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.87485784919654,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.508984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.9921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.2070661844,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.62583958196,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003076,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3034622.4182405816,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150685.3278777923,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.862986045891653,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81242696184945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1768425918792,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.812630208333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.02201171743,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.02201171743,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002224,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3096261.540643914,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2943376.203276417,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.48942658178887,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784599806683,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83085632013048,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08249435578047,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.869661458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.14816232024,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.14816232024,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003164,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3099713.73065187,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2941035.5240607667,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.524174802263726,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.6177085491694,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.93081939412626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.622526041666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.65695216352,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.07933629413,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003206,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3029636.74404824,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149047.5156171573,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.811511467507767,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784657392475,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.61973045831323,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.03790704832257,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.044010416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98320.7816159497,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98320.7816159497,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004405,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3032157.0343555454,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149976.2448294968,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.83943175105583,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82198955289805,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.158340159184,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.9796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.34644736268,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.34644736268,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003043,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3103557.8870266313,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2946619.0531340097,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.563206330211543,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784689479464,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.61772836586968,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 64.98816278888803,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.381380208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.90234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.47530523938,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.90071681872,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003315,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3032026.844510551,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1149803.4009539397,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 30.835874888071633,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.760704409017,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26419631144378,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.512760416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.23265333034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.23265333034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001892,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3100783.3738755034,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2948816.4598736987,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.534384595707383,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784743698573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84721344304066,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15799596711649,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.661588541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.4375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.34900724096,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.34900724096,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001821,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3080673.0014653187,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150915.8503561842,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.329828812449964,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88348977839249,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.39895326529046,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.344010416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.86328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.70369020125,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.70369020125,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002825,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3143457.9016003255,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2942559.532780899,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.968873621117215,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784773561706,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84351023164962,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28335104335105,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.599348958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.12890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.96504677135,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96659.37807073859,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002869,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3074886.9859743905,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1150251.0724994896,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.81157428639862,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93482074811283,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.38325135344161,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.422786458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.8046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.10063957762,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.10063957762,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003193,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3138546.723362265,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2944341.4008851927,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.91912284430909,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784830262527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9261194656151,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3581131781739,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.737760416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.45624190374,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.45624190374,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002976,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3220645.3771197055,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027450.848422743,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.753950384377056,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8160538351014,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28613434335627,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.565234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.78510335892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.17201830295,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001929,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3142800.460917903,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165876.6951839842,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.96171018934971,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784859204820,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.94005241210661,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.42645535369154,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.299479166666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.80859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.77051798157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.16423801579,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002174,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3224545.026765488,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035143.8438005312,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.35857099427887,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77356968178711,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14059699190126,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.779817708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.65625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.54678591766,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.93767281904,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002072,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3140292.545877553,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166408.237118488,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.936281260814955,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784916742938,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83112436175472,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2616746633648,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.875260416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.86865174804,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.27639842547,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002679,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3140598.0552399517,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165597.5032587603,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.940577537117466,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91046836212966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33140165263727,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.598567708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.4921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.10719443596,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.10719443596,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003189,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3222426.3415175737,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031612.4193318537,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.77217912011144,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1784945861110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.00867843721061945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78085161431027,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13640423394885,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.63203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.2578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.04369368628,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98316.51061691412,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001804,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3144765.880567205,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167045.4889290833,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.986142112189523,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87878937200414,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26254141943438,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.428645833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.94921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.57843829409,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.57843829409,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001681,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3215894.0376479253,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3023935.684708557,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.70492341978861,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785002302306,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77115796951361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18079116835327,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.039322916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99987.17796895762,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98320.6805628775,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006094,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3144652.509723144,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166418.9614660987,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.98363245372466,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90097570663646,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.40505972434916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.594791666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.09375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.40708013863,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.40708013863,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003006,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3219143.6736630034,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3021249.19081644,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.738694434858175,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785032363704,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81419772702317,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15143542521086,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.572916666666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.59765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.31850463041,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.70208644876,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001809,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3140875.4817189653,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167299.2309218575,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.942935867224257,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9343984984916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.42088112536713,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.269921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.7734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.50831545987,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.50831545987,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002334,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3224937.689471233,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028218.9316392764,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.79725226658326,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785088833944,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.68742255434084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.07599199630599,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.986979166666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.3359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.2135138717,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.59884558269,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001872,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3140830.558808181,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165393.965277572,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.942512536491996,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86392290338158,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2365650969529,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.718489583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.7578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.86469456514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.86469456514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00232,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3228197.425925489,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3036396.992069706,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.831286098192066,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785118739553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.02666339776863,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 66.42286774368091,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.539192708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.66189047645,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.66189047645,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003054,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3227224.73733098,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3041292.9252005247,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.82179519854888,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84728159022963,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32387096774194,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.590755208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.12890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.11663108686,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.11663108686,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005014,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3144611.6161251366,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166418.4116328508,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.98177356782228,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785176576634,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93860078037338,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.38310120705664,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.834635416666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.4765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.76887921146,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.76887921146,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002175,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3230956.5917172753,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031401.490497546,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.85837675146161,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8124631355348,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27940649149923,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.473697916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.0078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.33183680603,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.71519641552,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001801,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3143619.850961852,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167368.9428661114,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.97084204267619,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785204900985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92104954841845,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21815229996135,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.352864583333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.8229586537,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.8229586537,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002142,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3216682.130222658,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3022969.032937685,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.71318948245465,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86794950234817,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1962505800464,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.327604166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.78515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.6582165667,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.6582165667,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002446,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3143224.7766641844,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168168.260491959,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.96716774143956,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785272754099,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82851798209313,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2196599690881,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.908333333333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.51171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.21861238744,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.62052623201,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002469,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3139496.589271799,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165473.7391508645,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.92926363653707,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90361178708199,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32464310571496,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.764583333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.60546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.8803157029,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.8803157029,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002107,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3220365.731847023,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3024547.383654553,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.75063207142685,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785291927020,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.95664400783149,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.50116993363173,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.368229166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.71316106046,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96663.10785324586,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002209,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3235544.6728294822,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3036465.6656237887,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.47238408412952,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3334221839904785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82543418062376,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.36563135428793,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.234114583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.77734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99985.53670642759,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96652.59660447728,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007079,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3141937.1785010453,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166558.6592884078,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.50752994623116,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785348583745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87442006814477,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18081509550692,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.21953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.85546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.81640356663,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.81640356663,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002146,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3227572.3794124573,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029701.07451932,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.823943921198925,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81062697690409,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26113266097751,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.995052083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.52734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.65784033676,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.65784033676,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001836,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3142770.5199590665,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166188.7678767901,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.962222928794446,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785380778589,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78414051137639,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22791333385105,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.601432291666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.1953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.97686884309,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.36614474306,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002014,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3139665.6703164107,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166053.7626094574,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.93074108885066,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.96021873786778,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.42983145198701,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.820703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.44921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.22354331653,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.22354331653,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003118,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3223663.4974159924,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032029.7614021543,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.78472224199059,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785440192159,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7242331504821777,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81185277218893,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.37375009647295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.982291666666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.46484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96658.1677783386,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.77986345948,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001965,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3128338.9409186738,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166462.2847355008,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.81638387864076,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92191203481951,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28504833346223,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.902734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.9609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.88966719367,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.88966719367,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002915,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3218496.9031700366,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026306.999580033,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.73295481041434,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785464489334,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9606434277013,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.4308261405672,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.995442708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.2890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.97899838003,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.97899838003,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002657,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3224595.3175414484,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033006.987679306,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.79394691563485,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76968330895528,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27311563580484,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.552213541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.1953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.33337006542,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.33337006542,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002034,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3137095.0607872754,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167172.9038344787,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.9046083450721,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785525848681,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8763719414909,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1787436016752,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.35078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.1875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.73857651278,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.73857651278,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004838,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3222998.636230192,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3036192.075852939,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.77978911991064,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81650689635502,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2223218426341,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.077473958333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.79296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.72339014652,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.72339014652,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001796,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3141657.8164892728,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166865.4855637166,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.950885347194973,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785550598941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82081331421945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13898808602816,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.51796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.1484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.96731195926,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.43452342661,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00482,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3134797.3725980096,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167989.2454125292,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.88185628461297,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677965961396694,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9307170093611,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.36056913083823,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.68203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.1171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.34808608537,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98319.8151853701,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003042,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3224120.8942342014,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027348.0702699763,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.79217813983389,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785607225500,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7698974191955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10955046649704,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.7234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.59375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.38016597243,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.76272007433,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001772,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3143255.7743297513,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167621.1753385453,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.96712390658343,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.94458865520943,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.47498261071179,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.298307291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.56439796003,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.56439796003,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00291,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3222178.5580778318,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032195.668162841,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.76950678377524,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785637099798,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.96967137623047,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.35611398563596,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.738151041666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.16015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.31367292034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.31367292034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003063,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3228580.1857300294,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032082.87232851,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.83469496354417,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78908927411591,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1267968992248,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.593098958333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.15351925143,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.53985089941,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001908,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3139732.1657043444,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165934.4624651095,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.931360943895566,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785693602225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8646710233818,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.31796400710589,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.842447916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.31640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.19076895612,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.19076895612,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003138,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3213182.4054092346,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3025827.039452753,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.67814021880377,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7122229950353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11409238374787,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.224739583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.29296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.5115522435,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96657.9492123793,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003756,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3139193.2777277334,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166861.7510335473,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.47734204281759,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785723464414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80711909168969,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14678419546895,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.209635416666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.8203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.96175868166,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.96175868166,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001854,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3141824.2079704776,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166845.8746981726,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.95347500547661,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.008677965961396694,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.95689030734563,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.29317035432462,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.639192708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.15625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.33620598858,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98316.80356664324,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00488,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3228455.392186453,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032011.0362264556,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.837269673826114,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785786088755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77224676863342,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08846171747732,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.883463541666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.7109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99997.00342313075,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.3866994119,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001798,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3138284.552709691,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167135.7446095517,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.915714542069026,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.7241379022598267,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.98154613931555,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.42808444822519,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.506380208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.08984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96663.26411976965,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.87212183465,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002112,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3232749.793323183,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3039611.7017697366,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.87657884185669,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785810852271,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667555570602417,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82994714058876,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10737913486005,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.619270833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.59765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.43860972355,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.82574687547,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001937,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3139279.3403595095,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167330.6495907973,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.927636957538855,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.95679657391292,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.36804704425874,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.614973958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.52734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.7769973711,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.7769973711,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004611,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3219938.078581658,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026473.4688857566,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.74764946599658,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785872227937,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82366221517401,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23653129109616,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.157682291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.11328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.61195552192,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.61195552192,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001864,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3133722.053807044,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167144.7252442213,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.870214189932426,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9590811215085,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.48006805877804,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.296354166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.23828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.0744201531,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.0744201531,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003209,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3231666.4706577095,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3044198.9644983266,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.86616248426557,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785896081937,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80788593406426,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2555603849736,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.07890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.91015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.09663431912,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.09663431912,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001975,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3137720.923683862,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166362.8968540211,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.910076681331525,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90049824287685,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.4222000778513,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.071354166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.90197298914,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.90197298914,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005145,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3232291.396947796,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3034359.777866474,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.873578636627975,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785954953222,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72495484769574,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08878229924184,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.886067708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.26953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.58611334296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.58611334296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00249,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3134576.909268723,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165189.4875502936,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.87924073409236,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92242008503628,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.4442648535888,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.749609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.3046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.38577672234,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.38577672234,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003019,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3225647.0829935493,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031969.4319699155,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.80484122172144,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1785983321613,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91987536935068,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28395825280248,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.675520833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.97265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.61155753589,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.61155753589,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002271,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3220210.8087163297,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031329.0847527133,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.74914603758074,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86462937683649,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.29336530280764,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.336979166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.78515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.88559007515,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.88559007515,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002714,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3139590.296421492,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168069.2111192055,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.929481124297286,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786071507502,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89809021493262,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24161088351241,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.028645833333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.4375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.01133976445,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.01133976445,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003451,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3230368.9969026614,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3037259.461898482,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.85399058577317,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74111803042527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10330738087823,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.61171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.96011998296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.96011998296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001855,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3133202.2324950765,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165929.702651833,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.865786964690603,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786125455082,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9676087051904,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20925563851897,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.476302083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.00390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.41199631292,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.41199631292,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003003,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3217765.7196017085,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026919.254346789,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.72467900450143,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80903536351828,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24201545595054,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.034114583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.23504616355,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.23504616355,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002094,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3145723.8329064143,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168353.086860181,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.992395915836866,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786157741207,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.95964169692441,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34313901345291,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.385677083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.29073082369,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.29073082369,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003077,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3233563.053760664,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035179.0169848683,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.88537845748411,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8597891677803,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.29738089758344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.895442708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.40093963924,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.40093963924,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002603,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3145301.5595266586,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1169277.0564748724,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.988372700201865,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786211196684,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80698342066017,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20225082864411,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.059244791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.65625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.05024849254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.05024849254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0018,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3143229.820109149,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166274.9643765981,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.96774181315345,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92592669378084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34782838078982,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.165625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.59189236183,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.59189236183,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002283,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3217092.5625861185,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3020902.5541040073,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.71744040296398,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786240765770,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86701907978765,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1685797995374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.470182291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.26953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.63325918061,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.63325918061,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001851,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3149389.016964973,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168419.0609013734,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.02954157010509,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89216984258661,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.31143825069294,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.009375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.87109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.51973203196,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.51973203196,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004768,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3216915.1152723036,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3017955.376755904,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.71699070637421,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786297647837,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91142019751337,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.26966860690726,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.14453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.15759647918,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.15759647918,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004989,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3222216.5363243152,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3025153.2257182267,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.771028443687904,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81520613532335,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21806451612903,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.951041666666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.5693482322,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.5693482322,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00189,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3138933.643462731,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167050.8753569121,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.923230323593522,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786327237939,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88181867275429,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33192556256327,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.452604166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.98828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.21298790784,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.21298790784,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001904,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3133365.815011986,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165924.574025599,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.86574827614084,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89522184051295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21071196911197,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.264192708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.35791842265,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96661.77558082227,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003036,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3222662.4420431578,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3024588.150647595,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.33957422857992,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786385523381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.008677965961396694,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93518684834528,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30336452935262,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.338802083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.4453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.41527376273,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98336.94818030852,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003001,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3224700.146167938,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030225.881972818,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.79235532360835,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8319605803476,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17479722436391,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.150911458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.75,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.5996075969,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.5996075969,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002075,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3139539.397688993,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167415.866459658,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.93035527445383,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786413867441,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7241379022598267,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91866632517443,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24679944418712,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.732552083333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96661.80135599841,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.3841379984,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00302,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3218053.1810144666,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3019992.2148602027,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.72761175957197,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75788117292129,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28148296902756,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.881901041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98320.24253286415,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98320.24253286415,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004734,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3134818.4941670164,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167240.7660060893,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.88375469191081,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786479123046,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93950512195127,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30251556662515,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.238932291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.0390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.65452818871,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.65452818871,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002855,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3211000.722593568,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3023673.418069172,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.65579843435204,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79573742962259,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.14550435865505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.22421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.84765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.69717021212,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.69717021212,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001812,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3140331.975198184,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166979.2194598843,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.93740996254646,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786501343118,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78485694436283,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22702349463202,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.293359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.75,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.4038356494,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.4038356494,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001991,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3136226.5613249894,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168485.7952042385,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.89575275034288,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87673852389662,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19479158640881,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.27578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.14284893402,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.14284893402,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004998,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3221927.524250125,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032941.4636695567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.76809400826673,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786557510885,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89993856999419,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27356982765284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.659114583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.65625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.32186652886,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.32186652886,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003058,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3219298.407056833,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3028845.7668744354,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.74029644710827,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82484267822151,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24830906024836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.079296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.90625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98322.38857779543,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.38857779543,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001797,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3142352.655649909,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168434.7359177612,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.959685897618236,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786588563197,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86223184124607,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34902057867863,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.348567708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.17643794266,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.17643794266,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00274,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3122109.589186215,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1169046.6539353218,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.752578024395113,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.3334221839904785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90343496246858,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.29168316831682,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.301953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99985.49171826417,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96652.5531159593,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007106,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3201152.6040596315,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027775.7475301023,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.1202073908905,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786653504035,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 3.3333334922790527,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.8279637117366,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.45111024904958,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.329036458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.82421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99987.43324609818,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96654.51880456158,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007541,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3216079.531494121,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3042444.0956384134,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 33.27396971472315,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.00867843721061945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75803001597498,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.0302533464418,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.985286458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.09531626198,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98331.62822394543,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002993,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3122023.4049105938,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167942.524905708,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.74994110542276,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786673424470,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.94990189153853,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17431465017395,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.061979166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.41015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.56027210658,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.56027210658,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004133,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3196739.907445884,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3037609.2726175007,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.51145874115094,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.57335911794715,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15378613830853,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.431640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.25390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98318.56588473744,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96652.1495138097,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009012,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3117196.298671097,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168430.5048703973,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.25170174022577,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786730447687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.82437902546589,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32746967472765,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.089322916666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.05353598828,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.05353598828,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002815,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3106376.36445377,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166004.9246062448,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.592606971831803,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6581333875656128,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86716504573056,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.18192670480903,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.720572916666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.33203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99997.51506175072,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98339.4229320068,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001491,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194903.12912551,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026567.4964857236,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48852834264147,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786759082119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72940104094393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.04840640222686,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.545572916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.03057067374,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96660.47073049283,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003846,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3111740.193458811,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167272.1819901122,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.19247920005392,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6581776142120361,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83305332689676,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21160003093341,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.476041666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.2265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.30205400164,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98334.25214821273,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003019,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3199676.140811892,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035300.907080194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.53877535967052,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786816043743,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7574634247631,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09402476056995,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.406770833333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.15234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.68155461185,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.68155461185,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003042,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3111642.0102532343,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168724.9747931599,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.64627959913985,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81634473380315,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10501668089069,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.110546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.32346850714,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.71807736535,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002206,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191450.5579892304,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3034462.938983676,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.456622681234705,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786845464816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.89410030106423,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.55582656656964,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.564713541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.0390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98314.5571228894,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96648.11831395587,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008204,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3109500.214987681,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166462.748635665,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.173417022839985,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.88607216071865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.37970297029703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.8375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.32421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.31531164193,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.31531164193,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003062,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3196593.464814256,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3026889.03047529,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.50938912848214,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786902455308,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84016463408689,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23247892335061,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.813411458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98320.12730156793,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98320.12730156793,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008059,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3195248.130741688,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031146.726591168,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.49841327952321,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949152946472168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.833346827506496,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.20932637645043,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.770703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.3046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.48901644113,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96661.90445684044,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002956,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3113644.615420218,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166438.4034052568,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.21170359632694,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786931838735,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76224939976102,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.04378486669748,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.382682291666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.2379002453,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96657.68019867635,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003923,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110045.9702045866,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167344.1504429555,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.17588052818979,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91131944062177,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19741935483871,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.9875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98330.68168261729,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98330.68168261729,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001618,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3203305.806277955,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3040734.7506405804,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.57686971618167,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1786988950265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92986337263169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25931493079719,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.150260416666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.23014212363,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.62630642157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002262,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3188207.785500107,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3033271.921414114,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.4236743823758,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.69500732421875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81596451679982,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12006224227807,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.715364583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.42578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.50825439791,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96656.91772314202,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002741,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107167.878553014,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166817.765570938,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.14635798187761,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787019077702,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.62077223229143,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.1134015503876,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.314192708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 96656.37486130295,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96656.37486130295,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003078,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3114094.808613709,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168613.620406893,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.218204056196804,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.93409234573787,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32189949285385,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.026432291666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.77734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.56075585233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.56075585233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002302,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3202086.2863566685,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032134.901785948,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.56483870915785,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787090363778,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79107408460926,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09541698901438,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.203385416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.0859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.64473038529,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.64473038529,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001844,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3106617.697479284,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1165522.822288265,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.594550098272357,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.87120765758641,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.40661514683153,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.217317708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99983.34943954,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98316.96028221434,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009992,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3195989.8714894163,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3032992.85539436,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.50700451189168,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787107184054,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7907436209179,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23268058778035,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.346484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.9375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.72335502003,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.72335502003,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002813,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115020.8068749458,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167770.392984764,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.67966287559771,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83648168326756,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.13819559335137,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.87890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.3401339511,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.73446505192,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002196,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3193576.324152598,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3027198.6137837814,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.47823602419724,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787161958268,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83899168565644,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17227674115027,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.013671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.75390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.03591960967,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.03591960967,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003436,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3116022.5404507653,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167597.4212554595,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.691038923173323,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6667110919952393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.83783725662181,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.28648399723693,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.586067708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.18359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.79188653735,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.18424555798,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002125,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194611.0944969407,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3030718.205126956,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.4896021279208,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787191075483,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.70189403989377,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08087789799073,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.82421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.63894899508,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96659.05750019316,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003068,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3112494.869981003,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167335.5827648505,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.20075749212414,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91640410286217,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23247500193784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.123958333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.44679292395,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.83934637521,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002132,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3198357.085460919,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035577.2386264256,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.52682102117989,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787257953555,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84611907050018,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.39971223021583,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.081640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.59375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.88359512271,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.30220187067,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00307,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3192523.0286510647,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3029641.1568146436,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.467997078773195,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.90547359179445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27309600862999,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.338932291666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.71484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.26533209109,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.26533209109,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003296,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3115549.9085584735,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1169688.3954542927,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 31.68615816123946,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787292456635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80244269694998,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.41722772277228,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.930859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.4375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98323.90362082928,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96657.3515851668,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004127,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3107294.493379509,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167466.7858194811,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.14752362257316,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.94474030942638,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.3856301095172,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.219921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.1735662892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.58734018439,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002896,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3191257.4639682323,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031593.84247604,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.45503215588298,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787334675867,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.703639268875122,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78763490447086,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17749324272144,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.983072916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.77734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98316.89410841698,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96641.92874314856,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008405,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110030.768685126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1167730.218208636,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.18096750687638,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.9497390470692,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16641638225255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.389583333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.95703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.57340196444,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.57340196444,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003108,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3198590.495127482,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3031685.663963317,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.53060607184394,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787362444231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91443431015959,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.19988283357743,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.261979166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.7109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.84026624226,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.25959513822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003096,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3187566.462036388,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3020124.123979025,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.41760278439826,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6949613094329834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81603357058215,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23310195563113,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.862239583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.77495936243,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96659.19120523754,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002985,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3108859.381508309,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1166857.0638099723,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.16310153999978,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1787419429286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89993594703945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24089713843773,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.101302083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.93359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.20960171985,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.20960171985,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002313,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3194445.42439035,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 3035646.372029163,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.48890011249553,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.7036857604980469,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7882307839797,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27654614139233,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.872005208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98314.14425993562,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 96639.18031818715,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008456,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 3110190.9246371593,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1168770.5201482063,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 32.18353999275212,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}