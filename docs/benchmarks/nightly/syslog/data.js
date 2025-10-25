window.BENCHMARK_DATA = {
  "lastUpdate": 1761387183039,
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
        "date": 1761387180785,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 17.19632788861427,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.189623040592686,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.855078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 298035.70764201815,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13100997.077641955,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.188098484127503,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.44303317088313,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.085546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.1796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 542218.9448764318,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13187572.706173185,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.68043038435104,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.93712187654321,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.538671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.64453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 875609.514036014,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12969689.08514535,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 21.40888864910276,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 23.885728915978703,
            "unit": "%",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.319921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.48828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 125200,
            "unit": "count",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 334293.36205169477,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12937091.263444658,
            "unit": "bits/sec",
            "extra": "Nightly - Syslog/SYSLOG-3164-CEF-ATTR-OTAP - Network Utilization"
          }
        ]
      }
    ]
  }
}