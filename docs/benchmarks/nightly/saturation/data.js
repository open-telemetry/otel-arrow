window.BENCHMARK_DATA = {
  "lastUpdate": 1770442605978,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770442605479,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8146860599517822,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.83107533411253,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.04131153393244,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.84986979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.69921875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 235668.6666692444,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 237588.62647808573,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001256,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5384041.218407265,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5211968.890766329,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.2465636730194092,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.64822764117307,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.78597243105446,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 82.45208333333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 86.515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 766586.8095412808,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 776142.801691116,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008421,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17634346.54436607,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16886079.63676188,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.2694835662841797,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.68328317887052,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.90491606725034,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.87486979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.4296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 364715.86921102885,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 372993.03617330623,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001206,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8223062.85324261,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8009226.653661823,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.7573071718215942,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.12730001434335,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.33147170161728,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 285.1244791666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 302.34375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2033868.2440016873,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2069609.554736221,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.022421,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47360906.076947875,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45549408.75850722,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.084608554840088,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.51263523376994,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.83561576687592,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 159.17083333333332,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 167.55859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1477687.162646972,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1508491.1554932168,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002611,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34269912.06713828,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32954418.89745833,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}