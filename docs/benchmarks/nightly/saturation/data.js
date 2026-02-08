window.BENCHMARK_DATA = {
  "lastUpdate": 1770519720543,
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
      }
    ]
  }
}