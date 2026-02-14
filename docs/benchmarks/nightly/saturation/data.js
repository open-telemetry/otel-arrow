window.BENCHMARK_DATA = {
  "lastUpdate": 1771037470450,
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
      }
    ]
  }
}