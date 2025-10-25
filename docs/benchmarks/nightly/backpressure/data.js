window.BENCHMARK_DATA = {
  "lastUpdate": 1761387184945,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "fb291bfcb973afe17b8b98827bb12ebcb1d59203",
          "message": "Retry processor (stateless) Ack/Nack (#1295)\n\nReplaces stateful retry processor with new stateless Ack/Nack support.\n\nThe component has been restructured with exactly same configuration used\nby the OTel Collector retry configuration, with 3 duration fields (in\nseconds) instead of a retry limit.\n\nNew tests written. Metrics updated.\n\nNote: stores the num_items calculated from the initial data in a 3-word\ncalldata, as a temporary measure. In the 10/21/2025 SIG meeting we\ndiscussed placing num_items in another field of the context,\npotentially, that could be a separate change.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-10-24T15:24:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb291bfcb973afe17b8b98827bb12ebcb1d59203"
        },
        "date": 1761387184432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03589010286916346,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.053237760790672536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.326171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58798.97160678726,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4794.929870890139,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.33101484298095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.51468776230269,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.43828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.7265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13239094.349966377,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12711120.185954576,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}