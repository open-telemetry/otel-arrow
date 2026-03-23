window.BENCHMARK_DATA = {
  "lastUpdate": 1774233779571,
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770488249588,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7046704292297363,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.88274515157501,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.04750716339466,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.391536458333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.38671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 232766.6337141308,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 236734.53769085454,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001452,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5366075.917919224,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5238417.255159494,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.0075185298919678,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.5343267804164,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.85619432286023,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.95104166666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 87.36328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 767224.5410555042,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 782626.7153659988,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001918,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17763339.17878751,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17075594.983243976,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.699682354927063,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.8486516844761,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.03791222291022,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.40442708333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.0078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 421715.2324142226,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 428883.05166571384,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001513,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9621636.625853373,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9291528.84125375,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9433798789978027,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.57022281078305,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.73246487019489,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 270.44752604166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 281.13671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2050366.996705368,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2090213.415079729,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006397,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47877632.32163214,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46102894.670870535,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.2069677114486694,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.72517452031079,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.93096896178245,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 156.12486979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 164.81640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1488075.2963548785,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1506035.8849814408,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006942,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33886295.807709835,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32698620.103840392,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770519720152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6314725875854492,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.83254439179055,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.02795218532819,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.746484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.8125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 236473.87758097827,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 237967.1453100827,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002636,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5327256.54118704,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5194829.456071515,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.9142777919769287,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.14756989424777,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.44176199365374,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 87.20755208333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 90.2109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 737267.3336811658,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 744008.0050805677,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005892,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16758832.15551614,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16137220.242313676,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9602808952331543,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.90222828632125,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.33021513261983,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.780078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.37109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 365656.3635383954,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 372824.25566161895,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000903,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8311143.450010507,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8071565.102306775,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.1708993911743164,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.53207126059698,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.97621756848261,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 288.77135416666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 302.0078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2142030.682848878,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2188532.018137454,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006879,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50081041.82451328,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48089235.88782391,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.637510299682617,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.14213195437438,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.48712872343727,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 156.44322916666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 166.828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1423474.4568211404,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1461018.7406772585,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003808,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32777765.013332956,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31539400.411605824,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770574567220,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.676201581954956,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.84259110057131,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.07262270536607,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.809505208333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.20703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 234177.429216816,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 238102.71498877005,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000727,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5356613.19235018,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5177613.336714614,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.301835536956787,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.01400485851684,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.10786220742003,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 76.80013020833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 81.6796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 578296.5565309457,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 591607.9923479243,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002543,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13224271.674545903,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12697687.915781725,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.857032060623169,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.75405885988863,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.34689309825242,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.65989583333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.81640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 363010.1080544208,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 369751.32206826605,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001062,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8415779.915457403,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8098390.746296337,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.919479250907898,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.74245747463944,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.91360251910164,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 279.888671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 294.703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2362545.485971535,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2407894.059301565,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008062,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 55310747.27697618,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 53427550.81312358,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0106068849563599,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 58.621354709428054,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 59.42027081596281,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 151.655859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 161.03515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 954039.1537428936,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 963680.7392664369,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006728,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 21827649.022669833,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20777771.654217478,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770605622290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1310566663742065,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.86034912667117,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.05701921523102,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.89140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.05859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 233877.6786509213,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 236522.96763157606,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001006,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5315890.045524256,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5077759.088559642,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.071977138519287,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.5037042690155,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.72374952388301,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 83.571875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 87.5078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 772093.8371440556,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 780370.5069643584,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00481,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17741109.810614027,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17101404.23553783,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.383210301399231,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.6342453666828,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.77609980038685,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.55494791666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.8203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 419494.39143791475,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 425296.8809352354,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001832,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9623205.48478199,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9179254.743444579,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.14249587059021,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.56650166993577,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.87077787497103,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 288.2109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 308.59375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1923662.320681184,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1964876.707824834,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002348,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45041428.24090019,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 43152258.35510015,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.0403213500976562,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.55108446952732,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.98815530683135,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 148.21419270833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 155.2578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1196101.84837499,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1220506.169335742,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002489,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 27591968.876341257,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 26579903.470065277,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6",
          "message": "InternalLogs - catch more scenarios of direct use of tracing (#2006)\n\nFollow up to\nhttps://github.com/open-telemetry/otel-arrow/pull/1987/changes#diff-01748cfa22e108f927f1500697086488ddb8d06bcd3e66db97f7b4cbc6927678",
          "timestamp": "2026-02-10T01:22:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6"
        },
        "date": 1770692415373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8269513845443726,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.82179580985017,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.00913832702557,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.089192708333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.77734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 237332.6160150637,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 239295.24146659585,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00126,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5321840.225813204,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5102394.170696799,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4386870861053467,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.83164410829616,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.97411209673669,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 88.67552083333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 92.75,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 720639.6391980586,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 731007.3884713221,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001451,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16447101.121518923,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15832240.072887802,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.8612831830978394,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.20041637942974,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.35476098315041,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.9203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.52734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 417195.01602443564,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 424960.1970308016,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001177,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9579500.840368448,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9242492.151244031,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.559196710586548,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 72.899576368167,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.17327412083236,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 282.5006510416667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 298.78515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1908718.1403032935,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1957565.9937216043,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006731,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 44832684.49310178,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 43155155.620958455,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.3109257221221924,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 87.74013947362846,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.90563950556243,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 149.696484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 156.5,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1386508.6970812383,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1418549.8823771046,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002774,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31717167.817428157,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 30685110.682953086,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "8ae3f080c7df5bf627db50c83925e0a756adeadb",
          "message": "feat: Add event name to missing otel_error logs (#1978)\n\n# Change Summary\n\nAdd eventName to missing logs.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #N/A\n\n## How are these changes tested?\n\nlocal builds and tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T14:46:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8ae3f080c7df5bf627db50c83925e0a756adeadb"
        },
        "date": 1770748476381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7187005877494812,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.83205505873417,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.99074436498151,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.8828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.78125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 237459.40230959974,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 239166.02443342897,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001566,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5365630.335498315,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5218588.554934678,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.9618560075759888,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.97828803348871,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.10002617280124,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 77.1578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 81.68359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 594340.6956360975,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 600057.3969880963,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006633,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13367746.84396391,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12965814.453618053,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.804762840270996,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.66479284689382,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.78301412639405,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.68984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.67578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 423170.4819954408,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 430807.70611646917,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000858,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9683113.196297001,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9311311.563292164,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.6452908515930176,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 80.0371855986478,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 80.24242785134822,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 274.14166666666665,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 288.31640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2025516.9436565235,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2079097.7594261765,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.009538,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47597375.51379123,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45723609.859504804,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9346601963043213,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.06093106949476,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.4949173568282,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 160.27018229166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 169.109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1517239.894315095,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1546593.3310727614,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002514,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35193910.01510758,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33927369.00371776,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0d0af9a8664649f5c330cdcb2becf5bd611ca404",
          "message": "Add support for schema key aliases in query engine Parsers (#1725)\n\nDraft PR to open discussion - The current `otlp-bridge` for the\n`recordset` engine uses the OpenTelemetry [log data model\nspec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md)\nfor its initial schema keys (`Attributes`, `Timestamp`,\n`ObservedTimestamp`, `SeverityText`, etc).\n\nHowever, many well-versed in the OpenTelemetry space may be more used to\nthe snake case representation (`attributes`, `time_unix_nano`,\n`observed_time_unix_nano`, `severity_text`, etc) from the\n[proto](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otlp/proto/logs.rs)\nrepresentation.\n\nDo we have any significant risks if we plan to support both? Inspired by\n`severity_text` reference in #1722, been on the back of my mind for a\nwhile.\n\nThis is still somewhat incomplete, could need more wiring for\nuser-provided aliases in bridge, but for the moment just doing it for\nknown OpenTelemetry fields.",
          "timestamp": "2026-02-10T23:42:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d0af9a8664649f5c330cdcb2becf5bd611ca404"
        },
        "date": 1770780484046,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1142367124557495,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.89677583589372,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.00573279432187,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.70911458333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.01171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 233578.35130467525,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 236180.966916699,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001177,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5385347.894109293,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5178756.570368717,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.004628896713257,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 91.56171328186802,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.7371983531961,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.74674479166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 87.58203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 730021.5528451527,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 744655.7748753733,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001823,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16645371.404973645,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16106845.209490286,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.1068272590637207,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.1756109616431,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.49819760408606,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.740234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 419202.4059914991,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 428034.27689898596,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000877,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9576903.929873753,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9202836.581956258,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9275556802749634,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.45401578192036,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.73132377930821,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 285.0278645833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 304.8515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2040495.3440599178,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2079827.0280416217,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010652,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47697092.15092309,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46026492.271905966,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0797576904296875,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.7733010597438,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.02408129022302,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 143.46497395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 151.52734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1299898.3881115525,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1313934.1405513026,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006758,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 29803270.724879216,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28690777.30774165,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0d0af9a8664649f5c330cdcb2becf5bd611ca404",
          "message": "Add support for schema key aliases in query engine Parsers (#1725)\n\nDraft PR to open discussion - The current `otlp-bridge` for the\n`recordset` engine uses the OpenTelemetry [log data model\nspec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md)\nfor its initial schema keys (`Attributes`, `Timestamp`,\n`ObservedTimestamp`, `SeverityText`, etc).\n\nHowever, many well-versed in the OpenTelemetry space may be more used to\nthe snake case representation (`attributes`, `time_unix_nano`,\n`observed_time_unix_nano`, `severity_text`, etc) from the\n[proto](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otlp/proto/logs.rs)\nrepresentation.\n\nDo we have any significant risks if we plan to support both? Inspired by\n`severity_text` reference in #1722, been on the back of my mind for a\nwhile.\n\nThis is still somewhat incomplete, could need more wiring for\nuser-provided aliases in bridge, but for the moment just doing it for\nknown OpenTelemetry fields.",
          "timestamp": "2026-02-10T23:42:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d0af9a8664649f5c330cdcb2becf5bd611ca404"
        },
        "date": 1770834767200,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5600337982177734,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.8095494635288,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.9668262705045,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.425130208333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.7578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 232470.63212492986,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 236097.2524917432,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000766,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5288509.604295859,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5103031.609073837,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.408782482147217,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.45620417138502,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.68250544667697,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 86.366015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 88.265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 768723.5022139039,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 787240.38034065,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001475,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17775222.863938477,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17089686.331891336,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.248875617980957,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.88814039726783,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.06930677409359,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.7984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.25,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 358573.80944039806,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 366637.6882134205,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000902,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8314554.075328116,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7830598.579886291,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.932436227798462,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 89.63680734926474,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.0382296496124,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 289.4712239583333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 304.53125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2263972.6249091127,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2307722.4537614933,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.035892,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53041497.02120178,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 51390656.591378294,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.8930141925811768,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.68848783852158,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.99126463920719,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 129.45872395833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 135.90234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1145897.8119201427,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1179048.7997099822,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001832,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26604635.895391725,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 25568908.82728798,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "70c62ad23a1d932f7e95bf93f57d4c86c82927c3",
          "message": "Emit warning and skip unconnected nodes during engine build (#2023)\n\n# Change Summary\n\nAdd a pre-processing step at the start of pipeline build that gracefully\nremoves unconnected nodes from the incoming `PipelineConfig`.\n\nInput with unconnected nodes:\n```yaml\nnodes:\n  unconnected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  connected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  unconnected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:batch:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      otap:\n        min_size: 1\n        sizer: items\n      flush_timeout: 5s\n\n  connected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop_exporter\n        dispatch_strategy: round_robin\n    config:\n      verbosity: detailed\n      mode: signal\n\n  noop_exporter:\n    kind: exporter\n    plugin_urn: \"urn:otel:noop:exporter\"  \n```\n\nOutput (confirmed that log was able to pass through remaining connected\nnodes with debug processor):\n```log\n2026-02-11T19:01:57.699Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_receiver, node_kind=receiver] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.706Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\n2026-02-11T19:01:57.701Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_proc, node_kind=processor] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.702Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=2] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.725Z  INFO  otap-df-otap::receiver.start: Starting Syslog/CEF Receiver [protocol=Tcp, listening_addr=127.0.0.1:5514] entity/node.attrs: node.id=connected_receiver node.urn=urn:otel:syslog_cef:receiver node.type=receiver pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n\nReceived 1 resource logs\nReceived 1 log records\nReceived 0 events\nLogRecord #0:\n   -> ObservedTimestamp: 1770836524675426978\n   -> Timestamp: 1770836524000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Attributes:\n      -> syslog.facility: 16\n      -> syslog.severity: 6\n      -> syslog.host_name: securityhost\n      -> syslog.tag: myapp[1234]\n      -> syslog.app_name: myapp\n      -> syslog.process_id: 1234\n      -> syslog.content: User admin logged in from 10.0.0.1 successfully [test_id=234tg index=1]\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0\n```\n\nInput with no connected nodes:\n```yaml\nnodes:\n  recv:\n    kind: receiver\n    plugin_urn: \"urn:test:a:receiver\"\n    config: {}\n  proc:\n    kind: processor\n    plugin_urn: \"urn:test:b:processor\"\n    out_ports:\n      out:\n        destinations: [exp]\n        dispatch_strategy: round_robin\n    config: {}\n  exp:\n    kind: exporter\n    plugin_urn: \"urn:test:c:exporter\"\n    config: {}\n```\n\nOutput:\n```log\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=proc, node_kind=processor]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=recv, node_kind=receiver]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=exp, node_kind=exporter]\n2026-02-11T19:00:02.759Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=3]\n2026-02-11T19:00:02.759Z  ERROR otap-df-state::state.observed_error: [observed_event=EngineEvent { key: DeployedPipelineKey { pipeline_group_id: \"default_pipeline_group\", pipeline_id: \"default_pipeline\", core_id: 0 }, node_id: None, node_kind: None, time: SystemTime { tv_sec: 1770836402, tv_nsec: 759880158 }, type: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: Some(\"Pipeline encountered a runtime error.\") }]\n2026-02-11T19:00:02.760Z  ERROR otap-df-state::state.report_failed: [error=InvalidTransition { phase: Starting, event: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: \"event not valid for current phase\" }]\n2026-02-11T19:00:02.760Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\nPipeline failed to run: Pipeline runtime error: Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\n```\n\n\n## What issue does this PR close?\n\n* Closes #2012\n\n## How are these changes tested?\n\nUnit tests and local engine runs.\n\n## Are there any user-facing changes?\n\n1. Engine is now more flexible and does not crash with unconnected nodes\npresent in the config.\n2. Engine provides visible error if there are no nodes provided instead\nof starting up successfully.",
          "timestamp": "2026-02-11T23:33:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70c62ad23a1d932f7e95bf93f57d4c86c82927c3"
        },
        "date": 1770864806701,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8443214893341064,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.78354296371981,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.93906063236206,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.725651041666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.75,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 237504.00742813345,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 239509.3047998006,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001076,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5385956.513622106,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5246491.176659458,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.123236656188965,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.46202078036842,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.63798715667312,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 86.60494791666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 89.84765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 767587.9708733779,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 783885.6800496303,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003525,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17621284.86995889,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17048034.698349793,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6269752979278564,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.01124642888105,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.2966381548397,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.979817708333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.5859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 419584.386403164,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 426410.9205212932,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001165,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9615824.5428781,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9169012.569397097,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.6987087726593018,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.5823069622068,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.83140471188801,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 278.8274739583333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 294.3046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2009403.7605175655,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2063631.712845713,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001528,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47306581.10461053,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45490820.71236735,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.3189419507980347,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 89.1122006884096,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.35191877532085,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 157.04609375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 165.44140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1409851.9831652907,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1428447.112296364,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.024321,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32572390.113817457,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31305052.11904341,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "70c62ad23a1d932f7e95bf93f57d4c86c82927c3",
          "message": "Emit warning and skip unconnected nodes during engine build (#2023)\n\n# Change Summary\n\nAdd a pre-processing step at the start of pipeline build that gracefully\nremoves unconnected nodes from the incoming `PipelineConfig`.\n\nInput with unconnected nodes:\n```yaml\nnodes:\n  unconnected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  connected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  unconnected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:batch:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      otap:\n        min_size: 1\n        sizer: items\n      flush_timeout: 5s\n\n  connected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop_exporter\n        dispatch_strategy: round_robin\n    config:\n      verbosity: detailed\n      mode: signal\n\n  noop_exporter:\n    kind: exporter\n    plugin_urn: \"urn:otel:noop:exporter\"  \n```\n\nOutput (confirmed that log was able to pass through remaining connected\nnodes with debug processor):\n```log\n2026-02-11T19:01:57.699Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_receiver, node_kind=receiver] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.706Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\n2026-02-11T19:01:57.701Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_proc, node_kind=processor] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.702Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=2] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.725Z  INFO  otap-df-otap::receiver.start: Starting Syslog/CEF Receiver [protocol=Tcp, listening_addr=127.0.0.1:5514] entity/node.attrs: node.id=connected_receiver node.urn=urn:otel:syslog_cef:receiver node.type=receiver pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n\nReceived 1 resource logs\nReceived 1 log records\nReceived 0 events\nLogRecord #0:\n   -> ObservedTimestamp: 1770836524675426978\n   -> Timestamp: 1770836524000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Attributes:\n      -> syslog.facility: 16\n      -> syslog.severity: 6\n      -> syslog.host_name: securityhost\n      -> syslog.tag: myapp[1234]\n      -> syslog.app_name: myapp\n      -> syslog.process_id: 1234\n      -> syslog.content: User admin logged in from 10.0.0.1 successfully [test_id=234tg index=1]\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0\n```\n\nInput with no connected nodes:\n```yaml\nnodes:\n  recv:\n    kind: receiver\n    plugin_urn: \"urn:test:a:receiver\"\n    config: {}\n  proc:\n    kind: processor\n    plugin_urn: \"urn:test:b:processor\"\n    out_ports:\n      out:\n        destinations: [exp]\n        dispatch_strategy: round_robin\n    config: {}\n  exp:\n    kind: exporter\n    plugin_urn: \"urn:test:c:exporter\"\n    config: {}\n```\n\nOutput:\n```log\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=proc, node_kind=processor]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=recv, node_kind=receiver]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=exp, node_kind=exporter]\n2026-02-11T19:00:02.759Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=3]\n2026-02-11T19:00:02.759Z  ERROR otap-df-state::state.observed_error: [observed_event=EngineEvent { key: DeployedPipelineKey { pipeline_group_id: \"default_pipeline_group\", pipeline_id: \"default_pipeline\", core_id: 0 }, node_id: None, node_kind: None, time: SystemTime { tv_sec: 1770836402, tv_nsec: 759880158 }, type: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: Some(\"Pipeline encountered a runtime error.\") }]\n2026-02-11T19:00:02.760Z  ERROR otap-df-state::state.report_failed: [error=InvalidTransition { phase: Starting, event: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: \"event not valid for current phase\" }]\n2026-02-11T19:00:02.760Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\nPipeline failed to run: Pipeline runtime error: Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\n```\n\n\n## What issue does this PR close?\n\n* Closes #2012\n\n## How are these changes tested?\n\nUnit tests and local engine runs.\n\n## Are there any user-facing changes?\n\n1. Engine is now more flexible and does not crash with unconnected nodes\npresent in the config.\n2. Engine provides visible error if there are no nodes provided instead\nof starting up successfully.",
          "timestamp": "2026-02-11T23:33:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70c62ad23a1d932f7e95bf93f57d4c86c82927c3"
        },
        "date": 1770920942129,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7074872255325317,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.81499119028841,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.93308801735492,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.580729166666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.1953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 232385.22242025298,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 236353.17044013392,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000786,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5277837.097928735,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5143388.558325893,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.9194271564483643,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.12900887238652,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.31091235555556,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 282.6569010416667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 298.859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2040132.4244712784,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2099692.604500429,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002505,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47956902.968447074,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46319329.935220316,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6447700262069702,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.62187431808599,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.77070128842887,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.77526041666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 422828.1669581562,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 429782.71794745815,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000998,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9635956.449446749,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9302423.314838951,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.2408456802368164,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.6414591705708,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.20842641462659,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 145.63203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 155.89453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1273779.812686008,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1302323.2527588585,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001177,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 29518538.61027507,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28318081.62913427,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.3160524368286133,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.54060767307514,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.6840560599274,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 88.185546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 92.40625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 768342.610559353,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 778454.4027620233,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001233,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17603054.9535802,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16927821.88207315,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "75a2f71ba0765bd22358a4a2c772bf4eabc66c35",
          "message": "Reindex implementation (#2021)\n\n# Change Summary\n\nThis is a reimplementation of reindex as a part of the ongoing work of\n#1926. This now reindexes all the required columns, has support for\nmetrics, and has support for dictionary encoded columns.\n\nI also was able to uncomment most of the batching tests after this\nchange 🥳. One more to go which requires split.\n\nOther minor changes:\n\n- Made it so that `allowed_payload_types` returns payload types in the\nexact same order that they are stored. This is occasionally handy to\nhave.\n\nThings deferred:\n\n- Benchmarks. I had nothing to compare it to since the original didn't\nreindex a bunch of the necessary columns anyway like scope or resource\nid nor did it support dictionaries. I'll add these in when I get to the\nnext point..\n- Some optimization opportunities like using naive offsets instead of\nsorting and reindexing everything starting at 0. We need this path\nbecause it's possible to get into situations where we absolutely need to\ncompact things down to fit into u16, but we can likely skip it a decent\nportion of the time.\n\n## What issue does this PR close?\n\nPart of #1926.\n\n## How are these changes tested?\n\nI added a big unit test suite.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-13T02:10:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/75a2f71ba0765bd22358a4a2c772bf4eabc66c35"
        },
        "date": 1770951370103,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8805455565452576,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.90524606349686,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.11736764354902,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.651171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.62109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 227734.18778275306,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 229739.4910699588,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000899,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5746175.29536503,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5042440.811269701,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1539405584335327,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 72.27584328535156,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.6063162555032,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 260.41614583333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 270.796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1948336.1568561394,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1970818.7963945454,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007189,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49440361.53519241,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 44363509.39349239,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.114372968673706,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.85396636779244,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.07626113861386,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.47890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.55078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 407211.97198749724,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 415821.95249367657,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001065,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10292694.201739436,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9206402.807487987,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.5260815620422363,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.8305690803303,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.44240155916474,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 160.54661458333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 166.5859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1403404.0658416245,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1438855.1972195818,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00824,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35542746.34410252,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31872043.100514434,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.1332814693450928,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.7446444270342,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.91715249573974,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 91.596875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 94.328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 703997.9179232991,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 719016.1760933435,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001632,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17783409.364305466,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15827820.583120685,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771007123277,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.106402039527893,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.88413741239452,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.0108786130031,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.40026041666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 227520.22439068,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 230037.5130001857,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001066,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5744672.714619289,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5135215.791954077,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.4125618934631348,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 59.61632554113637,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 60.12434296319208,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 288.4350260416667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 301.0625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1550945.2200066294,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1588362.7323430404,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001851,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 39994568.366125,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 35390889.4408832,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.8750916719436646,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.84761916763718,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.1921417904406,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.04908854166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 407291.7783124758,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 414928.8724774769,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001879,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10335981.555477496,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9122037.328221094,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1189671754837036,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.60824830453944,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.21347056332586,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 146.875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 150.0859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1220116.6746329668,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1233769.379941856,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00276,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30498161.107318584,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27211466.945064433,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.7884052991867065,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.04210251724948,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.18755747703234,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 91.22591145833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 94.94140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 724766.3629670647,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 737728.1236497784,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001725,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18379247.09190915,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16454723.516513834,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771037470071,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9750533699989319,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.85109040665644,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.99246895741557,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.62291666666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.1171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 231914.65837313066,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 234175.95002332123,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001106,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5729572.984407712,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5193744.332917798,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.8607990741729736,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.65484635960424,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.85373964803952,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 286.01731770833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 293.10546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1937303.7311771952,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1973353.060030915,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006665,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 48832911.856835075,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 43593337.499823295,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.724822759628296,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.03438158344811,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.29342651587916,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.159114583333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.7734375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 408150.8328673136,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 415190.7115467377,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001034,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10177596.17681063,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9176403.794306543,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.733229398727417,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.95809394083663,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.38253533992584,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 158.2265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 163.50390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1428619.8177136916,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1467667.2746387958,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001654,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36564770.74973769,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32848345.430557486,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.3197654485702515,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 91.2619877010244,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.4597231554667,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 89.367578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 92.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 714413.663648212,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 723842.2481947892,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004765,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17867245.669608984,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16036261.556466931,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771092943398,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4111842215061188,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.73803284616118,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.92616624884224,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.006640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.87109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 228274.89909473836,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 229213.5294364108,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002322,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5652689.992556975,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4996977.766790773,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.569697141647339,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.08008494071305,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.26840333410655,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 288.7421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 301.453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2123340.8803882753,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2177904.3128531873,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007955,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 55137074.5223226,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 49248852.66394067,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.8763079643249512,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.68413894904917,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.0135889351816,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.33580729166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 354734.55888952117,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 361390.47180799383,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000785,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8936881.021682464,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8028622.3311757445,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.3652803897857666,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 88.07746173808793,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.61366115479116,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 157.07395833333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 161.34765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1345593.8385718216,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1377420.9068787938,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00427,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 34421779.01230914,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 30537573.925583746,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0738805532455444,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 91.21472489955738,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.35791897157904,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.94140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 88.55078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 707151.5608604123,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 714745.5240729534,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005558,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17847561.07869536,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15861745.790837616,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771124083641,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9517411589622498,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.89254892172927,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.08618213604186,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.079296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.1796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 228630.4410894056,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 230806.41102475434,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000829,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5672231.246289399,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5076354.0823562285,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6632622480392456,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.2063410318455,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.58396335292298,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 288.84947916666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 297.91796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2080014.1546423,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2114610.244015775,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.011407,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53446460.531497516,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47472768.67697157,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.314983606338501,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 49.38698882057862,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.45755488037166,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.78346354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.8671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 234064.82758540232,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 239483.38994474357,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001155,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5865113.39965068,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5170628.202926633,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5642738342285156,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.1468197043091,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.53733293790548,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 148.43684895833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 155.1328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1262833.9378100997,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1282588.117536236,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001479,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31883161.87790262,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28575925.970746268,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.4830777645111084,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.50350614161957,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.59846896423595,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.12408854166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 88.19140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 752597.0024659596,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 771284.5704716792,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001387,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19228734.291918933,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17117752.764409803,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771179330158,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2755101919174194,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.82969224431706,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.00669797849463,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.565234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.19140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 230806.95726033938,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 233750.92355192534,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000687,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5736226.65047477,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5104193.808100656,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.309507369995117,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 80.41956110201976,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 80.74600610320695,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 296.29973958333335,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 307.5859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2046862.79901054,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2094135.245447972,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002818,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52786841.156931415,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 47049909.97165727,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6428898572921753,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.67078739286266,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.86619340523883,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.55104166666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.80859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 410328.3819439614,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 417069.62516934355,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000802,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10347352.563830387,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9311110.163346112,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.411365032196045,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.04363414158415,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.794611885163,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 153.61158854166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 159.58984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1426718.7878651735,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1446854.9985853392,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007318,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35941311.72573722,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32239219.421016134,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1075737476348877,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.6024335410725,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.7709482630273,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 92.61640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 94.98046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 766288.5644872545,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 774775.7758687142,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.024427,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19217740.15943122,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17085546.57958541,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771210363543,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4442193508148193,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.84232414565015,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.00614880765077,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.42747395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.2578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 227478.2067834155,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 230763.49111125758,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000895,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5761551.5484559955,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5087822.545982163,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -3.1110026836395264,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.64268767893125,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 71.00617157594009,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 300.32213541666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 312.34765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1821258.0338284725,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1877917.4228004408,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002059,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47354938.238706514,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 42333761.04886957,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6989022493362427,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.71038423545573,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.9083974546019,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.85299479166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.7890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 359123.7214215365,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 365224.8823970723,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001695,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9071249.725024432,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8057292.596835931,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9219722747802734,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.08985891880289,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.28651309683902,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 162.95260416666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 168.98828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1396307.3953922857,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1423144.035445751,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00155,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35149796.14269137,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31549782.310709827,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0319427251815796,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.50728936290582,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.7922527997523,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 87.469921875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 89.76953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 719313.1790744065,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 726736.079495424,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008888,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17772490.57612219,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16048160.277168645,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "44cd962e942288b48ad9c46d7d5b5d5dca9262ae",
          "message": "chore(deps): update golang docker tag to v1.26 (#2045)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| golang | stage | minor | `1.25` → `1.26` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-16T16:03:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/44cd962e942288b48ad9c46d7d5b5d5dca9262ae"
        },
        "date": 1771266257781,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9251378774642944,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.84444618059717,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.95501880235805,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.62721354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.1640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 230592.4695773343,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 232725.76778235717,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000988,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5787868.759745113,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5193417.662068523,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.4886269569396973,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 80.99735621159813,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.2906451997836,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 275.2201822916667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 286.9375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2093309.1577352446,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2145403.812824899,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001549,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53950975.53732422,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48317185.77121143,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.1447830200195312,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.04461032649562,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.25183148251315,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.47018229166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.2265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 417747.5274153631,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 426707.30566085153,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001485,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10490687.83174573,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9296164.331981542,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4091583490371704,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 91.97975695313751,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.43999182999923,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 157.32213541666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 164.484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1426676.866020615,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1446781.0019621947,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001186,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36065921.798448764,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32188492.071901347,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.299572467803955,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.55945784899619,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.7096721009835,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 89.26770833333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 93.34765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 760692.236430906,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 778184.9067920325,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002274,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19263038.214876536,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17367692.49620054,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "8d843949245c99f14780fa2794e2bf5bdfb8983b",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.51.0 (#2046)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.50.0` → `v1.51.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.51.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0/v1.51.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.51.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1510v01450)\n\n##### 💡 Enhancements 💡\n\n- `pkg/scraperhelper`: ScraperID has been added to the logs for metrics,\nlogs, and profiles\n([#&#8203;14461](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14461))\n\n##### 🧰 Bug fixes 🧰\n\n- `exporter/otlp_grpc`: Fix the OTLP exporter balancer to use\nround\\_robin by default, as intended.\n([#&#8203;14090](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14090))\n\n- `pkg/config/configoptional`: Fix `Unmarshal` methods not being called\nwhen config is wrapped inside `Optional`\n([#&#8203;14500](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14500))\nThis bug notably manifested in the fact that the\n`sending_queue::batch::sizer` config for exporters\nstopped defaulting to `sending_queue::sizer`, which sometimes caused the\nwrong units to be used\n  when configuring `sending_queue::batch::min_size` and `max_size`.\n\nAs part of the fix, `xconfmap` exposes a new\n`xconfmap.WithForceUnmarshaler` option, to be used in the `Unmarshal`\nmethods\nof wrapper types like `configoptional.Optional` to make sure the\n`Unmarshal` method of the inner type is called.\n\nThe default behavior remains that calling `conf.Unmarshal` on the\n`confmap.Conf` passed as argument to an `Unmarshal`\nmethod will skip any top-level `Unmarshal` methods to avoid infinite\nrecursion in standard use cases.\n\n- `pkg/confmap`: Fix an issue where configs could fail to decode when\nusing interpolated values in string fields.\n([#&#8203;14413](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14413))\nFor example, a header can be set via an environment variable to a string\nthat is parseable as a number, e.g. `1234`\n\n- `pkg/service`: Don't error on startup when process metrics are enabled\non unsupported OSes (e.g. AIX)\n([#&#8203;14307](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14307))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-17T00:36:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8d843949245c99f14780fa2794e2bf5bdfb8983b"
        },
        "date": 1771296804723,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8654660582542419,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.91025299861556,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.02656557551461,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.03736979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 231700.08939954045,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 233705.37504049696,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001427,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5723650.584741273,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5212265.9182760175,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.0490241050720215,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.11017743190702,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.27086555632877,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 284.35625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 294.98828125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2106571.348749144,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2149735.5074435114,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008305,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 53790015.76616888,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48329325.75092197,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9131852388381958,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.64012230035037,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.79131421481195,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.295833333333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.48046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 408106.1332420731,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 415913.95977819356,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001333,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10317598.966286818,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9245818.94855318,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.9501544833183289,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.35911284327325,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.69199385043693,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 154.06640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 160.8984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1306566.3209496147,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1318980.718871563,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007743,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32698872.944068525,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28989343.16802714,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.8912583589553833,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46739536344138,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.94014388471372,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 87.29192708333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 89.55859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 753262.3041452051,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 767508.4400657777,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.019082,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18904129.669758197,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16880592.7195633,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "a6e0e74750abf6f949784dfada1df293d2c7250b",
          "message": "chore(deps): update dependency pyarrow to v23.0.1 (#2049)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==23.0.0` →\n`==23.0.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/23.0.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/23.0.0/23.0.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNS4zIiwidXBkYXRlZEluVmVyIjoiNDMuMTUuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-17T13:18:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a6e0e74750abf6f949784dfada1df293d2c7250b"
        },
        "date": 1771352963259,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.792539119720459,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.8394797013125,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.96742400984388,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.916666666666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.40625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 231487.6523461299,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 233322.28244250332,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001196,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5851815.723448869,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5146919.807794063,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.0643155574798584,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.34507036838039,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.62766693072359,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 280.596484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 295.109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1988060.1987393887,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2029100.0368384006,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008034,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 51229376.52357831,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45656409.892690524,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9827263355255127,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.79496343365811,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.59945843100775,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.856770833333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.54296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 411011.5737936762,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 419160.8087929072,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000724,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10472361.540056627,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9350370.606216123,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.581853985786438,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 79.32897237317901,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 80.3874604630992,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 146.890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 154.55859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1240712.2644826188,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1260338.5219418174,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001251,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 31538802.859000903,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 28075046.436053336,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.313863754272461,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.70362388944511,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.83885674303406,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 89.15651041666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 91.109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 706205.1075718349,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 722545.732266744,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002602,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17981513.586693455,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15952872.6035163,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "7962c78a3cd1bd64c8108288975f39780c072ccf",
          "message": "feat: Upgrade bytes library to resolve RUSTSEC-2026-0007 (#2055)\n\n# Change Summary\n\nResolves security vulnerability reported at\nhttps://rustsec.org/advisories/RUSTSEC-2026-0007.html\n\n## What issue does this PR close?\n\n\n* Closes #N/A\n\n## How are these changes tested?\n\nBuilding and running local tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-17T23:34:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7962c78a3cd1bd64c8108288975f39780c072ccf"
        },
        "date": 1771384763356,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.647050142288208,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.8211882029753,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.94518542989215,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.9453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.48046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 230550.28776086585,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 234347.5665399607,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000862,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5780272.257877036,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5221056.4467776595,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.1737782955169678,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.70191132264105,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.97098700968617,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 298.56223958333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 307.14453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1924835.5095198774,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1966677.1629861558,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008336,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49380498.62569002,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 44138606.911809675,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.7969346046447754,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.30523812134672,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.61558950243,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.02291666666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.00390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 358531.53779390675,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 364974.1146940019,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000836,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 8968235.104133535,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8035010.648718045,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.316938638687134,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.11069992662648,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.5273364509136,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 162.02708333333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 167.03125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1283479.0436193896,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1313216.4647837332,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002513,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32832399.602291208,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 29208163.023455627,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.127971887588501,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.54085149744238,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.73073121113688,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 90.183203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 92.87890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 755865.2910715702,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 771949.8914832781,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002734,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19126579.048347697,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17096050.77980719,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "305fb777d4f44ba6e0559938e671fb5169087319",
          "message": "[query-engine] KQL RecordSet processor logging and perf improvements (#2052)\n\n# Changes\n\n* Switch logs from \"debug\" to \"error\" level when KQL RecordSet processor\nencounters an error processing logs.\n* Add an option to OTLP Bridge for opting out of serialization of\ndropped records. Note: A count of dropped records will always be\nreturned.\n* Use this feature in KQL RecordSet processor. This basically avoids\nwasting cycles making an OTLP blob which was just being dropped on the\nfloor.",
          "timestamp": "2026-02-18T18:01:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/305fb777d4f44ba6e0559938e671fb5169087319"
        },
        "date": 1771440406650,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8237835168838501,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.81097751349238,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.91387858823529,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.47213541666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.46875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 233066.2642391731,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 234986.22577594363,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001202,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5710791.400930365,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5177293.092438617,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0692152976989746,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.73594116386225,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.92519490217308,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 295.933203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 309.12890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2346127.04086604,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2371212.190774925,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006817,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 60099326.56370849,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 53637969.01050035,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.042879819869995,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.73518061934128,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.95017534010465,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.984765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.0078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 413527.45009906596,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 421975.3190162999,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000931,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10482465.901838647,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9368705.3281636,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1520534753799438,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.55013164728597,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.87717112623763,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 155.299609375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 157.984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1458989.9801504079,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1475798.3255845702,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008286,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36863207.48105211,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32929943.04986544,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.32806134223938,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.53047145306829,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.67190391341322,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.65885416666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 89.10546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 760180.6485800872,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 777878.1203229353,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002243,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19357818.914330002,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17332685.86663382,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "74b09ca8e6f889ef8aa43fa068528aff67e26c9f",
          "message": "feat: add content_router processor (#2030)\n\nDescription:\n\n   Implements the content router processor described in #2029\n\n   - Registered as `urn:otel:content_router:processor`\n- Zero-copy routing via `RawLogsData`/`RawMetricsData`/`RawTraceData`\nprotobuf views\n- Native `OtapLogsView` for Arrow logs (metrics/traces Arrow views\npending — falls back to OTLP conversion)\n   - Destination-aware mixed-batch detection with single-pass fold\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-19T17:52:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74b09ca8e6f889ef8aa43fa068528aff67e26c9f"
        },
        "date": 1771526373669,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2218601703643799,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.79849094244283,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.91829533034195,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.530598958333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 230464.43327492653,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 233280.38643623894,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000998,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5738553.42534925,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5104238.959365751,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.4919426441192627,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.90083223544704,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.00131937172775,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 82.2125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 83.25390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 582469.9755416448,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 596984.7928342957,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001582,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14493719.978048809,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13047710.904999582,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.599238872528076,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.66370401335264,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.80786003872667,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.34231770833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.06640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 410371.27374150825,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 421037.80369881593,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000769,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10344725.069689216,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9241450.429090101,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.2810221910476685,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.13285707746,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.41952327960323,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 290.916015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 300.1875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2051548.3843937414,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2077829.1737360228,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004286,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 51778672.35611368,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46224418.312308446,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.031022787094116,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.74327331704464,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.95474554455446,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 146.03515625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 152.34375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1331843.1235859392,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1358893.1604375811,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001397,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33395290.62345858,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 29884851.213086527,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "c5777cfeedaed5f80add973ea5e3493432639c7b",
          "message": "Add TLS & mTLS support to OTLP HTTP Exporter (#2109)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for TLS & mTLS to the OTLP HTTP Exporter. These can be can\nbe configured using the same config options as the OTLP gRPC exporter.\n\nSame as for the OTLP gRPC exporter, we don't yet support hot-reloading\nthe client-cert.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #1145\n\n## How are these changes tested?\n\nNew unit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n New TLS config options for this component are user facing.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-26T01:49:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c5777cfeedaed5f80add973ea5e3493432639c7b"
        },
        "date": 1772075861849,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7795678973197937,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.90934805095092,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.06967820849421,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.469401041666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.12890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 229868.37030100447,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 231660.35032042724,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000669,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5752481.783180116,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5119179.224987987,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1079477071762085,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.7997081087194,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.1356571003487,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 282.519140625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 293.0390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2016728.0920528965,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2039072.3844643075,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.012104,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 51825098.5985126,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45993551.8281575,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.163642406463623,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.78077772921458,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.97155640511794,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.48385416666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.1640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 408194.56008627074,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 417026.43084656197,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000878,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10332638.801040685,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9191358.030373191,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0857242345809937,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.07369860557343,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.08175988872576,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 159.49361979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 165.3203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1422400.7355855017,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1437844.0849632183,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007708,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35737739.21550421,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32107194.224093176,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6013078689575195,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.75221354167664,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.9320087129323,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 88.30625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 92.3203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 762020.7203075801,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 774223.0184648423,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001812,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19251479.016903803,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17253888.139265385,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "45c3ea7b3beee63354c2c362f5e1f45cd091e017",
          "message": "URN rename to microsoft:exporter:azure_monitor (#2119)\n\nChanges URN from urn:microsoft_azure:exporter:monitor to\nurn:microsoft:exporter:azure_monitor to align with the naming convention\nused by other Microsoft components (urn:microsoft:exporter:geneva,\nurn:microsoft:processor:recordset_kql)",
          "timestamp": "2026-02-26T15:54:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/45c3ea7b3beee63354c2c362f5e1f45cd091e017"
        },
        "date": 1772132094353,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8416564464569092,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.91911657795507,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.04067553077816,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.945963541666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.39453125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 228118.2656713501,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 230038.2376077603,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000877,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5731753.830782522,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4997022.141706408,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.687523603439331,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.38206204370789,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.93538247225646,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 291.1763020833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 302.41796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2016180.667543534,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2070365.997638725,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00148,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52558504.16357085,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46991598.30612416,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.0568864345550537,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.81587317073355,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.9739835399598,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.940104166666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.1171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 410712.4041916909,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 419160.29183478525,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000798,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10373411.552408814,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9374740.603818137,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.472604274749756,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.65783309608128,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 78.99130772424478,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 151.55651041666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 153.5859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1190623.1977971515,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1220062.5977240496,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001223,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30312342.37901236,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27150666.413841072,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.481931209564209,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.12841707226711,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.38069923267327,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 94.128125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 96.67578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 716857.2021534042,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 727480.5323524352,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003783,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18029593.854564156,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16211012.727470716,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "274e9e1c0d2e09d205ef1a45aa5c327f3a6a3e9f",
          "message": "   [Geneva exporter] Add UserManagedIdentityByResourceId auth variant (#2096)\n\n### Description:\n\n- Adds `UserManagedIdentityByResourceId` variant to the Geneva\nexporter's `AuthConfig` enum, enabling authentication via Azure Resource\nManager resource ID. This maps to the existing\n`AuthMethod::UserManagedIdentityByResourceId` in `geneva-uploader` - no\nnew auth logic added.\n\n### Changes:\n- New `AuthConfig` variant with `resource_id` and `msi_resource` fields\n- Mapping to `AuthMethod::UserManagedIdentityByResourceId` in\n`from_config`\n   - `msi_resource` extraction for the new variant\n- Extended `test_auth_config_variants` to cover all five auth variants\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-02-27T00:29:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/274e9e1c0d2e09d205ef1a45aa5c327f3a6a3e9f"
        },
        "date": 1772160808580,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7654109001159668,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.95054168914785,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.1037774818027,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.06458333333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.58984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 228544.94895582434,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 230294.25689503743,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000871,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5652263.809549269,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5075605.843137975,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.922086238861084,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 80.7939460386464,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.08270833655257,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 285.105859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 299.625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2036682.6264451968,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2075829.4219466061,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.019421,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52518851.60489612,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46731605.154029235,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5179555416107178,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.05455482049908,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.26788706537718,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.24752604166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.83203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 410369.9879320258,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 416599.2219090772,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000957,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10330989.670464346,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9254178.829249598,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0264414548873901,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.2945127155742,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 79.01858495107481,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 161.05950520833332,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 165.265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1213703.7807116702,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1226161.7389357793,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003412,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30642535.830266632,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27443472.9420192,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.0922446250915527,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.40937251325678,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.7229479628483,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 87.46705729166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 90.5859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 734106.3559643489,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 749465.6576280999,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002728,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18666541.688296616,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16541620.665520333,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "pradhyum",
            "username": "pradhyum6144",
            "email": "pradhyum314@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "05a881850552252d5d19119ab26ae61c0362082f",
          "message": "feat: add aggregated status checks for CI workflows (#2033)\n\n## Summary\n\nThis PR adds aggregated status check jobs to both Go and Rust CI\nworkflows to simplify maintenance of required status checks.\n\n### Changes\n\n- **go-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `gen_otelarrowcol`\n  - `codeql`\n\n- **rust-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `fmt`\n  - `pest-fmt`\n  - `clippy`\n  - `deny`\n  - `docs`\n  - `structure_check`\n\n### Benefits\n\n- **Easy maintenance**: Add/remove required checks via PR instead of\nupdating GitHub settings\n- **Single check**: Repository admins only need to set `Go-CI /\nrequired-status-check` and `Rust-CI / required-status-check` as required\nin branch protection\n- **Industry pattern**: Follows the same pattern used by\n[opentelemetry-java-instrumentation](https://github.com/open-telemetry/opentelemetry-java-instrumentation)\n\n### How It Works\n\nEach aggregated job:\n- Uses `if: always()` to run even if some jobs fail\n- Depends on all required jobs via the `needs` field\n- Checks each job's result and fails if any required job didn't succeed\n- Provides clear error messages indicating which job failed\n\n### After Merge\n\nRepository admins should update branch protection rules to require only:\n- `Go-CI / required-status-check`\n- `Rust-CI / required-status-check`\n\nThis will allow all other checks to be managed in code going forward.\n\n\nCloses #1964",
          "timestamp": "2026-02-27T16:20:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05a881850552252d5d19119ab26ae61c0362082f"
        },
        "date": 1772216788666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5962580442428589,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.93115850942952,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.04473825607579,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.02122395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.56640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 229863.80372435448,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 233533.02325072885,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001861,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5677261.303918587,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5068355.127157359,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -3.3128042221069336,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.36714573168499,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.64591477520699,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 291.3471354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 298.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1940961.6084956427,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2005261.8651395962,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006479,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50015867.88743446,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 44920268.780388,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.188978433609009,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.02735475293022,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.28312209514955,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.326822916666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.66015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 411263.73228946235,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 420266.20630318514,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001284,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10372034.093680965,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9275536.302894633,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.5549168586730957,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.47921286125319,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.86395227571285,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 160.94661458333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 167.41015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1391044.0675883691,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1426584.0877179587,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002217,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 35430350.91791855,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 31942706.185289875,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.3740015029907227,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.99607901409418,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.16474790985828,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 85.75377604166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 87.61328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 710263.3584624721,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 727125.0199789236,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00073,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17933044.19536043,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16200978.991451206,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "532cc7dfa1a3ef24a3544b3af1419ccaeab71714",
          "message": "Columnar query engine expression evaluation: simple arithmetic (#2126)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a module to the columnar query engine with the ability evaluate\nsimple arithmetic expressions on OTAP record batches. For example, it\ncould evaluate expressions such as `severity_number + attributes[\"x\"] *\n2`.\n\nNote that the expression evaluation isn't yet integrated into any\n`PipelineStage` implementation, but the intention is that this can soon\nbe used to implement more advanced filtering, attribute insertion,\ncolumn updates and so on.\n\nWhile on the surface, this isolated simple arithmetic not appear\nterribly useful, the main/important contributions in this PR are to lay\ndown some foundations for future expression evaluation:\n\n**Transparent joins in the expression tree**\nThis PR adds is the ability to evaluate a set of DataFusion expressions\nwhile transparently joining data from different record batches as the\nexpression evaluates.\n\nFor example, consider we had `severity_number * 2 + attributes[\"x\"] +\nattributes[\"y\"]`, we'd need to first evaluate:\n- 1. `severity_number * 2`\n- 2. `attributes where key = \"x\"`, then select the value column based on\nthe type\n- 3. `attributes where key = \"y\"`, then select the value column based on\nthe type,\n\nThen we need to join these three expressions on the ID/parent ID\nrelationship, then perform the additions.\n\nThis PR builds the expression tree in such a way that it can manage\nwhere these joins need to happen on either side of a binary expression,\nand it performs the joins automatically during expression evaluation\nwhile keeping track of the ID scope/row order of the current data at\neach stage.\n\n**Type evaluation and coercion**\n\nWhile planning the expression, the planner attempts to keep track of\nwhat are the possible types that the expression could produce if it were\nto successfully evaluate. When it detects invalid types, it is able to\nproduce an error indicating an invalid expression.\n\nFor example, we'd be able to detect at planning time that `severity_text\n+ 2` is invalid, because text can't be added to a number.\n\nFor expressions where we can't determine that the types are invalid at\nplanning time, it will be determined and runtime and an error will be\nproduced when the expression evaluates on some batch. For example\n`attributes[\"x\"] + 2`, it's unknown whether `attributes[\"x\"]` is an\nint64, so it's assumed that the expression will produce an int64, and if\n`attributes[\"x\"]` is found not to be this type, an ExecutionError will\nbe produced.\n\nThe planner automatically coerces integer types when necessary.\nCurrently when adding two integers, they will be coerced into the\nlargest type that could contain the value, while keeping the signed-ness\nof one side. For example, uint8 + int32 will produce an int32. I realize\nthis type of automatic integer coercion is probably controversial, so in\nthe future I'm happy to get rid of this in favour of forcing explicit\ncasting if that is preferred.\n\n**Missing data / null propagation**\n\nWhen one side of an expression is null, for the purposes of arithmetic\nthe expression will evaluate to null. This includes the case of null\nvalues, missing attributes, missing columns, and missing optional record\nbatches.\n\nFor example: `attributes[\"x\"] + 2` would evaluate as null if the\nattribtues record batch was not present, there were no attributes with\n`key == \"x\"`, or the attributes where `key==\"x\"` had type empty, and so\non.\n\n**Relocated the projection code**\n\nAdds a new module called `pipeline::project` which has the projection\ncode that was previously inside the filter module. We need to project\nthe input record batches into a known schema to evaluate the\nexpressions, and also consider the expression evaluation to result in a\n`null` if the projection could evaluate due to missing data.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to https://github.com/open-telemetry/otel-arrow/issues/2058\n\n## How are these changes tested?\n\nThere are 64 new unit tests covering these changes\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nNo\n\n\n## Future work/followups:\nThere are many, but most pressing are:\n- Other types of expression evaluation, including string expressions,\nunary math expressions, function invocation and bridging this with the\nfiltering code (for expressions that produce boolean arrays).\n- Integrating expression evaluation with various pipeline stages\nincluding those which set attributes, set values, and filtering\n- OPL Parser support for the type of expressions we're able to evaluate\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-28T00:22:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/532cc7dfa1a3ef24a3544b3af1419ccaeab71714"
        },
        "date": 1772247057041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5960643291473389,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.87941928711507,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.9579688080808,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.79127604166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 227222.71048453168,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 230849.3310326737,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000763,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5738329.892674457,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5085882.912063497,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.7138724327087402,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.42276623773265,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.62890646695492,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 286.51796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 295.92578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1982146.202900688,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2035939.12230714,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001354,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 51541287.083708085,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46029660.05580391,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.037824869155884,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 49.45664302741784,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 49.54281015617752,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.58151041666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.26953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 238681.80509794992,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 243545.72233029167,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001021,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5963345.008658852,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5301869.761139472,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.2004668712615967,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 73.48755364941097,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 73.7069857757154,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 141.04114583333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 143.25390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1154933.8941919128,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1168798.4931847164,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008948,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 29026415.59910487,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 26068225.826277886,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.269867420196533,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 88.12439666665922,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.03234570897834,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 97.54583333333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 100.76953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 691694.2744446622,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 707394.8180406525,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003018,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17415889.553873938,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15349003.013640737,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772302533681,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.902294397354126,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.98968407460583,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.11887992605145,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.801822916666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.1875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 231701.88890807933,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 233792.5220897712,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000961,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5775249.614442643,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5108088.759974749,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9386287927627563,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.80256495306156,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 77.00712942594453,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 295.7233072916667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 304.6015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1975115.4619261543,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2013405.6190632374,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.038406,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50813545.70667209,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45579466.42081783,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.1417043209075928,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.10314662299491,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.29981635883905,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.631770833333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.76953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 412375.44735553616,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 421207.3098728763,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000934,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10451804.331106449,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9364209.051138764,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9126776456832886,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.99939752889833,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.19185842769731,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 154.2578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 159.75,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1472251.2470867892,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1500410.666533425,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001237,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37452568.43850095,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33472158.186233316,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.18058180809021,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.63397439984402,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.25851078449612,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 87.31614583333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 92.171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 745469.6968790428,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 761725.2745049507,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001559,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18887978.425180744,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16842058.349602487,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772336613204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7000479102134705,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.96698593813882,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.06835638383214,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.791796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 231600.31271610147,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 233221.62583706455,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000748,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5833818.493465182,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5144287.330641022,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.8229904174804688,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.73684229887698,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.91651182855811,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 280.9955729166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 291.59375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1930308.3778382665,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1984800.8022721,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001588,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49599503.169890195,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 44423199.65952924,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.8325315713882446,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.09543828224089,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.8189677511442,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.059635416666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.671875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 409770.9431154962,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 417280.12513881177,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001209,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10176947.994835103,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9186862.777076432,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5476574897766113,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.47106870718588,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.01877081799907,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 146.76796875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 150.12109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1480400.1656832232,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1503311.6895053864,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001247,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37090932.489118904,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33038239.602246325,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8659716844558716,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.7001803172478,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.89345101404537,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 86.77486979166666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 89.15625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 778407.5046736223,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 785148.2933512306,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004848,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19268842.694937155,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17315550.80491018,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772388901890,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3783854246139526,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.92312468941181,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.06478567939403,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.09544270833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.59765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 229055.71227558758,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 232212.982831858,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001193,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5817454.397030236,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5090503.129883568,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -3.233947277069092,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 70.20118814938691,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 70.62759758961681,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 281.5463541666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 292.42578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1825875.3593932318,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1884923.202588863,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002869,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47039213.564769536,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 42168481.03016178,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.8192476034164429,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.84672129622918,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.96750059405942,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.60703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 422144.5759080775,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 429824.4311428063,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001131,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10496317.054035733,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9456242.639145292,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4351750612258911,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.74647725878206,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.02661835105563,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 146.890234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 150.109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1480466.1775609925,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1501713.4586946475,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00203,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37085864.05615899,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33356774.842869062,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8728910088539124,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.56195987111681,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.75362323440527,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 90.225,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 92.61328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 742876.001379289,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 749360.4990770189,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007732,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18375330.210456558,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16377465.969338516,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "308023048a93dd306b0e1525808232b53afcdd7b",
          "message": "chore(deps): update docker digest updates (#2138)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | digest | `4c7eb94` → `51c04d7` |\n| golang | stage | digest | `c83e68f` → `9edf713` |\n| python | final | digest | `9b81fe9` → `6a27522` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My40My4yIiwidXBkYXRlZEluVmVyIjoiNDMuNDMuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-03-02T00:50:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/308023048a93dd306b0e1525808232b53afcdd7b"
        },
        "date": 1772420027119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4005086421966553,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.93891342935294,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.0759131003487,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.874348958333336,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.234375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 231532.04690048198,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 234774.67314240817,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000748,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 5816820.132107663,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5174219.245005035,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.269908905029297,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.67653135804639,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.85316125957446,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 283.77447916666665,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 296.1015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1947260.672382612,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1991461.713924318,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002206,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49937186.885327,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 44629860.814575486,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.1204237937927246,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.99809368054376,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.29200536050641,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.383854166666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.6328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 416514.43833871896,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 425346.3090990102,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000878,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10504792.783206515,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9353364.412647983,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.197094678878784,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.77455988567687,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.12341638717633,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 157.75104166666668,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 160.62109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1444399.8397832124,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1476134.67016653,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001203,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 36760087.598803796,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 32470693.94923534,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.9663470983505249,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.60628204493581,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.7466179611425,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 91.73450520833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 93.9296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 781349.8313681853,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 788900.3827948942,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.011511,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19170215.18099953,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17247846.195692122,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "eb91467c84869e57ea46dbde762123e63132189a",
          "message": "[WIP] fix renovate PR #2140 failing markdown CI (#2148)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-02T17:40:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eb91467c84869e57ea46dbde762123e63132189a"
        },
        "date": 1772475822557,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1674764156341553,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.02175740006408,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.23335013155858,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.413020833333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.12109375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 226581.87272819362,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 229227.16254650944,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000987,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6084313.090609286,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5329785.578240022,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.936922311782837,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.13708045528473,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 75.43990402726146,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 284.1772135416667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 292.01953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1935036.848129581,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1991867.3771660542,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001553,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 52905369.39376331,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 46997488.68905638,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.078052520751953,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.04252476570706,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.01199384184744,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.06041666666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 373677.2460758792,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 381442.45568353945,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000956,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10034489.41986496,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8826752.760032512,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.3569576740264893,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.35939790400928,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.15198302040817,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 144.273046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 149.0859375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1142235.0630880978,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1169157.060294434,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001492,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30893579.654270012,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27239196.884428684,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.267977237701416,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.75190345886509,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.86941095664358,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 88.77890625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 90.4765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 743088.6489276534,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 759941.7308691812,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000895,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 19940809.20542063,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17565270.024575986,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "63a23cf282c43a3dceb453f6fd17c794a4e9bb70",
          "message": "fix: Mark time_unix_nano as required for metrics histogram dp tables  (#2151)\n\n# Change Summary\n\nRemove `schema.Optional` metadata from histogram datapoint types.\n\n## What issue does this PR close?\n\n\n* Closes #2150\n\n## How are these changes tested?\n\nRan the unit tests\n\n## Are there any user-facing changes?\n\nNo\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-02T22:57:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63a23cf282c43a3dceb453f6fd17c794a4e9bb70"
        },
        "date": 1772512390133,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0767459869384766,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.00417726087618,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.10897975048431,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.44609375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.04296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 229824.0570560216,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 232298.67839587404,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001099,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6090891.26056538,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5362053.407983699,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.7633728981018066,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.04398980743271,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.26963723391088,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 285.19518229166664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 294.7578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2171831.167582411,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2231846.962229995,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00767,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59323126.72719909,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 52951252.74662225,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.0624632835388184,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.96999246608237,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.81762184858337,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.47630208333333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.8203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 405462.91973530856,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 413825.4431240745,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001028,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10869821.383618452,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9636630.521133298,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6415506601333618,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 90.15991425998263,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.6402249941892,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 149.52122395833334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 152.3984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1388416.5405081564,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1411208.1021612445,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002558,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 37435594.32772506,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33088398.070847955,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0217187404632568,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 74.04887980281443,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 74.14774470242695,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 81.11380208333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 84.84375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 584595.9808066385,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 590568.9079809239,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00408,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15300490.115769075,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13504590.718311066,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Arthur Câmara",
            "username": "alochaus",
            "email": "arthur_camara@outlook.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "281e61fd2c23b2e0374a7a43c50839632ec48acc",
          "message": "feat: handle Ack and Nack messages in OTAP Exporter. (#1994)\n\n# Change Summary\n\nPart of #1325 (see also #1324).\n\n## Problem: flaky test due to sleep-based synchronization.\n\nThe OTAP Exporter had two related issues:\n\n### 1. Dropped Context\n\nWhen the exporter received pipeline data (OtapPdata), it split the\nmessage into context and payload, then threw away the context:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L233-L235\n\nThe context carries routing information that tells the pipeline who to\nnotify when a message succeeds or fails. Without it, the exporter was a\nblack hole — data went in, but no confirmation ever came back.\n\n### 2. Flaky Test\n\nThe test `test_receiver_not_ready_on_start` had no way to know when the\nexporter finished processing, so it used arbitrary sleeps:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L866-L868\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L870-L872\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L883-L885\n\nOn a slow machine or under load, these might not be long enough. On a\nfast machine, they waste time.\n\n## Solution: thread context through for ACK/NACK.\n\nThe core idea is to preserve the `OtapPdata` context by using\n`take_payload()` instead of `into_parts()`.\n\nThis extracts the payload for gRPC transmission while keeping the\ncontext alive in the original OtapPdata. Then pass that context through\nthe entire streaming pipeline so it can be returned with ACK (success)\nor NACK (failure) notifications.\n\n## Key changes\n\n1. Preserve context at the entry point\n\n```rust\n  // Before: context discarded\n  let (_context, payload) = pdata.into_parts();\n\n  // After: payload extracted, context preserved\n  let payload = pdata.take_payload();\n```\n\n2. Pair each batch with its context through the pipeline\n\nThe internal channels changed from carrying just data to carrying\n(context, data) tuples:\n\n```rust\n  // Before\n  channel::<OtapArrowRecords>(64)\n\n  // After\n  channel::<(OtapPdata, OtapArrowRecords)>(64)\n```\n\n3. Correlate gRPC responses with their original requests\n\nThe exporter uses bidirectional gRPC streaming — requests go out on one\nstream, responses come back on another. A FIFO correlation channel pairs\nthem:\n\n```\n  create_req_stream ──sends pdata──→ [correlation channel] ──recv pdata──→ handle_res_stream\n    (yielded batch)                                                         (got response)\n```\n\nSince both streams are ordered, the first response always corresponds to\nthe first request.\n\n\n4. Send ACK/NACK in the main loop\n\n```rust\n  Some(PDataMetricsUpdate::Exported(signal_type, pdata)) => {\n      self.pdata_metrics.inc_exported(signal_type);\n      effect_handler.notify_ack(AckMsg::new(pdata)).await?;\n  },\n  Some(PDataMetricsUpdate::Failed(signal_type, pdata)) => {\n      self.pdata_metrics.inc_failed(signal_type);\n      effect_handler.notify_nack(NackMsg::new(\"export failed\", pdata)).await?;\n  },\n```\n\nThe effect_handler uses the context inside pdata to route ACK/NACK back\nthrough the pipeline.\n\n\n5. Replace sleeps with deterministic waits in the test\n\n```Rust\n  // Before: sleep and hope\n  tokio::time::sleep(Duration::from_millis(5)).await;\n\n  // After: wait for the actual event\n  timeout(Duration::from_secs(5), async {\n      loop {\n          match pipeline_ctrl_msg_rx.recv().await {\n              Ok(PipelineControlMsg::DeliverNack { .. }) => break,\n              Ok(_) => continue,\n              Err(_) => panic!(\"pipeline ctrl channel closed\"),\n          }\n      }\n  }).await.expect(\"Timed out waiting for NACK\");\n```\n\nThe test is now event-driven: it proceeds as soon as the NACK/ACK\narrives, with a 5-second timeout as a safety net.\n\n## What issue does this PR close?\n\n* https://github.com/open-telemetry/otel-arrow/issues/1611\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-04T00:59:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/281e61fd2c23b2e0374a7a43c50839632ec48acc"
        },
        "date": 1772593665480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2597299814224243,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.04023714371485,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 99.28656643352333,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.844401041666664,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 226922.31546421364,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 229780.92395819744,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001221,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 6083205.8983662175,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5317863.141010622,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6112297773361206,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 72.4563343138049,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 72.73243866357308,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 283.5611979166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 293.70703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1866041.144694281,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1896107.3538252562,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.027521,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 49350516.04013662,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 43808269.58275739,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.893751859664917,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 82.25797819622365,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.50688777107501,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.850390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.5078125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 344707.058236225,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 351234.95455147186,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000953,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9075065.601104943,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 8090648.527553034,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.73681640625,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.6786030373097,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.60352217862913,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 152.501171875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 157.35546875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1436493.3590974922,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1461442.6123495745,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005323,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 38367504.96938954,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 33623359.310896166,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.6079107522964478,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 91.00569627249446,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 91.18106642746675,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 95.759765625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 97.86328125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 708477.0403823252,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 719868.7183775605,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001696,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 18770920.433525905,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16545048.21747165,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "d389678b03da242781069e748a409d90ffddf610",
          "message": "fix: Temporarily disable the nightly otap-filter-otap Go collector scenario (#2396)\n\n# Change Summary\n\nThis scenario has been blocking all the nightly benchmarks for a few\nweeks now and we can't fix it until this is released and we take a\nversion bump:\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/pull/46879\n\nIt looks like it will be another couple of weeks for the next otel\ncollector contrib release as the last one was just a few days ago. I'm\nproposing to disable the scenario for now to unblock everything else.",
          "timestamp": "2026-03-21T01:34:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d389678b03da242781069e748a409d90ffddf610"
        },
        "date": 1774114738493,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9829009771347046,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.16795915662598,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.03900549918674,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.432552083333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.2421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 293433.44386314554,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 290549.2835739242,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002213,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7974987.415858514,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7600688.900621085,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.650263786315918,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 78.71556045887722,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.98025990099009,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 115.644921875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 124.56640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2660537.8675177232,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2731049.1431245403,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00697,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 76037189.30314746,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 72332799.14826056,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8088557720184326,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 75.82738960763227,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 76.58711008468052,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.198046875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.2265625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 369200.32580647845,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 372186.62411324895,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007401,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 9972186.308757998,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 9577972.58192596,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.118849992752075,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.93178972733928,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.96372826338975,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 66.08984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 67.41015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1790904.7883118999,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1828851.3743934473,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001814,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 50491407.965670586,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 48167656.798049375,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.058753490447998,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.92719929017878,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.2708734402852,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.87421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 760010.988936999,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 768057.6320547372,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002164,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 20675901.834655683,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19805386.18435357,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "d389678b03da242781069e748a409d90ffddf610",
          "message": "fix: Temporarily disable the nightly otap-filter-otap Go collector scenario (#2396)\n\n# Change Summary\n\nThis scenario has been blocking all the nightly benchmarks for a few\nweeks now and we can't fix it until this is released and we take a\nversion bump:\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/pull/46879\n\nIt looks like it will be another couple of weeks for the next otel\ncollector contrib release as the last one was just a few days ago. I'm\nproposing to disable the scenario for now to unblock everything else.",
          "timestamp": "2026-03-21T01:34:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d389678b03da242781069e748a409d90ffddf610"
        },
        "date": 1774144779234,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4377662241458893,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.23558549211201,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.79964164349707,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.399348958333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.17578125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 288484.50942639273,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 287221.62172540167,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002168,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7896144.208827295,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7573764.11047458,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.24484258890151978,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.54402570703482,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.45844248335655,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 121.75703125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 126.46875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2412777.7552084615,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2406870.2478177506,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.06188,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 67904217.99418007,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 64868197.93869249,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.2844327688217163,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.03936396283727,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.58048449313378,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.566015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.19921875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 467929.3263740477,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 466598.3820040964,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.011524,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12829901.829156877,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12203234.469250668,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5496355891227722,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 88.40847425707817,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.98423154310478,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 64.42682291666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 66.91015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2000945.1660227163,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1989947.258309468,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008505,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 55504667.80167313,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 53492662.49694004,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.2837110757827759,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.02534665934542,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.5089647696477,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.965885416666666,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.9375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 935921.7600837919,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 947936.2915310482,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002007,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25811127.88453669,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 24742390.47278731,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Google Antigravity",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "993e4a845f4a50d3cdbdbe074f40517bbe1741ee",
          "message": "feat: implement OtapMetricsView for zero-copy OTAP metrics traversal (#2367)\n\n## Summary\n\nImplement zero-copy OTAP Arrow-backed views for metrics data, following\nthe same pattern as OtapLogsView. This enables direct traversal of\nmetrics Arrow RecordBatches without intermediate conversion to protobuf\nor Prost types.\n\n## New file: views/otap/metrics.rs \n\nComplete metrics hierarchy:\n- OtapMetricsView → ResourceMetrics → ScopeMetrics → MetricView →\nDataView\n- Gauge/Sum/Histogram/ExpHistogram/Summary views\n- NumberDataPoint, HistogramDataPoint, ExpHistogramDataPoint,\nSummaryDataPoint views\n- ExemplarView, BucketsView, ValueAtQuantileView\n\n## Modified files (visibility only)\n- MetricsArrays/QuantileArrays/PositiveNegativeArrayAccess fields →\npub(crate)\n- Shared helpers in logs.rs → pub(crate) for reuse\n- views/otap.rs: added mod metrics + re-export\n\n## Design\n- Pre-computed BTreeMap indexes at construction (same as OtapLogsView)\n- Reuses RowGroup, OtapAttributeView, OtapAnyValueView from logs module\n- Introduces Otap32AttributeIter for u32-keyed dp/exemplar attributes\n\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-22T14:45:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993e4a845f4a50d3cdbdbe074f40517bbe1741ee"
        },
        "date": 1774200115619,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.0191118717193604,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.5015610868914,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.907852494577,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.278125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.91015625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 295566.70836679667,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 292554.5526980664,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002211,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7985310.382595059,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7571976.183698528,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.671398401260376,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 76.99059075407277,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.97861808075349,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 121.20221354166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 127.953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2733059.930254725,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2806070.847884425,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.02127,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 77874790.9540789,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 74252517.83864263,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.342902272939682,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.10121418977411,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.67387951060864,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.086848958333334,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.83203125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 475298.07342608756,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473668.26548683364,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002162,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12763679.53086365,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12363689.81774663,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.19123774766921997,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 79.74070155119936,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 82.12345633170882,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 67.579296875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 68.71484375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1624167.4467495908,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1621061.4256471533,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002168,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 44880490.36293235,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 42998137.66995414,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.7665982246398926,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.35255663741702,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.4945398202665,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.11744791666667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.390625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 935006.0281143117,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 942173.7676864407,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00218,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26093118.525278978,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 24658999.029923808,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Google Antigravity",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "993e4a845f4a50d3cdbdbe074f40517bbe1741ee",
          "message": "feat: implement OtapMetricsView for zero-copy OTAP metrics traversal (#2367)\n\n## Summary\n\nImplement zero-copy OTAP Arrow-backed views for metrics data, following\nthe same pattern as OtapLogsView. This enables direct traversal of\nmetrics Arrow RecordBatches without intermediate conversion to protobuf\nor Prost types.\n\n## New file: views/otap/metrics.rs \n\nComplete metrics hierarchy:\n- OtapMetricsView → ResourceMetrics → ScopeMetrics → MetricView →\nDataView\n- Gauge/Sum/Histogram/ExpHistogram/Summary views\n- NumberDataPoint, HistogramDataPoint, ExpHistogramDataPoint,\nSummaryDataPoint views\n- ExemplarView, BucketsView, ValueAtQuantileView\n\n## Modified files (visibility only)\n- MetricsArrays/QuantileArrays/PositiveNegativeArrayAccess fields →\npub(crate)\n- Shared helpers in logs.rs → pub(crate) for reuse\n- views/otap.rs: added mod metrics + re-export\n\n## Design\n- Pre-computed BTreeMap indexes at construction (same as OtapLogsView)\n- Reuses RowGroup, OtapAttributeView, OtapAnyValueView from logs module\n- Introduces Otap32AttributeIter for u32-keyed dp/exemplar attributes\n\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-22T14:45:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993e4a845f4a50d3cdbdbe074f40517bbe1741ee"
        },
        "date": 1774233778691,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.491904079914093,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 93.57069272714418,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.10573886090663,
            "unit": "%",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.408984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 291427.7327941363,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 289994.18789004546,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002306,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 7965572.13716144,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7537375.450578506,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 1 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5.620754718780518,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 77.51484488873402,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 81.70542943595314,
            "unit": "%",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 118.53997395833333,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 124.6953125,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 2423019.7126776786,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 2559211.710851888,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.011279,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 70735328.6109338,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 67398171.0892348,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 16 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.1405367851257324,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 92.99115344255834,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.50059858068497,
            "unit": "%",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.470572916666665,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.640625,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 571593.2035723575,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 578112.4345688237,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002169,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 15964863.276372483,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 15236821.930432508,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 2 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.7831273674964905,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 81.26995673315055,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 83.74534659525631,
            "unit": "%",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 65.03502604166667,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 66.984375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1700731.9501165256,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1714050.8478892127,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007368,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 47282339.209669776,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 45068293.483923405,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 8 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.14830182492733002,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 89.41753953867232,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 93.54742066845947,
            "unit": "%",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.518359375,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.57421875,
            "unit": "MiB",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1052946.9491881682,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 1054508.4887045375,
            "unit": "logs/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002324,
            "unit": "seconds",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 28894659.78953488,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 27657959.537231546,
            "unit": "bytes/sec",
            "extra": "Continuous - Saturation - 4 Core(s)/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}